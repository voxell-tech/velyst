use bevy::{
    asset::{io::Reader, AssetLoader, LoadContext},
    prelude::*,
    utils::{
        thiserror::{self, Error},
        BoxedFuture,
    },
};
use bevy_vello::vello_svg::usvg;
use serde::{Deserialize, Serialize};
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

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        settings: &'a Self::Settings,
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let asset_path = load_context.asset_path().clone();
            let typst_asset = load_context
                .load_direct_with_reader(reader, asset_path)
                .await
                .or_else(|_| Err(SvgAssetLoaderError::LoadDirectError))?;

            let typst_asset = typst_asset
                .take::<TypstAsset>()
                .expect("TypstAsset missing after loading.");

            let svg_str = typst_svg::svg_merged(typst_asset.document(), Abs::raw(settings.padding));
            let tree = usvg::Tree::from_str(
                &svg_str,
                &usvg::Options::default(),
                &usvg::fontdb::Database::default(),
            )?;

            Ok(SvgAsset(tree))
        })
    }

    fn extensions(&self) -> &[&str] {
        &["typ"]
    }
}

#[derive(Default, Serialize, Deserialize)]
pub struct SvgAssetLoaderSettings {
    pub padding: f64,
}

/// Possible errors that can be produced by [`SvgAssetLoader`].
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum SvgAssetLoaderError {
    /// A [`LoadDirectError`].
    #[error("Could not load typst file.")]
    LoadDirectError,

    #[error("Could parse typst as Svg: {0}")]
    UsvgError(#[from] usvg::Error),
}
