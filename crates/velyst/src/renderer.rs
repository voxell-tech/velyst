use bevy::camera::primitives::Aabb;
use bevy::prelude::*;
use bevy::ui::ContentSize;
use bevy_vello::prelude::*;
use imaging_vello::VelloSceneSink;
use imaging_vello::vello;
use kanva::prelude::*;
use typst::layout::{Abs, Axes, Frame, Region, Size};
use vello::Scene;
use vello::peniko::kurbo::{Affine, Rect};

use crate::VelystSet;
use crate::func::VelystContent;
use crate::world::VelystWorld;

pub struct VelystRendererPlugin;

impl Plugin for VelystRendererPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            (
                (
                    (layout_ui_content, layout_world_content),
                    comemo_evict,
                    build_kanva_scene,
                )
                    .chain()
                    .in_set(VelystSet::Layout),
                (
                    render_ui_scene,
                    render_world_scene,
                    render_ui_kanva,
                    render_world_kanva,
                )
                    .in_set(VelystSet::Render),
            ),
        );
    }
}

/// Layout [`VelystContent`] into a [`VelystFrame`] in UI coordinates.
fn layout_ui_content(
    world: VelystWorld,
    mut q_contents: Query<
        (
            &VelystContent,
            &mut VelystFrame,
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
            let frame_size = frame.size();
            let size = Vec2::new(
                frame_size.x.to_pt() as f32,
                frame_size.y.to_pt() as f32,
            ) * scale_factor;
            *content_size = ContentSize::fixed_size(size);
            scene.0 = Some(frame);
        }
    }
}

/// Layout [`VelystContent`] into a [`VelystFrame`] in world
/// coordinates.
fn layout_world_content(
    world: VelystWorld,
    mut q_contents: Query<
        (
            &VelystContent,
            &mut VelystFrame,
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
            let frame_size = frame.size();
            let width = frame_size.x.to_pt() as f32;
            let height = frame_size.y.to_pt() as f32;
            let anchor = world_scene.anchor;

            // Bevy_vello flips Y when rendering world scenes, so the
            // scene occupies [0, width] × [0, -height] in
            // local space. Anchor shifts the origin
            // within that rect (normalized 0..1).
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
            scene.0 = Some(frame);
        }
    }
}

/// Clear cache regularly to prevent memory build ups.
fn comemo_evict() {
    typst::comemo::evict(4);
}

/// Render [`VelystFrame`] into a [`UiVelloScene`].
fn render_ui_scene(
    mut q_scenes: Query<
        (&VelystFrame, &mut UiVelloScene, &Visibility),
        (
            Or<(Changed<VelystFrame>, Changed<Visibility>)>,
            With<UiScene>,
            Without<VelystKanva>,
        ),
    >,
) {
    for (scene, mut vello_scene, viz) in q_scenes.iter_mut() {
        if viz == Visibility::Hidden {
            continue;
        }
        let Some(frame) = &scene.0 else { continue };
        *vello_scene =
            UiVelloScene::from(frame_to_scene(frame, Vec2::ZERO));
    }
}

/// Render [`VelystFrame`] into a [`VelloScene2d`].
fn render_world_scene(
    mut q_scenes: Query<
        (&VelystFrame, &WorldScene, &mut VelloScene2d, &Visibility),
        (
            Or<(Changed<VelystFrame>, Changed<Visibility>)>,
            With<WorldScene>,
            Without<VelystKanva>,
        ),
    >,
) {
    for (scene, world_scene, mut vello_scene, viz) in
        q_scenes.iter_mut()
    {
        if viz == Visibility::Hidden {
            continue;
        }
        let Some(frame) = &scene.0 else { continue };
        *vello_scene = VelloScene2d::from(frame_to_scene(
            frame,
            world_scene.anchor,
        ));
    }
}

/// Build a [`VelystKanva`] from the laid-out [`VelystFrame`] frame.
fn build_kanva_scene(
    mut q_scenes: Query<
        (&VelystFrame, &mut VelystKanva),
        Changed<VelystFrame>,
    >,
) {
    for (scene, mut kanva) in q_scenes.iter_mut() {
        let Some(frame) = &scene.0 else { continue };
        let mut builder = KanvaBuilder::new();
        kanva_typst::render_frame(frame, &mut builder);
        kanva.0 = builder.build();
    }
}

/// Render [`VelystKanva`] into a [`UiVelloScene`].
fn render_ui_kanva(
    mut q_scenes: Query<
        (&VelystKanva, &VelystFrame, &mut UiVelloScene, &Visibility),
        (
            Or<(Changed<VelystKanva>, Changed<Visibility>)>,
            With<UiScene>,
        ),
    >,
) {
    for (kanva, scene, mut vello_scene, viz) in q_scenes.iter_mut() {
        if viz == Visibility::Hidden {
            continue;
        }
        let Some(frame) = &scene.0 else { continue };
        *vello_scene = UiVelloScene::from(kanva_to_scene(
            &kanva.0,
            frame,
            Vec2::ZERO,
        ));
    }
}

/// Render [`VelystKanva`] into a [`VelloScene2d`].
fn render_world_kanva(
    mut q_scenes: Query<
        (
            &VelystKanva,
            &VelystFrame,
            &WorldScene,
            &mut VelloScene2d,
            &Visibility,
        ),
        (
            Or<(Changed<VelystKanva>, Changed<Visibility>)>,
            With<WorldScene>,
        ),
    >,
) {
    for (kanva, scene, world_scene, mut vello_scene, viz) in
        q_scenes.iter_mut()
    {
        if viz == Visibility::Hidden {
            continue;
        }
        let Some(frame) = &scene.0 else { continue };
        *vello_scene = VelloScene2d::from(kanva_to_scene(
            &kanva.0,
            frame,
            world_scene.anchor,
        ));
    }
}

fn kanva_to_scene(
    kanva: &Kanva,
    frame: &Frame,
    anchor: Vec2,
) -> Scene {
    let frame_size = frame.size();
    let w = frame_size.x.to_pt();
    let h = frame_size.y.to_pt();

    let surface_clip = Rect::new(0.0, 0.0, w, h);

    let mut inner = Scene::new();
    let mut sink = VelloSceneSink::new(&mut inner, surface_clip);
    kanva.render(&mut sink);
    let _ = sink.finish();

    let mut scene = Scene::new();
    scene.append(
        &inner,
        Some(Affine::translate((
            -w * anchor.x as f64,
            -h * anchor.y as f64,
        ))),
    );
    scene
}

fn frame_to_scene(frame: &Frame, anchor: Vec2) -> Scene {
    let frame_size = frame.size();
    let w = frame_size.x.to_pt();
    let h = frame_size.y.to_pt();

    let surface_clip = Rect::new(0.0, 0.0, w, h);

    let mut inner = Scene::new();
    let mut sink = VelloSceneSink::new(&mut inner, surface_clip);
    typst_imaging::render_frame(frame, &mut sink);
    let _ = sink.finish();

    let mut scene = Scene::new();
    scene.append(
        &inner,
        Some(Affine::translate((
            -w * anchor.x as f64,
            -h * anchor.y as f64,
        ))),
    );
    scene
}

/// The laid-out Typst frame, ready to be rendered.
///
/// Add [`UiScene`] or [`WorldScene`] to control which coordinate
/// space this entity renders in.
#[derive(Component, Default)]
pub struct VelystFrame(pub Option<Frame>);

/// Stores a [`Kanva`] built from the last laid-out Typst frame.
///
/// Add this alongside [`UiScene`] or [`WorldScene`] to opt into kanva
/// rendering. Mutate `path_mods` / `group_mods` each frame via their
/// builder methods and mark this component changed to trigger a
/// re-render without a Typst recompile.
#[derive(Component, Default, Deref, DerefMut)]
pub struct VelystKanva(pub Kanva);

/// Marker: render this entity's [`VelystFrame`] in Bevy UI
/// coordinates.
///
/// Requires [`UiVelloScene`] and [`ContentSize`] which are inserted
/// automatically.
#[derive(Component, Default)]
#[require(VelystFrame, UiVelloScene, ContentSize)]
pub struct UiScene;

/// Marker: render this entity's [`VelystFrame`] in world coordinates
/// via Bevy's [`Transform`].
///
/// Requires [`VelloScene2d`] which is inserted automatically.
#[derive(Component, Default)]
#[require(VelystFrame, VelloScene2d)]
pub struct WorldScene {
    /// Normalized anchor point within the scene (0..1 in each axis).
    /// `(0, 0)` = top-left origin, `(0.5, 0.5)` = center.
    pub anchor: Vec2,
    /// Optional width constraint for Typst layout (in points).
    pub width: Option<f64>,
    /// Optional height constraint for Typst layout (in points).
    pub height: Option<f64>,
}

impl WorldScene {
    pub fn with_anchor(mut self, anchor: Vec2) -> Self {
        self.anchor = anchor;
        self
    }

    pub fn with_width(mut self, width: impl Into<f64>) -> Self {
        self.width = Some(width.into());
        self
    }

    pub fn with_height(mut self, height: impl Into<f64>) -> Self {
        self.height = Some(height.into());
        self
    }
}
