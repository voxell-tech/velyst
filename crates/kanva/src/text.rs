use alloc::vec::Vec;

use imaging::record::Glyph;
use imaging::{
    Composite, FillRef, GeometryRef, GlyphRunRef, PaintSink,
    StrokeRef,
};
use peniko::kurbo::{Affine, BezPath};
use peniko::{Brush, FontData, Style};

use crate::shape::KanvaStroke;

/// Text as raw glyph IDs + positions. The backend resolves outlines from the font.
/// Lower memory cost; use when you don't need per-glyph animation.
#[derive(Debug, Clone)]
pub struct KanvaGlyphRun {
    pub font: FontData,
    pub font_size: f32,
    pub hint: bool,
    pub brush: Brush,
    pub stroke: Option<KanvaStroke>,
    pub transform: Affine,
    pub glyphs: Vec<GlyphPos>,
}

impl KanvaGlyphRun {
    pub fn render(&self, tf: Affine, sink: &mut impl PaintSink) {
        let run_tf = tf * self.transform;
        let fill_style = Style::Fill(peniko::Fill::NonZero);
        sink.glyph_run(
            GlyphRunRef {
                font: &self.font,
                transform: run_tf,
                glyph_transform: None,
                font_size: self.font_size,
                font_embolden: peniko::kurbo::Vec2::ZERO,
                hint: self.hint,
                normalized_coords: &[],
                style: &fill_style,
                brush: (&self.brush).into(),
                composite: Composite::default(),
            },
            &mut self.glyphs.iter().map(|g| Glyph {
                id: g.id,
                x: g.x,
                y: g.y,
            }),
        );
        if let Some(stroke) = &self.stroke {
            let stroke_style = Style::Stroke(stroke.style.clone());
            sink.glyph_run(
                GlyphRunRef {
                    font: &self.font,
                    transform: run_tf,
                    glyph_transform: None,
                    font_size: self.font_size,
                    font_embolden: peniko::kurbo::Vec2::ZERO,
                    hint: self.hint,
                    normalized_coords: &[],
                    style: &stroke_style,
                    brush: (&stroke.brush).into(),
                    composite: Composite::default(),
                },
                &mut self.glyphs.iter().map(|g| Glyph {
                    id: g.id,
                    x: g.x,
                    y: g.y,
                }),
            );
        }
    }
}

/// Glyphs as pre-decomposed bezier outlines.
/// Higher memory cost; use when you need to trace, morph, or animate
/// individual paths.
#[derive(Debug, Clone)]
pub struct KanvaOutlinedGlyphs {
    pub brush: Brush,
    pub stroke: Option<KanvaStroke>,
    pub glyphs: Vec<KanvaGlyph>,
}

impl KanvaOutlinedGlyphs {
    pub fn render(&self, tf: Affine, sink: &mut impl PaintSink) {
        for glyph in &self.glyphs {
            let glyph_tf = tf * glyph.transform;
            sink.fill(FillRef {
                transform: glyph_tf,
                fill_rule: peniko::Fill::NonZero,
                brush: (&self.brush).into(),
                brush_transform: glyph.fill_transform,
                shape: GeometryRef::Path(&glyph.path),
                composite: Composite::default(),
            });
            if let Some(stroke) = &self.stroke {
                sink.stroke(StrokeRef {
                    transform: glyph_tf,
                    stroke: &stroke.style,
                    brush: (&stroke.brush).into(),
                    brush_transform: glyph.stroke_transform,
                    shape: GeometryRef::Path(&glyph.path),
                    composite: Composite::default(),
                });
            }
        }
    }
}

/// A positioned glyph inside a [`KanvaGlyphRun`].
#[derive(Debug, Clone, Copy)]
pub struct GlyphPos {
    pub id: u32,
    pub x: f32,
    pub y: f32,
}

/// A single glyph as a bezier outline, for use in [`KanvaOutlinedGlyphs`].
#[derive(Debug, Clone)]
pub struct KanvaGlyph {
    pub path: BezPath,
    pub transform: Affine,
    pub fill_transform: Option<Affine>,
    pub stroke_transform: Option<Affine>,
}
