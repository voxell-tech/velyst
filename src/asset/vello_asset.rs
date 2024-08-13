use bevy::{
    asset::{io::Reader, AssetLoader, LoadContext},
    prelude::*,
};
use bevy_vello::{integrations::VelloAsset, vello_svg::usvg};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use typst::layout::Abs;

use crate::{compiler::TypstScene, prelude::TypstDocAsset};

pub struct VelloAssetPlugin;

impl Plugin for VelloAssetPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset_loader::<VelloAssetLoader>();
    }
}

#[derive(Default)]
pub struct VelloAssetLoader;

impl AssetLoader for VelloAssetLoader {
    type Asset = VelloAsset;

    type Settings = VelloAssetLoaderSettings;

    type Error = VelloAssetLoaderError;

    async fn load<'a>(
        &'a self,
        _reader: &'a mut Reader<'_>,
        settings: &'a Self::Settings,
        load_context: &'a mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let asset_path = load_context.asset_path().clone();
        let direct_loader = load_context.loader().direct();

        let typst_asset = direct_loader
            .load::<TypstDocAsset>(asset_path)
            .await
            .map_err(|_| VelloAssetLoaderError::LoadDirectError)?
            .take();

        let vello_asset =
            TypstScene::from_document(typst_asset.document(), Abs::pt(settings.padding))?
                .as_asset();
        Ok(vello_asset)
    }

    fn extensions(&self) -> &[&str] {
        &["typ"]
    }
}

#[derive(Default, Serialize, Deserialize)]
pub struct VelloAssetLoaderSettings {
    /// Padding around the document (in [`Abs::pt()`]).
    pub padding: f64,
}

/// Possible errors that can be produced by [`VelloAssetLoader`].
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum VelloAssetLoaderError {
    /// A [`bevy::asset::LoadDirectError`].
    #[error("Could not load typst file.")]
    LoadDirectError,

    /// A [`usvg::Error`] when parsing string into a [`usvg::Tree`].
    #[error("Could parse typst as Svg: {0}")]
    UsvgError(#[from] usvg::Error),
}
