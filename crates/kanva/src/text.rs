use alloc::vec::Vec;

use imaging::record::Glyph;
use imaging::{
    Composite, FillRef, GeometryRef, GlyphRunRef, PaintSink,
    StrokeRef,
};
use peniko::kurbo::{Affine, BezPath};
use peniko::{Brush, FontData, Style};
use ttf_parser::OutlineBuilder;

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
    /// Decompose this glyph run into per-glyph bezier outlines.
    /// Returns `None` if the font cannot be parsed.
    pub fn to_outlined_glyphs(&self) -> Option<KanvaOutlinedGlyphs> {
        let face = ttf_parser::Face::parse(
            self.font.data.data(),
            self.font.index,
        )
        .ok()?;
        let upem = face.units_per_em() as f64;
        let scale = self.font_size as f64 / upem;

        let glyphs = self
            .glyphs
            .iter()
            .filter_map(|g| {
                let mut builder = BezPathBuilder(BezPath::new());
                face.outline_glyph(
                    ttf_parser::GlyphId(g.id as u16),
                    &mut builder,
                )?;
                Some(KanvaGlyph {
                    path: builder.0,
                    transform: self.transform
                        * Affine::translate((g.x as f64, g.y as f64))
                        * Affine::scale_non_uniform(scale, -scale),
                    fill_transform: None,
                    stroke_transform: None,
                })
            })
            .collect();

        Some(KanvaOutlinedGlyphs {
            brush: self.brush.clone(),
            stroke: self.stroke.clone(),
            glyphs,
        })
    }

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

struct BezPathBuilder(BezPath);

impl OutlineBuilder for BezPathBuilder {
    fn move_to(&mut self, x: f32, y: f32) {
        self.0.move_to((x as f64, y as f64));
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.0.line_to((x as f64, y as f64));
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.0.quad_to((x1 as f64, y1 as f64), (x as f64, y as f64));
    }

    fn curve_to(
        &mut self,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        x: f32,
        y: f32,
    ) {
        self.0.curve_to(
            (x1 as f64, y1 as f64),
            (x2 as f64, y2 as f64),
            (x as f64, y as f64),
        );
    }

    fn close(&mut self) {
        self.0.close_path();
    }
}
