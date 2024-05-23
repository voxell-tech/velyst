use std::sync::Arc;

use bevy::{
    asset::{io::Reader, AssetLoader, LoadContext},
    prelude::*,
    utils::{
        thiserror::{self, Error},
        BoxedFuture,
    },
};
use bevy_vello::{
    integrations::{VectorFile, VectorLoaderError, VelloAsset},
    vello,
    vello_svg::{self, usvg},
};
use serde::{Deserialize, Serialize};
use typst::layout::Abs;

use crate::prelude::TypstAsset;

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

            let mut scene = vello::Scene::new();
            vello_svg::render_tree(&mut scene, &tree);

            let view_size = tree.view_box().rect.size();
            let view_width = view_size.width();
            let view_height = view_size.height();

            let image_size = tree.size();
            let image_width = image_size.width();
            let image_height = image_size.height();

            // Use ration to calculate actual width and height
            let width = view_width * view_width / image_width;
            let height = view_height * view_height / image_height;

            let local_transform_center = Transform::from_xyz(width * 0.5, -height * 0.5, 0.0);

            let vello_asset = VelloAsset {
                file: VectorFile::Svg(Arc::new(scene)),
                local_transform_center,
                width,
                height,
                alpha: 1.0,
            };

            Ok(vello_asset)
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

    #[error("Could not load Svg into Vello scene.")]
    VectorLoaderError(#[from] VectorLoaderError),
}
