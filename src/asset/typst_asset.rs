use std::sync::Arc;

use bevy::{
    asset::{io::Reader, AssetLoader, AsyncReadExt, LoadContext},
    prelude::*,
};
use ecow::EcoVec;
use thiserror::Error;
use typst::{diag::SourceDiagnostic, foundations::Module};

use crate::compiler::world::TypstWorld;

pub struct TypstAssetPlugin(pub Arc<TypstWorld>);

impl Plugin for TypstAssetPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<TypstAsset>()
            .register_asset_loader(TypstModAssetLoader(self.0.clone()));
    }
}

#[derive(Asset, TypePath)]
pub struct TypstAsset(Module);

impl TypstAsset {
    pub fn module(&self) -> &Module {
        &self.0
    }
}

pub struct TypstModAssetLoader(Arc<TypstWorld>);

impl AssetLoader for TypstModAssetLoader {
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

        let module = self.0.eval_str(&text).map_err(SourceDiagnosticError)?;
        Ok(TypstAsset(module))
    }

    fn extensions(&self) -> &[&str] {
        &["typ"]
    }
}

/// Possible errors that can be produced by [`TypstDocAssetLoader`] and [`TypstModAssetLoader`].
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum TypstAssetLoaderError {
    /// An [Io](std::io) Error
    #[error("Could not load typst file: {0}")]
    Io(#[from] std::io::Error),

    /// [SourceDiagnostic] Error
    #[error("Could not compile typst file: {0}")]
    SourceDiagnosticError(#[from] SourceDiagnosticError),
}

#[derive(Debug, Error)]
pub struct SourceDiagnosticError(EcoVec<SourceDiagnostic>);

impl std::fmt::Display for SourceDiagnosticError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.0, f)
    }
}
