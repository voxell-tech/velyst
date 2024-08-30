//! # Typst Vello
//!
//! A Vello scene drawer for Typst's frames.

use bevy_utils::{default, HashMap};
use image::{render_image, ImageScene};
use shape::{convert_path, render_shape, ShapeScene};
use text::render_text;
use typst::{
    foundations::Label,
    layout::{Frame, FrameItem, FrameKind, GroupItem, Point, Size, Transform},
};
use utils::convert_transform;
use vello::{kurbo, peniko};

pub mod image;
pub mod shape;
pub mod text;
pub mod utils;

/// Every group is layouted in a flat list.
/// Each group will have a parent index associated with it.
#[derive(Default)]
pub struct TypstScene {
    pub groups: Vec<TypstGroup>,
    pub group_scenes: Vec<vello::Scene>,
    pub group_map: HashMap<Label, Vec<usize>>,
}

impl TypstScene {
    pub fn render(&self) -> vello::Scene {
        let mut scene = vello::Scene::new();
        let mut computed_transforms = Vec::with_capacity(self.groups.len());

        for group in self.groups.iter() {
            let transform = match group.parent {
                Some(parent_index) => {
                    let transform = computed_transforms[parent_index] * group.transform;
                    computed_transforms.push(transform);
                    transform
                }
                None => {
                    computed_transforms.push(group.transform);
                    group.transform
                }
            };

            let mut pushed_clip = false;
            if let Some(clip_path) = &group.clip_path {
                scene.push_layer(
                    peniko::BlendMode::new(peniko::Mix::Clip, peniko::Compose::SrcOver),
                    1.0,
                    group.transform,
                    clip_path,
                );
                pushed_clip = true;
            }

            scene.append(
                &group.render(),
                (transform != kurbo::Affine::IDENTITY).then_some(transform),
            );

            if pushed_clip {
                scene.pop_layer();
            }
        }

        scene
    }

    pub fn from_frame(frame: &Frame) -> Self {
        let mut typst_scene = TypstScene::default();

        let group_paths = TypstGroup::default();
        typst_scene.append_group(group_paths);
        typst_scene.handle_frame(
            frame,
            RenderState::new(frame.size(), Transform::identity()),
            0,
        );

        typst_scene
    }

    /// Populate [`GroupPaths`] with items inside the [`Frame`] and recursively
    /// populate the [`TypstScene`] itself if the frame contains any groups.
    fn handle_frame(&mut self, frame: &Frame, state: RenderState, group_index: usize) {
        for (pos, item) in frame.items() {
            let pos = *pos;
            let local_transform = Transform::translate(pos.x, pos.y);

            match item {
                FrameItem::Group(group) => {
                    self.handle_group(
                        group,
                        state.pre_translate(pos),
                        local_transform,
                        Some(group_index),
                    );
                }
                FrameItem::Text(text) => {
                    let shapes = &mut self.groups[group_index].shapes;
                    shapes.extend(render_text(text, state.pre_translate(pos), local_transform));
                }
                FrameItem::Shape(shape, _) => {
                    let shapes = &mut self.groups[group_index].shapes;
                    shapes.push(render_shape(
                        shape,
                        state.pre_translate(pos),
                        local_transform,
                    ));
                }
                FrameItem::Image(image, size, _) => {
                    if size.any(|p| p.to_pt() == 0.0) {
                        // Image size invalid!
                        continue;
                    }

                    let images = &mut self.groups[group_index].images;
                    let image = render_image(image, *size, local_transform);
                    images.push(image);
                }
                // TODO: Support links
                FrameItem::Link(_, _) => {}
                FrameItem::Tag(_) => {}
            }
        }
    }

    /// Convert [`GroupItem`] into [`GroupPaths`] and append it.
    fn handle_group(
        &mut self,
        group: &GroupItem,
        state: RenderState,
        local_transform: Transform,
        parent: Option<usize>,
    ) {
        // Generate GroupPaths for the underlying frame.
        let group_paths = TypstGroup {
            transform: convert_transform(local_transform.pre_concat(group.transform)),
            parent,
            clip_path: group.clip_path.as_ref().map(convert_path),
            label: group.label,
            ..default()
        };

        // Update state based on group frame.
        let state = match group.frame.kind() {
            FrameKind::Soft => state.pre_concat(group.transform),
            FrameKind::Hard => state
                .with_transform(Transform::identity())
                .with_size(group.frame.size()),
        };

        let group_index = self.groups.len();
        self.append_group(group_paths);
        self.handle_frame(&group.frame, state, group_index);
    }

    /// Add a group to the [group list][Self::groups].
    fn append_group(&mut self, group: TypstGroup) {
        if let Some(label) = group.label {
            let index = self.groups.len();
            match self.group_map.get_mut(&label) {
                Some(map) => {
                    map.push(index);
                }
                None => {
                    self.group_map.insert(label, vec![index]);
                }
            }
        }
        self.groups.push(group);
    }
}

#[derive(Default, Debug)]
pub struct TypstGroup {
    pub transform: kurbo::Affine,
    pub shapes: Vec<ShapeScene>,
    pub images: Vec<ImageScene>,
    pub parent: Option<usize>,
    pub clip_path: Option<kurbo::BezPath>,
    pub label: Option<Label>,
}

impl TypstGroup {
    /// Create [`GroupPaths`] from a single [`ShapeScene`].
    pub fn from_shape_scene(shape_scene: ShapeScene, parent: Option<usize>) -> Self {
        Self {
            shapes: vec![shape_scene],
            parent,
            ..default()
        }
    }

    pub fn render(&self) -> vello::Scene {
        let mut scene = vello::Scene::new();

        for shape in self.shapes.iter() {
            shape.render(&mut scene);
        }

        for fixed_scene in self.images.iter() {
            scene.append(
                &fixed_scene.scene,
                (fixed_scene.transform != kurbo::Affine::IDENTITY).then_some(fixed_scene.transform),
            )
        }

        scene
    }
}

/// Contextual information for rendering.
#[derive(Default, Debug, Clone, Copy)]
pub struct RenderState {
    /// The transform of the current item.
    transform: Transform,
    /// The size of the first hard frame in the hierarchy.
    size: Size,
}

impl RenderState {
    pub fn new(size: Size, transform: Transform) -> Self {
        Self { size, transform }
    }

    /// Pre translate the current item's transform.
    pub fn pre_translate(self, pos: Point) -> Self {
        self.pre_concat(Transform::translate(pos.x, pos.y))
    }

    /// Pre concat the current item's transform.
    pub fn pre_concat(self, transform: Transform) -> Self {
        Self {
            transform: self.transform.pre_concat(transform),
            ..self
        }
    }

    /// Sets the size of the first hard frame in the hierarchy.
    pub fn with_size(self, size: Size) -> Self {
        Self { size, ..self }
    }

    /// Sets the current item's transform.
    pub fn with_transform(self, transform: Transform) -> Self {
        Self { transform, ..self }
    }
}
