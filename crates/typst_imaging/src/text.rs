use std::sync::Arc;

use imaging::peniko::kurbo::Vec2;
use imaging::peniko::{Blob, Fill, FontData, Style};
use imaging::record::Glyph;
use imaging::{Composite, GlyphRunRef, PaintSink};
use typst_library::text::TextItem;

use crate::RenderState;
use crate::convert::convert_fixed_stroke;
use crate::paint::text_paint;

pub(crate) fn render_text(
    text: &TextItem,
    sink: &mut impl PaintSink,
    state: RenderState,
) {
    let bytes = text.font.data();

    let font_data = FontData::new(
        Blob::new(Arc::new(bytes.clone())),
        text.font.index(),
    );
    let font_size = text.size.to_pt() as f32;

    let glyphs: Vec<Glyph> = {
        let mut x = 0.0f64;
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
            .collect()
    };

    if glyphs.is_empty() {
        return;
    }

    let (fill_brush, fill_brush_transform) =
        text_paint(&text.fill, &state);

    let fill_style = Style::Fill(Fill::NonZero);
    sink.glyph_run(
        GlyphRunRef {
            font: &font_data,
            transform: state.transform,
            glyph_transform: None,
            font_size,
            font_embolden: Vec2::ZERO,
            hint: false,
            normalized_coords: &[],
            style: &fill_style,
            brush: (&fill_brush).into(),
            brush_transform: fill_brush_transform,
            composite: Composite::default(),
        },
        &mut glyphs.iter().copied(),
    );

    if let Some(stroke) = &text.stroke {
        let (stroke_brush, stroke_brush_transform) =
            text_paint(&stroke.paint, &state);

        let stroke_style =
            Style::Stroke(convert_fixed_stroke(stroke));

        sink.glyph_run(
            GlyphRunRef {
                font: &font_data,
                transform: state.transform,
                glyph_transform: None,
                font_size,
                font_embolden: Vec2::ZERO,
                hint: false,
                normalized_coords: &[],
                style: &stroke_style,
                brush: (&stroke_brush).into(),
                brush_transform: stroke_brush_transform,
                composite: Composite::default(),
            },
            &mut glyphs.iter().copied(),
        );
    }
}
