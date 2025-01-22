use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use gta2_viewer::{
    loader::{StyleFileAsset, StyleFileAssetLoader},
    map::{
        map_box::{BoxFaceBuilder, FaceType},
        Map, MapFileAsset, MapFileAssetLoader,
    },
    MapMaterialIndex, Style,
};

use bevy::{asset::RenderAssetUsages, prelude::*};
use wgpu::{TextureDimension, TextureFormat};

const IMAGE_SIZE: u32 = 64;
const MAP_PATH: &str = "data/bil.gmp";
const STYLE_PATH: &str = "data/bil.sty";

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum AppState {
    #[default]
    SetupTilesIndex,
    SetupMap,
    Wait,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(PanOrbitCameraPlugin)
        .insert_state(AppState::SetupTilesIndex)
        .insert_resource(ClearColor(Color::BLACK))
        .init_asset::<MapFileAsset>()
        .init_asset_loader::<MapFileAssetLoader>()
        .init_asset::<StyleFileAsset>()
        .init_asset_loader::<StyleFileAssetLoader>()
        .add_systems(
            Startup,
            (load_style_file, load_map_file, setup_camera_and_light),
        )
        .add_systems(
            Update,
            setup_assets.run_if(in_state(AppState::SetupTilesIndex)),
        )
        .add_systems(Update, setup_map.run_if(in_state(AppState::SetupMap)))
        .add_systems(Update, nop.run_if(in_state(AppState::Wait)))
        .run();
}

fn nop() {}

fn load_style_file(mut commands: Commands, asset_server: Res<AssetServer>) {
    let asset = asset_server.load(STYLE_PATH);
    commands.insert_resource(Style { asset });
}

fn load_map_file(mut commands: Commands, asset_server: Res<AssetServer>) {
    let asset = asset_server.load(MAP_PATH);
    commands.insert_resource(Map { asset });
}

fn setup_assets(
    style: Res<Style>,
    styles: Res<Assets<StyleFileAsset>>,
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let Some(style_file) = styles.get(&style.asset.clone()) else {
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
            ..default()
        });

        map_materials.index.insert(id, material_handler);
    }

    commands.insert_resource(map_materials);
    next_state.set(AppState::SetupMap);
}

fn setup_map(
    map: Res<Map>,
    map_materials: Res<MapMaterialIndex>,
    maps: Res<Assets<MapFileAsset>>,
    //styles: Res<Assets<StyleFileAsset>>,
    //asset_server: Res<AssetServer>,
    //mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let Some(map_file) = maps.get(&map.asset.clone()) else {
        return;
    };

    let unknown_tile_color = materials.add(Color::srgba_u8(0, 255, 128, 255));

    // setup faces meshs
    let front = meshes.add(BoxFaceBuilder::new(1.0, FaceType::Front));
    let back = meshes.add(BoxFaceBuilder::new(1.0, FaceType::Back));
    let left = meshes.add(BoxFaceBuilder::new(1.0, FaceType::Left));
    let right = meshes.add(BoxFaceBuilder::new(1.0, FaceType::Right));
    let top = meshes.add(BoxFaceBuilder::new(1.0, FaceType::Top));
    //let bottom = meshes.add(BoxFaceBuilder::new(1.0, FaceType::Bottom));

    const X_MAX: usize = 256;
    const Y_MAX: usize = 256;

    for (i, voxel) in map_file.0.uncompressed_map.0.iter().enumerate() {
        //const Z_MAX: usize = 8;

        let x = i % X_MAX;
        let y = (i / X_MAX) % Y_MAX;
        let z = i / (X_MAX * Y_MAX);

        if voxel.lid.tile_id != 0 {
            let _front = commands.spawn((
                Mesh3d(front.clone()),
                MeshMaterial3d(
                    map_materials
                        .index
                        .get(&(voxel.lid.tile_id))
                        .cloned()
                        .unwrap_or(unknown_tile_color.clone()),
                ),
                Transform::from_xyz(x as f32, y as f32, z as f32),
            ));

            let _back = commands.spawn((
                Mesh3d(back.clone()),
                MeshMaterial3d(
                    map_materials
                        .index
                        .get(&(voxel.lid.tile_id))
                        .cloned()
                        .unwrap_or(unknown_tile_color.clone()),
                ),
                Transform::from_xyz(x as f32, y as f32, z as f32),
            ));
        }

        if voxel.left.tile_id != 0 {
            let _left = commands.spawn((
                Mesh3d(left.clone()),
                MeshMaterial3d(
                    map_materials
                        .index
                        .get(&(voxel.left.tile_id))
                        .cloned()
                        .unwrap_or(unknown_tile_color.clone()),
                ),
                Transform::from_xyz(x as f32, y as f32, z as f32),
            ));
        }

        if voxel.right.tile_id != 0 {
            let _right = commands.spawn((
                Mesh3d(right.clone()),
                MeshMaterial3d(
                    map_materials
                        .index
                        .get(&(voxel.right.tile_id))
                        .cloned()
                        .unwrap_or(unknown_tile_color.clone()),
                ),
                Transform::from_xyz(x as f32, y as f32, z as f32),
            ));
        }

        if voxel.top.tile_id != 0 {
            let _top = commands.spawn((
                Mesh3d(top.clone()),
                MeshMaterial3d(
                    map_materials
                        .index
                        .get(&(voxel.top.tile_id))
                        .cloned()
                        .unwrap_or(unknown_tile_color.clone()),
                ),
                Transform::from_xyz(x as f32, y as f32, z as f32),
            ));
        }
    }

    next_state.set(AppState::Wait)
}

fn setup_camera_and_light(mut commands: Commands) {
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 30.0),
    ));

    commands.spawn((
        PanOrbitCamera::default(),
        Transform::from_xyz(128., 128., 25.0).looking_at(
            Vec3 {
                x: 128.0,
                y: 156.0,
                z: 0.0,
            },
            Vec3::Y,
        ),
    ));
    //commands.spawn((
    //    Camera3d::default(),
    //    Transform::from_xyz(128., 128., 25.0).looking_at(
    //        Vec3 {
    //            x: 128.0,
    //            y: 156.0,
    //            z: 0.0,
    //        },
    //        Vec3::Z,
    //    ),
    //));
}

#[derive(Debug, Component)]
struct Voxel {}

//fn create_image_asset(
//    index: usize,
//    style: &StyleFile,
//    images: &mut ResMut<Assets<Image>>,
//) -> Handle<Image> {
//    let index = if index > 922 { 50 } else { index };
//
//    let palette_index = style.palette_index.physical_index.get(index).unwrap();
//    let phys_palette = style.physical_palette.get(*palette_index as usize).unwrap();
//    let tile = style.tiles.get(index).unwrap();
//    let tile = tile
//        .0
//        .iter()
//        .map(|p| *phys_palette.colors.get(*p as usize).unwrap())
//        .flat_map(|c| {
//            let c = c.to_ne_bytes();
//            if c == [0, 0, 0, 0] {
//                [0, 0, 0, 0]
//            } else {
//                [c[0], c[1], c[2], 255]
//            }
//        })
//        .collect();
//
//    let size = wgpu::Extent3d {
//        width: IMAGE_SIZE,
//        height: IMAGE_SIZE,
//        depth_or_array_layers: 1,
//    };
//
//    images.add(Image::new(
//        size,
//        TextureDimension::D2,
//        tile,
//        TextureFormat::Bgra8UnormSrgb,
//        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
//    ))
//}
