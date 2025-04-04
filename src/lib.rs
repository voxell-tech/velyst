pub use {typst, typst_element, typst_library, typst_vello};

use std::path::PathBuf;

use asset::TypstAssetPlugin;
use renderer::VelystRendererPlugin;
use world::{TypstWorld, TypstWorldPlugin, TypstWorldRef};

use bevy::prelude::*;

pub mod prelude {
    pub use crate::asset::TypstAsset;
    pub use crate::renderer::{
        TypstAssetHandle, TypstContent, TypstContext, TypstFunc, TypstLabel, TypstPath,
        VelystAppExt, VelystScene, VelystSceneTag, VelystSet,
    };
    pub use crate::world::{TypstWorld, TypstWorldRef};
    pub use velyst_macros::{TypstFunc, TypstPath};
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
        let world = TypstWorld::new(assets_path, &self.font_paths);
        let world_ref = TypstWorldRef::new(world);

        app.add_plugins((
            TypstWorldPlugin(world_ref.clone()),
            TypstAssetPlugin(world_ref.clone()),
            VelystRendererPlugin,
        ));
    }
}
