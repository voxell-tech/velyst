use std::sync::Arc;

use bevy::prelude::*;
use world::TypstWorldMeta;

pub mod fonts;
pub mod world;

mod download;
mod package;

#[derive(Resource)]
pub struct TypstCompiler {
    pub world_builder: Arc<TypstWorldMeta>,
}
