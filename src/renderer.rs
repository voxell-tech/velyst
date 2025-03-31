use bevy::prelude::*;
use bevy::ui::UiSystem;
use bevy_vello::prelude::*;
use typst::foundations::{Content, NativeElement, Value};
use typst::layout::{Abs, Axes, Region, Size};
use typst_element::elem::FuncCall;
use typst_element::prelude::ScopeExt;
use typst_vello::TypstScene;

use crate::asset::{VelystModules, VelystSourceHandle};
use crate::world::VelystWorld;

// pub use velyst_macros::TypstFunc;

pub struct VelystRendererPlugin;

impl Plugin for VelystRendererPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            PostUpdate,
            (
                VelystSet::PrepareFunc,
                VelystSet::Compile,
                VelystSet::Layout.in_set(UiSystem::PostLayout),
                VelystSet::PostLayout,
                VelystSet::Render,
            )
                .chain(),
        );

        app.add_systems(
            PostUpdate,
            (
                (check_source_ready, compile_velyst_func)
                    .chain()
                    .in_set(VelystSet::Compile),
                layout_content.in_set(VelystSet::Layout),
                render_scene.in_set(VelystSet::Render),
            ),
        );
    }
}

pub trait TypstFuncAppExt {
    fn register_typst_func<Func: TypstFuncComp>(&mut self) -> &mut Self;
}

impl TypstFuncAppExt for App {
    fn register_typst_func<Func: TypstFuncComp>(&mut self) -> &mut Self {
        self.add_systems(
            PostUpdate,
            spawn_or_apply_typst_func::<Func>.in_set(VelystSet::PrepareFunc),
        )
    }
}

fn spawn_or_apply_typst_func<Func: TypstFuncComp>(
    mut commands: Commands,
    mut q_funcs: Query<(&Func, Option<&mut VelystFunc>, Entity), Changed<Func>>,
) {
    for (func, velyst_func, entity) in q_funcs.iter_mut() {
        match velyst_func {
            Some(mut velyst_func) => velyst_func.apply_typst_func(func),
            None => {
                let mut velyst_func = VelystFunc::default();
                velyst_func.apply_typst_func(func);
                commands.entity(entity).insert(velyst_func);
            }
        }
    }
}

fn compile_velyst_func(
    mut commands: Commands,
    mut q_funcs: Query<
        (
            &VelystFunc,
            &mut VelystContent,
            &VelystSourceHandle,
            &Visibility,
            Entity,
        ),
        (
            Or<(Changed<VelystFunc>, Changed<Visibility>)>,
            With<VelystSourceReady>,
        ),
    >,
    modules: Res<VelystModules>,
) {
    for (func, mut content, handle, viz, entity) in q_funcs.iter_mut() {
        if viz == Visibility::Hidden {
            continue;
        }

        if let Some(module) = modules.get(&handle.id()) {
            match module.scope().get_func(func.name) {
                Ok(typst_func) => {
                    content.0 = typst_func
                        .call_with_named(&func.positional_args, &func.named_args)
                        .pack();
                }
                Err(err) => error!("Unable to get typst function {}: {err}", func.name),
            }
        } else {
            // Check again for module availability next frame.
            commands.entity(entity).remove::<VelystSourceReady>();
        }
    }
}

fn check_source_ready(
    mut commands: Commands,
    mut q_funcs: Query<(&VelystSourceHandle, Entity), Without<VelystSourceReady>>,
    modules: Res<VelystModules>,
) {
    for (handle, entity) in q_funcs.iter_mut() {
        if modules.contains_key(&handle.id()) {
            commands.entity(entity).insert(VelystSourceReady);
        }
    }
}

/// Layout [`Content`] into a [`VelystScene`].
fn layout_content(
    world: VelystWorld,
    mut q_contents: Query<
        (
            &VelystContent,
            &mut VelystScene,
            &Visibility,
            Option<&ComputedNode>,
        ),
        Or<(Changed<VelystContent>, Changed<Visibility>)>,
    >,
) {
    for (content, mut scene, viz, node) in q_contents.iter_mut() {
        if viz == Visibility::Hidden {
            continue;
        }
        if let Some(frame) = world.layout_frame(
            &content.0,
            // Constraint to node size if it exists.
            node.map(|node| {
                let size = node.size().as_dvec2();
                Region::new(
                    Size::new(Abs::pt(size.x), Abs::pt(size.y)),
                    Axes::splat(false),
                )
            }),
        ) {
            scene.0 = TypstScene::from_frame(&frame);
        }
    }

    // Clear cache regularly to prevent memory build ups.
    typst::comemo::evict(4);
}

/// Render [`VelystScene`] into a [`VelloScene`].
fn render_scene(
    mut q_scenes: Query<
        (&mut VelystScene, &mut VelloScene, &Visibility),
        Or<(Changed<VelystScene>, Changed<Visibility>)>,
    >,
) {
    for (mut velyst_scene, mut vello_scene, viz) in q_scenes.iter_mut() {
        if viz == Visibility::Hidden {
            continue;
        }
        *vello_scene = VelloScene::from(velyst_scene.render());
    }
}

#[derive(Component, Default)]
#[require(VelystContent)]
pub struct VelystFunc {
    pub name: &'static str,
    pub positional_args: Vec<Value>,
    pub named_args: Vec<(&'static str, Value)>,
}

impl VelystFunc {
    pub fn apply_typst_func<F: TypstFunc>(&mut self, func: &F) {
        self.name = F::NAME;
        func.apply_positional_args(&mut self.positional_args);
        func.apply_named_args(&mut self.named_args);
    }
}

/// Marker component that is inserted when the [module][typst::foundations::Module]
/// needed from [`VelystModules`] for the [`VelystSourceHandle`] is ready.
///
/// Will be removed when the [module][typst::foundations::Module]
/// needed becomes unavailable again.
#[derive(Component)]
pub struct VelystSourceReady;

#[derive(Component, Default, Deref, DerefMut)]
#[require(VelystScene)]
pub struct VelystContent(pub Content);

#[derive(Component, Default, Deref, DerefMut)]
#[require(VelloScene)]
pub struct VelystScene(pub TypstScene);

pub trait TypstFunc {
    const NAME: &str;

    fn apply_positional_args(&self, args: &mut Vec<Value>);

    fn apply_named_args(&self, args: &mut Vec<(&'static str, Value)>);
}

pub trait TypstFuncComp: TypstFunc + Component {}

impl<T: TypstFunc + Component> TypstFuncComp for T {}

/// Helper macro for creating Typst function struct with
/// [`TypstFunc`] trait implementation.
///
/// # Example
///
/// ```
/// use velyst::typst_func;
/// use velyst::typst::foundations::IntoValue;
/// use bevy::prelude::*;
///
/// typst_func!(
///    /// A main function from Typst.
///    #[derive(Component, Reflect)]
///    #[reflect(Component)]
///    pub struct MainFunc<T: IntoValue + Clone + 'static> {
///        #[reflect(ignore)]
///        pos_arg0: f64,
///        /// Documentation for `pos_arg1`.
///        pos_arg1: T,
///    },
///    // Named variables will be placed here.
///    named {
///        named_arg0: bool,
///        named_arg1: i32,
///        #[reflect(ignore)]
///        named_arg2: String,
///    },
///    // The literal function name from the Typst scope.
///    "main"
/// );
/// ```
#[macro_export]
macro_rules! typst_func {
    (
        // Attributes.
        $( #[$attr:meta] )*
        $vis:vis struct $struct_name:ident
        // Lifetimes and generics.
        $(< $( $generic:tt $( : $bound:tt $(+ $_bound:tt )* )? ),+ >)? {
            // Positional ags
            $(
                $( #[$pos_attr:meta] )*
                $pos_arg:ident: $pos_type:ty,
            )*
        },
        $(
            named {
                // Optional named args.
                $(
                    $( #[$named_attr:meta] )*
                    $named_arg:ident: $named_type:ty,
                )*
            },
        )?
        $str_name:literal
    ) => {
        // Define the struct.
        // Attributes.
        $( #[$attr] )*
        $vis struct $struct_name
        // Lifetimes and generics.
        $(< $( $generic $( : $bound $(+ $_bound )* )? ),+ >)? {
            // Positional ags
            $(
                $( #[$pos_attr] )*
                $pos_arg: $pos_type,
            )*
            // Optional named args.
            $($(
                $( #[$named_attr] )*
                $named_arg: Option<$named_type>,
            )*)?
        }

        // Implement Typst func.
        // Lifetimes and generics.
        impl $(< $( $generic $( : $bound $(+ $_bound )* )? ),+ >)?
        $crate::renderer::TypstFunc for $struct_name
        // Bounds are not requeired.
        $(< $( $generic ),+ >)?
        {
            const NAME: &'static str = $str_name;

            fn apply_positional_args(&self, args: &mut Vec<$crate::typst::foundations::Value>) {
                args.clear();
                $(
                    args.push(
                        $crate::typst::foundations::IntoValue::into_value(self.$pos_arg.clone())
                    );
                )*
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

#[derive(Bundle)]
pub struct VelystFuncBundle<Func: TypstFuncComp> {
    pub handle: VelystSourceHandle,
    pub func: Func,
}

// #[derive(Component, Deref, DerefMut)]
// pub struct TypstLabel(TypLabel);

/// Velyst rendering pipeline.
#[derive(SystemSet, Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum VelystSet {
    /// Applying data to [`VelystFunc`] should happen here.
    ///
    /// Data from registered [`TypstFunc`] is applied to [`VelystFunc`] here.
    PrepareFunc,
    /// Compile [`VelystFunc`] into a [`VelystContent`].
    ///
    /// Custom compilation could also happen here.
    Compile,
    /// Layout [`VelystContent`] into a [`VelystScene`].
    Layout,
    /// Post processing of [`VelystScene`] should happen here.
    PostLayout,
    /// Render [`VelystScene`] into a [`VelloScene`].
    Render,
}
