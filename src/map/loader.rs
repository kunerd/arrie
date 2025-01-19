use super::file;

use bevy::{
    asset::{io::Reader, AssetLoader, LoadContext},
    prelude::*,
    reflect::TypePath,
};

#[derive(Asset, TypePath)]
pub struct MapFileAsset(pub file::Map);

#[derive(Default)]
pub struct MapFileAssetLoader;

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum MapFileAssetLoaderError {
    /// An [IO](std::io) Error
    #[error("Could not load file: {0}")]
    Io(#[from] std::io::Error),
}

impl AssetLoader for MapFileAssetLoader {
    type Asset = MapFileAsset;
    type Settings = ();
    type Error = MapFileAssetLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;

        let map = file::Map::from_bytes(bytes);

        Ok(MapFileAsset(map))
    }
}
