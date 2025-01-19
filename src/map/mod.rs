pub mod file;
pub mod map_box;

mod loader;

use bevy::{asset::Handle, prelude::Component};
pub use loader::{MapFileAsset, MapFileAssetLoader, MapFileAssetLoaderError};

#[derive(Component)]
pub struct Map {
    pub asset: Handle<MapFileAsset>
}
