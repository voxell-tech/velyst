use std::sync::Arc;

use bevy::{
    asset::{io::Reader, AssetLoader, AsyncReadExt, LoadContext},
    prelude::*,
};
use ecow::EcoVec;
use thiserror::Error;
use typst::{diag::SourceDiagnostic, model::Document};

use crate::compiler::world::TypstWorldMeta;

pub struct TypstAssetPlugin(pub Arc<TypstWorldMeta>);

impl Plugin for TypstAssetPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<TypstAsset>()
            .register_asset_loader(TypstAssetLoader {
                world_meta: self.0.clone(),
            });
    }
}

#[derive(Asset, TypePath)]
pub struct TypstAsset(Document);

impl TypstAsset {
    pub fn document(&self) -> &Document {
        &self.0
    }
}

pub struct TypstAssetLoader {
    world_meta: Arc<TypstWorldMeta>,
}

impl AssetLoader for TypstAssetLoader {
    type Asset = TypstAsset;

    type Settings = ();

    type Error = TypstAssetLoaderError;

    async fn load<'a>(
        &'a self,
        reader: &'a mut Reader<'_>,
        _settings: &'a Self::Settings,
        _load_context: &'a mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut text = String::new();
        reader.read_to_string(&mut text).await?;

        let document = self
            .world_meta
            .compile_str(&text)
            .map_err(TypstCompileError)?;

        Ok(TypstAsset(document))
    }

    fn extensions(&self) -> &[&str] {
        &["typ"]
    }
}

/// Possible errors that can be produced by [`TypstAssetLoader`].
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum TypstAssetLoaderError {
    /// An [Io](std::io) Error
    #[error("Could not load typst file: {0}")]
    Io(#[from] std::io::Error),

    /// A [`typst::compile`] Error
    #[error("Could not compile typst file: {0}")]
    TypstCompileError(#[from] TypstCompileError),
}

#[derive(Debug, Error)]
pub struct TypstCompileError(EcoVec<SourceDiagnostic>);

impl std::fmt::Display for TypstCompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.0, f)
    }
}
