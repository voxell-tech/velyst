use imaging::{ClipRef, ContextRef, PaintSink};
use peniko::kurbo::Affine;
use typst_library::layout::{
    Frame, FrameItem, FrameKind, GroupItem, Point, Size, Transform,
};

pub mod convert;
pub mod image;
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
        let pos = *pos;
        match item {
            FrameItem::Group(group) => {
                render_group(group, sink, state, pos)
            }
            FrameItem::Text(text) => {
                text::render_text(
                    text,
                    sink,
                    state.pre_translate(pos),
                );
            }
            FrameItem::Shape(shape, _) => {
                shape::render_shape(
                    shape,
                    sink,
                    state.pre_translate(pos),
                );
            }
            FrameItem::Image(image, size, _) => {
                if !size.any(|p| p.to_pt() == 0.0) {
                    image::render_image(
                        image,
                        *size,
                        sink,
                        state.pre_translate(pos),
                    );
                }
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
    let group_affine = convert::convert_transform(
        Transform::translate(pos.x, pos.y)
            .pre_concat(group.transform),
    );

    let state = match group.frame.kind() {
        FrameKind::Soft => state.pre_concat(group_affine),
        FrameKind::Hard => state
            .pre_concat(group_affine)
            .with_container(state.transform)
            .with_size(group.frame.size()),
    };

    if let Some(label) = group.label {
        let resolved = label.resolve();
        sink.push_context(ContextRef::new(&resolved, None));
    }

    if let Some(clip) = &group.clip {
        let clip_path = convert::convert_curve(clip);
        sink.push_clip(
            ClipRef::fill(clip_path).with_transform(state.transform),
        );
    }

    render_items(&group.frame, sink, state);

    if group.clip.is_some() {
        sink.pop_clip();
    }
    if group.label.is_some() {
        sink.pop_context();
    }
}

/// State threaded through the frame walk.
#[derive(Copy, Clone)]
pub(crate) struct RenderState {
    /// Accumulated screen-space transform.
    pub transform: Affine,
    /// Transform at the most recent hard-frame boundary (for gradient `relative: parent`).
    pub container_transform: Affine,
    /// Size of the most recent hard frame.
    pub size: Size,
}

impl RenderState {
    pub fn new(size: Size, transform: Transform) -> Self {
        let affine = convert::convert_transform(transform);
        Self {
            transform: affine,
            container_transform: affine,
            size,
        }
    }

    pub fn pre_translate(self, pos: Point) -> Self {
        self.pre_concat(Affine::translate((
            pos.x.to_pt(),
            pos.y.to_pt(),
        )))
    }

    pub fn pre_concat(self, t: Affine) -> Self {
        Self {
            transform: self.transform * t,
            ..self
        }
    }

    pub fn with_container(self, container: Affine) -> Self {
        Self {
            container_transform: container,
            ..self
        }
    }

    pub fn with_size(self, size: Size) -> Self {
        Self { size, ..self }
    }
}
