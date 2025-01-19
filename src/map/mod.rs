pub mod file;

mod loader;
mod map_box;

use bevy::{asset::Handle, prelude::Component};
pub use loader::{MapFileAsset, MapFileAssetLoader, MapFileAssetLoaderError};

#[derive(Component)]
pub struct Map {
    pub asset: Handle<MapFileAsset>
}
