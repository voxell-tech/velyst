use imaging::{ClipRef, ContextRef, GroupRef, PaintSink};
use peniko::kurbo::{Affine, Vec2};
use typst_library::layout::{
    Frame, FrameItem, FrameKind, GroupItem, Point, Size, Transform,
};

pub mod convert;
pub mod image;
pub mod paint;
pub mod shape;
pub mod text;

/// Walk a Typst [`Frame`] and emit all draw commands into `sink`.
pub fn render_frame(frame: &Frame, sink: &mut impl PaintSink) {
    let state = RenderState::new(frame.size(), Transform::identity());
    render_items(frame, sink, state);
}

fn render_items(
    frame: &Frame,
    sink: &mut impl PaintSink,
    state: RenderState,
) {
    for (pos, item) in frame.items() {
        match item {
            FrameItem::Group(group) => {
                render_group(group, sink, state, *pos);
            }
            FrameItem::Text(text) => {
                text::render_text(
                    text,
                    sink,
                    state.pre_translate(*pos),
                );
            }
            FrameItem::Shape(shape, _) => {
                shape::render_shape(
                    shape,
                    sink,
                    state.pre_translate(*pos),
                );
            }
            FrameItem::Image(image, size, _) => {
                image::render_image(
                    image,
                    *size,
                    sink,
                    state.pre_translate(*pos),
                );
            }
            FrameItem::Link(_, _) | FrameItem::Tag(_) => {}
        }
    }
}

fn render_group(
    group: &GroupItem,
    sink: &mut impl PaintSink,
    state: RenderState,
    pos: Point,
) {
    let group_transform = convert::convert_transform(group.transform);

    let state = match group.frame.kind() {
        FrameKind::Soft => {
            state.pre_translate(pos).pre_concat(group_transform)
        }
        FrameKind::Hard => state
            .pre_translate(pos)
            .pre_concat(group_transform)
            .pre_concat_container(
                state.container_transform.inverse() * state.transform,
            )
            .pre_concat_container(Affine::translate((
                pos.x.to_pt(),
                pos.y.to_pt(),
            )))
            .pre_concat_container(group_transform)
            .with_size(group.frame.size()),
    };

    if let Some(label) = group.label {
        let resolved = label.resolve();
        sink.push_context(ContextRef::new(&resolved, None));
    }

    let mut group_ref = GroupRef::new();
    if let Some(clip) = &group.clip {
        let clip_path = convert::convert_curve(clip);
        group_ref = group_ref.with_clip(
            ClipRef::fill(clip_path).with_transform(state.transform),
        );
    }
    sink.push_group(group_ref);

    render_items(&group.frame, sink, state);

    sink.pop_group();
    if group.label.is_some() {
        sink.pop_context();
    }
}

/// State threaded through the frame walk.
#[derive(Copy, Clone)]
pub(crate) struct RenderState {
    /// Accumulated screen-space transform.
    pub transform: Affine,
    /// Transform at the most recent hard-frame boundary.
    pub container_transform: Affine,
    /// Size of the most recent hard frame.
    pub container_size: Size,
}

impl RenderState {
    pub fn new(size: Size, transform: Transform) -> Self {
        let affine = convert::convert_transform(transform);
        Self {
            transform: affine,
            container_transform: affine,
            container_size: size,
        }
    }

    /// `pos` is applied before the current transform.
    pub fn pre_translate(self, pos: Point) -> Self {
        Self {
            transform: self.transform.pre_translate(Vec2::new(
                pos.x.to_pt(),
                pos.y.to_pt(),
            )),
            ..self
        }
    }

    /// `transform` is applied before the current transform.
    pub fn pre_concat(self, transform: Affine) -> Self {
        Self {
            transform: self.transform * transform,
            ..self
        }
    }

    /// Sets the size of the most recent hard frame.
    pub fn with_size(self, size: Size) -> Self {
        Self {
            container_size: size,
            ..self
        }
    }

    /// `transform` is applied before the current container transform.
    fn pre_concat_container(self, transform: Affine) -> Self {
        Self {
            container_transform: self.container_transform * transform,
            ..self
        }
    }
}
