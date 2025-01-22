pub mod file;
pub mod map_box;

mod loader;

use bevy::{asset::Handle, prelude::Resource};
pub use loader::{MapFileAsset, MapFileAssetLoader, MapFileAssetLoaderError};

#[derive(Resource, Debug)]
pub struct Map {
    pub asset: Handle<MapFileAsset>
}
