use gta2_viewer::map;
use gta2_viewer::StyleFile;

use bevy::{asset::RenderAssetUsages, prelude::*, render::mesh::Indices};
use wgpu::{PrimitiveTopology, TextureDimension, TextureFormat};

use std::fs::File;

const IMAGE_SIZE: u32 = 64;
const MAP_PATH: &str = "data/bil.gmp";
const DATA_PATH: &str = "data/bil.sty";

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, (setup_tiles, load_map, setup_camera))
        .run();
}

fn setup_tiles(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let file = File::open(DATA_PATH).unwrap();
    let style = StyleFile::from_file(&file);

    let index = 716;

    let tile = style.tiles.get(index).unwrap();

    let palette_index = style.palette_index.physical_index.get(index).unwrap();
    dbg!(palette_index);
    let phys_palette = style.physical_palette.get(*palette_index as usize).unwrap();

    let colored_image: Vec<u8> = tile
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
    //BGRA
    //0123
    //ARGB
    //3210
    //ABGR
    //RGBA

    let size = wgpu::Extent3d {
        width: IMAGE_SIZE,
        height: IMAGE_SIZE,
        depth_or_array_layers: 1,
    };

    let image = images.add(Image::new(
        size,
        TextureDimension::D2,
        colored_image,
        //TextureFormat::Bgra8Unorm,
        TextureFormat::Bgra8UnormSrgb,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    ));

    //commands.spawn(Camera2d);

    let sprite = Sprite {
        image,
        custom_size: Some(Vec2::new(256.0, 256.0)),
        ..Default::default()
    };

    commands.spawn(sprite);
}

fn load_map() {
    map::file::Map::from_file(MAP_PATH);
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
