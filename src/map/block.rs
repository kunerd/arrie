pub mod face;

pub use face::Face;

use crate::map::{
    file::{self, BlockInfo},
    Flatness, MyExtension, TextureIndex,
};

use bevy::{
    asset::Assets,
    ecs::{
        component::Component,
        system::{Commands, Res, ResMut},
    },
    gltf::{Gltf, GltfMesh},
    hierarchy::{BuildChildren, ChildBuild},
    math::Vec3,
    pbr::{ExtendedMaterial, MeshMaterial3d, StandardMaterial},
    render::{alpha::AlphaMode, mesh::Mesh3d, view::Visibility},
    transform::components::Transform,
    utils::default,
};

#[derive(Component)]
struct Block {
    pos: Position,
}

#[derive(Debug, Clone, Copy)]
pub struct Position {
    pub x: u8,
    pub y: u8,
    pub z: u8,
}

#[derive(Component)]
struct Normal;

pub fn spawn_normal(
    pos: Position,
    voxel: &BlockInfo,
    block_gltf: &Gltf,
    assets_gltfmesh: &Res<Assets<GltfMesh>>,
    textures: &Res<TextureIndex>,
    ext_materials: &mut ResMut<Assets<ExtendedMaterial<StandardMaterial, MyExtension>>>,
    commands: &mut Commands,
) {
    let mut get_face = |face: &file::Face, name| {
        if face.tile_id == 0 {
            return None;
        }

        let handle = block_gltf
            .named_meshes
            .get(name)
            .unwrap_or_else(|| panic!("named mesh [{name}] to be found"))
            .clone();

        let mesh = &assets_gltfmesh
            .get(&handle)
            .unwrap_or_else(|| panic!("mesh [{name}] to exist"))
            .primitives[0]
            .mesh;

        let base_color_texture = textures
            .index
            .get(&face.tile_id)
            .unwrap_or_else(|| panic!("texture for tile_id: {} to be found", face.tile_id))
            .clone();

        // TODO could be optimized by re-using ext material with same properties
        let ext_material = ext_materials.add(ExtendedMaterial {
            base: StandardMaterial {
                base_color_texture: Some(base_color_texture),
                // NOTE: transperency is only allowed in flat faces
                alpha_mode: if face.flat {
                    AlphaMode::AlphaToCoverage
                } else {
                    AlphaMode::Opaque
                },
                ..default()
            },
            extension: MyExtension::new(face.flip, face.rotate.clockwise_rad()),
        });

        Some(Face {
            mesh: Mesh3d(mesh.clone()),
            material: MeshMaterial3d(ext_material),
        })
    };

    let lid = get_face(&voxel.lid, "block.lid");
    let left = get_face(&voxel.left, "block.left");
    let right = get_face(&voxel.right, "block.right");
    let top = get_face(&voxel.top, "block.top");
    let bottom = get_face(&voxel.bottom, "block.bottom");

    let transform = Transform::from_translation(Vec3::from(pos));

    let lr_flat = match (voxel.left.flat, voxel.right.flat) {
        (true, true) => Flatness::Both,
        (true, false) => Flatness::Left,
        (false, true) => Flatness::Right,
        (false, false) => Flatness::None,
    };

    let tb_flat = match (voxel.top.flat, voxel.right.flat) {
        (true, true) => Flatness::Both,
        (true, false) => Flatness::Left,
        (false, true) => Flatness::Right,
        (false, false) => Flatness::None,
    };

    commands
        .spawn((Block { pos }, Normal, transform, Visibility::Visible))
        .with_children(|parent| {
            lid.map(|face| parent.spawn((face::Lid, face)));

            match lr_flat {
                Flatness::None => {
                    left.map(|face| parent.spawn((face::Left, face)));
                    right.map(|face| parent.spawn((face::Right, face)));
                }
                Flatness::Left => {
                    left.map(|face| parent.spawn((face::Left, face)));
                    right.map(|face| {
                        parent.spawn((face::Right, face, Transform::from_xyz(-1.0, 0.0, 0.0)))
                    });
                }
                Flatness::Right => {
                    left.map(|face| {
                        parent.spawn((face::Left, face, Transform::from_xyz(1.0, 0.0, 0.0)))
                    });
                    right.map(|face| parent.spawn((face::Right, face)));
                }
                Flatness::Both => {
                    left.clone().map(|face| parent.spawn((face::Left, face)));
                    right.clone().map(|face| parent.spawn((face::Right, face)));
                    left.map(|face| {
                        parent.spawn((face::Left, face, Transform::from_xyz(1.0, 0.0, 0.0)))
                    });
                    right.map(|face| {
                        parent.spawn((face::Right, face, Transform::from_xyz(-1.0, 0.0, 0.0)))
                    });
                }
            }

            match tb_flat {
                Flatness::None => {
                    top.map(|face| parent.spawn((face::Top, face)));
                    bottom.map(|face| parent.spawn((face::Bottom, face)));
                }
                Flatness::Left => {
                    top.map(|face| parent.spawn((face::Top, face)));
                    bottom.map(|face| {
                        parent.spawn((face::Bottom, face, Transform::from_xyz(0.0, 1.0, 0.0)))
                    });
                }
                Flatness::Right => {
                    top.map(|face| {
                        parent.spawn((face::Top, face, Transform::from_xyz(0.0, -1.0, 0.0)))
                    });
                    bottom.map(|face| parent.spawn((face::Bottom, face)));
                }
                Flatness::Both => {
                    top.clone().map(|face| parent.spawn((face::Top, face)));
                    bottom
                        .clone()
                        .map(|face| parent.spawn((face::Bottom, face)));
                    top.map(|face| {
                        parent.spawn((face::Top, face, Transform::from_xyz(0.0, -1.0, 0.0)))
                    });
                    bottom.map(|face| {
                        parent.spawn((face::Bottom, face, Transform::from_xyz(0.0, 1.0, 0.0)))
                    });
                }
            }
        });
}

impl From<Position> for Vec3 {
    fn from(pos: Position) -> Self {
        Vec3 {
            x: f32::from(pos.x),
            y: f32::from(pos.y),
            z: f32::from(pos.z),
        }
    }
}
