use bevy_vello_graphics::{
    bevy_vello::vello::{kurbo, peniko},
    brush::Brush,
    fill::Fill,
    stroke::Stroke,
};
use ttf_parser::{GlyphId, OutlineBuilder};
use typst::{
    layout::{Abs, Point, Ratio, Size, Transform},
    text::{Font, TextItem},
    visualize as viz,
};

use crate::{
    shape::ShapeScene,
    utils::{convert_fixed_stroke, convert_paint_to_brush, convert_transform},
    RenderState,
};

pub fn render_text(
    text: &TextItem,
    state: RenderState,
    local_transform: Transform,
) -> Vec<ShapeScene> {
    let mut shape_scenes = Vec::new();
    let scale = text.size.to_pt() / text.font.units_per_em();

    let mut x = 0.0;
    for glyph in &text.glyphs {
        let id = GlyphId(glyph.id);
        let offset = x + glyph.x_offset.at(text.size).to_pt();

        let offset_transform = Transform::translate(Abs::pt(offset), Abs::zero())
            .pre_concat(Transform::scale(Ratio::one(), -Ratio::one()));

        let shape_scene = render_outline_glyph(
            text,
            state.pre_concat(offset_transform),
            id,
            scale,
            local_transform.pre_concat(offset_transform),
        );

        if let Some(shape_scene) = shape_scene {
            shape_scenes.push(shape_scene);
        }

        x += glyph.x_advance.at(text.size).to_pt();
    }

    shape_scenes
}

fn render_outline_glyph(
    text: &TextItem,
    state: RenderState,
    glyph_id: GlyphId,
    scale: f64,
    local_transform: Transform,
) -> Option<ShapeScene> {
    let scale = Ratio::new(scale);

    let glyph_size = text.font.ttf().glyph_bounding_box(glyph_id)?;
    let width = glyph_size.width() as f64 * scale.get();
    let height = glyph_size.height() as f64 * scale.get();

    let brush_size = Size::new(Abs::pt(width), Abs::pt(height));

    Some(ShapeScene {
        transform: convert_transform(local_transform),
        path: convert_outline_glyph_to_path(&text.font, glyph_id, scale)?,
        fill: {
            let transform = convert_transform(text_paint_transform(state, &text.fill));
            let brush = convert_paint_to_brush(&text.fill, brush_size);

            Some(Fill {
                style: peniko::Fill::NonZero,
                brush: Brush::from_brush(brush).with_transform(transform),
            })
        },
        stroke: text.stroke.as_ref().map(|stroke| {
            let transform = convert_transform(text_paint_transform(state, &stroke.paint));
            let brush = convert_paint_to_brush(&stroke.paint, brush_size);

            Stroke {
                style: convert_fixed_stroke(stroke),
                brush: Brush::from_brush(brush).with_transform(transform),
            }
        }),
    })
}

fn text_paint_transform(state: RenderState, paint: &viz::Paint) -> Transform {
    match paint {
        viz::Paint::Solid(_) => Transform::identity(),
        viz::Paint::Gradient(gradient) => match gradient.unwrap_relative(true) {
            viz::RelativeTo::Self_ => Transform::identity(),
            viz::RelativeTo::Parent => Transform::scale(
                Ratio::new(state.size.x.to_pt()),
                Ratio::new(state.size.y.to_pt()),
            )
            .post_concat(state.transform.invert().unwrap()),
        },
        viz::Paint::Pattern(pattern) => match pattern.unwrap_relative(true) {
            viz::RelativeTo::Self_ => Transform::identity(),
            viz::RelativeTo::Parent => state.transform.invert().unwrap(),
        },
    }
}

fn convert_outline_glyph_to_path(font: &Font, id: GlyphId, scale: Ratio) -> Option<kurbo::BezPath> {
    let mut builder = GlyphPathBuilder(kurbo::BezPath::new(), scale);
    font.ttf().outline_glyph(id, &mut builder)?;
    Some(builder.0)
}

struct GlyphPathBuilder(kurbo::BezPath, Ratio);

impl OutlineBuilder for GlyphPathBuilder {
    // Y axis is inverted.
    fn move_to(&mut self, x: f32, y: f32) {
        let scale = self.1.get();
        self.0.move_to((scale * x as f64, scale * y as f64));
    }

    fn line_to(&mut self, x: f32, y: f32) {
        let scale = self.1.get();
        self.0.line_to((scale * x as f64, scale * y as f64));
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let scale = self.1.get();
        self.0.quad_to(
            (scale * x1 as f64, scale * y1 as f64),
            (scale * x as f64, scale * y as f64),
        );
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        let scale = self.1.get();
        self.0.curve_to(
            (scale * x1 as f64, scale * y1 as f64),
            (scale * x2 as f64, scale * y2 as f64),
            (scale * x as f64, scale * y as f64),
        );
    }

    fn close(&mut self) {
        self.0.close_path();
    }
}
