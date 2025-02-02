use std::thread::spawn;

use bevy::prelude::*;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(PanOrbitCameraPlugin)
        .add_systems(Startup, (setup_camera, load_scene))
        .run();
}

#[derive(Component)]
struct ThreeSidedDiagonal {
    right: Handle<Mesh>,
    back: Handle<Mesh>,
    top: Handle<Mesh>,
    bottom: Handle<Mesh>,
}

fn setup_camera(mut commands: Commands) {
    // Camera
    commands.spawn((
        Transform::from_translation(Vec3::new(0.0, 0.0, 3.0)),
        PanOrbitCamera::default(),
    ));
}

fn load_scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh: Handle<Mesh> = asset_server.load("3-side-diagonal.glb#Mesh0");
    let material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.0, 0.0),
        ..StandardMaterial::default()
    });

    //commands.spawn((
    //    Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
    //    MeshMaterial3d(material.clone()),
    //    Transform::from_xyz(1.0, 1.0, 1.0),
    //));
    //let my_gltf = asset_server.load("3-side-diagonal.glb#Scene0");
    //commands.spawn(SceneBundle {
    //    scene: SceneRoot(my_gltf),
    //    transform: Transform::from_xyz(2.0, 0.0, -5.0),
    //    ..Default::default()
    //});

    commands.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(material),
        Transform::from_xyz(-1.0, -1.0, -1.0),
    ));
}
