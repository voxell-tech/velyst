use std::sync::Arc;

use bevy::prelude::*;
use bevy_vello::{
    integrations::{VectorFile, VelloAsset},
    vello,
    vello_svg::{self, usvg},
    VelloScene,
};
use typst::{
    diag::SourceResult,
    foundations::{Content, Module},
    layout::Abs,
    model::Document,
};
use world::TypstWorldMeta;

pub mod fonts;
pub mod world;

mod download;
mod package;

#[derive(Resource)]
pub struct TypstCompiler(pub Arc<TypstWorldMeta>);

impl TypstCompiler {
    pub fn eval_str(&self, text: impl Into<String>) -> SourceResult<Module> {
        self.0.eval_str(text)
    }

    pub fn compile_str(&self, text: impl Into<String>) -> SourceResult<Document> {
        self.0.compile_str(text)
    }

    pub fn compile_content(&self, content: Content) -> SourceResult<Document> {
        self.0.compile_content(content)
    }
}

pub struct TypstScene {
    pub scene: vello::Scene,
    pub width: f32,
    pub height: f32,
}

impl TypstScene {
    pub fn from_document(document: &Document, padding: Abs) -> Result<Self, usvg::Error> {
        let svg_str = typst_svg::svg_merged(document, padding);

        let tree = usvg::Tree::from_str(&svg_str, &usvg::Options::default())?;
        let scene = vello_svg::render_tree(&tree);
        let size = tree.size();

        Ok(Self {
            scene,
            width: size.width(),
            height: size.height(),
        })
    }

    pub fn as_asset(self) -> VelloAsset {
        let local_transform_center = Transform::from_xyz(self.width * 0.5, -self.height * 0.5, 0.0);

        VelloAsset {
            file: VectorFile::Svg(Arc::new(self.scene)),
            local_transform_center,
            width: self.width,
            height: self.height,
            alpha: 1.0,
        }
    }

    pub fn as_component(self) -> VelloScene {
        VelloScene::from(self.scene)
    }
}
