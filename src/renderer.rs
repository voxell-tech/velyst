use bevy::prelude::*;
use bevy::ui::UiSystem;
use bevy_vello::prelude::*;
use typst::foundations::{Content, NativeElement, Value};
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
                VelystSet::PreLayout,
                VelystSet::Layout.in_set(UiSystem::PostLayout),
                VelystSet::PostLayout,
                VelystSet::Render,
            )
                .chain(),
        );

        app.add_systems(
            PostUpdate,
            (
                (check_func_ready, compile_func)
                    .chain()
                    .in_set(VelystSet::PreLayout),
                layout_content.in_set(VelystSet::Layout),
                render_scene.in_set(VelystSet::Render),
            ),
        );
    }
}

/// Velyst rendering pipeline.
#[derive(SystemSet, Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum VelystSet {
    /// Compilation of [`Content`] should happen here.
    PreLayout,
    /// Layout [`Content`] into a [`VelystScene`].
    Layout,
    /// Post processing of [`VelystScene`] should happen here.
    PostLayout,
    /// Render [`VelystScene`] into a [`VelloScene`].
    Render,
}

fn compile_func(
    mut commands: Commands,
    mut q_funcs: Query<
        (&VelystFunc, &mut VelystContent, &Visibility, Entity),
        (
            Or<(Changed<VelystFunc>, Changed<Visibility>)>,
            With<VelystFuncReady>,
        ),
    >,
    modules: Res<VelystModules>,
) {
    for (func, mut content, viz, entity) in q_funcs.iter_mut() {
        if viz == Visibility::Hidden {
            continue;
        }

        if let Some(module) = modules.get(&func.handle.id()) {
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
            commands.entity(entity).remove::<VelystFuncReady>();
        }
    }
}

fn check_func_ready(
    mut commands: Commands,
    mut q_funcs: Query<(&VelystFunc, Entity), Without<VelystFuncReady>>,
    modules: Res<VelystModules>,
) {
    for (func, entity) in q_funcs.iter_mut() {
        if modules.contains_key(&func.handle.id()) {
            commands.entity(entity).insert(VelystFuncReady);
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
    pub handle: Handle<VelystSource>,
    pub name: &'static str,
    pub positional_args: Vec<Value>,
    pub named_args: Vec<(&'static str, Value)>,
}

/// Marker component that is inserted when the [module][typst::foundations::Module]
/// needed from [`VelystModules`] for the [`VelystFunc`] is ready.
///
/// Will be removed when the [module][typst::foundations::Module]
/// needed becomes unavailable again.
#[derive(Component)]
pub struct VelystFuncReady;

#[derive(Component, Default, Deref, DerefMut)]
#[require(VelystScene)]
pub struct VelystContent(pub Content);

#[derive(Component, Default, Deref, DerefMut)]
#[require(VelloScene)]
pub struct VelystScene(pub TypstScene);

// #[derive(Component, Deref, DerefMut)]
// pub struct TypstLabel(TypLabel);
