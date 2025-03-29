use asset::TypstAssetPlugin;
use bevy::prelude::*;
use renderer::VelystRendererPlugin;
use world::VelystWorldPlugin;

pub use {typst, typst_element, typst_vello};

pub mod prelude {
    pub use crate::asset::TypstSource;
    pub use crate::renderer::{
        TypstAssetHandle, TypstContent, TypstContext, TypstFunc, TypstLabel, TypstPath,
        VelystAppExt, VelystScene, VelystSet,
    };
    pub use crate::world::VelystWorld;
    pub use velyst_macros::{TypstFunc, TypstPath};
}

pub mod asset;
pub mod renderer;
pub mod world;

/// Plugin for loading and rendering [Typst][typst] content.
pub struct VelystPlugin;

impl Plugin for VelystPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((TypstAssetPlugin, VelystWorldPlugin, VelystRendererPlugin));
    }
}
