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
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
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
        .run();
}

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
    //mut next_state: ResMut<NextState<AppState>>,
) {
    let Some(map_file) = maps.get(&map.asset.clone()) else {
        return;
    };

    fn is_valid_tile(id: u16) -> bool {
        id > 0 && id < 992
    }

    let unknown_tile_color = materials.add(Color::srgba_u8(0, 255, 128, 255));

    // setup faces meshs
    let front = meshes.add(BoxFaceBuilder::new(1.0, FaceType::Front));
    let back = meshes.add(BoxFaceBuilder::new(1.0, FaceType::Back));
    let left = meshes.add(BoxFaceBuilder::new(1.0, FaceType::Left));
    let right = meshes.add(BoxFaceBuilder::new(1.0, FaceType::Right));
    let top = meshes.add(BoxFaceBuilder::new(1.0, FaceType::Top));
    //let bottom = meshes.add(BoxFaceBuilder::new(1.0, FaceType::Bottom));

    let voxel = map_file
        .0
        .uncompressed_map
        .0
        .iter()
        .find(|i| is_valid_tile(i.lid) && is_valid_tile(i.left) && is_valid_tile(i.top))
        .unwrap();

    // let lid_image = create_image_asset(first_cube.lid as usize, &style_file.0, &mut images);
    let _front = commands.spawn((
        Mesh3d(front.clone()),
        MeshMaterial3d(
            map_materials
                .index
                .get(&(voxel.lid as usize))
                .cloned()
                .unwrap_or(unknown_tile_color.clone())
        ),
    ));

    let _back = commands.spawn((
        Mesh3d(back.clone()),
        MeshMaterial3d(
            map_materials
                .index
                .get(&(voxel.lid as usize))
                .cloned()
                .unwrap_or(unknown_tile_color.clone())
        ),
    ));

    let _left = commands.spawn((
        Mesh3d(left.clone()),
        MeshMaterial3d(
            map_materials
                .index
                .get(&(voxel.left as usize))
                .cloned()
                .unwrap_or(unknown_tile_color.clone())
        ),
    ));

    let _right = commands.spawn((
        Mesh3d(right.clone()),
        MeshMaterial3d(
            map_materials
                .index
                .get(&(voxel.right as usize))
                .cloned()
                .unwrap_or(unknown_tile_color.clone())
        ),
    ));

    let _top = commands.spawn((
        Mesh3d(top.clone()),
        MeshMaterial3d(
            map_materials
                .index
                .get(&(voxel.top as usize))
                .cloned()
                .unwrap_or(unknown_tile_color.clone())
        ),
    ));
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

//#[derive(Debug, Component)]
//struct Voxel {
//}

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
