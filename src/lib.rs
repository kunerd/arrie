extern crate byteorder;

mod camera;
mod dev_tools;
pub mod map;
mod style;
mod window;

pub use style::{loader, StyleFile, Tile};

use bevy::{prelude::*, utils::HashMap};

#[derive(Resource, Debug)]
pub struct Style {
    pub asset: Handle<loader::StyleFileAsset>,
}

#[derive(Resource, Debug, Default)]
pub struct MapMaterialIndex {
    pub index: HashMap<usize, Handle<StandardMaterial>>,
}

pub struct Arrie;

impl Plugin for Arrie {
    fn build(&self, app: &mut App) {
        app.add_plugins((window::plugin, camera::plugin, map::plugin));

        #[cfg(feature = "dev")]
        app.add_plugins(dev_tools::plugin);
    }
}
