use bevy::asset::AsAssetId;
use bevy::prelude::*;
use typst::foundations::{Content, IntoValue, NativeElement, Value};
use typst_element::elem::FuncCall;
use typst_element::prelude::ScopeExt;

use crate::VelystSet;
use crate::asset::{VelystModules, VelystSource};
use crate::renderer::VelystFrame;

pub trait TypstFuncAppExt {
    fn register_typst_func<F: TypstFunc>(&mut self) -> &mut Self;
}

impl TypstFuncAppExt for App {
    /// Register a [`TypstFunc`] type so that [`VelystFunc<F>`]
    /// entities are compiled into [`VelystContent`] when they
    /// change.
    fn register_typst_func<F: TypstFunc>(&mut self) -> &mut Self {
        self.add_systems(
            PostUpdate,
            (check_source_ready::<F>, compile_velyst_func::<F>)
                .chain()
                .in_set(VelystSet::Compile),
        )
    }
}

/// Insert or remove [`VelystSourceReady`] based on whether the module
/// for the entity's handle is loaded.
fn check_source_ready<F: TypstFunc>(
    mut commands: Commands,
    q_funcs: Query<(Entity, &VelystFunc<F>, Has<VelystSourceReady>)>,
    modules: Res<VelystModules>,
) {
    for (entity, func, is_ready) in q_funcs.iter() {
        let module_ready = modules.contains_key(&func.handle.id());

        if module_ready && !is_ready {
            commands.entity(entity).insert(VelystSourceReady);
        } else if !module_ready && is_ready {
            commands.entity(entity).remove::<VelystSourceReady>();
        }
    }
}

/// Compile a [`VelystFunc<F>`] into a [`VelystContent`].
fn compile_velyst_func<F: TypstFunc>(
    mut q_funcs: Query<(
        Ref<VelystFunc<F>>,
        &mut VelystContent,
        Ref<Visibility>,
        Ref<VelystSourceReady>,
    )>,
    modules: Res<VelystModules>,
    mut asset_events: MessageReader<AssetEvent<VelystSource>>,
) {
    let changed_assets: smallvec::SmallVec<
        [AssetId<VelystSource>; 4],
    > = asset_events
        .read()
        .filter_map(|e| match e {
            AssetEvent::Added { id }
            | AssetEvent::Modified { id } => Some(*id),
            _ => None,
        })
        .collect();

    for (func, mut content, viz, ready) in q_funcs.iter_mut() {
        let needs_recompile = func.is_changed()
            || viz.is_changed()
            || ready.is_added()
            || changed_assets.contains(&func.handle.id());

        if !needs_recompile || *viz == Visibility::Hidden {
            continue;
        }

        let Some(module) = modules.get(&func.handle.id()) else {
            continue;
        };

        match module.scope().get_func(F::NAME) {
            Ok(typst_func) => {
                let mut positional_args = Vec::new();
                let mut named_args = Vec::new();
                func.data.apply_positional_args(&mut positional_args);
                func.data.apply_named_args(&mut named_args);
                content.0 = typst_func
                    .call_with_named(&positional_args, &named_args)
                    .pack();
            }
            Err(err) => error!(
                "Unable to get typst function {}: {err}",
                F::NAME
            ),
        }
    }
}

/// A Typst function component. Holds the source asset handle and the
/// typed function data. Register it with
/// [`TypstFuncAppExt::register_typst_func`] and add [`UiScene`] or
/// [`WorldScene`] to control which coordinate space this entity
/// renders in.
///
/// [`UiScene`]: crate::renderer::UiScene
/// [`WorldScene`]: crate::renderer::WorldScene
#[derive(Component)]
#[require(VelystContent)]
pub struct VelystFunc<F: TypstFunc> {
    pub handle: Handle<VelystSource>,
    pub data: F,
}

impl<F: TypstFunc> AsAssetId for VelystFunc<F> {
    type Asset = VelystSource;

    fn as_asset_id(&self) -> AssetId<Self::Asset> {
        self.handle.id()
    }
}

impl<F: TypstFunc> VelystFunc<F> {
    pub fn new(handle: Handle<VelystSource>, data: F) -> Self {
        Self { handle, data }
    }
}

/// Marker component that is inserted when the
/// [module][typst::foundations::Module] needed for this entity's
/// [`VelystFunc`] handle is ready.
///
/// Will be removed when the [module][typst::foundations::Module]
/// needed becomes unavailable again.
#[derive(Component)]
pub struct VelystSourceReady;

#[derive(Component, Default, Deref, DerefMut)]
#[require(VelystFrame)]
pub struct VelystContent(pub Content);

pub trait TypstValue:
    IntoValue + Clone + Send + Sync + 'static
{
}

impl<T: IntoValue + Clone + Send + Sync + 'static> TypstValue for T {}

pub trait TypstFunc: Send + Sync + 'static {
    const NAME: &str;

    fn apply_positional_args(&self, args: &mut Vec<Value>);

    fn apply_named_args(&self, args: &mut Vec<(&'static str, Value)>);
}

/// Helper macro for creating Typst function struct with
/// [`TypstFunc`] trait implementation.
///
/// # Example
///
/// ```
/// use bevy::prelude::*;
/// use velyst::prelude::*;
///
/// typst_func!(
///     // The literal function name from the Typst scope,
///     // usually from the source file.
///     "button",
///     /// A button function from Typst.
///     #[derive(Component, Reflect)]
///     #[reflect(Component)]
///     struct ButtonFunc<T: TypstValue> {},
///     // Positional arguments, order matters here!
///     positional_args {
///         /// Label size.
///         size: f64,
///         custom_data: T,
///         /// Button label.
///         #[reflect(ignore)]
///         label: String,
///     },
///     // Named arguments, order doesn't really matters here.
///     named_args {
///         icon_index: u32,
///         #[reflect(ignore)]
///         icon_label: String,
///     },
/// );
/// ```
///
/// Arguments can be also omitted if there aren't any:
///
/// ```
/// use velyst::typst_func;
/// typst_func!("empty", struct EmptyFunc {});
/// ```
#[macro_export]
macro_rules! typst_func {
    (
        // The literal function name from the Typst scope,
        // usually from the source file.
        $str_name:literal,
        // Attributes.
        $( #[$attr:meta] )*
        $vis:vis struct $struct_name:ident
        // Lifetimes and generics.
        $(< $( $generic:tt $( : $bound:tt $(+ $_bound:tt )* )? ),+ >)? {}$(,)?
        // Positional args
        $(
            positional_args {
                // Optional positional args.
                $(
                    $( #[$positional_attr:meta] )*
                    $positional_arg:ident: $positional_type:ty
                ),*$(,)?
            }$(,)?
        )?
        // Named args.
        $(
            named_args {
                $(
                    $( #[$named_attr:meta] )*
                    $named_arg:ident: $named_type:ty
                ),*$(,)?
            }$(,)?
        )?
    ) => {
        // Define the struct.
        // Attributes.
        $( #[$attr] )*
        $vis struct $struct_name
        // Lifetimes and generics.
        $(< $( $generic $( : $bound $(+ $_bound )* )? ),+ >)? {
            // Positional ags
            $($(
                $( #[$positional_attr] )*
                $positional_arg: $positional_type,
            )*)?
            // Optional named args.
            $($(
                $( #[$named_attr] )*
                $named_arg: Option<$named_type>,
            )*)?
        }

        // Implement Typst func.
        // Lifetimes and generics.
        impl $(< $( $generic $( : $bound $(+ $_bound )* )? ),+ >)?
        $crate::func::TypstFunc for $struct_name
        // Bounds are not required.
        $(< $( $generic ),+ >)?
        {
            const NAME: &'static str = $str_name;

            fn apply_positional_args(&self, args: &mut Vec<$crate::typst::foundations::Value>) {
                args.clear();
                $($(
                    args.push(
                        $crate::typst::foundations::IntoValue::into_value(
                            self.$positional_arg.clone()
                        )
                    );
                )*)?
            }

            fn apply_named_args(&self, args: &mut Vec<(&'static str, $crate::typst::foundations::Value)>) {
                args.clear();
                $($(
                    if let Some(arg) = self.$named_arg.as_ref() {
                        args.push(
                            (stringify!($named_arg),
                            $crate::typst::foundations::IntoValue::into_value(arg.clone()))
                        );
                    }
                )*)?
            }
        }
    };
}
