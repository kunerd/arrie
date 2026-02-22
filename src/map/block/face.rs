use bevy::{
    ecs::{bundle::Bundle, component::Component},
    pbr::{ExtendedMaterial, MeshMaterial3d, StandardMaterial},
    render::mesh::Mesh3d,
};

use crate::map::MyExtension;

#[derive(Component)]
// #[require(Face)]
pub struct Lid;

#[derive(Component)]
// #[require(Face)]
pub struct Left;

#[derive(Component)]
// #[require(Face)]
pub struct Right;

#[derive(Component)]
// #[require(Face)]
pub struct Top;

#[derive(Component)]
// #[require(Face)]
pub struct Bottom;

#[derive(Bundle, Default, Clone)]
pub struct Face {
    pub mesh: Mesh3d,
    pub material: MeshMaterial3d<ExtendedMaterial<StandardMaterial, MyExtension>>,
}
