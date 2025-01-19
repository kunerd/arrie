use super::StyleFile;

use bevy::{
    asset::{io::Reader, AssetLoader, LoadContext},
    prelude::*,
    reflect::TypePath,
};

#[derive(Asset, TypePath)]
pub struct StyleFileAsset(pub StyleFile);

#[derive(Default)]
pub struct StyleFileAssetLoader;

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum StyleFileAssetLoaderError {
    /// An [IO](std::io) Error
    #[error("Could not load file: {0}")]
    Io(#[from] std::io::Error),
}

impl AssetLoader for StyleFileAssetLoader {
    type Asset = StyleFileAsset;
    type Settings = ();
    type Error = StyleFileAssetLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;

        let style = StyleFile::from_bytes(bytes);

        Ok(StyleFileAsset(style))
    }
}
