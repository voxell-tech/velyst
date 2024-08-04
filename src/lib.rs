pub use {typst, typst_svg};

use {
    asset::{
        svg_asset::SvgAssetPlugin, typst_asset::TypstAssetPlugin, vello_asset::VelloAssetPlugin,
    },
    compiler::{world::TypstWorldMeta, TypstCompiler},
    std::{path::PathBuf, sync::Arc},
};

use bevy::prelude::*;

pub mod prelude {
    pub use crate::{
        asset::{
            svg_asset::{SvgAsset, SvgAssetLoaderSettings},
            typst_asset::TypstAsset,
        },
        TypstPlugin,
    };
}

pub mod asset;
pub mod compiler;

#[derive(Default)]
pub struct TypstPlugin {
    font_paths: Vec<PathBuf>,
}

impl TypstPlugin {
    pub fn new(font_paths: Vec<PathBuf>) -> Self {
        Self { font_paths }
    }
}

impl Plugin for TypstPlugin {
    fn build(&self, app: &mut App) {
        // Using assets/ as the root path
        let mut assets_path = PathBuf::from(".");
        assets_path.push("assets");
        let world_meta = Arc::new(TypstWorldMeta::new(assets_path, &self.font_paths));

        app.add_plugins(TypstAssetPlugin(world_meta.clone()))
            .add_plugins((SvgAssetPlugin, VelloAssetPlugin))
            .insert_resource(TypstCompiler(world_meta));
    }
}
