use bevy::prelude::*;
use bevy::ui::{ContentSize, UiSystems};
use bevy_vello::prelude::*;
use typst::foundations::{Content, NativeElement, Value};
use typst::layout::{Abs, Axes, Region, Size};
use typst_element::elem::FuncCall;
use typst_element::prelude::ScopeExt;
use typst_vello::TypstScene;

use crate::asset::{VelystModules, VelystSourceHandle};
use crate::world::VelystWorld;

pub struct VelystRendererPlugin;

impl Plugin for VelystRendererPlugin {
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
    fn register_typst_func<Func: TypstFuncComp>(
        &mut self,
    ) -> &mut Self;
}

impl TypstFuncAppExt for App {
    /// Spawns the necessary components for rendering a [`TypstFunc`]
    /// when it is spawned via a [`VelystFuncBundle`].
    fn register_typst_func<Func: TypstFuncComp>(
        &mut self,
    ) -> &mut Self {
        self.add_systems(
            PostUpdate,
            apply_typst_func::<Func>.in_set(VelystSet::PrepareFunc),
        )
        .add_observer(spawn_velyst_func::<Func>)
    }
}

/// Spawn [`VelystFunc`] for a newly added [`TypstFuncComp`].
fn spawn_velyst_func<Func: TypstFuncComp>(
    trigger: On<Add, Func>,
    mut commands: Commands,
    mut q_func: Query<&Func, Without<VelystFunc>>,
) {
    let entity = trigger.event_target();

    if let Ok(func) = q_func.get_mut(entity) {
        let mut velyst_func = VelystFunc::default();
        velyst_func.apply_typst_func(func);
        commands.entity(entity).insert(velyst_func);
    }
}

/// Apply name and arguments from a [`TypstFunc`] to [`VelystFunc`].
fn apply_typst_func<Func: TypstFuncComp>(
    mut q_funcs: Query<(&Func, &mut VelystFunc), Changed<Func>>,
) {
    for (func, mut velyst_func) in q_funcs.iter_mut() {
        velyst_func.apply_typst_func(func);
    }
}

/// Compile a [`VelystFunc`] into a [`VelystContent`].
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
            Or<(
                Changed<VelystFunc>,
                Changed<Visibility>,
                Changed<VelystSourceHandle>, // TODO: Use AssetChanged in 0.16.
                Added<VelystSourceReady>,
            )>,
            With<VelystSourceReady>,
        ),
    >,
    modules: Res<VelystModules>,
) {
    for (func, mut content, handle, viz, entity) in q_funcs.iter_mut()
    {
        if viz == Visibility::Hidden {
            continue;
        }

        if let Some(module) = modules.get(&handle.id()) {
            match module.scope().get_func(func.name) {
                Ok(typst_func) => {
                    content.0 = typst_func
                        .call_with_named(
                            &func.positional_args,
                            &func.named_args,
                        )
                        .pack();
                }
                Err(err) => error!(
                    "Unable to get typst function {}: {err}",
                    func.name
                ),
            }
        } else {
            // Check again for module availability next frame.
            commands.entity(entity).remove::<VelystSourceReady>();
        }
    }
}

fn check_source_ready(
    mut commands: Commands,
    mut q_funcs: Query<
        (&VelystSourceHandle, Entity),
        Without<VelystSourceReady>,
    >,
    modules: Res<VelystModules>,
) {
    for (handle, entity) in q_funcs.iter_mut() {
        if modules.contains_key(&handle.id()) {
            commands.entity(entity).insert(VelystSourceReady);
        }
    }
}

/// Layout [`Content`] into a [`UiVelystScene`].
fn layout_content(
    world: VelystWorld,
    mut q_contents: Query<
        (
            &VelystContent,
            &mut UiVelystScene,
            &Visibility,
            &Node,
            &ComputedNode,
            &mut ContentSize,
            &ComputedUiRenderTargetInfo,
        ),
        Or<(
            Changed<VelystContent>,
            Changed<Visibility>,
            Changed<ComputedNode>,
        )>,
    >,
) {
    for (
        content,
        mut scene,
        viz,
        node,
        computed_node,
        mut content_size,
        target_info,
    ) in q_contents.iter_mut()
    {
        let scale_factor = target_info.scale_factor();
        if scale_factor == 0.0 {
            continue;
        }

        if viz == Visibility::Hidden {
            continue;
        }

        let mut size = Size::splat(Abs::inf());

        if node.width != Val::Auto {
            size.x =
                Abs::pt((computed_node.size.x / scale_factor) as f64);
        }
        if node.height != Val::Auto {
            size.y =
                Abs::pt((computed_node.size.y / scale_factor) as f64);
        }

        if let Some(frame) = world.layout_frame(
            &content.0,
            Region::new(size, Axes::splat(false)),
        ) {
            scene.0 = TypstScene::from_frame(&frame);

            let Axes { x, y } = frame.size();
            let size = Vec2::new(x.to_pt() as f32, y.to_pt() as f32)
                * scale_factor;
            *content_size = ContentSize::fixed_size(size);
        }
    }

    // Clear cache regularly to prevent memory build ups.
    typst::comemo::evict(4);
}

/// Render [`UiVelystScene`] into a [`UiVelloScene`].
fn render_scene(
    mut q_scenes: Query<
        (&mut UiVelystScene, &mut UiVelloScene, &Visibility),
        Or<(Changed<UiVelystScene>, Changed<Visibility>)>,
    >,
) {
    for (mut velyst_scene, mut vello_scene, viz) in
        q_scenes.iter_mut()
    {
        if viz == Visibility::Hidden {
            continue;
        }
        *vello_scene = UiVelloScene::from(velyst_scene.render());
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
    /// Apply name and arguments.
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
#[require(UiVelystScene)]
pub struct VelystContent(pub Content);

#[derive(Component, Default, Deref, DerefMut)]
#[require(UiVelloScene, ContentSize)]
pub struct UiVelystScene(pub TypstScene);

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
///     // The literal function name from the Typst scope,
///     // usually from the source file.
///     "button",
///     /// A button function from Typst.
///     #[derive(Component, Reflect)]
///     #[reflect(Component)]
///     pub(super) struct ButtonFunc<T: IntoValue + Clone> {},
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
        $crate::renderer::TypstFunc for $struct_name
        // Bounds are not requeired.
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
    /// Layout [`VelystContent`] into a [`UiVelystScene`].
    Layout,
    /// Post processing of [`UiVelystScene`] should happen here.
    PostLayout,
    /// Render [`UiVelystScene`] into a [`UiVelloScene`].
    Render,
}
