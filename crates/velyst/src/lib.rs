#![doc = include_str!("../README.md")]

use asset::TypstAssetPlugin;
use bevy::prelude::*;
use bevy::ui::UiSystems;
use renderer::VelystRendererPlugin;
use world::VelystWorldPlugin;

pub use imaging;
pub use kanva;
pub use typst;
pub use typst_element;

pub mod prelude {
    pub use crate::VelystSet;
    pub use crate::asset::{VelystModules, VelystSource};
    pub use crate::func::{
        TypstFunc, TypstFuncAppExt, TypstValue, VelystContent,
        VelystFunc, VelystSourceReady,
    };
    pub use crate::renderer::{
        UiScene, VelystFrame, VelystKanva, WorldScene,
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

/// Velyst rendering pipeline.
#[derive(SystemSet, Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum VelystSet {
    /// Custom data preparation before compilation should happen here.
    PrepareFunc,
    /// Compile [`func::VelystFunc`] into [`func::VelystContent`].
    ///
    /// One system per registered [`func::TypstFunc`] type runs here.
    Compile,
    /// Layout [`func::VelystContent`] and render into [`UiVelloScene`][bevy_vello::prelude::UiVelloScene] or [`VelloScene2d`][bevy_vello::prelude::VelloScene2d].
    Layout,
    /// Post-layout hook for downstream systems.
    PostLayout,
    /// Hook for downstream rendering systems.
    Render,
}
