#![doc = include_str!("../README.md")]

extern crate self as velyst;

use asset::TypstAssetPlugin;
use bevy::prelude::*;
use bevy::ui::UiSystems;
use renderer::{VelystRendererPlugin, VelystSet};
use world::VelystWorldPlugin;

pub use {typst, typst_element, typst_vello};

pub mod prelude {
    pub use crate::asset::{VelystModules, VelystSource};
    pub use crate::func::{
        TypstFunc, TypstFuncAppExt, TypstValue, VelystContent,
        VelystFunc, VelystSourceReady,
    };
    pub use crate::renderer::{
        UiScene, VelystScene, VelystSet, WorldScene,
    };
    pub use crate::typst_func;
    pub use crate::world::VelystWorld;
    pub use typst_element::prelude::*;
}

pub mod asset;
pub mod func;
pub mod renderer;
pub mod world;

/// Plugin for loading and rendering [Typst][typst] content.
pub struct VelystPlugin;

impl Plugin for VelystPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            PostUpdate,
            (
                VelystSet::PrepareFunc,
                VelystSet::Compile,
                VelystSet::Layout.in_set(UiSystems::PostLayout),
                VelystSet::PostLayout,
                VelystSet::Render,
            )
                .chain(),
        );

        app.add_plugins((
            TypstAssetPlugin,
            VelystWorldPlugin,
            VelystRendererPlugin,
        ));
    }
}
