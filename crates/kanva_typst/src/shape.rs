use kanva::imaging::Composite;
use kanva::imaging::peniko::Fill;
use kanva::prelude::*;
use typst_imaging::RenderState;
use typst_imaging::convert::{
    convert_fixed_stroke, convert_geometry,
};
use typst_imaging::paint::shape_paint;
use typst_library::visualize as viz;

pub fn render_shape(
    shape: &viz::Shape,
    sink: &mut impl KanvaSink,
    state: RenderState,
) {
    let path = convert_geometry(&shape.geometry);

    let fill = shape.fill.as_ref().map(|paint| {
        let (brush, brush_transform) =
            shape_paint(paint, shape, &state);
        KanvaFill {
            rule: match shape.fill_rule {
                viz::FillRule::NonZero => Fill::NonZero,
                viz::FillRule::EvenOdd => Fill::EvenOdd,
            },
            brush,
            brush_transform,
            composite: Composite::default(),
        }
    });

    let stroke = shape.stroke.as_ref().map(|s| {
        let (brush, brush_transform) =
            shape_paint(&s.paint, shape, &state);
        KanvaStroke {
            stroke: convert_fixed_stroke(s),
            brush,
            brush_transform,
            composite: Composite::default(),
        }
    });

    sink.draw_path(
        path,
        state.transform,
        fill,
        stroke,
        Default::default(),
    );
}
