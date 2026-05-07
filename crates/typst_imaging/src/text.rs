use std::sync::Arc;

use imaging::record::Glyph;
use imaging::{GlyphRunRef, PaintSink};
use peniko::kurbo::Vec2;
use peniko::{Blob, Fill, FontData, Style};
use typst_library::text::TextItem;

use crate::RenderState;
use crate::convert::{convert_fixed_stroke, convert_paint};

pub(crate) fn render_text(text: &TextItem, sink: &mut impl PaintSink, state: RenderState) {
    // Copy font bytes into a peniko Blob. Typst already caches the underlying data;
    // the Arc::from copy is a one-time cost per text item and avoids unsafe lifetime tricks.
    let font_bytes: Arc<Vec<u8>> = Arc::new(text.font.data().as_ref().to_vec());
    let font_data = FontData::new(Blob::new(font_bytes), text.font.index());

    let font_size = text.size.to_pt() as f32;

    let glyphs: Vec<Glyph> = {
        let mut x = 0.0f64;
        text.glyphs
            .iter()
            .map(|g| {
                let glyph_x = (x + g.x_offset.at(text.size).to_pt()) as f32;
                x += g.x_advance.at(text.size).to_pt();
                Glyph { id: g.id as u32, x: glyph_x, y: 0.0 }
            })
            .collect()
    };

    let (fill_brush, _) = convert_paint(&text.fill, state.size, state.container_transform);
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
            composite: imaging::Composite::default(),
        },
        &mut glyphs.iter().copied(),
    );

    if let Some(stroke) = &text.stroke {
        let (stroke_brush, _) =
            convert_paint(&stroke.paint, state.size, state.container_transform);
        let stroke_style = Style::Stroke(convert_fixed_stroke(stroke));
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
                composite: imaging::Composite::default(),
            },
            &mut glyphs.iter().copied(),
        );
    }
}
