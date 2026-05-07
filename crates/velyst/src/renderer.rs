use bevy::camera::primitives::Aabb;
use bevy::prelude::*;
use bevy::ui::ContentSize;
use bevy_vello::prelude::*;
use typst::layout::{Abs, Axes, Region, Size};
use typst_vello::TypstScene;

use crate::func::VelystContent;
use crate::world::VelystWorld;

pub struct VelystRendererPlugin;

impl Plugin for VelystRendererPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            (
                (
                    (layout_content, layout_world_content),
                    comemo_evict,
                )
                    .chain()
                    .in_set(VelystSet::Layout),
                (render_ui_scene, render_world_scene)
                    .in_set(VelystSet::Render),
            ),
        );
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
}

/// Clear cache regularly to prevent memory build ups.
fn comemo_evict() {
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
