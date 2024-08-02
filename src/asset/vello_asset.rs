use std::sync::Arc;

use bevy::{
    asset::{io::Reader, AssetLoader, LoadContext},
    prelude::*,
};
use bevy_vello::{
    integrations::{VectorFile, VelloAsset},
    vello_svg::{self, usvg},
};
use thiserror::Error;
use typst::layout::Abs;

use crate::prelude::TypstAsset;

use super::svg_asset::SvgAssetLoaderSettings;

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

    type Settings = SvgAssetLoaderSettings;

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
            .load::<TypstAsset>(asset_path)
            .await
            .map_err(|_| VelloAssetLoaderError::LoadDirectError)?
            .take();

        let svg_str = typst_svg::svg_merged(typst_asset.document(), Abs::raw(settings.padding));
        let tree = usvg::Tree::from_str(&svg_str, &usvg::Options::default())?;

        let scene = vello_svg::render_tree(&tree);

        let size = tree.size();
        let width = size.width();
        let height = size.height();

        let local_transform_center = Transform::from_xyz(width * 0.5, -height * 0.5, 0.0);

        let vello_asset = VelloAsset {
            file: VectorFile::Svg(Arc::new(scene)),
            local_transform_center,
            width,
            height,
            alpha: 1.0,
        };

        Ok(vello_asset)
    }

    fn extensions(&self) -> &[&str] {
        &["typ"]
    }
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
