use std::sync::Arc;

use imaging::record::Glyph;
use imaging::{Composite, GlyphRunRef, PaintSink};
use peniko::kurbo::Vec2;
use peniko::{Blob, Fill, FontData, Style};
use typst_library::text::TextItem;

use crate::RenderState;
use crate::convert::convert_fixed_stroke;
use crate::paint::text_paint;

pub(crate) fn render_text(
    text: &TextItem,
    sink: &mut impl PaintSink,
    state: RenderState,
) {
    let font_bytes: Arc<Vec<u8>> =
        Arc::new(text.font.data().as_ref().to_vec());
    let font_data =
        FontData::new(Blob::new(font_bytes), text.font.index());
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

    let Some(last_glyph) = glyphs.last() else {
        // Skips if there are no glyphs at all.
        return;
    };

    // Debug: outline the container rect used for gradient sampling.
    // #[cfg(debug_assertions)]
    // {
    //     let w = state.container_size.x.to_pt();
    //     let h = state.container_size.y.to_pt();
    //     let rect = peniko::kurbo::Rect::new(0.0, 0.0, w, h);
    //     sink.stroke(StrokeRef {
    //         transform: state.container_transform,
    //         stroke: &Stroke::default(),
    //         brush: (&Brush::Solid(Color::from_rgba8(
    //             255, 0, 255, 220,
    //         )))
    //             .into(),
    //         brush_transform: None,
    //         shape: GeometryRef::Rect(rect),
    //         composite: Composite::default(),
    //     });
    // }

    let fill_brush =
        text_paint(&text.fill, &state, last_glyph.x as f64);

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
            composite: Composite::default(),
        },
        &mut glyphs.iter().copied(),
    );

    if let Some(stroke) = &text.stroke {
        let stroke_brush =
            text_paint(&stroke.paint, &state, last_glyph.x as f64);

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
                composite: Composite::default(),
            },
            &mut glyphs.iter().copied(),
        );
    }
}
