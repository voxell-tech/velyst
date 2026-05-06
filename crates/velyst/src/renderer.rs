use bevy::asset::AsAssetId;
use bevy::camera::primitives::Aabb;
use bevy::prelude::*;
use bevy::ui::{ContentSize, UiSystems};
use bevy_vello::prelude::*;
use typst::foundations::{Content, IntoValue, NativeElement, Value};
use typst::layout::{Abs, Axes, Region, Size};
use typst_element::elem::FuncCall;
use typst_element::prelude::ScopeExt;
use typst_vello::TypstScene;

use crate::asset::{VelystModules, VelystSource};
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
                (layout_content, layout_world_content)
                    .in_set(VelystSet::Layout),
                (render_ui_scene, render_world_scene)
                    .in_set(VelystSet::Render),
            ),
        );
    }
}

pub trait TypstFuncAppExt {
    fn register_typst_func<F: TypstFunc>(&mut self) -> &mut Self;
}

impl TypstFuncAppExt for App {
    /// Register a [`TypstFunc`] type so that [`VelystFunc<F>`] entities
    /// are compiled into [`VelystContent`] when they change.
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

/// Layout [`VelystContent`] into a [`VelystScene`] in UI coordinates.
fn layout_content(
    world: VelystWorld,
    mut q_contents: Query<
        (
            &VelystContent,
            &mut VelystScene,
            &Visibility,
            &Node,
            &ComputedNode,
            &mut ContentSize,
            &ComputedUiRenderTargetInfo,
        ),
        (
            Or<(
                Changed<VelystContent>,
                Changed<Visibility>,
                Changed<ComputedNode>,
            )>,
            With<UiScene>,
        ),
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

/// Layout [`VelystContent`] into a [`VelystScene`] in world coordinates.
fn layout_world_content(
    world: VelystWorld,
    mut q_contents: Query<
        (
            &VelystContent,
            &mut VelystScene,
            &WorldScene,
            &Visibility,
            &mut Aabb,
        ),
        (
            Or<(
                Changed<VelystContent>,
                Changed<Visibility>,
                Changed<WorldScene>,
            )>,
            With<WorldScene>,
        ),
    >,
) {
    for (content, mut scene, world_scene, viz, mut aabb) in
        q_contents.iter_mut()
    {
        if viz == Visibility::Hidden {
            continue;
        }

        let mut size = Size::splat(Abs::inf());

        if let Some(width) = world_scene.width {
            size.x = Abs::pt(width);
        }
        if let Some(height) = world_scene.height {
            size.y = Abs::pt(height);
        }

        if let Some(frame) = world.layout_frame(
            &content.0,
            Region::new(size, Axes::splat(false)),
        ) {
            scene.0 = TypstScene::from_frame(&frame);

            let Axes { x, y } = frame.size();
            let width = x.to_pt() as f32;
            let height = y.to_pt() as f32;
            let anchor = world_scene.anchor;

            // Bevy_vello flips Y when rendering world scenes, so the scene
            // occupies [0, width] × [0, -height] in local space.
            // Anchor shifts the origin within that rect (normalized 0..1).
            let center = Vec3A::new(
                width * (0.5 - anchor.x),
                height * (anchor.y - 0.5),
                0.0,
            );
            let half_extents =
                Vec3A::new(width / 2.0, height / 2.0, 0.0);
            *aabb = Aabb {
                center,
                half_extents,
            };
        }
    }

    // Clear cache regularly to prevent memory build ups.
    typst::comemo::evict(4);
}

/// Render [`VelystScene`] into a [`UiVelloScene`].
fn render_ui_scene(
    mut q_scenes: Query<
        (&mut VelystScene, &mut UiVelloScene, &Visibility),
        (
            Or<(Changed<VelystScene>, Changed<Visibility>)>,
            With<UiScene>,
        ),
    >,
) {
    for (mut scene, mut vello_scene, viz) in q_scenes.iter_mut() {
        if viz == Visibility::Hidden {
            continue;
        }
        *vello_scene = UiVelloScene::from(scene.render());
    }
}

/// Render [`VelystScene`] into a [`VelloScene2d`].
fn render_world_scene(
    mut q_scenes: Query<
        (&mut VelystScene, &mut VelloScene2d, &Visibility),
        (
            Or<(Changed<VelystScene>, Changed<Visibility>)>,
            With<WorldScene>,
        ),
    >,
) {
    for (mut scene, mut vello_scene, viz) in q_scenes.iter_mut() {
        if viz == Visibility::Hidden {
            continue;
        }
        *vello_scene = VelloScene2d::from(scene.render());
    }
}

/// A Typst function component. Holds the source asset handle and the
/// typed function data. Register it with
/// [`TypstFuncAppExt::register_typst_func`] and add [`UiScene`] or
/// [`WorldScene`] to control which coordinate space this entity renders in.
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

/// Marker component that is inserted when the [module][typst::foundations::Module]
/// needed for this entity's [`VelystFunc`] handle is ready.
///
/// Will be removed when the [module][typst::foundations::Module]
/// needed becomes unavailable again.
#[derive(Component)]
pub struct VelystSourceReady;

#[derive(Component, Default, Deref, DerefMut)]
#[require(VelystScene)]
pub struct VelystContent(pub Content);

/// The laid-out Typst scene, ready to be rendered.
///
/// Add [`UiScene`] or [`WorldScene`] to control which coordinate space
/// this entity renders in.
#[derive(Component, Default, Deref, DerefMut)]
pub struct VelystScene(pub TypstScene);

/// Marker: render this entity's [`VelystScene`] in Bevy UI coordinates.
///
/// Requires [`UiVelloScene`] and [`ContentSize`] which are inserted
/// automatically.
#[derive(Component, Default)]
#[require(VelystScene, UiVelloScene, ContentSize)]
pub struct UiScene;

/// Marker: render this entity's [`VelystScene`] in world coordinates
/// via Bevy's [`Transform`].
///
/// Requires [`VelloScene2d`] which is inserted automatically.
#[derive(Component, Default)]
#[require(VelystScene, VelloScene2d)]
pub struct WorldScene {
    /// Normalized anchor point within the scene (0..1 in each axis).
    /// `(0, 0)` = top-left origin, `(0.5, 0.5)` = center.
    pub anchor: Vec2,
    /// Optional width constraint for Typst layout (in points).
    pub width: Option<f64>,
    /// Optional height constraint for Typst layout (in points).
    pub height: Option<f64>,
}

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
/// use velyst::prelude::*;
/// use bevy::prelude::*;
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
        $crate::renderer::TypstFunc for $struct_name
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

// #[derive(Component, Deref, DerefMut)]
// pub struct TypstLabel(TypLabel);

/// Velyst rendering pipeline.
#[derive(SystemSet, Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum VelystSet {
    /// Custom data preparation before compilation should happen here.
    PrepareFunc,
    /// Compile [`VelystFunc`] into [`VelystContent`].
    ///
    /// One system per registered [`TypstFunc`] type runs here.
    Compile,
    /// Layout [`VelystContent`] into a [`VelystScene`].
    Layout,
    /// Post processing of [`VelystScene`] should happen here.
    PostLayout,
    /// Render [`VelystScene`] into a [`UiVelloScene`] or [`VelloScene2d`].
    Render,
}
