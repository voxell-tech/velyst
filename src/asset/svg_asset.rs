use bevy::{
    asset::{io::Reader, AssetLoader, LoadContext},
    prelude::*,
};
use bevy_vello::vello_svg::usvg;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use typst::layout::Abs;

use crate::prelude::TypstAsset;

pub struct SvgAssetPlugin;

impl Plugin for SvgAssetPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<SvgAsset>()
            .init_asset_loader::<SvgAssetLoader>();
    }
}

#[derive(Asset, TypePath)]
pub struct SvgAsset(usvg::Tree);

impl SvgAsset {
    pub fn tree(&self) -> &usvg::Tree {
        &self.0
    }
}

#[derive(Default)]
pub struct SvgAssetLoader;

impl AssetLoader for SvgAssetLoader {
    type Asset = SvgAsset;

    type Settings = SvgAssetLoaderSettings;

    type Error = SvgAssetLoaderError;

    async fn load<'a>(
        &'a self,
        _reader: &'a mut Reader<'_>,
        settings: &'a Self::Settings,
        load_context: &'a mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let asset_path = load_context.asset_path().clone();
        let direct_loader = load_context.loader().direct();
        let typst_asset = direct_loader
            .load::<TypstAsset>(asset_path)
            .await
            .map_err(|_| SvgAssetLoaderError::LoadDirectError)?
            .take();

        let svg_str = typst_svg::svg_merged(typst_asset.document(), Abs::pt(settings.padding));
        let tree = usvg::Tree::from_str(&svg_str, &usvg::Options::default())?;

        Ok(SvgAsset(tree))
    }

    fn extensions(&self) -> &[&str] {
        &["typ"]
    }
}

#[derive(Default, Serialize, Deserialize)]
pub struct SvgAssetLoaderSettings {
    /// Padding around the document (in [`Abs::pt()`]).
    pub padding: f64,
}

/// Possible errors that can be produced by [`SvgAssetLoader`].
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum SvgAssetLoaderError {
    /// A [`bevy::asset::LoadDirectError`].
    #[error("Could not load typst file.")]
    LoadDirectError,

    /// A [`usvg::Error`] when parsing string to a [`usvg::Tree`].
    #[error("Could parse typst as Svg: {0}")]
    UsvgError(#[from] usvg::Error),
}
