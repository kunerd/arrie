use std::thread::spawn;

use bevy::{
    pbr::{ExtendedMaterial, MaterialExtension, OpaqueRendererMethod},
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef, ShaderType},
};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(MaterialPlugin::<
            ExtendedMaterial<StandardMaterial, MyExtension>,
        >::default())
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
    mut ext_materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, MyExtension>>>,
) {
    let mesh: Handle<Mesh> = asset_server.load("gta2_block_model.glb#Mesh1/Primitive0");
    let texture = asset_server.load("uv_check.png");
    let material = StandardMaterial {
        //base_color: Color::srgb(1.0, 0.0, 0.0),
        base_color_texture: Some(texture),
        opaque_render_method: OpaqueRendererMethod::Auto,
        ..StandardMaterial::default()
    };
    let ext_material = ext_materials.add(ExtendedMaterial {
        base: material,
        extension: MyExtension {
            holder: MyExtensionHolder {
                flip: 1,
                angle: 0.25,
            },
        },
    });

    commands.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(ext_material),
        Transform::from_xyz(-1.0, -1.0, -1.0),
    ));
}

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
struct MyExtension {
    #[uniform(100)]
    holder: MyExtensionHolder,
}

#[derive(ShaderType, Reflect, Default, Clone, Debug)]
struct MyExtensionHolder {
    flip: u32,
    angle: f32,
}

const SHADER_ASSET_PATH: &str = "shaders/extended_material.wgsl";

impl MaterialExtension for MyExtension {
    fn fragment_shader() -> ShaderRef {
        SHADER_ASSET_PATH.into()
    }

    fn deferred_fragment_shader() -> ShaderRef {
        SHADER_ASSET_PATH.into()
    }
}
