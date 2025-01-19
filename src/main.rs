use gta2_viewer::{
    loader::{StyleFileAsset, StyleFileAssetLoader},
    map::{
        map_box::{BoxFaceBuilder, FaceType},
        Map, MapFileAsset, MapFileAssetLoader,
    },
    Style, StyleFile,
};

use bevy::{
    asset::{LoadState, RenderAssetUsages},
    prelude::*,
};
use wgpu::{TextureDimension, TextureFormat};

const IMAGE_SIZE: u32 = 64;
const MAP_PATH: &str = "data/bil.gmp";
const STYLE_PATH: &str = "data/bil.sty";

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .insert_resource(ClearColor(Color::BLACK))
        .init_asset::<MapFileAsset>()
        .init_asset_loader::<MapFileAssetLoader>()
        .init_asset::<StyleFileAsset>()
        .init_asset_loader::<StyleFileAssetLoader>()
        .add_systems(Startup, (setup_tiles, setup_map, setup_camera_and_light))
        .add_systems(Update, on_map_and_style_loaded)
        .run();
}

fn setup_tiles(mut commands: Commands, asset_server: Res<AssetServer>) {
    let asset = asset_server.load(STYLE_PATH);
    commands.spawn(Style { asset });
}

fn setup_map(mut commands: Commands, asset_server: Res<AssetServer>) {
    let asset = asset_server.load(MAP_PATH);
    commands.spawn(Map { asset });
}

fn on_map_and_style_loaded(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    map: Query<&Map>,
    style: Query<&Style>,
    maps: Res<Assets<MapFileAsset>>,
    styles: Res<Assets<StyleFileAsset>>,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (map, style) in map.iter().zip(style.iter()) {
        let map_state = asset_server.get_load_state(&map.asset);
        let style_state = asset_server.get_load_state(&style.asset);

        if let (Some(LoadState::Loaded), Some(LoadState::Loaded)) = (map_state, style_state) {
            let map = maps.get(&map.asset.clone()).unwrap();
            let style = styles.get(&style.asset.clone()).unwrap();

            fn is_valid_tile(id: u16) -> bool {
                id > 0 && id < 992
            };

            let first_cube = map
                .0
                .uncompressed_map
                .0
                .iter()
                .find(|i| is_valid_tile(i.lid) && is_valid_tile(i.left) && is_valid_tile(i.top) )
                .unwrap();
            dbg!(first_cube);

            let lid_image = create_image_asset(first_cube.lid as usize, &style.0, &mut images);
            let front = commands.spawn((
                Mesh3d(meshes.add(BoxFaceBuilder::new(1.0, FaceType::Front))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color_texture: Some(lid_image.clone()),
                    ..default()
                })),
            ));

            let back = commands.spawn((
                Mesh3d(meshes.add(BoxFaceBuilder::new(1.0, FaceType::Back))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color_texture: Some(lid_image.clone()),
                    ..default()
                })),
            ));

            let left_image = create_image_asset(first_cube.left as usize, &style.0, &mut images);
            let left = commands.spawn((
                Mesh3d(meshes.add(BoxFaceBuilder::new(1.0, FaceType::Left))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color_texture: Some(left_image),
                    ..default()
                })),
            ));

            let right_image = create_image_asset(first_cube.right as usize, &style.0, &mut images);
            let right = commands.spawn((
                Mesh3d(meshes.add(BoxFaceBuilder::new(1.0, FaceType::Right))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color_texture: Some(right_image),
                    ..default()
                })),
            ));

            let top_image = create_image_asset(first_cube.top as usize, &style.0, &mut images);
            let top = commands.spawn((
                Mesh3d(meshes.add(BoxFaceBuilder::new(1.0, FaceType::Top))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color_texture: Some(top_image),
                    ..default()
                })),
            ));
        }
    }
}

fn setup_camera_and_light(mut commands: Commands) {
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(-10.0, 8.0, 4.0),
    ));

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-3.0, 3.0, 3.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn create_image_asset(
    index: usize,
    style: &StyleFile,
    images: &mut ResMut<Assets<Image>>,
) -> Handle<Image> {
    let index = if index > 922 { 50 } else { index };

    let palette_index = style.palette_index.physical_index.get(index).unwrap();

    let phys_palette = style.physical_palette.get(*palette_index as usize).unwrap();

    let tile = style.tiles.get(index).unwrap();
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

    images.add(Image::new(
        size,
        TextureDimension::D2,
        tile,
        TextureFormat::Bgra8UnormSrgb,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    ))
}
