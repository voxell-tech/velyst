pub use {typst, typst_element, typst_vello};

use {
    asset::TypstAssetPlugin,
    renderer::VelystRendererPlugin,
    std::{path::PathBuf, sync::Arc},
    world::{TypstWorld, TypstWorldRef},
};

use bevy::prelude::*;

pub mod prelude {
    pub use crate::asset::TypstAsset;
    pub use crate::renderer::{
        TypstAssetHandle, TypstContent, TypstContext, TypstFunc, TypstLabel, TypstPath,
        VelystCommandExt, VelystScene, VelystSet,
    };
    pub use crate::world::{TypstWorld, TypstWorldRef};
}

pub mod asset;
pub mod renderer;
pub mod world;

/// Plugin for loading and rendering [Typst][typst] content.
#[derive(Default)]
pub struct VelystPlugin {
    font_paths: Vec<PathBuf>,
}

impl VelystPlugin {
    pub fn new(font_paths: Vec<PathBuf>) -> Self {
        Self { font_paths }
    }
}

impl Plugin for VelystPlugin {
    fn build(&self, app: &mut App) {
        // Using assets/ as the root path
        let mut assets_path = PathBuf::from(".");
        assets_path.push("assets");
        let world = Arc::new(TypstWorld::new(assets_path, &self.font_paths));

        app.add_plugins(TypstAssetPlugin(world.clone()))
            .add_plugins(VelystRendererPlugin)
            .insert_resource(TypstWorldRef::new(world));
    }
}
