use gta2_viewer::{
    loader::{StyleFileAsset, StyleFileAssetLoader},
    map::{Map, MapFileAsset, MapFileAssetLoader},
    Style
};

use bevy::{asset::RenderAssetUsages, prelude::*};
use wgpu::{TextureDimension, TextureFormat};

use std::fs::File;

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
        .add_systems(Startup, (setup_tiles, setup_map, setup_camera))
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

fn setup_camera(
    mut commands: Commands,
    //mut meshes: ResMut<Assets<Mesh>>,
    //mut materials: ResMut<Assets<StandardMaterial>>,
) {
    //let file = File::open(DATA_PATH).unwrap();
    //let style = StyleFile::from_file(&file);
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(-10.0, 8.0, 4.0),
    ));
    // camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-3.0, 3.0, 3.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn create_image(mut images: ResMut<Assets<Image>>) {
    //let size = wgpu::Extent3d {
    //    width: IMAGE_SIZE,
    //    height: IMAGE_SIZE,
    //    depth_or_array_layers: 1,
    //};

    //let image = images.add(Image::new(
    //    size,
    //    TextureDimension::D2,
    //    colored_image_1,
    //    //TextureFormat::Bgra8Unorm,
    //    TextureFormat::Bgra8UnormSrgb,
    //    RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    //));
}
