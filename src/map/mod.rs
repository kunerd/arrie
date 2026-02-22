mod block;
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
use file::{BlockInfo, DiagonalType, SlopeDirection, SlopeLevel, SlopeType};
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
        // .insert_resource(CurrentMap(Maps::Residential))
        // .insert_resource(CurrentMap(Maps::Industrial))
        .add_plugins(MaterialPlugin::<
            ExtendedMaterial<StandardMaterial, MyExtension>,
        >::default())
        .add_plugins(MeshPickingPlugin)
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
    pos: block::Position,
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

        commands.spawn(UnloadedBlock {
            info: block_info,
            pos: block::Position {
                x: x as u8,
                y: y as u8,
                z: z as u8,
            },
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
    fn new(flip: bool, angle: f32) -> Self {
        let flip = if flip { 1 } else { 0 };

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

    for (entity, block) in &mut blocks {
        let pos = Vec3::from(block.pos);
        let voxel = &block.info;

        commands.entity(entity).despawn();

        let builder = match &voxel.slope_type {
            SlopeType::None => {
                block::spawn_normal(
                    block.pos,
                    voxel,
                    block_gltf,
                    &assets_gltfmesh,
                    &textures,
                    &mut ext_materials,
                    &mut commands,
                );

                None
            }
            SlopeType::SlopeAbove => Some(spawn_normal_block(
                pos,
                voxel,
                block_gltf,
                &assets_gltfmesh,
                &textures,
                &mut ext_materials,
            )),
            SlopeType::Diagonal(diagonal_type) => {
                block::spawn_diagonal(
                    block.pos,
                    voxel,
                    diagonal_type,
                    block_gltf,
                    &assets_gltfmesh,
                    &textures,
                    &mut ext_materials,
                    &mut commands,
                );
                None
            }
            SlopeType::ThreeSidedDiagonal(diagonal_type) => Some(spawn_3_sided_diagonal_block(
                pos,
                diagonal_type,
                voxel,
                block_gltf,
                &assets_gltfmesh,
                &textures,
                &mut ext_materials,
            )),
            SlopeType::Degree26 { direction, level } => Some(spawn_degree_26_block(
                pos,
                direction,
                level,
                voxel,
                block_gltf,
                &assets_gltfmesh,
                &textures,
                &mut ext_materials,
            )),
            SlopeType::Degree45(slope_direction) => Some(spawn_degree_45_block(
                pos,
                slope_direction,
                voxel,
                block_gltf,
                &assets_gltfmesh,
                &textures,
                &mut ext_materials,
            )),
            SlopeType::Degree7 { direction, index } => spawn_7_degree_block(
                pos,
                direction,
                *index,
                voxel,
                block_gltf,
                &assets_gltfmesh,
                &textures,
                &mut ext_materials,
            ),
            SlopeType::PartialBlock(partial_pos) => Some(create_partial_block(
                pos,
                voxel,
                partial_pos,
                block_gltf,
                &assets_gltfmesh,
                &textures,
                &mut ext_materials,
            )),
            SlopeType::PartialCornerBlock(partial_pos) => Some(create_partial_corner_block(
                pos,
                voxel,
                partial_pos,
                block_gltf,
                &assets_gltfmesh,
                &textures,
                &mut ext_materials,
            )),
            _ => None, //SlopeType::Degree7 { direction, level \} => todo!(),
                       //SlopeType::PartialCornerBlock => todo!(),
                       //SlopeType::Ignore => todo!(),
        };

        let Some(mut builder) = builder else {
            continue;
        };

        // FIXME depends on rotation
        match (voxel.left.flat, voxel.right.flat) {
            (true, true) => builder.left_right = Flatness::Both,
            (true, false) => builder.left_right = Flatness::Left,
            (false, true) => builder.left_right = Flatness::Right,
            (false, false) => builder.left_right = Flatness::None,
        }

        match (voxel.top.flat, voxel.bottom.flat) {
            (true, true) => builder.top_bottom = Flatness::Both,
            (true, false) => builder.top_bottom = Flatness::Left,
            (false, true) => builder.top_bottom = Flatness::Right,
            (false, false) => builder.top_bottom = Flatness::None,
        }

        commands.queue(builder);
    }
}

enum Flatness {
    None,
    Left,
    Right,
    Both,
}

struct BlockBuilder {
    lid: Option<BlockFace>,
    left: Option<BlockFace>,
    right: Option<BlockFace>,
    top: Option<BlockFace>,
    bottom: Option<BlockFace>,
    left_right: Flatness,
    top_bottom: Flatness,
    position: Vec3,
    rotation: Option<f32>,
}

impl Command for BlockBuilder {
    fn apply(self, world: &mut World) {
        let mut transform = Transform::from_translation(self.position);

        if let Some(angle) = self.rotation {
            transform = transform.with_rotation(Quat::from_rotation_z(angle));
        }

        world
            .spawn((Block, Visibility::Visible, transform))
            .with_children(|parent| {
                let spawn_child_maybe = |parent: &mut WorldChildBuilder, maybe_child| {
                    if let Some(child) = maybe_child {
                        parent.spawn(child);
                    }
                };

                spawn_child_maybe(parent, self.lid);
                match self.left_right {
                    Flatness::Left => {
                        if let Some(right) = self.right {
                            parent.spawn((
                                right,
                                Transform::from_translation(Vec3::new(-1.0, 0.0, 0.0)),
                            ));
                        }
                        spawn_child_maybe(parent, self.left);
                    }
                    Flatness::Right => {
                        if let Some(left) = self.left {
                            parent.spawn((
                                left,
                                Transform::from_translation(Vec3::new(1.0, 0.0, 0.0)),
                            ));
                        }
                        spawn_child_maybe(parent, self.right);
                    }
                    Flatness::Both => {
                        if let Some(right) = self.right.clone() {
                            parent.spawn((
                                right,
                                Transform::from_translation(Vec3::new(-1.0, 0.0, 0.0)),
                            ));
                        }
                        spawn_child_maybe(parent, self.left.clone());

                        if let Some(left) = self.left {
                            parent.spawn((
                                left,
                                Transform::from_translation(Vec3::new(1.0, 0.0, 0.0)),
                            ));
                        }
                        spawn_child_maybe(parent, self.right);
                    }
                    Flatness::None => {
                        spawn_child_maybe(parent, self.left);
                        spawn_child_maybe(parent, self.right);
                    }
                }

                match self.top_bottom {
                    Flatness::Left => {
                        if let Some(bottom) = self.bottom {
                            parent.spawn((
                                bottom,
                                Transform::from_translation(Vec3::new(0.0, 1.0, 0.0)),
                            ));
                        }
                        spawn_child_maybe(parent, self.top);
                    }
                    Flatness::Right => {
                        if let Some(top) = self.top {
                            parent.spawn((
                                top,
                                Transform::from_translation(Vec3::new(0.0, -1.0, 0.0)),
                            ));
                        }
                        spawn_child_maybe(parent, self.bottom);
                    }
                    Flatness::Both => {
                        if let Some(bottom) = self.bottom.clone() {
                            parent.spawn((
                                bottom,
                                Transform::from_translation(Vec3::new(0.0, 1.0, 0.0)),
                            ));
                        }
                        spawn_child_maybe(parent, self.top.clone());

                        if let Some(top) = self.top {
                            parent.spawn((
                                top,
                                Transform::from_translation(Vec3::new(0.0, -1.0, 0.0)),
                            ));
                        }
                        spawn_child_maybe(parent, self.bottom);
                    }
                    Flatness::None => {
                        spawn_child_maybe(parent, self.top);
                        spawn_child_maybe(parent, self.bottom);
                    }
                }
            });
    }
}

#[derive(Component)]
struct Block;

#[derive(Bundle, Clone)]
struct BlockFace {
    mesh: Mesh3d,
    material: MeshMaterial3d<ExtendedMaterial<StandardMaterial, MyExtension>>,
    info: FaceInfo,
}

fn spawn_normal_block(
    position: Vec3,
    voxel: &BlockInfo,
    block_gltf: &Gltf,
    assets_gltfmesh: &Res<Assets<GltfMesh>>,
    textures: &Res<TextureIndex>,
    ext_materials: &mut ResMut<Assets<ExtendedMaterial<StandardMaterial, MyExtension>>>,
) -> BlockBuilder {
    let get_mesh = |name: &str| {
        let handle = block_gltf.named_meshes[name].clone();
        &assets_gltfmesh.get(&handle).unwrap().primitives[0].mesh
    };

    let mut spawn_face_maybe = |mesh: Handle<Mesh>, face: FaceInfo| -> Option<BlockFace> {
        if face.tile_id == 0 {
            return None;
        }

        let base_color_texture = textures.index.get(&face.tile_id).cloned();
        let ext_material = ext_materials.add(ExtendedMaterial {
            base: StandardMaterial {
                base_color_texture,
                alpha_mode: AlphaMode::AlphaToCoverage,
                ..default()
            },
            extension: MyExtension::new(face.flip, face.rotate.clockwise_rad()),
        });

        Some(BlockFace {
            mesh: Mesh3d(mesh),
            material: MeshMaterial3d(ext_material),
            info: face,
        })
    };

    let lid = get_mesh("block.lid");
    let left = get_mesh("block.left");
    let right = get_mesh("block.right");
    let top = get_mesh("block.top");
    let bottom = get_mesh("block.bottom");

    BlockBuilder {
        lid: spawn_face_maybe(lid.clone(), FaceInfo(voxel.lid.clone())),
        left: spawn_face_maybe(left.clone(), FaceInfo(voxel.left.clone())),
        right: spawn_face_maybe(right.clone(), FaceInfo(voxel.right.clone())),
        top: spawn_face_maybe(top.clone(), FaceInfo(voxel.top.clone())),
        bottom: spawn_face_maybe(bottom.clone(), FaceInfo(voxel.bottom.clone())),
        left_right: Flatness::None,
        top_bottom: Flatness::None,
        position,
        rotation: None,
    }
}

fn create_partial_block(
    position: Vec3,
    voxel: &BlockInfo,
    partial_pos: &file::PartialPosition,
    block_gltf: &Gltf,
    assets_gltfmesh: &Res<Assets<GltfMesh>>,
    textures: &Res<TextureIndex>,
    ext_materials: &mut ResMut<Assets<ExtendedMaterial<StandardMaterial, MyExtension>>>,
) -> BlockBuilder {
    let get_mesh = |name: &str| {
        let name = format!("partial.{name}");
        let handle = block_gltf.named_meshes[name.as_str()].clone();
        &assets_gltfmesh.get(&handle).unwrap().primitives[0].mesh
    };

    let mut spawn_face_maybe =
        |mesh: Handle<Mesh>, face: FaceInfo, angle: Option<f32>| -> Option<BlockFace> {
            if face.tile_id == 0 {
                return None;
            }

            let angle = angle.unwrap_or(0.0);
            let rotation = if face.flip {
                face.rotate.clockwise_rad() - angle
            } else {
                face.rotate.clockwise_rad() + angle
            };
            let base_color_texture = textures.index.get(&face.tile_id).cloned();
            let ext_material = ext_materials.add(ExtendedMaterial {
                base: StandardMaterial {
                    base_color_texture,
                    alpha_mode: AlphaMode::AlphaToCoverage,
                    ..default()
                },
                extension: MyExtension::new(face.flip, rotation),
            });

            Some(BlockFace {
                mesh: Mesh3d(mesh),
                material: MeshMaterial3d(ext_material),
                info: face,
            })
        };

    let lid = get_mesh("lid");
    let left = get_mesh("left");
    let right = get_mesh("right");
    let top = get_mesh("top");
    let bottom = get_mesh("bottom");

    const PARTIAL_POS_OFFSET: f32 = (64.0 - 24.0) / 64.0 / 2.0;

    let mut position = position;
    let (rotation, left_face, right_face, top_face, bottom_face) = match partial_pos {
        file::PartialPosition::Left => {
            position.x -= PARTIAL_POS_OFFSET;
            (
                Some(0.75 * TAU),
                &voxel.bottom,
                &voxel.top,
                &voxel.right,
                &voxel.left,
            )
        }
        file::PartialPosition::Right => {
            position.x += PARTIAL_POS_OFFSET;
            (
                Some(0.25 * TAU),
                &voxel.bottom,
                &voxel.top,
                &voxel.left,
                &voxel.right,
            )
        }
        file::PartialPosition::Top => {
            position.y += PARTIAL_POS_OFFSET;
            (
                Some(0.5 * TAU),
                &voxel.right,
                &voxel.left,
                &voxel.bottom,
                &voxel.top,
            )
        }
        file::PartialPosition::Bottom => {
            position.y -= PARTIAL_POS_OFFSET;
            (None, &voxel.left, &voxel.right, &voxel.top, &voxel.bottom)
        }
    };

    BlockBuilder {
        lid: spawn_face_maybe(lid.clone(), FaceInfo(voxel.lid.clone()), rotation),
        left: spawn_face_maybe(left.clone(), FaceInfo(left_face.clone()), None),
        right: spawn_face_maybe(right.clone(), FaceInfo(right_face.clone()), None),
        top: spawn_face_maybe(top.clone(), FaceInfo(top_face.clone()), None),
        bottom: spawn_face_maybe(bottom.clone(), FaceInfo(bottom_face.clone()), None),
        left_right: Flatness::None,
        top_bottom: Flatness::None,
        position,
        rotation,
    }
}

fn create_partial_corner_block(
    position: Vec3,
    voxel: &BlockInfo,
    partial_pos: &file::CornerPosition,
    block_gltf: &Gltf,
    assets_gltfmesh: &Res<Assets<GltfMesh>>,
    textures: &Res<TextureIndex>,
    ext_materials: &mut ResMut<Assets<ExtendedMaterial<StandardMaterial, MyExtension>>>,
) -> BlockBuilder {
    let get_mesh = |name: &str| {
        let name = format!("partial_corner.{name}");
        let handle = block_gltf.named_meshes[name.as_str()].clone();
        &assets_gltfmesh.get(&handle).unwrap().primitives[0].mesh
    };

    let mut spawn_face_maybe =
        |mesh: Handle<Mesh>, face: FaceInfo, angle: Option<f32>| -> Option<BlockFace> {
            if face.tile_id == 0 {
                return None;
            }

            let angle = angle.unwrap_or(0.0);
            let rotation = if face.flip {
                face.rotate.clockwise_rad() - angle
            } else {
                face.rotate.clockwise_rad() + angle
            };
            let base_color_texture = textures.index.get(&face.tile_id).cloned();
            let ext_material = ext_materials.add(ExtendedMaterial {
                base: StandardMaterial {
                    base_color_texture,
                    alpha_mode: AlphaMode::AlphaToCoverage,
                    ..default()
                },
                extension: MyExtension::new(face.flip, rotation),
            });

            Some(BlockFace {
                mesh: Mesh3d(mesh),
                material: MeshMaterial3d(ext_material),
                info: face,
            })
        };

    let lid = get_mesh("lid");
    let left = get_mesh("left");
    let right = get_mesh("right");
    let top = get_mesh("top");
    let bottom = get_mesh("bottom");

    const PARTIAL_POS_OFFSET: f32 = (64.0 - 24.0) / 64.0 / 2.0;

    let mut position = position;

    // FIXME: rotation doesn't work when UV flip is true
    // FIXME: UV maps of sides are wrong in model file
    let (rotation, left_face, right_face, top_face, bottom_face) = match partial_pos {
        file::CornerPosition::TopLeft => {
            position.x -= PARTIAL_POS_OFFSET;
            position.y += PARTIAL_POS_OFFSET;
            (None, &voxel.left, &voxel.right, &voxel.top, &voxel.bottom)
        }
        file::CornerPosition::TopRight => {
            position.x += PARTIAL_POS_OFFSET;
            position.y += PARTIAL_POS_OFFSET;
            (
                Some(-0.75 * TAU),
                &voxel.bottom,
                &voxel.top,
                &voxel.left,
                &voxel.right,
            )
        }
        file::CornerPosition::BottomLeft => {
            position.x -= PARTIAL_POS_OFFSET;
            position.y -= PARTIAL_POS_OFFSET;
            (
                Some(-0.25 * TAU),
                &voxel.top,
                &voxel.left,
                &voxel.bottom,
                &voxel.right,
            )
        }
        file::CornerPosition::BottomRight => {
            position.x += PARTIAL_POS_OFFSET;
            position.y -= PARTIAL_POS_OFFSET;
            (
                Some(0.5 * TAU),
                &voxel.right,
                &voxel.left,
                &voxel.bottom,
                &voxel.top,
            )
        }
    };

    BlockBuilder {
        lid: spawn_face_maybe(lid.clone(), FaceInfo(voxel.lid.clone()), rotation),
        left: spawn_face_maybe(left.clone(), FaceInfo(left_face.clone()), None),
        right: spawn_face_maybe(right.clone(), FaceInfo(right_face.clone()), None),
        top: spawn_face_maybe(top.clone(), FaceInfo(top_face.clone()), None),
        bottom: spawn_face_maybe(bottom.clone(), FaceInfo(bottom_face.clone()), None),
        left_right: Flatness::None,
        top_bottom: Flatness::None,
        position,
        rotation,
    }
}

fn spawn_degree_26_block(
    position: Vec3,
    direction: &SlopeDirection,
    level: &file::SlopeLevel,
    voxel: &BlockInfo,
    block_gltf: &Gltf,
    assets_gltfmesh: &Res<Assets<GltfMesh>>,
    textures: &Res<TextureIndex>,
    ext_materials: &mut ResMut<Assets<ExtendedMaterial<StandardMaterial, MyExtension>>>,
) -> BlockBuilder {
    let level_name = match level {
        SlopeLevel::Low => "low",
        SlopeLevel::High => "high",
    };

    let base_name = format!("slope_2.{level_name}");
    let get_mesh = |name| {
        let name = format!("{base_name}.{name}");
        let handle = block_gltf.named_meshes[name.as_str()].clone();
        &assets_gltfmesh.get(&handle).unwrap().primitives[0].mesh
    };
    let get_mesh_1 = |name| {
        let handle = block_gltf.named_meshes[name].clone();
        &assets_gltfmesh.get(&handle).unwrap().primitives[0].mesh
    };

    let mut spawn_face_maybe =
        |mesh: Handle<Mesh>, face: FaceInfo, angle: Option<f32>| -> Option<BlockFace> {
            if face.tile_id == 0 {
                return None;
            }

            let base_color_texture = textures.index.get(&face.tile_id).cloned();

            let angle = angle.unwrap_or(0.0);
            let rotation = if face.flip {
                face.rotate.clockwise_rad() - angle
            } else {
                face.rotate.clockwise_rad() + angle
            };

            let ext_material = ext_materials.add(ExtendedMaterial {
                base: StandardMaterial {
                    base_color_texture,
                    alpha_mode: AlphaMode::AlphaToCoverage,
                    ..default()
                },
                extension: MyExtension::new(face.flip, rotation),
            });

            Some(BlockFace {
                mesh: Mesh3d(mesh),
                material: MeshMaterial3d(ext_material),
                info: face,
            })
        };

    // setup faces meshs
    let lid = get_mesh("lid");
    let left = get_mesh("left");
    let right = get_mesh("right");
    let top = if matches!(level, file::SlopeLevel::High) {
        Some(get_mesh_1("block.top"))
    } else {
        None
    };

    let (angle, left_face, right_face, top_face) = match direction {
        SlopeDirection::Down => (0.5 * TAU, &voxel.right, &voxel.left, &voxel.bottom),
        SlopeDirection::Up => (0.0, &voxel.left, &voxel.right, &voxel.top),
        SlopeDirection::Left => (0.25 * TAU, &voxel.bottom, &voxel.top, &voxel.left),
        SlopeDirection::Right => (0.75 * TAU, &voxel.top, &voxel.bottom, &voxel.right),
    };

    BlockBuilder {
        lid: spawn_face_maybe(lid.clone(), FaceInfo(voxel.lid.clone()), Some(angle)),
        left: spawn_face_maybe(left.clone(), FaceInfo(left_face.clone()), None),
        right: spawn_face_maybe(right.clone(), FaceInfo(right_face.clone()), None),
        top: top.and_then(|top| spawn_face_maybe(top.clone(), FaceInfo(top_face.clone()), None)),
        bottom: None,
        left_right: Flatness::None,
        top_bottom: Flatness::None,
        position,
        rotation: Some(angle),
    }
}

fn spawn_7_degree_block(
    position: Vec3,
    direction: &SlopeDirection,
    index: u8,
    voxel: &BlockInfo,
    block_gltf: &Gltf,
    assets_gltfmesh: &Res<Assets<GltfMesh>>,
    textures: &Res<TextureIndex>,
    ext_materials: &mut ResMut<Assets<ExtendedMaterial<StandardMaterial, MyExtension>>>,
) -> Option<BlockBuilder> {
    let level_name = match index {
        0 => "0",
        1 => "1",
        2 => "2",
        3 => "3",
        4 => "4",
        5 => "5",
        6 => "6",
        7 => "7",
        _ => panic!("index out of bounds"),
    };

    let base_name = format!("slope_8.{level_name}");
    let get_mesh = |name| {
        let name = format!("{base_name}.{name}");
        let handle = block_gltf.named_meshes[name.as_str()].clone();
        &assets_gltfmesh.get(&handle).unwrap().primitives[0].mesh
    };
    let get_mesh_1 = |name| {
        let handle = block_gltf.named_meshes[name].clone();
        &assets_gltfmesh.get(&handle).unwrap().primitives[0].mesh
    };

    let mut spawn_face_maybe =
        |mesh: Handle<Mesh>, face: FaceInfo, angle: Option<f32>| -> Option<BlockFace> {
            if face.tile_id == 0 {
                return None;
            }

            let base_color_texture = textures.index.get(&face.tile_id).cloned();

            let angle = angle.unwrap_or(0.0);
            let rotation = if face.flip {
                face.rotate.clockwise_rad() - angle
            } else {
                face.rotate.clockwise_rad() + angle
            };

            let ext_material = ext_materials.add(ExtendedMaterial {
                base: StandardMaterial {
                    base_color_texture,
                    alpha_mode: AlphaMode::AlphaToCoverage,
                    ..default()
                },
                extension: MyExtension::new(face.flip, rotation),
            });

            Some(BlockFace {
                mesh: Mesh3d(mesh),
                material: MeshMaterial3d(ext_material),
                info: face,
            })
        };

    // setup faces meshs
    let lid = get_mesh("lid");
    let left = get_mesh("left");
    let right = get_mesh("right");
    let top = if index > 0 {
        Some(get_mesh_1("block.top"))
    } else {
        None
    };

    let (angle, left_face, right_face, top_face) = match direction {
        SlopeDirection::Down => (0.5 * TAU, &voxel.right, &voxel.left, &voxel.bottom),
        SlopeDirection::Up => (0.0, &voxel.left, &voxel.right, &voxel.top),
        SlopeDirection::Left => (0.25 * TAU, &voxel.bottom, &voxel.top, &voxel.left),
        SlopeDirection::Right => (0.75 * TAU, &voxel.top, &voxel.bottom, &voxel.right),
    };

    Some(BlockBuilder {
        lid: spawn_face_maybe(lid.clone(), FaceInfo(voxel.lid.clone()), Some(angle)),
        left: spawn_face_maybe(left.clone(), FaceInfo(left_face.clone()), None),
        right: spawn_face_maybe(right.clone(), FaceInfo(right_face.clone()), None),
        top: top.and_then(|top| spawn_face_maybe(top.clone(), FaceInfo(top_face.clone()), None)),
        bottom: None,
        left_right: Flatness::None,
        top_bottom: Flatness::None,
        position,
        rotation: Some(angle),
    })
}

fn spawn_degree_45_block(
    position: Vec3,
    direction: &SlopeDirection,
    voxel: &BlockInfo,
    block_gltf: &Gltf,
    assets_gltfmesh: &Res<Assets<GltfMesh>>,
    textures: &Res<TextureIndex>,
    ext_materials: &mut ResMut<Assets<ExtendedMaterial<StandardMaterial, MyExtension>>>,
) -> BlockBuilder {
    let get_mesh = |name| {
        let handle = block_gltf.named_meshes[name].clone();
        &assets_gltfmesh.get(&handle).unwrap().primitives[0].mesh
    };

    let mut spawn_face_maybe =
        |mesh: Handle<Mesh>, face: FaceInfo, angle: Option<f32>| -> Option<BlockFace> {
            if face.tile_id == 0 {
                return None;
            }

            let base_color_texture = textures.index.get(&face.tile_id).cloned();

            let angle = angle.unwrap_or(0.0);
            let rotation = if face.flip {
                face.rotate.clockwise_rad() - angle
            } else {
                face.rotate.clockwise_rad() + angle
            };

            let ext_material = ext_materials.add(ExtendedMaterial {
                base: StandardMaterial {
                    base_color_texture,
                    alpha_mode: AlphaMode::AlphaToCoverage,
                    ..default()
                },
                extension: MyExtension::new(face.flip, rotation),
            });

            Some(BlockFace {
                mesh: Mesh3d(mesh),
                material: MeshMaterial3d(ext_material),
                info: face,
            })
        };

    // setup faces meshs
    let lid = get_mesh("degree_45.lid");
    let left = get_mesh("degree_45.left");
    let right = get_mesh("degree_45.right");
    let top = get_mesh("block.top");

    let (angle, left_face, right_face, top_face) = match direction {
        SlopeDirection::Down => (0.5 * TAU, &voxel.right, &voxel.left, &voxel.bottom),
        SlopeDirection::Up => (0.0, &voxel.left, &voxel.right, &voxel.top),
        SlopeDirection::Left => (0.25 * TAU, &voxel.bottom, &voxel.top, &voxel.right),
        SlopeDirection::Right => (0.75 * TAU, &voxel.top, &voxel.bottom, &voxel.left),
    };

    BlockBuilder {
        lid: spawn_face_maybe(lid.clone(), FaceInfo(voxel.lid.clone()), Some(angle)),
        left: spawn_face_maybe(left.clone(), FaceInfo(left_face.clone()), None),
        right: spawn_face_maybe(right.clone(), FaceInfo(right_face.clone()), None),
        top: spawn_face_maybe(top.clone(), FaceInfo(top_face.clone()), None),
        bottom: None,
        left_right: Flatness::None,
        top_bottom: Flatness::None,
        position,
        rotation: Some(angle),
    }
}

fn spawn_3_sided_diagonal_block(
    mut position: Vec3,
    diagonal_type: &DiagonalType,
    voxel: &BlockInfo,
    block_gltf: &Gltf,
    assets_gltfmesh: &Res<Assets<GltfMesh>>,
    textures: &Res<TextureIndex>,
    ext_materials: &mut ResMut<Assets<ExtendedMaterial<StandardMaterial, MyExtension>>>,
) -> BlockBuilder {
    const THREE_SIDED_LID_TILE_ID: usize = 1023;

    // current workaround it's 4-sided
    if voxel.lid.tile_id != THREE_SIDED_LID_TILE_ID {
        return spawn_4_sided_diagonal_block(
            position,
            diagonal_type,
            voxel,
            block_gltf,
            assets_gltfmesh,
            textures,
            ext_materials,
        );
    }

    let get_mesh = |name| {
        let handle = block_gltf.named_meshes[name].clone();
        &assets_gltfmesh.get(&handle).unwrap().primitives[0].mesh
    };

    let mut spawn_face_maybe = |mesh: Handle<Mesh>, face: FaceInfo, _angle| -> Option<BlockFace> {
        if face.tile_id == 0 {
            return None;
        }

        let base_color_texture = textures.index.get(&face.tile_id).cloned();

        let ext_material = ext_materials.add(ExtendedMaterial {
            base: StandardMaterial {
                base_color_texture,
                alpha_mode: AlphaMode::AlphaToCoverage,
                ..default()
            },
            extension: MyExtension::new(face.flip, face.rotate.clockwise_rad()),
        });

        Some(BlockFace {
            mesh: Mesh3d(mesh),
            material: MeshMaterial3d(ext_material),
            info: face,
        })
    };

    let lid = get_mesh("3_sided.lid");
    let right = get_mesh("3_sided.right");
    let top = get_mesh("3_sided.top");

    // FIXME: flat not working -> example are trees
    let (angle, left_face, top_face, right_face) = match diagonal_type {
        DiagonalType::UpRight => {
            position.x -= 0.25;
            position.y -= 0.25;
            (Some(0.5 * TAU), &voxel.right, &voxel.bottom, &voxel.left)
        }
        DiagonalType::UpLeft => {
            position.x += 0.25;
            position.y -= 0.25;
            (Some(0.75 * TAU), &voxel.left, &voxel.right, &voxel.bottom)
        }
        DiagonalType::DownLeft => {
            position.x += 0.25;
            position.y += 0.25;
            (None, &voxel.left, &voxel.top, &voxel.right)
        }
        DiagonalType::DownRight => {
            position.x -= 0.25;
            position.y += 0.25;
            (Some(0.25 * TAU), &voxel.right, &voxel.left, &voxel.top)
        }
    };

    BlockBuilder {
        lid: None,
        left: spawn_face_maybe(lid.clone(), FaceInfo(left_face.clone()), angle),
        right: spawn_face_maybe(right.clone(), FaceInfo(right_face.clone()), angle),
        top: spawn_face_maybe(top.clone(), FaceInfo(top_face.clone()), angle),
        bottom: None,
        left_right: Flatness::None,
        top_bottom: Flatness::None,
        position,
        rotation: angle,
    }
}

fn spawn_4_sided_diagonal_block(
    position: Vec3,
    diagonal_type: &DiagonalType,
    voxel: &BlockInfo,
    block_gltf: &Gltf,
    assets_gltfmesh: &Res<Assets<GltfMesh>>,
    textures: &Res<TextureIndex>,
    ext_materials: &mut ResMut<Assets<ExtendedMaterial<StandardMaterial, MyExtension>>>,
) -> BlockBuilder {
    let get_mesh = |name| {
        let handle = block_gltf.named_meshes[name].clone();
        &assets_gltfmesh.get(&handle).unwrap().primitives[0].mesh
    };

    let mut spawn_face_maybe = |mesh: Handle<Mesh>, face: FaceInfo, angle| -> Option<BlockFace> {
        if face.tile_id == 0 {
            return None;
        }

        let base_color_texture = textures.index.get(&face.tile_id).cloned();

        let rotation = if face.flip {
            face.rotate.clockwise_rad() - angle
        } else {
            face.rotate.clockwise_rad() + angle
        };

        let ext_material = ext_materials.add(ExtendedMaterial {
            base: StandardMaterial {
                base_color_texture,
                alpha_mode: AlphaMode::AlphaToCoverage,
                ..default()
            },
            extension: MyExtension::new(face.flip, rotation),
        });

        Some(BlockFace {
            mesh: Mesh3d(mesh),
            material: MeshMaterial3d(ext_material),
            info: face,
        })
    };

    let lid = get_mesh("4_sided.lid");
    let left = get_mesh("4_sided.left");
    let right = get_mesh("block.right");
    let top = get_mesh("block.top");

    let (angle, lid_face, left_face, top_face, right_face) = match diagonal_type {
        DiagonalType::UpRight => (
            -0.5 * TAU,
            &voxel.lid,
            &voxel.right,
            &voxel.bottom,
            &voxel.left,
        ),
        DiagonalType::UpLeft => (
            -0.25 * TAU,
            &voxel.lid,
            &voxel.left,
            &voxel.right,
            &voxel.bottom,
        ),
        DiagonalType::DownLeft => (0.0, &voxel.lid, &voxel.left, &voxel.top, &voxel.right),
        DiagonalType::DownRight => (
            0.25 * TAU,
            &voxel.lid,
            &voxel.right,
            &voxel.left,
            &voxel.top,
        ),
    };

    BlockBuilder {
        lid: spawn_face_maybe(lid.clone(), FaceInfo(lid_face.clone()), angle),
        left: spawn_face_maybe(left.clone(), FaceInfo(left_face.clone()), 0.0),
        right: spawn_face_maybe(right.clone(), FaceInfo(right_face.clone()), 0.0),
        top: spawn_face_maybe(top.clone(), FaceInfo(top_face.clone()), 0.0),
        bottom: None,
        left_right: Flatness::None,
        top_bottom: Flatness::None,
        position,
        rotation: Some(angle),
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Default, States)]
enum MapState {
    #[default]
    NotLoaded,
    SetupAssets,
    SetupMap,
    Loaded,
}

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

#[allow(dead_code)]
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
            Maps::Downtown => "wil",
            Maps::Residential => "ste",
            Maps::Industrial => "bil",
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

#[derive(Component, Debug, Clone)]
struct FaceInfo(file::Face);

impl std::ops::Deref for FaceInfo {
    type Target = file::Face;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

fn add_debug_observer(
    mut commands: Commands,
    blocks: Query<(Entity, Ref<block::Block>)>,
    faces: Query<(Entity, Ref<FaceInfo>)>,
) {
    for (entity, info) in &faces {
        if info.is_added() {
            commands.entity(entity).observe(on_click_show_debug);
        }
    }

    for (entity, block) in &blocks {
        if block.is_added() {
            commands.entity(entity).observe(on_click_show_pos);
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
    let Ok(face) = faces.get(click.entity()) else {
        return;
    };

    for mut span in &mut query {
        **span = format!("{:#?}", face);
    }
}

fn on_click_show_pos(
    click: Trigger<Pointer<Click>>,
    blocks: Query<&block::Block>,
    mut query: Query<&mut TextSpan, With<FaceDebugText>>,
) {
    let Ok(block) = blocks.get(click.entity()) else {
        return;
    };

    for mut span in &mut query {
        **span = format!("Pos: {:#?}\n", block.pos);
    }
}
