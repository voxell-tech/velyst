use imaging::{FillRef, GeometryRef, PaintSink, StrokeRef};
use peniko::Fill;
use typst_library::visualize as viz;

use crate::RenderState;
use crate::convert::{
    convert_fixed_stroke, convert_geometry, convert_paint, shape_brush_transform,
};

pub(crate) fn render_shape(shape: &viz::Shape, sink: &mut impl PaintSink, state: RenderState) {
    let path = convert_geometry(&shape.geometry);
    let shape_geom = GeometryRef::OwnedPath(path);

    if let Some(paint) = &shape.fill {
        let brush_transform =
            shape_brush_transform(paint, shape, state.container_transform, state.size);
        let (brush, _) = convert_paint(paint, state.size, state.container_transform);
        let fill_rule = match shape.fill_rule {
            viz::FillRule::NonZero => Fill::NonZero,
            viz::FillRule::EvenOdd => Fill::EvenOdd,
        };
        sink.fill(
            FillRef {
                transform: state.transform,
                fill_rule,
                brush: (&brush).into(),
                brush_transform,
                shape: shape_geom.clone(),
                composite: imaging::Composite::default(),
            },
        );
    }

    if let Some(stroke) = &shape.stroke {
        let brush_transform =
            shape_brush_transform(&stroke.paint, shape, state.container_transform, state.size);
        let (brush, _) =
            convert_paint(&stroke.paint, state.size, state.container_transform);
        let style = convert_fixed_stroke(stroke);
        sink.stroke(StrokeRef {
            transform: state.transform,
            stroke: &style,
            brush: (&brush).into(),
            brush_transform,
            shape: shape_geom,
            composite: imaging::Composite::default(),
        });
    }
}
