use std::path::PathBuf;

use bevy::prelude::*;
use typst::{diag::SourceResult, eval::Tracer, model::Document};

use world::TypstWorld;

pub mod fonts;
pub mod world;

mod download;
mod package;

#[derive(Default)]
pub struct TypstCompilerPlugin {
    font_paths: Vec<PathBuf>,
}

impl TypstCompilerPlugin {
    pub fn new(font_paths: Vec<PathBuf>) -> Self {
        Self { font_paths }
    }
}

impl Plugin for TypstCompilerPlugin {
    fn build(&self, app: &mut App) {
        let mut assets = PathBuf::from(".");
        assets.push("assets");
        app.insert_resource(TypstWorld::new(assets, &self.font_paths));
    }
}

#[derive(bevy::ecs::system::SystemParam)]
pub struct TypstCompiler<'w, 's> {
    world: ResMut<'w, TypstWorld>,
    tracer: Local<'s, Tracer>,
}

impl<'w, 's> TypstCompiler<'w, 's> {
    pub fn compile(&mut self, text: &str) -> SourceResult<Document> {
        let world = &mut *self.world;
        let source = world.get_main_source_mut();
        source.replace(text);
        typst::compile(world, &mut self.tracer)
    }
}
