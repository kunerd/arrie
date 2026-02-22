pub mod face;

use std::f32::consts::TAU;

pub use face::Face;

use crate::map::{
    file::{self, BlockInfo, DiagonalType},
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
    math::{Quat, Vec3},
    pbr::{ExtendedMaterial, MeshMaterial3d, StandardMaterial},
    render::{alpha::AlphaMode, mesh::Mesh3d, view::Visibility},
    transform::components::Transform,
    utils::default,
};

#[derive(Component, Debug)]
pub(crate) struct Block {
    pub pos: Position,
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

    let (left, right, lr_flat) = match (voxel.left.flat, voxel.right.flat) {
        (true, true) => (voxel.left.clone(), voxel.right.clone(), Flatness::Both),
        (true, false) => {
            let mut right = voxel.right.clone();
            right.tile_id = voxel.left.tile_id;
            right.flat = voxel.left.flat;
            (voxel.left.clone(), right, Flatness::Left)
        }
        (false, true) => {
            let mut left = voxel.left.clone();
            left.tile_id = voxel.right.tile_id;
            left.flat = voxel.right.flat;
            (left, voxel.right.clone(), Flatness::Right)
        }
        (false, false) => (voxel.left.clone(), voxel.right.clone(), Flatness::None),
    };

    let left = get_face(&left, "block.left");
    let right = get_face(&right, "block.right");

    let (top, bottom, tb_flat) = match (voxel.top.flat, voxel.bottom.flat) {
        (true, true) => (voxel.top.clone(), voxel.bottom.clone(), Flatness::Both),
        (true, false) => {
            let mut bottom = voxel.bottom.clone();
            bottom.tile_id = voxel.top.tile_id;
            bottom.flat = voxel.top.flat;
            (voxel.top.clone(), bottom, Flatness::Left)
        }
        (false, true) => {
            let mut top = voxel.top.clone();
            top.tile_id = voxel.bottom.tile_id;
            top.flat = voxel.bottom.flat;
            (top, voxel.bottom.clone(), Flatness::Right)
        }
        (false, false) => (voxel.top.clone(), voxel.bottom.clone(), Flatness::None),
    };

    let top = get_face(&top, "block.top");
    let bottom = get_face(&bottom, "block.bottom");

    let transform = Transform::from_translation(Vec3::from(pos));
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
                    right.map(|face| parent.spawn((face::Right, face)));
                    left.map(|face| {
                        parent.spawn((face::Left, face, Transform::from_xyz(1.0, 0.0, 0.0)))
                    });
                }
                Flatness::Both => {
                    left.clone().map(|face| parent.spawn((face::Left, face)));
                    right.clone().map(|face| {
                        parent.spawn((face::Right, face, Transform::from_xyz(-1.0, 0.0, 0.0)))
                    });
                    right.map(|face| parent.spawn((face::Right, face)));
                    left.map(|face| {
                        parent.spawn((face::Left, face, Transform::from_xyz(1.0, 0.0, 0.0)))
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
                    bottom.clone().map(|face| {
                        parent.spawn((face::Bottom, face, Transform::from_xyz(0.0, 1.0, 0.0)))
                    });
                    top.map(|face| {
                        parent.spawn((face::Top, face, Transform::from_xyz(0.0, -1.0, 0.0)))
                    });
                    bottom.map(|face| parent.spawn((face::Bottom, face)));
                }
            }
        });
}

pub fn spawn_diagonal(
    pos: Position,
    voxel: &BlockInfo,
    diagonal_type: &DiagonalType,
    block_gltf: &Gltf,
    assets_gltfmesh: &Res<Assets<GltfMesh>>,
    textures: &Res<TextureIndex>,
    ext_materials: &mut ResMut<Assets<ExtendedMaterial<StandardMaterial, MyExtension>>>,
    commands: &mut Commands,
) {
    let mut get_face = |face: &file::Face, name, angle| {
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

        let base_color_texture = textures.index.get(&face.tile_id).cloned();

        if base_color_texture.is_none() {
            dbg!(format!("texture for tile_id: {} to be found", face.tile_id));
        }

        let rotation = if face.flip {
            face.rotate.clockwise_rad() - angle
        } else {
            face.rotate.clockwise_rad() + angle
        };

        // TODO could be optimized by re-using ext material with same properties
        let ext_material = ext_materials.add(ExtendedMaterial {
            base: StandardMaterial {
                base_color_texture,
                // NOTE: transperency is only allowed in flat faces
                alpha_mode: if face.flat {
                    AlphaMode::AlphaToCoverage
                } else {
                    AlphaMode::Opaque
                },
                ..default()
            },
            extension: MyExtension::new(face.flip, rotation),
        });

        Some(Face {
            mesh: Mesh3d(mesh.clone()),
            material: MeshMaterial3d(ext_material),
        })
    };

    let (angle, diagonal, right, top) = match diagonal_type {
        DiagonalType::DownLeft => (0.0, &voxel.left, &voxel.right, &voxel.top),
        DiagonalType::DownRight => (0.25 * TAU, &voxel.right, &voxel.top, &voxel.left),
        DiagonalType::UpRight => (0.5 * TAU, &voxel.right, &voxel.left, &voxel.bottom),
        DiagonalType::UpLeft => (0.75 * TAU, &voxel.left, &voxel.bottom, &voxel.right),
    };

    // NOTE: angle is used to compensate the rotation of the lid's UV map which
    // occurs when rotating the whole blog
    let lid = get_face(&voxel.lid, "diagonal.lid", angle);
    let diagonal = get_face(diagonal, "diagonal.front", 0.0);
    let right = get_face(right, "block.right", 0.0);
    let top = get_face(top, "block.top", 0.0);

    let transform =
        Transform::from_translation(Vec3::from(pos)).with_rotation(Quat::from_rotation_z(angle));
    commands
        .spawn((Block { pos }, Normal, transform, Visibility::Visible))
        .with_children(|parent| {
            // NOTE: diagonals can not be flat
            lid.map(|face| parent.spawn((face::Lid, face)));
            diagonal.map(|face| parent.spawn(face));
            right.map(|face| parent.spawn(face));
            top.map(|face| parent.spawn(face));
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
