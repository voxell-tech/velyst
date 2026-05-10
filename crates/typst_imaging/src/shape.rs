use imaging::{
    Composite, FillRef, GeometryRef, PaintSink, StrokeRef,
};
use peniko::Fill;
use typst_library::visualize as viz;

use crate::RenderState;
use crate::convert::{convert_fixed_stroke, convert_geometry};
use crate::paint::shape_paint;

pub(crate) fn render_shape(
    shape: &viz::Shape,
    sink: &mut impl PaintSink,
    state: RenderState,
) {
    let path = convert_geometry(&shape.geometry);
    let shape_geom = GeometryRef::OwnedPath(path);

    if let Some(paint) = &shape.fill {
        let (brush, brush_transform) =
            shape_paint(paint, shape, &state);
        let fill_rule = match shape.fill_rule {
            viz::FillRule::NonZero => Fill::NonZero,
            viz::FillRule::EvenOdd => Fill::EvenOdd,
        };
        sink.fill(FillRef {
            transform: state.transform,
            fill_rule,
            brush: (&brush).into(),
            brush_transform,
            shape: shape_geom.clone(),
            composite: Composite::default(),
        });
    }

    if let Some(stroke) = &shape.stroke {
        let (brush, brush_transform) =
            shape_paint(&stroke.paint, shape, &state);
        let style = convert_fixed_stroke(stroke);
        sink.stroke(StrokeRef {
            transform: state.transform,
            stroke: &style,
            brush: (&brush).into(),
            brush_transform,
            shape: shape_geom,
            composite: Composite::default(),
        });
    }
}
