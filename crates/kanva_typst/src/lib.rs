use kanva::imaging::kurbo::Affine;
use kanva::imaging::peniko::Style;
use kanva::{Group, KanvaClip, KanvaSink};
pub use typst_imaging::RenderState;
use typst_imaging::convert::convert_transform;
use typst_library::layout::{
    Frame, FrameItem, FrameKind, GroupItem, Point, Transform,
};

pub mod image;
pub mod shape;
pub mod text;

/// Walk a Typst [`Frame`] and emit all draw commands into `sink`.
pub fn render_frame(frame: &Frame, sink: &mut impl KanvaSink) {
    let state = RenderState::new(frame.size(), Transform::identity());
    render_items(frame, sink, state);
}

pub fn render_items(
    frame: &Frame,
    sink: &mut impl KanvaSink,
    state: RenderState,
) {
    for (pos, item) in frame.items() {
        match item {
            FrameItem::Group(group) => {
                render_group(group, sink, state, *pos)
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
            FrameItem::Image(img, size, _) => {
                image::render_image(
                    img,
                    *size,
                    sink,
                    state.pre_translate(*pos),
                );
            }
            FrameItem::Link(_, _) | FrameItem::Tag(_) => {}
        }
    }
}

pub fn render_group(
    group: &GroupItem,
    sink: &mut impl KanvaSink,
    state: RenderState,
    pos: Point,
) {
    let group_transform = convert_transform(group.transform);

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
        sink.push_context(&label.resolve());
    }

    let clip = group.clip.as_ref().map(|curve| {
        let path = typst_imaging::convert::convert_curve(curve);
        KanvaClip {
            path,
            transform: state.transform,
            style: Style::Fill(kanva::imaging::peniko::Fill::NonZero),
        }
    });

    sink.push_group(Group {
        clip,
        ..Default::default()
    });
    render_items(&group.frame, sink, state);
    sink.pop_group();

    if group.label.is_some() {
        sink.pop_context();
    }
}
