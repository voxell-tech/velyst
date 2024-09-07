pub use {typst, typst_element};

use {
    asset::typst_asset::TypstAssetPlugin,
    std::{path::PathBuf, sync::Arc},
    world::{TypstWorld, TypstWorldRef},
};

use bevy::prelude::*;

pub mod prelude {
    pub use crate::asset::typst_asset::TypstAsset;
    pub use crate::typst_template;
    pub use crate::world::{TypstWorld, TypstWorldRef};
}

pub mod asset;
pub mod world;

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
        let world = Arc::new(TypstWorld::new(assets_path, &self.font_paths));

        app.add_plugins(TypstAssetPlugin(world.clone()))
            .insert_resource(TypstWorldRef::new(world));
    }
}

/// Create a template struct easily.
///
/// This macro will also help you construct a `new(scope: &Scope)` method that
/// initialize the template with a given [`Scope`][Scope].
///
/// # Example
///
/// ```
/// use bevy::prelude::*;
/// use bevy_typst::prelude::*;
/// use bevy_typst::typst_element::prelude::*;
///
/// typst_template! {
///     #[derive(Resource)]
///     pub struct UiTemplate {
///         foundations::Func => (
///             parent,
///             frame,
///             important_frame,
///             danger_frame,
///         ),
///     }
/// }
///
/// fn load_ui_template(
///     mut commands: Commands,
///     ui_asset: Res<UiAsset>,
///     typst_assets: Res<Assets<TypstAsset>>,
/// ) {
///     let Some(asset) = typst_assets.get(&ui_asset.0) else {
///         return;
///     };
///
///     let ui_template = UiTemplate::new(asset.module().scope());
///     commands.insert_resource(ui_template);
/// }
///
/// #[derive(Resource)]
/// pub struct UiAsset(Handle<TypstAsset>);
/// ```
///
/// [Scope]: typst::foundations::Scope
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
                    $($($field: $crate::typst_element::extensions::ScopeExt::get_unchecked(
                        scope, stringify!($field)
                    ),)*)*
                }
            }
        }
    };
}
