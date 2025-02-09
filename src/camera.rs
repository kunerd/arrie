use bevy::prelude::*;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};

pub fn plugin(app: &mut App) {
    app.add_plugins(PanOrbitCameraPlugin)
        .add_systems(Startup, setup_camera);
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        PanOrbitCamera::default(),
        Transform::from_xyz(10.0, 0.0, 18.0).looking_at(
            Vec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            Vec3::Y,
        ),
    ));
}
