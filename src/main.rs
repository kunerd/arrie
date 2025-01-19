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
        .insert_resource(ClearColor(Color::rgb(0.3, 0.3, 0.3)))
        .add_systems(Startup, (setup_tiles, load_map, setup))
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
    map::Map::from_file(MAP_PATH);
}

fn change_colors(mut query: Query<&mut Sprite>) {
    for mut sprite in query.iter_mut() {
        // your color changing logic here instead:
        sprite.color.set_alpha(0.5);
    }
}

fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let file = File::open(DATA_PATH).unwrap();
    let style = StyleFile::from_file(&file);

    let index = 50;

    let tile = style.tiles.get(index).unwrap();

    let palette_index = style.palette_index.physical_index.get(index).unwrap();
    dbg!(palette_index);
    let phys_palette = style.physical_palette.get(*palette_index as usize).unwrap();

    let colored_image_1: Vec<u8> = tile
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

    let index = 1;

    let tile = style.tiles.get(index).unwrap();

    let palette_index = style.palette_index.physical_index.get(index).unwrap();
    dbg!(palette_index);
    let phys_palette = style.physical_palette.get(*palette_index as usize).unwrap();

    let colored_image_2: Vec<u8> = tile
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

    let image = images.add(Image::new(
        size,
        TextureDimension::D2,
        colored_image_1,
        //TextureFormat::Bgra8Unorm,
        TextureFormat::Bgra8UnormSrgb,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    ));

    let front = commands.spawn((
        Mesh3d(meshes.add(BoxFaceBuilder::new(1.0, FaceType::Front))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(image.clone()),
            ..default()
        })),
    ));

    let back = commands.spawn((
        Back,
        Mesh3d(meshes.add(BoxFaceBuilder::new(1.0, FaceType::Back))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(image.clone()),
            ..default()
        })),
    ));

    let left = commands.spawn((
        Left,
        Mesh3d(meshes.add(BoxFaceBuilder::new(1.0, FaceType::Left))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(image.clone()),
            ..default()
        })),
    ));

    let right = commands.spawn((
        Right,
        Mesh3d(meshes.add(BoxFaceBuilder::new(1.0, FaceType::Right))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(image.clone()),
            ..default()
        })),
    ));

    let top = commands.spawn((
        Top,
        Mesh3d(meshes.add(BoxFaceBuilder::new(1.0, FaceType::Top))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(image.clone()),
            ..default()
        })),
    ));

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

struct TilesIter<I> {
    iter: I,
}

impl<I> TilesIter<I> {
    pub fn new(iter: I) -> Self {
        TilesIter { iter }
    }
}

impl<I: Iterator<Item = String>> Iterator for TilesIter<I> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}
