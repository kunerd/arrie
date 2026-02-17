use bevy::prelude::*;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};

pub fn plugin(app: &mut App) {
    app.add_plugins(PanOrbitCameraPlugin)
        .add_systems(Startup, setup_camera);
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        PanOrbitCamera {
            focus: Vec3 {
                x: 128.0,
                y: 128.0,
                z: 0.0,
            },
            ..Default::default()
        },
        Transform::from_xyz(128.0, 128.0, 18.0),
    ));
}
