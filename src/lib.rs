pub use {typst, typst_element, typst_svg};

use {
    asset::{
        svg_asset::SvgAssetPlugin, typst_asset::TypstAssetPlugin, vello_asset::VelloAssetPlugin,
    },
    compiler::{world::TypstWorldMeta, TypstCompiler},
    std::{path::PathBuf, sync::Arc},
};

use bevy::prelude::*;

pub mod prelude {
    pub use crate::{
        asset::{
            svg_asset::{SvgAsset, SvgAssetLoaderSettings},
            typst_asset::{TypstDocAsset, TypstModAsset},
        },
        typst_template, TypstPlugin,
    };
}

pub mod asset;
pub mod compiler;

#[derive(Default)]
pub struct TypstPlugin {
    font_paths: Vec<PathBuf>,
}

impl TypstPlugin {
    pub fn new(font_paths: Vec<PathBuf>) -> Self {
        Self { font_paths }
    }
}

impl Plugin for TypstPlugin {
    fn build(&self, app: &mut App) {
        // Using assets/ as the root path
        let mut assets_path = PathBuf::from(".");
        assets_path.push("assets");
        let world_meta = Arc::new(TypstWorldMeta::new(assets_path, &self.font_paths));

        app.add_plugins(TypstAssetPlugin(world_meta.clone()))
            .add_plugins((SvgAssetPlugin, VelloAssetPlugin))
            .insert_resource(TypstCompiler(world_meta));
    }
}

#[macro_export]
macro_rules! typst_template {
    {
        $(#[$outer:meta])*
        $vis:vis struct $struct_name:ident {
            $(
                $typst_ty:ty => (
                    $($field:ident,)*
                ),
            )*
        }
    } => {
        $(#[$outer])*
        $vis struct $struct_name {
            $($(pub $field: $typst_ty,)*)*
        }

        impl $struct_name {
            /// Populate the template with a given scope.
            ///
            /// # Panic
            ///
            /// Will panic if any of the fields in the template does not exists
            /// or does not match the type from the given scope.
            pub fn new(scope: &$crate::typst::foundations::Scope) -> Self {
                Self {
                    $($($field: scope.get_unchecked(stringify!($field)),)*)*
                }
            }
        }
    };
}
