use alloc::vec::Vec;

use imaging::record::Glyph;
use imaging::{Composite, GlyphRunRef, PaintSink};
use peniko::kurbo::{Affine, BezPath};
use peniko::{Brush, FontData, Style};
use ttf_parser::OutlineBuilder;

use crate::shape::{KanvaFill, KanvaShape, KanvaStroke};

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
    pub composite: Composite,
}

impl KanvaGlyphRun {
    /// Decompose this glyph run into individual [`KanvaShape`]s, one per glyph.
    /// Returns an empty vec if the font cannot be parsed.
    pub fn to_shapes(&self) -> Vec<KanvaShape> {
        let Ok(face) = ttf_parser::Face::parse(
            self.font.data.data(),
            self.font.index,
        ) else {
            return Vec::new();
        };
        let upem = face.units_per_em() as f64;
        let scale = self.font_size as f64 / upem;

        self.glyphs
            .iter()
            .filter_map(|g| {
                let mut builder = BezPathBuilder(BezPath::new());
                face.outline_glyph(
                    ttf_parser::GlyphId(g.id as u16),
                    &mut builder,
                )?;
                Some(KanvaShape {
                    path: builder.0,
                    fill: Some(KanvaFill {
                        style: peniko::Fill::NonZero,
                        brush: self.brush.clone(),
                        transform: None,
                        composite: self.composite,
                    }),
                    stroke: self.stroke.clone(),
                    transform: self.transform
                        * Affine::translate((g.x as f64, g.y as f64))
                        * Affine::scale_non_uniform(scale, -scale),
                })
            })
            .collect()
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

/// A positioned glyph inside a [`KanvaGlyphRun`].
#[derive(Debug, Clone, Copy)]
pub struct GlyphPos {
    pub id: u32,
    pub x: f32,
    pub y: f32,
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
