pub mod block;
pub mod file;

mod loader;

use bevy::{
    asset::{Handle, RenderAssetUsages},
    color::palettes::{
        css::GOLD,
        tailwind::{PINK_100, RED_500},
    },
    gltf::GltfMesh,
    pbr::{ExtendedMaterial, MaterialExtension},
    picking::pointer::PointerInteraction,
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef, ShaderType},
    utils::HashMap,
};
use file::{BlockInfo, DiagonalType, Rotate, SlopeDirection, SlopeLevel, SlopeType};
pub use loader::{MapFileAsset, MapFileAssetLoader, MapFileAssetLoaderError};
use wgpu::{TextureDimension, TextureFormat};

use std::{
    f32::consts::TAU,
    fmt::Display,
    path::{Path, PathBuf},
    str::FromStr,
    sync::Arc,
};

use crate::loader::{StyleFileAsset, StyleFileAssetLoader};

pub fn plugin(app: &mut App) {
    let game_files_path = check_and_get_game_files_path();
    app.insert_resource(game_files_path)
        .insert_resource(CurrentMap(Maps::Downtown))
        .add_plugins(MaterialPlugin::<
            ExtendedMaterial<StandardMaterial, MyExtension>,
        >::default())
        .init_asset::<MapFileAsset>()
        .init_asset_loader::<MapFileAssetLoader>()
        .init_asset::<StyleFileAsset>()
        .init_asset_loader::<StyleFileAssetLoader>()
        .insert_state(MapState::NotLoaded)
        .add_systems(Startup, spawn_face_debug_text)
        .add_systems(Update, (add_debug_observer, draw_mesh_intersections))
        .add_systems(OnEnter(MapState::NotLoaded), load_map_resources)
        .add_systems(
            Update,
            setup_texture_index.run_if(in_state(MapState::SetupAssets)),
        )
        .add_systems(Update, setup_map.run_if(in_state(MapState::SetupMap)))
        .add_systems(Update, spawn_blocks.run_if(in_state(MapState::Loaded)));
}

fn load_map_resources(
    mut commands: Commands,
    game_files_path: Res<GameFilesPath>,
    current_map: Res<CurrentMap>,
    asset_server: Res<AssetServer>,
    mut next_state: ResMut<NextState<MapState>>,
) {
    let mut path = game_files_path.0.to_path_buf();
    path.push(current_map.0.get_style_file_name());
    let asset = asset_server.load(path);
    commands.insert_resource(Style { asset });

    let mut path = game_files_path.0.to_path_buf();
    path.push(current_map.0.get_map_file_name());
    let asset = asset_server.load(path);
    commands.insert_resource(Map { asset });

    let gltf = asset_server.load("gta2_block_model.glb");
    commands.insert_resource(BlockMesh(gltf));

    next_state.set(MapState::SetupAssets);
}

fn setup_texture_index(
    style: Res<Style>,
    style_asset: Res<Assets<StyleFileAsset>>,
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut next_state: ResMut<NextState<MapState>>,
) {
    let Some(style_file) = style_asset.get(&style.asset.clone()) else {
        return;
    };

    let mut texture_index = TextureIndex::default();

    let style_file = &style_file.0;

    for (id, tile) in style_file.tiles.iter().enumerate() {
        let palette_index = style_file.palette_index.physical_index.get(id).unwrap();
        let phys_palette = style_file
            .physical_palette
            .get(*palette_index as usize)
            .unwrap();

        let tile = tile
            .0
            .iter()
            .map(|p| *phys_palette.colors.get(*p as usize).unwrap())
            .flat_map(|c| {
                let c = c.to_ne_bytes();
                if c == [0, 0, 0, 0] {
                    [0, 0, 0, 0]
                } else {
                    [c[0], c[1], c[2], 255]
                }
            })
            .collect();

        const IMAGE_SIZE: u32 = 64;
        let size = wgpu::Extent3d {
            width: IMAGE_SIZE,
            height: IMAGE_SIZE,
            depth_or_array_layers: 1,
        };

        let image_handle = images.add(Image::new(
            size,
            TextureDimension::D2,
            tile,
            TextureFormat::Bgra8UnormSrgb,
            RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
        ));

        //let material_handler = materials.add(StandardMaterial {
        //    base_color_texture: Some(image_handle.clone()),
        //    alpha_mode: AlphaMode::AlphaToCoverage,
        //    ..default()
        //});

        texture_index.index.insert(id, image_handle);
    }

    commands.insert_resource(texture_index);
    next_state.set(MapState::SetupMap);
}

#[derive(Resource)]
struct BlockMesh(Handle<Gltf>);

#[derive(Component)]
struct UnloadedBlock {
    info: BlockInfo,
    pos: Vec3,
}

fn setup_map(
    map: Res<Map>,
    mut map_asset: ResMut<Assets<MapFileAsset>>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<MapState>>,
) {
    let Some(map_file) = map_asset.remove(&map.asset) else {
        return;
    };

    const X_MAX: usize = 256;
    const Y_MAX: usize = 256;

    for (i, block_info) in map_file
        .0
        .uncompressed_map
        .unwrap()
        .0
        .into_iter()
        .enumerate()
    {
        let x = i % X_MAX;
        let y = Y_MAX - (i / X_MAX) % Y_MAX;
        let z = i / (X_MAX * Y_MAX);

        let pos = Vec3 {
            x: x as f32,
            y: y as f32,
            z: z as f32,
        };

        commands.spawn(UnloadedBlock {
            info: block_info,
            pos,
        });
    }

    next_state.set(MapState::Loaded)
}

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
struct MyExtension {
    #[uniform(100)]
    holder: MyExtensionHolder,
}
impl MyExtension {
    fn new(flip: bool, rotate: Rotate) -> Self {
        //let angle = -compute_rotation(rotate, flip);
        let flip = if flip { 1 } else { 0 };

        let fraction = match rotate {
            Rotate::Degree0 => 0.0,
            Rotate::Degree90 => 0.25,
            Rotate::Degree180 => 0.5,
            Rotate::Degree270 => 0.75,
        };

        let angle = TAU * fraction;

        Self {
            holder: MyExtensionHolder { flip, angle },
        }
    }
}

#[derive(ShaderType, Reflect, Default, Clone, Debug)]
struct MyExtensionHolder {
    flip: u32,
    angle: f32,
}

const SHADER_ASSET_PATH: &str = "shaders/extended_material.wgsl";

impl MaterialExtension for MyExtension {
    fn fragment_shader() -> ShaderRef {
        SHADER_ASSET_PATH.into()
    }

    fn deferred_fragment_shader() -> ShaderRef {
        SHADER_ASSET_PATH.into()
    }
}

fn spawn_blocks(
    textures: Res<TextureIndex>,
    assets_gltf: Res<Assets<Gltf>>,
    assets_gltfmesh: Res<Assets<GltfMesh>>,
    block_mesh_res: Res<BlockMesh>,
    mut commands: Commands,
    mut ext_materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, MyExtension>>>,
    mut blocks: Query<(Entity, &mut UnloadedBlock)>, //mut next_state: ResMut<NextState<MapState>>,
) {
    let Some(block_gltf) = assets_gltf.get(&block_mesh_res.0) else {
        return;
    };

    //let marker_color = materials.add(Color::srgb(1.0, 0.0, 0.0));
    //let unknown_tile_color = materials.add(Color::srgba_u8(0, 255, 128, 255));

    for (entity, block) in &mut blocks {
        let pos = block.pos;
        let voxel = &block.info;

        commands.entity(entity).despawn();

        match &voxel.slope_type {
            SlopeType::None => spawn_normal_block(
                pos,
                &mut commands,
                voxel,
                block_gltf,
                &assets_gltfmesh,
                &textures,
                &mut ext_materials,
            ),
            //SlopeType::Diagonal(diagonal_type) => spawn_diagonal_block(
            //    pos,
            //    diagonal_type,
            //    &mut commands,
            //    voxel,
            //    block_gltf,
            //    &assets_gltfmesh,
            //    &map_materials,
            //    &unknown_tile_color,
            //),
            //SlopeType::ThreeSidedDiagonal(diagonal_type) => spawn_3_sided_diagonal_block(
            //    pos,
            //    diagonal_type,
            //    &mut commands,
            //    voxel,
            //    block_gltf,
            //    &assets_gltfmesh,
            //    &map_materials,
            //    &unknown_tile_color,
            //),
            //SlopeType::Degree26 { direction, level } => spawn_degree_26_block(
            //    pos,
            //    direction,
            //    level,
            //    &mut commands,
            //    voxel,
            //    block_gltf,
            //    &assets_gltfmesh,
            //    &map_materials,
            //    &marker_color,
            //),
            //SlopeType::Degree45(slope_direction) => spawn_degree_45_block(
            //    pos,
            //    slope_direction,
            //    &mut commands,
            //    voxel,
            //    block_gltf,
            //    &assets_gltfmesh,
            //    &map_materials,
            //    &marker_color,
            //),
            _ => {} //SlopeType::Degree7 { direction, level \} => todo!(),
                    //SlopeType::Degree26 { direction, level \} => todo!(),
                    //SlopeType::ThreeSidedDiagonal(diagonal_type) => todo!(),
                    //SlopeType::FourSidedDiagonal(diagonal_type) => todo!(),
                    //SlopeType::PartialBlock => todo!(),
                    //SlopeType::PartialCornerBlock => todo!(),
                    //SlopeType::Ignore => todo!(),
        }
    }
}

fn spawn_normal_block(
    pos: Vec3,
    commands: &mut Commands,
    voxel: &BlockInfo,
    block_gltf: &Gltf,
    assets_gltfmesh: &Res<Assets<GltfMesh>>,
    textures: &Res<TextureIndex>,
    ext_materials: &mut ResMut<Assets<ExtendedMaterial<StandardMaterial, MyExtension>>>,
) {
    let get_mesh = |name: &str, fliped| {
        let name = if fliped {
            format!("{name}.flip")
        } else {
            name.to_string()
        };
        let handle = block_gltf.named_meshes[name.as_str()].clone();
        &assets_gltfmesh.get(&handle).unwrap().primitives[0].mesh
    };

    // setup faces meshs
    let lid = get_mesh("block.lid", voxel.lid.flip);
    let left = get_mesh("block.left", voxel.left.flip);
    let right = get_mesh("block.right", voxel.right.flip);
    let top = get_mesh("block.top", voxel.top.flip);
    let bottom = get_mesh("block.bottom", voxel.bottom.flip);

    let face = &voxel.lid;
    if face.tile_id != 0 {
        let base_color_texture = textures.index.get(&face.tile_id).cloned();
        let ext_material = ext_materials.add(ExtendedMaterial {
            base: StandardMaterial {
                base_color_texture,
                alpha_mode: AlphaMode::AlphaToCoverage,
                ..default()
            },
            extension: MyExtension::new(face.flip, face.rotate),
        });

        commands.spawn((
            Mesh3d(lid.clone()),
            MeshMaterial3d(ext_material),
            FaceInfo::Lid(face.clone()),
            Transform::from_translation(pos),
        ));
    }

    let face = &voxel.left;
    if face.tile_id != 0 {
        let pos = if voxel.right.flat {
            pos.with_x(pos.x + 1.0)
        } else {
            pos
        };

        let base_color_texture = textures.index.get(&face.tile_id).cloned();
        let ext_material = ext_materials.add(ExtendedMaterial {
            base: StandardMaterial {
                base_color_texture,
                alpha_mode: AlphaMode::AlphaToCoverage,
                ..default()
            },
            extension: MyExtension::new(face.flip, face.rotate),
        });

        commands.spawn((
            Mesh3d(left.clone()),
            MeshMaterial3d(ext_material),
            FaceInfo::Normal(face.clone()),
            Transform::from_translation(pos),
        ));
    }

    let face = &voxel.right;
    if face.tile_id != 0 {
        let base_color_texture = textures.index.get(&face.tile_id).cloned();
        let ext_material = ext_materials.add(ExtendedMaterial {
            base: StandardMaterial {
                base_color_texture,
                alpha_mode: AlphaMode::AlphaToCoverage,
                ..default()
            },
            extension: MyExtension::new(face.flip, face.rotate),
        });

        commands.spawn((
            Mesh3d(right.clone()),
            MeshMaterial3d(ext_material),
            FaceInfo::Normal(face.clone()),
            Transform::from_translation(pos),
        ));
    }

    let face = &voxel.top;
    if face.tile_id != 0 {
        let base_color_texture = textures.index.get(&face.tile_id).cloned();
        let ext_material = ext_materials.add(ExtendedMaterial {
            base: StandardMaterial {
                base_color_texture,
                alpha_mode: AlphaMode::AlphaToCoverage,
                ..default()
            },
            extension: MyExtension::new(face.flip, face.rotate),
        });

        commands.spawn((
            Mesh3d(top.clone()),
            MeshMaterial3d(ext_material),
            FaceInfo::Normal(face.clone()),
            Transform::from_translation(pos),
        ));
    }

    let face = &voxel.bottom;
    if face.tile_id != 0 {
        let base_color_texture = textures.index.get(&face.tile_id).cloned();
        let ext_material = ext_materials.add(ExtendedMaterial {
            base: StandardMaterial {
                base_color_texture,
                alpha_mode: AlphaMode::AlphaToCoverage,
                ..default()
            },
            extension: MyExtension::new(face.flip, face.rotate),
        });

        commands.spawn((
            Mesh3d(bottom.clone()),
            MeshMaterial3d(ext_material),
            FaceInfo::Normal(face.clone()),
            Transform::from_translation(pos),
        ));
    }
}

//fn spawn_diagonal_block(
//    pos: Vec3,
//    diagonal_type: &DiagonalType,
//    commands: &mut Commands,
//    voxel: &BlockInfo,
//    block_gltf: &Gltf,
//    assets_gltfmesh: &Res<Assets<GltfMesh>>,
//    map_materials: &Res<TextureIndex>,
//    unknown_tile_color: &Handle<StandardMaterial>,
//) {
//    let get_mesh = |name| {
//        let handle = block_gltf.named_meshes[name].clone();
//        &assets_gltfmesh.get(&handle).unwrap().primitives[0].mesh
//    };
//
//    // setup faces meshs
//    let front = get_mesh("diagonal.lid");
//    let front_fliped = get_mesh("diagonal.lid.flip");
//
//    let left = get_mesh("diagonal.front");
//    let left_fliped = get_mesh("diagonal.front.flip");
//
//    let right = get_mesh("block.right");
//    let right_fliped = get_mesh("block.right.flip");
//
//    let top = get_mesh("block.top");
//    let top_fliped = get_mesh("block.top.flip");
//
//    //let bottom = get_mesh("block.bottom");
//    //let bottom_fliped = get_mesh("block.bottom.flip");
//
//    let (angle, front_face, top_face, right_face) = match diagonal_type {
//        DiagonalType::UpRight => (-0.5 * TAU, &voxel.right, &voxel.bottom, &voxel.left),
//        DiagonalType::UpLeft => (-0.25 * TAU, &voxel.left, &voxel.right, &voxel.bottom),
//        DiagonalType::DownLeft => (0.0, &voxel.left, &voxel.top, &voxel.right),
//        DiagonalType::DownRight => (0.25 * TAU, &voxel.right, &voxel.left, &voxel.top),
//    };
//    let face = &voxel.lid;
//    if face.tile_id != 0 {
//        let mesh = if face.flip {
//            front_fliped.clone()
//        } else {
//            front.clone()
//        };
//
//        commands.spawn((
//            Mesh3d(mesh),
//            MeshMaterial3d(
//                map_materials
//                    .index
//                    .get(&(face.tile_id))
//                    .cloned()
//                    .unwrap_or(unknown_tile_color.clone()),
//            ),
//            Transform::from_translation(pos)
//                .with_rotation(Quat::from_rotation_z(compute_rotation(
//                    face.rotate,
//                    face.flip,
//                )))
//                .with_rotation(Quat::from_rotation_z(angle)),
//            //MapPos(i),
//        ));
//        //.observe(on_click_show_debug);
//    }
//
//    let face = front_face;
//    if face.tile_id != 0 {
//        let mesh = if face.flip {
//            left_fliped.clone()
//        } else {
//            left.clone()
//        };
//
//        let pos = if voxel.right.flat {
//            pos.with_x(pos.x + 1.0)
//        } else {
//            pos
//        };
//
//        commands.spawn((
//            Mesh3d(mesh),
//            MeshMaterial3d(
//                map_materials
//                    .index
//                    .get(&(face.tile_id))
//                    .cloned()
//                    .unwrap_or(unknown_tile_color.clone()),
//            ),
//            Transform::from_translation(pos)
//                .with_rotation(Quat::from_rotation_x(compute_rotation(
//                    face.rotate,
//                    face.flip,
//                )))
//                .with_rotation(Quat::from_rotation_z(angle)),
//            //MapPos(i),
//        ));
//        //.observe(on_click_show_debug);
//    }
//
//    let face = right_face;
//    if face.tile_id != 0 {
//        let mesh = if face.flip {
//            right_fliped.clone()
//        } else {
//            right.clone()
//        };
//
//        commands.spawn((
//            Mesh3d(mesh.clone()),
//            MeshMaterial3d(
//                map_materials
//                    .index
//                    .get(&(face.tile_id))
//                    .cloned()
//                    .unwrap_or(unknown_tile_color.clone()),
//            ),
//            Transform::from_translation(pos)
//                .with_rotation(Quat::from_rotation_x(compute_rotation(
//                    face.rotate,
//                    face.flip,
//                )))
//                .with_rotation(Quat::from_rotation_z(angle)),
//            //MapPos(i),
//        ));
//        //.observe(on_click_show_debug);
//    }
//
//    let face = top_face;
//    if face.tile_id != 0 {
//        let mesh = if face.flip {
//            top_fliped.clone()
//        } else {
//            top.clone()
//        };
//
//        commands.spawn((
//            Mesh3d(mesh),
//            MeshMaterial3d(
//                map_materials
//                    .index
//                    .get(&(face.tile_id))
//                    .cloned()
//                    .unwrap_or(unknown_tile_color.clone()),
//            ),
//            Transform::from_translation(pos)
//                .with_rotation(Quat::from_rotation_y(compute_rotation(
//                    face.rotate,
//                    face.flip,
//                )))
//                .with_rotation(Quat::from_rotation_z(angle)),
//            //MapPos(i),
//        ));
//        //.observe(on_click_show_debug);
//    }
//
//    //let face = &voxel.bottom;
//    //if face.tile_id != 0 {
//    //    let mesh = if face.flip {
//    //        bottom_fliped.clone()
//    //    } else {
//    //        bottom.clone()
//    //    };
//
//    //    commands.spawn((
//    //        Mesh3d(mesh),
//    //        MeshMaterial3d(
//    //            map_materials
//    //                .index
//    //                .get(&(face.tile_id))
//    //                .cloned()
//    //                .unwrap_or(unknown_tile_color.clone()),
//    //        ),
//    //        Transform::from_translation(pos)
//    //            .with_rotation(Quat::from_rotation_y(compute_rotation(
//    //                face.rotate,
//    //                face.flip,
//    //            )))
//    //            .with_rotation(Quat::from_rotation_z(angle)),
//    //        //MapPos(i),
//    //    ));
//    //    //.observe(on_click_show_debug);
//    //}
//}
//
//fn spawn_degree_26_block(
//    pos: Vec3,
//    direction: &SlopeDirection,
//    level: &file::SlopeLevel,
//    commands: &mut Commands<'_, '_>,
//    voxel: &BlockInfo,
//    block_gltf: &Gltf,
//    assets_gltfmesh: &Assets<GltfMesh>,
//    map_materials: &TextureIndex,
//    unknown_tile_color: &Handle<StandardMaterial>,
//) {
//    let level_name = match level {
//        SlopeLevel::Low => "low",
//        SlopeLevel::High => "high",
//    };
//
//    let base_name = format!("slope_2.{level_name}");
//    let get_mesh = |name| {
//        let name = format!("{base_name}.{name}");
//        let handle = block_gltf.named_meshes[name.as_str()].clone();
//        &assets_gltfmesh.get(&handle).unwrap().primitives[0].mesh
//    };
//    let get_mesh_1 = |name| {
//        let handle = block_gltf.named_meshes[name].clone();
//        &assets_gltfmesh.get(&handle).unwrap().primitives[0].mesh
//    };
//
//    // setup faces meshs
//    let front = get_mesh("lid");
//    let front_fliped = get_mesh("lid.flip");
//
//    let left = get_mesh("left");
//    let left_fliped = get_mesh("left.flip");
//
//    let right = get_mesh("right");
//    let right_fliped = get_mesh("right.flip");
//
//    let top = if matches!(level, file::SlopeLevel::High) {
//        Some((get_mesh_1("block.top"), get_mesh_1("block.top.flip")))
//    } else {
//        None
//    };
//
//    let (angle, left_face, right_face, top_face) = match direction {
//        SlopeDirection::Down => (0.5 * TAU, &voxel.right, &voxel.left, &voxel.bottom),
//        SlopeDirection::Up => (0.0, &voxel.left, &voxel.right, &voxel.top),
//        SlopeDirection::Left => (0.25 * TAU, &voxel.bottom, &voxel.top, &voxel.right),
//        SlopeDirection::Right => (-0.25 * TAU, &voxel.top, &voxel.bottom, &voxel.left),
//    };
//
//    let face = &voxel.lid;
//    if face.tile_id != 0 {
//        let mesh = if face.flip {
//            front_fliped.clone()
//        } else {
//            front.clone()
//        };
//
//        commands.spawn((
//            Mesh3d(mesh),
//            MeshMaterial3d(
//                map_materials
//                    .index
//                    .get(&face.tile_id)
//                    .cloned()
//                    .unwrap_or(unknown_tile_color.clone()),
//            ),
//            Transform::from_translation(pos)
//                .with_rotation(Quat::from_rotation_z(compute_rotation(
//                    face.rotate,
//                    face.flip,
//                )))
//                .with_rotation(Quat::from_rotation_z(angle)),
//            //MapPos(i),
//        ));
//        //.observe(on_click_show_debug);
//    }
//
//    let face = &left_face;
//    if face.tile_id != 0 {
//        let mesh = if face.flip {
//            left_fliped.clone()
//        } else {
//            left.clone()
//        };
//
//        let pos = if voxel.right.flat {
//            pos.with_x(pos.x + 1.0)
//        } else {
//            pos
//        };
//
//        commands.spawn((
//            Mesh3d(mesh),
//            MeshMaterial3d(
//                map_materials
//                    .index
//                    .get(&face.tile_id)
//                    .cloned()
//                    .unwrap_or(unknown_tile_color.clone()),
//            ),
//            Transform::from_translation(pos)
//                .with_rotation(Quat::from_rotation_x(compute_rotation(
//                    face.rotate,
//                    face.flip,
//                )))
//                .with_rotation(Quat::from_rotation_z(angle)),
//            //MapPos(i),
//        ));
//        //.observe(on_click_show_debug);
//    }
//
//    let face = &right_face;
//    if face.tile_id != 0 {
//        let mesh = if face.flip {
//            right_fliped.clone()
//        } else {
//            right.clone()
//        };
//
//        commands.spawn((
//            Mesh3d(mesh.clone()),
//            MeshMaterial3d(
//                map_materials
//                    .index
//                    .get(&face.tile_id)
//                    .cloned()
//                    .unwrap_or(unknown_tile_color.clone()),
//            ),
//            Transform::from_translation(pos)
//                .with_rotation(Quat::from_rotation_x(compute_rotation(
//                    face.rotate,
//                    face.flip,
//                )))
//                .with_rotation(Quat::from_rotation_z(angle)),
//            //MapPos(i),
//        ));
//        //.observe(on_click_show_debug);
//    }
//
//    if let Some((top, top_fliped)) = top {
//        let face = &top_face;
//        if face.tile_id != 0 {
//            let mesh = if face.flip {
//                top_fliped.clone()
//            } else {
//                top.clone()
//            };
//
//            commands.spawn((
//                Mesh3d(mesh),
//                MeshMaterial3d(
//                    map_materials
//                        .index
//                        .get(&face.tile_id)
//                        .cloned()
//                        .unwrap_or(unknown_tile_color.clone()),
//                ),
//                Transform::from_translation(pos)
//                    .with_rotation(Quat::from_rotation_y(compute_rotation(
//                        face.rotate,
//                        face.flip,
//                    )))
//                    .with_rotation(Quat::from_rotation_z(angle)),
//                //MapPos(i),
//            ));
//            //.observe(on_click_show_debug);
//        }
//    }
//}
//
//fn spawn_degree_45_block(
//    pos: Vec3,
//    slope_direction: &SlopeDirection,
//    commands: &mut Commands,
//    voxel: &BlockInfo,
//    block_gltf: &Gltf,
//    assets_gltfmesh: &Res<Assets<GltfMesh>>,
//    map_materials: &Res<TextureIndex>,
//    unknown_tile_color: &Handle<StandardMaterial>,
//) {
//    let get_mesh = |name| {
//        let handle = block_gltf.named_meshes[name].clone();
//        &assets_gltfmesh.get(&handle).unwrap().primitives[0].mesh
//    };
//
//    // setup faces meshs
//    let front = get_mesh("degree_45.lid");
//    let front_fliped = get_mesh("degree_45.lid.flip");
//
//    let left = get_mesh("degree_45.left");
//    let left_fliped = get_mesh("degree_45.left.flip");
//
//    let right = get_mesh("degree_45.right");
//    let right_fliped = get_mesh("degree_45.right.flip");
//
//    let top = get_mesh("block.top");
//    let top_fliped = get_mesh("block.top.flip");
//
//    let (angle, left_face, right_face, top_face) = match slope_direction {
//        SlopeDirection::Down => (0.5 * TAU, &voxel.right, &voxel.left, &voxel.bottom),
//        SlopeDirection::Up => (0.0, &voxel.left, &voxel.right, &voxel.top),
//        SlopeDirection::Left => (0.25 * TAU, &voxel.bottom, &voxel.top, &voxel.right),
//        SlopeDirection::Right => (-0.25 * TAU, &voxel.top, &voxel.bottom, &voxel.left),
//    };
//
//    let face = &voxel.lid;
//    if face.tile_id != 0 {
//        let mesh = if face.flip {
//            front_fliped.clone()
//        } else {
//            front.clone()
//        };
//
//        commands.spawn((
//            Mesh3d(mesh),
//            MeshMaterial3d(
//                map_materials
//                    .index
//                    .get(&face.tile_id)
//                    .cloned()
//                    .unwrap_or(unknown_tile_color.clone()),
//            ),
//            Transform::from_translation(pos)
//                .with_rotation(Quat::from_rotation_z(compute_rotation(
//                    face.rotate,
//                    face.flip,
//                )))
//                .with_rotation(Quat::from_rotation_z(angle)),
//            //MapPos(i),
//        ));
//        //.observe(on_click_show_debug);
//    }
//
//    let face = &left_face;
//    if face.tile_id != 0 {
//        let mesh = if face.flip {
//            left_fliped.clone()
//        } else {
//            left.clone()
//        };
//
//        let pos = if voxel.right.flat {
//            pos.with_x(pos.x + 1.0)
//        } else {
//            pos
//        };
//
//        commands.spawn((
//            Mesh3d(mesh),
//            MeshMaterial3d(
//                map_materials
//                    .index
//                    .get(&face.tile_id)
//                    .cloned()
//                    .unwrap_or(unknown_tile_color.clone()),
//            ),
//            Transform::from_translation(pos)
//                .with_rotation(Quat::from_rotation_x(compute_rotation(
//                    face.rotate,
//                    face.flip,
//                )))
//                .with_rotation(Quat::from_rotation_z(angle)),
//            //MapPos(i),
//        ));
//        //.observe(on_click_show_debug);
//    }
//
//    let face = &right_face;
//    if face.tile_id != 0 {
//        let mesh = if face.flip {
//            right_fliped.clone()
//        } else {
//            right.clone()
//        };
//
//        commands.spawn((
//            Mesh3d(mesh.clone()),
//            MeshMaterial3d(
//                map_materials
//                    .index
//                    .get(&face.tile_id)
//                    .cloned()
//                    .unwrap_or(unknown_tile_color.clone()),
//            ),
//            Transform::from_translation(pos)
//                .with_rotation(Quat::from_rotation_x(compute_rotation(
//                    face.rotate,
//                    face.flip,
//                )))
//                .with_rotation(Quat::from_rotation_z(angle)),
//            //MapPos(i),
//        ));
//        //.observe(on_click_show_debug);
//    }
//
//    let face = &top_face;
//    if face.tile_id != 0 {
//        let mesh = if face.flip {
//            top_fliped.clone()
//        } else {
//            top.clone()
//        };
//
//        commands.spawn((
//            Mesh3d(mesh),
//            MeshMaterial3d(
//                map_materials
//                    .index
//                    .get(&face.tile_id)
//                    .cloned()
//                    .unwrap_or(unknown_tile_color.clone()),
//            ),
//            Transform::from_translation(pos)
//                .with_rotation(Quat::from_rotation_y(compute_rotation(
//                    face.rotate,
//                    face.flip,
//                )))
//                .with_rotation(Quat::from_rotation_z(angle)),
//            //MapPos(i),
//        ));
//        //.observe(on_click_show_debug);
//    }
//
//    //let face = &voxel.bottom;
//    //if bottom_material_id != 0 {
//    //    let mesh = if face.flip {
//    //        bottom_fliped.clone()
//    //    } else {
//    //        bottom.clone()
//    //    };
//
//    //    commands.spawn((
//    //        Mesh3d(mesh),
//    //        MeshMaterial3d(
//    //            map_materials
//    //                .index
//    //                .get(&bottom_material_id)
//    //                .cloned()
//    //                .unwrap_or(unknown_tile_color.clone()),
//    //        ),
//    //        Transform::from_translation(pos)
//    //            .with_rotation(Quat::from_rotation_y(compute_rotation(
//    //                face.rotate,
//    //                face.flip,
//    //            )))
//    //            .with_rotation(Quat::from_rotation_z(angle)),
//    //        //MapPos(i),
//    //    ));
//    //    //.observe(on_click_show_debug);
//    //}
//}
//
//fn spawn_3_sided_diagonal_block(
//    pos: Vec3,
//    diagonal_type: &DiagonalType,
//    commands: &mut Commands,
//    voxel: &BlockInfo,
//    block_gltf: &Gltf,
//    assets_gltfmesh: &Res<Assets<GltfMesh>>,
//    map_materials: &Res<TextureIndex>,
//    unknown_tile_color: &Handle<StandardMaterial>,
//) {
//    const THREE_SIDED_LID_TILE_ID: usize = 1023;
//
//    // current workaround it's 4-sided
//    if voxel.lid.tile_id != THREE_SIDED_LID_TILE_ID {
//        spawn_4_sided_diagonal_block(
//            pos,
//            diagonal_type,
//            commands,
//            voxel,
//            block_gltf,
//            assets_gltfmesh,
//            map_materials,
//            unknown_tile_color,
//        );
//        return;
//    }
//
//    let get_mesh = |name| {
//        let handle = block_gltf.named_meshes[name].clone();
//        &assets_gltfmesh.get(&handle).unwrap().primitives[0].mesh
//    };
//
//    // setup faces meshs
//    let front = get_mesh("3_sided.lid");
//    let front_fliped = get_mesh("3_sided.lid");
//
//    //let left = get_mesh("diagonal.front");
//    //let left_fliped = get_mesh("diagonal.front.flip");
//
//    let right = get_mesh("3_sided.right");
//    let right_fliped = get_mesh("3_sided.right");
//
//    let top = get_mesh("3_sided.top");
//    let top_fliped = get_mesh("3_sided.top");
//
//    //let bottom = get_mesh("block.bottom");
//    //let bottom_fliped = get_mesh("block.bottom.flip");
//
//    let (angle, front_face, top_face, right_face) = match diagonal_type {
//        DiagonalType::UpLeft => (-0.25 * TAU, &voxel.left, &voxel.right, &voxel.bottom),
//        DiagonalType::UpRight => (-0.5 * TAU, &voxel.right, &voxel.bottom, &voxel.left),
//        DiagonalType::DownLeft => (0.0, &voxel.left, &voxel.top, &voxel.right),
//        DiagonalType::DownRight => (0.25 * TAU, &voxel.right, &voxel.left, &voxel.top),
//    };
//
//    let face = front_face;
//    if face.tile_id != 0 {
//        let mesh = if face.flip {
//            front_fliped.clone()
//        } else {
//            front.clone()
//        };
//
//        commands.spawn((
//            Mesh3d(mesh),
//            MeshMaterial3d(
//                map_materials
//                    .index
//                    .get(&(face.tile_id))
//                    .cloned()
//                    .unwrap_or(unknown_tile_color.clone()),
//            ),
//            Transform::from_translation(pos)
//                .with_rotation(Quat::from_rotation_z(compute_rotation(
//                    face.rotate,
//                    face.flip,
//                )))
//                .with_rotation(Quat::from_rotation_z(angle)),
//            //MapPos(i),
//        ));
//        //.observe(on_click_show_debug);
//    }
//
//    //let face = &voxel.left;
//    //if face.tile_id != 0 {
//    //    let mesh = if face.flip {
//    //        left_fliped.clone()
//    //    } else {
//    //        left.clone()
//    //    };
//
//    //    let pos = if voxel.right.flat {
//    //        pos.with_x(pos.x + 1.0)
//    //    } else {
//    //        pos
//    //    };
//
//    //    commands.spawn((
//    //        Mesh3d(mesh),
//    //        MeshMaterial3d(
//    //            map_materials
//    //                .index
//    //                .get(&(face.tile_id))
//    //                .cloned()
//    //                .unwrap_or(unknown_tile_color.clone()),
//    //        ),
//    //        Transform::from_translation(pos)
//    //            .with_rotation(Quat::from_rotation_x(compute_rotation(
//    //                face.rotate,
//    //                face.flip,
//    //            )))
//    //            .with_rotation(Quat::from_rotation_z(angle)),
//    //        //MapPos(i),
//    //    ));
//    //    //.observe(on_click_show_debug);
//    //}
//
//    let face = right_face;
//    if face.tile_id != 0 {
//        let mesh = if face.flip {
//            right_fliped.clone()
//        } else {
//            right.clone()
//        };
//
//        commands.spawn((
//            Mesh3d(mesh.clone()),
//            MeshMaterial3d(
//                map_materials
//                    .index
//                    .get(&(face.tile_id))
//                    .cloned()
//                    .unwrap_or(unknown_tile_color.clone()),
//            ),
//            Transform::from_translation(pos)
//                .with_rotation(Quat::from_rotation_x(compute_rotation(
//                    face.rotate,
//                    face.flip,
//                )))
//                .with_rotation(Quat::from_rotation_z(angle)),
//            //MapPos(i),
//        ));
//        //.observe(on_click_show_debug);
//    }
//
//    let face = top_face;
//    if face.tile_id != 0 {
//        let mesh = if face.flip {
//            top_fliped.clone()
//        } else {
//            top.clone()
//        };
//
//        commands.spawn((
//            Mesh3d(mesh),
//            MeshMaterial3d(
//                map_materials
//                    .index
//                    .get(&(face.tile_id))
//                    .cloned()
//                    .unwrap_or(unknown_tile_color.clone()),
//            ),
//            Transform::from_translation(pos)
//                .with_rotation(Quat::from_rotation_y(compute_rotation(
//                    face.rotate,
//                    face.flip,
//                )))
//                .with_rotation(Quat::from_rotation_z(angle)),
//            //MapPos(i),
//        ));
//        //.observe(on_click_show_debug);
//    }
//
//    //let face = &voxel.bottom;
//    //if face.tile_id != 0 {
//    //    let mesh = if face.flip {
//    //        bottom_fliped.clone()
//    //    } else {
//    //        bottom.clone()
//    //    };
//
//    //    commands.spawn((
//    //        Mesh3d(mesh),
//    //        MeshMaterial3d(
//    //            map_materials
//    //                .index
//    //                .get(&(face.tile_id))
//    //                .cloned()
//    //                .unwrap_or(unknown_tile_color.clone()),
//    //        ),
//    //        Transform::from_translation(pos)
//    //            .with_rotation(Quat::from_rotation_y(compute_rotation(
//    //                face.rotate,
//    //                face.flip,
//    //            )))
//    //            .with_rotation(Quat::from_rotation_z(angle)),
//    //        //MapPos(i),
//    //    ));
//    //    //.observe(on_click_show_debug);
//    //}
//}
//
//fn spawn_4_sided_diagonal_block(
//    pos: Vec3,
//    diagonal_type: &DiagonalType,
//    commands: &mut Commands,
//    voxel: &BlockInfo,
//    block_gltf: &Gltf,
//    assets_gltfmesh: &Res<Assets<GltfMesh>>,
//    map_materials: &Res<TextureIndex>,
//    unknown_tile_color: &Handle<StandardMaterial>,
//) {
//    let get_mesh = |name| {
//        let handle = block_gltf.named_meshes[name].clone();
//        &assets_gltfmesh.get(&handle).unwrap().primitives[0].mesh
//    };
//
//    // setup faces meshs
//    let front = get_mesh("4_sided.lid");
//    let front_fliped = get_mesh("4_sided.lid");
//
//    let left = get_mesh("4_sided.left");
//    let left_fliped = get_mesh("4_sided.left");
//
//    let right = get_mesh("block.right");
//    let right_fliped = get_mesh("block.right");
//
//    let top = get_mesh("block.top");
//    let top_fliped = get_mesh("block.top");
//
//    //let bottom = get_mesh("block.bottom");
//    //let bottom_fliped = get_mesh("block.bottom.flip");
//
//    let (angle, front_face, left_face, top_face, right_face) = match diagonal_type {
//        DiagonalType::UpRight => (
//            -0.5 * TAU,
//            &voxel.lid,
//            &voxel.right,
//            &voxel.bottom,
//            &voxel.left,
//        ),
//        DiagonalType::UpLeft => (
//            -0.25 * TAU,
//            &voxel.lid,
//            &voxel.left,
//            &voxel.right,
//            &voxel.bottom,
//        ),
//        DiagonalType::DownLeft => (0.0, &voxel.lid, &voxel.left, &voxel.top, &voxel.right),
//        DiagonalType::DownRight => (
//            0.25 * TAU,
//            &voxel.lid,
//            &voxel.right,
//            &voxel.left,
//            &voxel.top,
//        ),
//    };
//
//    let face = front_face;
//    if face.tile_id != 0 {
//        let mesh = if face.flip {
//            front_fliped.clone()
//        } else {
//            front.clone()
//        };
//
//        commands.spawn((
//            Mesh3d(mesh),
//            MeshMaterial3d(
//                map_materials
//                    .index
//                    .get(&(face.tile_id))
//                    .cloned()
//                    .unwrap_or(unknown_tile_color.clone()),
//            ),
//            Transform::from_translation(pos)
//                .with_rotation(Quat::from_rotation_z(compute_rotation(
//                    face.rotate,
//                    face.flip,
//                )))
//                .with_rotation(Quat::from_rotation_z(angle)),
//            //MapPos(i),
//        ));
//        //.observe(on_click_show_debug);
//    }
//
//    let face = left_face;
//    if face.tile_id != 0 {
//        let mesh = if face.flip {
//            left_fliped.clone()
//        } else {
//            left.clone()
//        };
//
//        let pos = if voxel.right.flat {
//            pos.with_x(pos.x + 1.0)
//        } else {
//            pos
//        };
//
//        commands.spawn((
//            Mesh3d(mesh),
//            MeshMaterial3d(
//                map_materials
//                    .index
//                    .get(&(face.tile_id))
//                    .cloned()
//                    .unwrap_or(unknown_tile_color.clone()),
//            ),
//            Transform::from_translation(pos)
//                .with_rotation(Quat::from_rotation_x(compute_rotation(
//                    face.rotate,
//                    face.flip,
//                )))
//                .with_rotation(Quat::from_rotation_z(angle)),
//            //MapPos(i),
//        ));
//        //.observe(on_click_show_debug);
//    }
//
//    let face = right_face;
//    if face.tile_id != 0 {
//        let mesh = if face.flip {
//            right_fliped.clone()
//        } else {
//            right.clone()
//        };
//
//        commands.spawn((
//            Mesh3d(mesh.clone()),
//            MeshMaterial3d(
//                map_materials
//                    .index
//                    .get(&(face.tile_id))
//                    .cloned()
//                    .unwrap_or(unknown_tile_color.clone()),
//            ),
//            Transform::from_translation(pos)
//                .with_rotation(Quat::from_rotation_x(compute_rotation(
//                    face.rotate,
//                    face.flip,
//                )))
//                .with_rotation(Quat::from_rotation_z(angle)),
//            //MapPos(i),
//        ));
//        //.observe(on_click_show_debug);
//    }
//
//    let face = top_face;
//    if face.tile_id != 0 {
//        let mesh = if face.flip {
//            top_fliped.clone()
//        } else {
//            top.clone()
//        };
//
//        commands.spawn((
//            Mesh3d(mesh),
//            MeshMaterial3d(
//                map_materials
//                    .index
//                    .get(&(face.tile_id))
//                    .cloned()
//                    .unwrap_or(unknown_tile_color.clone()),
//            ),
//            Transform::from_translation(pos)
//                .with_rotation(Quat::from_rotation_y(compute_rotation(
//                    face.rotate,
//                    face.flip,
//                )))
//                .with_rotation(Quat::from_rotation_z(angle)),
//            //MapPos(i),
//        ));
//        //.observe(on_click_show_debug);
//    }
//
//    //let face = &voxel.bottom;
//    //if face.tile_id != 0 {
//    //    let mesh = if face.flip {
//    //        bottom_fliped.clone()
//    //    } else {
//    //        bottom.clone()
//    //    };
//
//    //    commands.spawn((
//    //        Mesh3d(mesh),
//    //        MeshMaterial3d(
//    //            map_materials
//    //                .index
//    //                .get(&(face.tile_id))
//    //                .cloned()
//    //                .unwrap_or(unknown_tile_color.clone()),
//    //        ),
//    //        Transform::from_translation(pos)
//    //            .with_rotation(Quat::from_rotation_y(compute_rotation(
//    //                face.rotate,
//    //                face.flip,
//    //            )))
//    //            .with_rotation(Quat::from_rotation_z(angle)),
//    //        //MapPos(i),
//    //    ));
//    //    //.observe(on_click_show_debug);
//    //}
//}

fn compute_rotation(rotate: Rotate, flip: bool) -> f32 {
    let angle = match rotate {
        Rotate::Degree0 => 0.0,
        Rotate::Degree90 => {
            if flip {
                TAU * 0.75
            } else {
                TAU * 0.25
            }
        }
        Rotate::Degree180 => TAU * 0.5,
        Rotate::Degree270 => {
            if flip {
                TAU * 0.25
            } else {
                TAU * 0.75
            }
        }
    };

    // rotate clock-wise
    -angle
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Default, States)]
enum MapState {
    #[default]
    NotLoaded,
    SetupAssets,
    SetupMap,
    Loaded,
}

#[derive(Component, Debug)]
struct MapPos(usize);

#[derive(Resource, Debug)]
pub struct Map {
    pub asset: Handle<MapFileAsset>,
}

#[derive(Resource, Debug)]
pub struct Style {
    pub asset: Handle<StyleFileAsset>,
}

#[derive(Resource, Debug, Default)]
pub struct TextureIndex {
    pub index: HashMap<usize, Handle<Image>>,
}

#[derive(Resource, Debug)]
struct GameFilesPath(Arc<Path>);

#[derive(Resource, Debug)]
struct CurrentMap(Maps);

#[derive(Debug, Default, Clone)]
enum Maps {
    #[default]
    Downtown,
    Residential,
    Industrial,
}

impl Display for Maps {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Maps::Downtown => "Downtown",
            Maps::Residential => "Residential",
            Maps::Industrial => "Industrial",
        };

        write!(f, "{name}")
    }
}

impl Maps {
    fn get_base_name(&self) -> &str {
        match self {
            Maps::Downtown => "bil",
            Maps::Residential => "wil",
            Maps::Industrial => "ste",
        }
    }

    fn get_style_file_name(&self) -> PathBuf {
        let name = self.get_base_name().to_string() + ".sty";
        PathBuf::from_str(&name).expect("valid file name")
    }

    fn get_map_file_name(&self) -> PathBuf {
        let name = self.get_base_name().to_string() + ".gmp";
        PathBuf::from_str(&name).expect("valid file name")
    }
}

fn check_and_get_game_files_path() -> GameFilesPath {
    let env_path = std::env::var("ARRIE_GAME_FILES").unwrap();
    let game_files_path = PathBuf::from_str(&env_path).unwrap();

    let mut downtown_map_file_path = game_files_path.to_path_buf();
    downtown_map_file_path.push(Maps::Downtown.get_map_file_name());

    let Ok(true) = std::fs::exists(downtown_map_file_path) else {
        panic!("not a GTA2 game files path");
    };

    GameFilesPath(Arc::from(game_files_path))
}

#[derive(Component, Debug)]
enum FaceInfo {
    Lid(file::LidFace),
    Normal(file::NormalFace),
}

fn add_debug_observer(mut commands: Commands, faces: Query<(Entity, Ref<FaceInfo>)>) {
    for (entity, info) in &faces {
        if info.is_added() {
            dbg!("Spawn observer");
            commands.entity(entity).observe(on_click_show_debug);
        }
    }
}

#[derive(Component)]
struct FaceDebugText;

fn spawn_face_debug_text(mut commands: Commands) {
    commands
        .spawn((
            Text::new("Block info\n"),
            TextFont {
                font_size: 12.0,
                ..default()
            },
        ))
        .with_child((
            TextSpan::default(),
            TextFont {
                font_size: 12.0,
                ..default()
            },
            TextColor(GOLD.into()),
            FaceDebugText,
        ));
}

/// A system that draws hit indicators for every pointer.
fn draw_mesh_intersections(pointers: Query<&PointerInteraction>, mut gizmos: Gizmos) {
    for (point, normal) in pointers
        .iter()
        .filter_map(|interaction| interaction.get_nearest_hit())
        .filter_map(|(_entity, hit)| hit.position.zip(hit.normal))
    {
        gizmos.sphere(point, 0.05, RED_500);
        gizmos.arrow(point, point + normal.normalize() * 0.5, PINK_100);
    }
}

fn on_click_show_debug(
    click: Trigger<Pointer<Click>>,
    faces: Query<&FaceInfo>,
    mut query: Query<&mut TextSpan, With<FaceDebugText>>,
) {
    dbg!("on click");
    let Ok(face) = faces.get(click.entity()) else {
        return;
    };

    for mut span in &mut query {
        **span = format!("{:#?}", face);
    }
}
