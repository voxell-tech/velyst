//! # Velyst
//!
//! Velyst is a [Typst](https://typst.app) renderer for [Bevy](https://bevyengine.org).
//!
//! ## Quickstart
//!
//! Velyst renders Typst content using Typst functions.
//! This example shows you how to render a simple white box in the center of the screen.
//! To get started rendering a simple box, create a function inside a `.typ` file:
//!
//! ```typ
//! #let main() = {
//!   box(width: 100%, height: 100%)[
//!     #place(center + horizon)[
//!       #box(width: 10em, height: 10em, fill: white)
//!     ]
//!   ]
//! }
//! ```
//!
//! Then, in your `.rs` file, load your Typst asset file and register your function.
//!
//! ```rs
//! use bevy::prelude::*;
//! use bevy_vello::prelude::*;
//! use velyst::prelude::*;
//!
//! fn main() {
//!     App::new()
//!         .add_plugins((
//!             DefaultPlugins,
//!             bevy_vello::VelloPlugin::default(),
//!             velyst::VelystPlugin,
//!         ))
//!         .register_typst_func::<MainFunc>()
//!         .add_systems(Startup, setup)
//!         .run();
//! }
//!
//! fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
//!     commands.spawn((Camera2d, VelloView));
//!
//!     let handle =
//!         VelystSourceHandle(asset_server.load("typst/box.typ"));
//!
//!     commands.spawn((
//!         VelystFuncBundle {
//!             handle,
//!             func: MainFunc::default(),
//!         },
//!         VelystSize {
//!             width: Val::Percent(100.0),
//!             height: Val::Percent(100.0),
//!         },
//!     ));
//! }
//!
//! typst_func!(
//!     "main",
//!     #[derive(Component, Default)]
//!     struct MainFunc {},
//! );
//! ```

extern crate self as velyst;

use asset::TypstAssetPlugin;
use bevy::prelude::*;
use renderer::VelystRendererPlugin;
use world::VelystWorldPlugin;

pub use {typst, typst_element, typst_vello};

pub mod prelude {
    pub use crate::asset::{
        VelystModules, VelystSource, VelystSourceHandle,
    };
    pub use crate::renderer::{
        TypstFuncAppExt, UiVelystScene, VelystFunc, VelystFuncBundle,
        VelystSet, VelystSourceReady,
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
        app.add_plugins((
            TypstAssetPlugin,
            VelystWorldPlugin,
            VelystRendererPlugin,
        ));
    }
}
