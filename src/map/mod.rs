pub mod file;
pub mod map_box;

mod loader;

use bevy::{
    asset::{Handle, RenderAssetUsages},
    gltf::GltfMesh,
    prelude::*,
    utils::HashMap,
};
use file::{DiagonalType, Rotate, SlopeType};
pub use loader::{MapFileAsset, MapFileAssetLoader, MapFileAssetLoaderError};
use wgpu::{TextureDimension, TextureFormat};

use std::{f32::consts::TAU, fmt::Display, path::PathBuf, str::FromStr};

use crate::loader::{StyleFileAsset, StyleFileAssetLoader};

pub fn plugin(app: &mut App) {
    let game_files_path = check_and_get_game_files_path();
    app.insert_resource(game_files_path)
        .insert_resource(CurrentMap(Maps::Downtown))
        .init_asset::<MapFileAsset>()
        .init_asset_loader::<MapFileAssetLoader>()
        .init_asset::<StyleFileAsset>()
        .init_asset_loader::<StyleFileAssetLoader>()
        .insert_state(MapState::NotLoaded)
        .add_systems(OnEnter(MapState::NotLoaded), load_map_resources)
        .add_systems(Update, setup_assets.run_if(in_state(MapState::SetupAssets)))
        .add_systems(Update, setup_map.run_if(in_state(MapState::SetupMap)));
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

fn setup_assets(
    style: Res<Style>,
    style_asset: Res<Assets<StyleFileAsset>>,
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut next_state: ResMut<NextState<MapState>>,
) {
    let Some(style_file) = style_asset.get(&style.asset.clone()) else {
        return;
    };

    let mut map_materials = MapMaterialIndex::default();

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

        let material_handler = materials.add(StandardMaterial {
            base_color_texture: Some(image_handle.clone()),
            alpha_mode: AlphaMode::AlphaToCoverage,
            ..default()
        });

        map_materials.index.insert(id, material_handler);
    }

    commands.insert_resource(map_materials);
    next_state.set(MapState::SetupMap);
}

#[derive(Resource)]
struct BlockMesh(Handle<Gltf>);

fn setup_map(
    map: Res<Map>,
    map_asset: Res<Assets<MapFileAsset>>,
    block_mesh_res: Res<BlockMesh>,
    map_materials: Res<MapMaterialIndex>,
    assets_gltf: Res<Assets<Gltf>>,
    assets_gltfmesh: Res<Assets<GltfMesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<MapState>>,
) {
    let Some(map_file) = map_asset.get(&map.asset.clone()) else {
        return;
    };

    let Some(block_gltf) = assets_gltf.get(&block_mesh_res.0) else {
        return;
    };

    //let marker_color = materials.add(Color::srgb(1.0, 0.0, 0.0));
    let unknown_tile_color = materials.add(Color::srgba_u8(0, 255, 128, 255));

    let get_mesh = |name| {
        let handle = block_gltf.named_meshes[name].clone();
        &assets_gltfmesh.get(&handle).unwrap().primitives[0].mesh
    };
    // setup faces meshs
    let front = get_mesh("block.lid");
    let front_fliped = get_mesh("block.lid.flip");
    //let front = meshes.add(BoxFaceBuilder::new(1.0, FaceType::Front));
    //let front_fliped = meshes.add(BoxFaceBuilder::new(1.0, FaceType::Front).set_flip(true));

    //let left = meshes.add(BoxFaceBuilder::new(1.0, FaceType::Left));
    //let left_fliped = meshes.add(BoxFaceBuilder::new(1.0, FaceType::Left).set_flip(true));
    let left = get_mesh("block.left");
    let left_fliped = get_mesh("block.left.flip");

    //let right = meshes.add(BoxFaceBuilder::new(1.0, FaceType::Right));
    //let right_fliped = meshes.add(BoxFaceBuilder::new(1.0, FaceType::Right).set_flip(true));
    let right = get_mesh("block.right");
    let right_fliped = get_mesh("block.right.flip");

    //let top = meshes.add(BoxFaceBuilder::new(1.0, FaceType::Top));
    //let top_fliped = meshes.add(BoxFaceBuilder::new(1.0, FaceType::Top).set_flip(true));
    let top = get_mesh("block.top");
    let top_fliped = get_mesh("block.top.flip");

    //let bottom = meshes.add(BoxFaceBuilder::new(1.0, FaceType::Bottom));
    //let bottom_fliped = meshes.add(BoxFaceBuilder::new(1.0, FaceType::Bottom).set_flip(true));
    let bottom = get_mesh("block.bottom");
    let bottom_fliped = get_mesh("block.bottom.flip");

    const X_MAX: usize = 256;
    const Y_MAX: usize = 256;

    dbg!(map_file.0.uncompressed_map.as_ref().unwrap().0.len());

    for (i, voxel) in map_file
        .0
        .uncompressed_map
        .as_ref()
        .unwrap()
        .0
        .iter()
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

        let face = &voxel.lid;
        if face.tile_id != 0 {
            match &voxel.slope_type {
                SlopeType::Diagonal(d_type) => {
                    let lid_mesh = block_gltf.named_meshes["diagonal.lid"].clone();
                    let lid_mesh = assets_gltfmesh.get(&lid_mesh).unwrap();

                    let angle = match d_type {
                        DiagonalType::UpLeft => 0.0,
                        DiagonalType::UpRight => -0.25 * TAU,
                        DiagonalType::DownLeft => 0.25 * TAU,
                        DiagonalType::DownRight => -0.5 * TAU,
                    };

                    commands.spawn((
                        Mesh3d(lid_mesh.primitives[0].mesh.clone()),
                        MeshMaterial3d(
                            //marker_color.clone(),
                            map_materials
                                .index
                                .get(&(face.tile_id))
                                .cloned()
                                .unwrap_or(unknown_tile_color.clone()),
                        ),
                        Transform::from_translation(pos)
                            .with_rotation(Quat::from_rotation_z(angle)),
                    ));
                }
                SlopeType::Ignore => {
                    let mesh = if face.flip {
                        front_fliped.clone()
                    } else {
                        front.clone()
                    };

                    commands.spawn((
                        Mesh3d(mesh),
                        MeshMaterial3d(
                            map_materials
                                .index
                                .get(&(face.tile_id))
                                .cloned()
                                .unwrap_or(unknown_tile_color.clone()),
                        ),
                        Transform::from_translation(pos).with_rotation(Quat::from_rotation_z(
                            compute_rotation(face.rotate, face.flip),
                        )),
                        MapPos(i),
                    ));
                    //.observe(on_click_show_debug);
                }
            };
        }

        let face = &voxel.left;
        if face.tile_id != 0 {
            let mesh = if face.flip {
                left_fliped.clone()
            } else {
                left.clone()
            };

            let pos = if voxel.right.flat {
                pos.with_x(pos.x + 1.0)
            } else {
                pos
            };

            commands.spawn((
                Mesh3d(mesh),
                MeshMaterial3d(
                    map_materials
                        .index
                        .get(&(face.tile_id))
                        .cloned()
                        .unwrap_or(unknown_tile_color.clone()),
                ),
                Transform::from_translation(pos).with_rotation(Quat::from_rotation_x(
                    compute_rotation(face.rotate, face.flip),
                )),
                MapPos(i),
            ));
            //.observe(on_click_show_debug);
        }

        let face = &voxel.right;
        if face.tile_id != 0 {
            let mesh = if face.flip {
                right_fliped.clone()
            } else {
                right.clone()
            };

            commands.spawn((
                Mesh3d(mesh.clone()),
                MeshMaterial3d(
                    map_materials
                        .index
                        .get(&(face.tile_id))
                        .cloned()
                        .unwrap_or(unknown_tile_color.clone()),
                ),
                Transform::from_translation(pos).with_rotation(Quat::from_rotation_x(
                    compute_rotation(face.rotate, face.flip),
                )),
                MapPos(i),
            ));
            //.observe(on_click_show_debug);
        }

        let face = &voxel.top;
        if face.tile_id != 0 {
            let mesh = if face.flip {
                top_fliped.clone()
            } else {
                top.clone()
            };

            commands.spawn((
                Mesh3d(mesh),
                MeshMaterial3d(
                    map_materials
                        .index
                        .get(&(face.tile_id))
                        .cloned()
                        .unwrap_or(unknown_tile_color.clone()),
                ),
                Transform::from_translation(pos).with_rotation(Quat::from_rotation_y(
                    compute_rotation(face.rotate, face.flip),
                )),
                MapPos(i),
            ));
            //.observe(on_click_show_debug);
        }

        let face = &voxel.bottom;
        if face.tile_id != 0 {
            let mesh = if face.flip {
                bottom_fliped.clone()
            } else {
                bottom.clone()
            };

            commands.spawn((
                Mesh3d(mesh),
                MeshMaterial3d(
                    map_materials
                        .index
                        .get(&(face.tile_id))
                        .cloned()
                        .unwrap_or(unknown_tile_color.clone()),
                ),
                Transform::from_translation(pos).with_rotation(Quat::from_rotation_y(
                    compute_rotation(face.rotate, face.flip),
                )),
                MapPos(i),
            ));
            //.observe(on_click_show_debug);
        }
    }

    next_state.set(MapState::Loaded)
}

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
pub struct MapMaterialIndex {
    pub index: HashMap<usize, Handle<StandardMaterial>>,
}

#[derive(Resource, Debug)]
struct GameFilesPath(PathBuf);

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

    GameFilesPath(game_files_path)
}

//fn spawn_face_debug_text(mut commands: Commands) {
//    commands
//        .spawn((
//            Text::new("Block info"),
//            TextFont {
//                font_size: 12.0,
//                ..default()
//            },
//        ))
//        .with_child((
//            TextSpan::default(),
//            TextFont {
//                font_size: 12.0,
//                ..default()
//            },
//            TextColor(GOLD.into()),
//            FaceDebugText,
//        ));
//}
//
///// A system that draws hit indicators for every pointer.
//fn draw_mesh_intersections(pointers: Query<&PointerInteraction>, mut gizmos: Gizmos) {
//    for (point, normal) in pointers
//        .iter()
//        .filter_map(|interaction| interaction.get_nearest_hit())
//        .filter_map(|(_entity, hit)| hit.position.zip(hit.normal))
//    {
//        gizmos.sphere(point, 0.05, RED_500);
//        gizmos.arrow(point, point + normal.normalize() * 0.5, PINK_100);
//    }
//}
//
//fn on_click_show_debug(
//    click: Trigger<Pointer<Click>>,
//    map: Res<Map>,
//    maps: Res<Assets<MapFileAsset>>,
//    positions: Query<&MapPos>,
//    mut query: Query<&mut TextSpan, With<FaceDebugText>>,
//) {
//    let Ok(pos) = positions.get(click.entity()) else {
//        return;
//    };
//
//    let Some(map_file) = maps.get(&map.asset.clone()) else {
//        return;
//    };
//
//    let Some(map_info) = map_file.0.uncompressed_map.as_ref().unwrap().0.get(pos.0) else {
//        return;
//    };
//
//    for mut span in &mut query {
//        **span = format!("map_pos: {}\n{:#?}", pos.0, map_info);
//    }
//}
