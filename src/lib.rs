extern crate byteorder;

pub mod map;
mod style;

use bevy::{asset::Handle, pbr::StandardMaterial, prelude::Resource, utils::HashMap};
pub use style::{loader, StyleFile, Tile};

#[derive(Resource, Debug)]
pub struct Style {
    pub asset: Handle<loader::StyleFileAsset>,
}

#[derive(Resource, Debug, Default)]
pub struct MapMaterialIndex {
    pub index: HashMap<usize, Handle<StandardMaterial>>
}

