use std::sync::Arc;

use kanva::imaging::Composite;
use kanva::imaging::peniko::{Blob, Fill, FontData};
use kanva::imaging::record::Glyph;
use kanva::prelude::*;
use typst_imaging::RenderState;
use typst_imaging::convert::convert_fixed_stroke;
use typst_imaging::paint::text_paint;
use typst_library::text::TextItem;

pub fn render_text(
    text: &TextItem,
    sink: &mut impl KanvaSink,
    state: RenderState,
) {
    let bytes = text.font.data();
    let font_data = FontData::new(
        Blob::new(Arc::new(bytes.clone())),
        text.font.index(),
    );
    let font_size = text.size.to_pt() as f32;

    let glyphs = {
        let mut x = 0.0;
        text.glyphs
            .iter()
            .map(|g| {
                let glyph_x =
                    (x + g.x_offset.at(text.size).to_pt()) as f32;
                x += g.x_advance.at(text.size).to_pt();
                Glyph {
                    id: g.id as u32,
                    x: glyph_x,
                    y: 0.0,
                }
            })
            .collect::<Vec<_>>()
    };

    if glyphs.is_empty() {
        return;
    }

    let (fill_brush, fill_brush_transform) =
        text_paint(&text.fill, &state);

    let fill = Some(KanvaFill {
        rule: Fill::NonZero,
        brush: fill_brush,
        brush_transform: fill_brush_transform,
        composite: Composite::default(),
    });

    let stroke = text.stroke.as_ref().map(|s| {
        let (stroke_brush, stroke_brush_transform) =
            text_paint(&s.paint, &state);
        KanvaStroke {
            stroke: convert_fixed_stroke(s),
            brush: stroke_brush,
            brush_transform: stroke_brush_transform,
            composite: Composite::default(),
        }
    });

    sink.glyph_run(
        GlyphRun {
            font: font_data,
            transform: state.transform,
            glyph_transform: None,
            font_size,
        },
        fill,
        stroke,
        &mut glyphs.iter().copied(),
    );
}
