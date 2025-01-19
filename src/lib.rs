extern crate byteorder;

pub mod map;
mod style;

use bevy::{asset::Handle, prelude::Component};
pub use style::{loader, StyleFile, Tile};

#[derive(Component)]
pub struct Style {
    pub asset: Handle<loader::StyleFileAsset>
}

