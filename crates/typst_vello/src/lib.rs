//! # Typst Vello
//!
//! A Vello scene drawer for Typst's frames.

pub use typst;

use ahash::AHashMap;
use image::{render_image, ImageScene};
use shape::{convert_path, render_shape, ShapeScene};
use smallvec::SmallVec;
use text::{render_text, TextScene};
use typst::foundations::Label;
use typst::layout::{Frame, FrameItem, FrameKind, GroupItem, Point, Size, Transform};
use utils::convert_transform;
use vello::kurbo::Shape;
use vello::{kurbo, peniko};

pub mod image;
pub mod shape;
pub mod text;
pub mod utils;

/// Every group is layouted in a flat Vec.
/// Each group will have a parent index associated with it.
#[derive(Default)]
pub struct TypstScene {
    size: kurbo::Vec2,
    group_scenes: Vec<TypstGroupScene>,
    group_map: AHashMap<Label, SmallVec<[usize; 1]>>,
    pub post_process_map: AHashMap<Label, PostProcess>,
}

impl TypstScene {
    pub fn from_frame(frame: &Frame) -> Self {
        let size = kurbo::Vec2::new(frame.width().to_pt(), frame.height().to_pt());
        let mut typst_scene = TypstScene {
            size,
            ..Default::default()
        };

        let group_paths = TypstGroup {
            size,
            ..Default::default()
        };
        typst_scene.append_group(group_paths);
        typst_scene.handle_frame(
            frame,
            RenderState::new(frame.size(), Transform::identity()),
            0,
        );

        typst_scene
    }

    pub fn update_frame(&mut self, frame: &Frame) {
        let size = kurbo::Vec2::new(frame.width().to_pt(), frame.height().to_pt());

        self.group_scenes.clear();
        self.group_map.clear();
        self.size = size;

        let group_paths = TypstGroup {
            size,
            ..Default::default()
        };
        self.append_group(group_paths);
        self.handle_frame(
            frame,
            RenderState::new(frame.size(), Transform::identity()),
            0,
        );
    }

    /// Render [`TypstScene`] into a [`vello::Scene`].
    pub fn render(&mut self) -> vello::Scene {
        let mut scene = vello::Scene::new();
        let mut computed_transforms = Vec::with_capacity(self.group_scenes.len());

        let mut layers = Vec::new();

        for (i, group_scene) in self.group_scenes.iter_mut().enumerate() {
            let group = &mut group_scene.group;

            let post_process = group
                .label
                .as_ref()
                .and_then(|label| self.post_process_map.get(label));

            let local_transform = post_process
                .map(|p| group.transform * p.transform.unwrap_or_default())
                .unwrap_or(group.transform);
            // Calculate accumulated transform from the group hierarchy.
            let transform = match group.parent {
                Some(parent_index) => {
                    let transform = computed_transforms[parent_index] * local_transform;
                    computed_transforms.push(transform);
                    transform
                }
                None => {
                    let transform = local_transform;
                    computed_transforms.push(transform);
                    transform
                }
            };
            let transform = (transform != kurbo::Affine::IDENTITY).then_some(transform);

            if let (Some(&last_layer), Some(parent_index)) = (layers.last(), group.parent) {
                if last_layer > parent_index {
                    scene.pop_layer();
                    layers.pop();
                }
            }

            if let Some(layer) = post_process
                .and_then(|p| p.layer.as_ref())
                .or(group.layer())
            {
                scene.push_layer(
                    layer.blend_mode,
                    layer.alpha,
                    transform.unwrap_or_default(),
                    layer.clip_path.as_ref().unwrap_or(
                        &kurbo::Rect::new(0.0, 0.0, group.size.x, group.size.y).to_path(0.1),
                    ),
                );

                layers.push(i);
            }

            if group_scene.updated {
                // Use the rendered group scene.
                scene.append(&group_scene.scene, transform);
            } else {
                // Scene needs to be re-rendered if it's not updated.
                let new_scene = group.render(&EmptySceneProcessor);
                scene.append(&new_scene, transform);
                // Update group scene to the newly rendered scene.
                group_scene.scene = new_scene;
            }

            // Flag the current group scene as updated.
            group_scene.updated = true;
        }

        // Pop the last layer if there is any left.
        if layers.len() > 0 {
            scene.pop_layer();
            layers.pop();
        }

        debug_assert!(layers.len() == 0);

        scene
    }

    /// Populate [`TypstGroup`] with items inside the [`Frame`] and recursively
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
                    let scenes = &mut self.get_group_mut(group_index).scenes;
                    scenes.push(SceneKind::Text(render_text(
                        text,
                        state.pre_translate(pos),
                        (local_transform.is_identity() == false).then_some(local_transform),
                    )));
                }
                FrameItem::Shape(shape, _) => {
                    let scenes = &mut self.get_group_mut(group_index).scenes;
                    scenes.push(SceneKind::Shape(render_shape(
                        shape,
                        state.pre_translate(pos),
                        local_transform,
                    )));
                }
                FrameItem::Image(image, size, _) => {
                    if size.any(|p| p.to_pt() == 0.0) {
                        // Image size invalid!
                        continue;
                    }

                    let scenes = &mut self.get_group_mut(group_index).scenes;
                    scenes.push(SceneKind::Image(render_image(
                        image,
                        *size,
                        local_transform,
                    )));
                }
                // TODO: Support links
                FrameItem::Link(_, _) => {}
                FrameItem::Tag(_) => {}
            }
        }
    }

    /// Convert [`GroupItem`] into [`TypstGroup`] and append it.
    fn handle_group(
        &mut self,
        group: &GroupItem,
        state: RenderState,
        local_transform: Transform,
        parent: Option<usize>,
    ) {
        // Generate TypstGroup for the underlying frame.
        let group_paths = TypstGroup {
            size: kurbo::Vec2::new(group.frame.width().to_pt(), group.frame.height().to_pt()),
            transform: convert_transform(local_transform.pre_concat(group.transform)),
            parent,
            layer: group.clip_path.as_ref().map(|path| Layer {
                clip_path: Some(convert_path(path)),
                ..Default::default()
            }),
            label: group.label,
            ..Default::default()
        };

        // Update state based on group frame.
        let state = match group.frame.kind() {
            FrameKind::Soft => state.pre_concat(group.transform),
            FrameKind::Hard => state
                .with_transform(Transform::identity())
                .with_size(group.frame.size()),
        };

        let group_index = self.group_scenes.len();
        self.append_group(group_paths);
        self.handle_frame(&group.frame, state, group_index);
    }

    /// Add a group to the [group list][Self::group_scenes].
    fn append_group(&mut self, group: TypstGroup) {
        if let Some(label) = group.label {
            let index = self.group_scenes.len();
            match self.group_map.get_mut(&label) {
                Some(map) => {
                    map.push(index);
                }
                None => {
                    self.group_map.insert(label, SmallVec::from_buf([index]));
                }
            }
        }
        self.group_scenes.push(TypstGroupScene::new(group));
    }
}

impl TypstScene {
    pub fn query(&self, label: Label) -> Option<&[usize]> {
        self.group_map.get(&label).map(|indices| indices.as_slice())
    }

    pub fn get_group(&self, index: usize) -> &TypstGroup {
        &self.group_scenes[index].group
    }

    pub fn get_group_mut(&mut self, index: usize) -> &mut TypstGroup {
        self.group_scenes[index].updated = false;
        &mut self.group_scenes[index].group
    }

    pub fn iter_groups(&self) -> impl Iterator<Item = &TypstGroup> {
        self.group_scenes
            .iter()
            .map(|group_scene| &group_scene.group)
    }

    pub fn iter_groups_mut(&mut self) -> impl Iterator<Item = &mut TypstGroup> {
        self.group_scenes
            .iter_mut()
            .map(|group_scene| &mut group_scene.group)
    }

    /// Number of groups in the scene.
    pub fn groups_len(&self) -> usize {
        self.group_scenes.len()
    }

    /// Width and height of the entire scene.
    pub fn size(&self) -> kurbo::Vec2 {
        self.size
    }
}

#[derive(Default)]
pub struct TypstGroupScene {
    group: TypstGroup,
    scene: vello::Scene,
    updated: bool,
}

impl TypstGroupScene {
    pub fn new(group: TypstGroup) -> Self {
        Self {
            group,
            ..Default::default()
        }
    }
}

#[derive(Default, Debug)]
pub struct TypstGroup {
    size: kurbo::Vec2,
    transform: kurbo::Affine,
    scenes: Vec<SceneKind>,
    parent: Option<usize>,
    layer: Option<Layer>,
    label: Option<Label>,
}

impl TypstGroup {
    /// Create [`TypstGroup`] from a single [`SceneKind`].
    pub fn from_scene(scene: SceneKind, parent: Option<usize>) -> Self {
        Self {
            scenes: vec![scene],
            parent,
            ..Default::default()
        }
    }

    pub fn render(&self, scene_processor: &impl SceneProcesser) -> vello::Scene {
        let mut vello_scene = vello::Scene::new();

        for (i, scene) in self.scenes.iter().enumerate() {
            vello_scene.append(&scene_processor.process_scene(i, scene), None);
        }

        vello_scene
    }
}

// Getters
impl TypstGroup {
    pub fn size(&self) -> kurbo::Vec2 {
        self.size
    }

    pub fn transform(&self) -> kurbo::Affine {
        self.transform
    }

    pub fn parent(&self) -> Option<usize> {
        self.parent
    }

    pub fn layer(&self) -> Option<&Layer> {
        self.layer.as_ref()
    }

    pub fn label(&self) -> Option<Label> {
        self.label
    }
}

pub struct PostProcess {
    /// Transform override.
    pub transform: Option<kurbo::Affine>,
    /// Layer override.
    pub layer: Option<Layer>,
    /// Post process for scenes.
    pub scene_processor: Box<dyn SceneProcesser>,
}

impl Default for PostProcess {
    fn default() -> Self {
        Self {
            transform: None,
            layer: None,
            scene_processor: Box::new(EmptySceneProcessor),
        }
    }
}

impl std::fmt::Debug for PostProcess {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PostProcess")
            .field("transform", &self.transform)
            .field("layer", &self.layer)
            .finish()
    }
}

/// Render [`SceneKind`] as usual without any post processing.
pub struct EmptySceneProcessor;

impl SceneProcesser for EmptySceneProcessor {
    fn process_scene(&self, _scene_index: usize, scene: &SceneKind) -> vello::Scene {
        let mut vello_scene = vello::Scene::new();
        scene.render(&mut vello_scene);

        vello_scene
    }
}

pub trait SceneProcesser: Send + Sync + 'static {
    /// Process a scene.
    fn process_scene(&self, scene_index: usize, scene: &SceneKind) -> vello::Scene;
}

#[derive(Debug, Clone)]
pub struct Layer {
    pub blend_mode: peniko::BlendMode,
    pub alpha: f32,
    pub clip_path: Option<kurbo::BezPath>,
}

impl Default for Layer {
    fn default() -> Self {
        Self {
            blend_mode: peniko::BlendMode::new(peniko::Mix::Normal, peniko::Compose::SrcOver),
            alpha: 1.0,
            clip_path: None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum SceneKind {
    Shape(ShapeScene),
    Text(TextScene),
    Image(ImageScene),
}

impl SceneKind {
    pub fn render(&self, scene: &mut vello::Scene) {
        match self {
            SceneKind::Shape(shape) => shape.render(scene),
            SceneKind::Text(text) => text.render(scene),
            SceneKind::Image(image) => image.render(scene),
        };
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
