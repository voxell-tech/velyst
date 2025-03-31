extern crate self as velyst;

use asset::TypstAssetPlugin;
use bevy::prelude::*;
use renderer::VelystRendererPlugin;
use world::VelystWorldPlugin;

pub use {typst, typst_element, typst_vello};

pub mod prelude {
    pub use crate::asset::{VelystModules, VelystSource, VelystSourceHandle};
    pub use crate::renderer::{
        TypstFuncAppExt, VelystFunc, VelystFuncBundle, VelystScene, VelystSet, VelystSourceReady,
    };
    pub use crate::typst_func;
    pub use crate::world::VelystWorld;
    pub use typst_element::prelude::*;
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
