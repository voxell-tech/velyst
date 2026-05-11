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

#[cfg(test)]
mod tests {
    use alloc::sync::Arc;
    use alloc::vec;
    use alloc::vec::Vec;

    use peniko::kurbo::{Affine, BezPath, PathEl, Vec2};
    use peniko::{Blob, Brush, Color, FontData};
    use ttf_parser::OutlineBuilder as _;

    use crate::builder::KanvaBuilder;
    use crate::shape::KanvaStroke;

    use super::{
        BezPathBuilder, GlyphPos, KanvaGlyph, KanvaGlyphRun,
        KanvaOutlinedGlyphs,
    };

    fn invalid_font() -> FontData {
        // 4 bytes of garbage - ttf_parser will fail to parse
        FontData::new(
            Blob::new(Arc::new(vec![0u8, 1, 2, 3])),
            0,
        )
    }

    // ── GlyphPos ──────────────────────────────────────────────────────────────

    #[test]
    fn glyph_pos_fields_are_accessible() {
        let g = GlyphPos {
            id: 42,
            x: 1.5,
            y: -2.0,
        };
        assert_eq!(g.id, 42);
        assert_eq!(g.x, 1.5);
        assert_eq!(g.y, -2.0);
    }

    #[test]
    fn glyph_pos_copy() {
        let g = GlyphPos { id: 1, x: 0.0, y: 0.0 };
        let g2 = g;
        assert_eq!(g.id, g2.id);
    }

    // ── KanvaGlyph ────────────────────────────────────────────────────────────

    #[test]
    fn kanva_glyph_default_transforms_are_none() {
        let glyph = KanvaGlyph {
            path: BezPath::new(),
            transform: Affine::IDENTITY,
            fill_transform: None,
            stroke_transform: None,
        };
        assert!(glyph.fill_transform.is_none());
        assert!(glyph.stroke_transform.is_none());
    }

    #[test]
    fn kanva_glyph_clone_preserves_transform() {
        let tf = Affine::translate((3.0, 4.0));
        let glyph = KanvaGlyph {
            path: BezPath::new(),
            transform: tf,
            fill_transform: None,
            stroke_transform: None,
        };
        let cloned = glyph.clone();
        assert_eq!(cloned.transform, tf);
    }

    // ── BezPathBuilder ────────────────────────────────────────────────────────

    #[test]
    fn bez_path_builder_move_to_adds_move_el() {
        let mut b = BezPathBuilder(BezPath::new());
        b.move_to(1.0, 2.0);
        let els: Vec<PathEl> = b.0.elements().to_vec();
        assert_eq!(els.len(), 1);
        assert!(matches!(els[0], PathEl::MoveTo(_)));
    }

    #[test]
    fn bez_path_builder_line_to_adds_line_el() {
        let mut b = BezPathBuilder(BezPath::new());
        b.move_to(0.0, 0.0);
        b.line_to(5.0, 5.0);
        let els: Vec<PathEl> = b.0.elements().to_vec();
        assert_eq!(els.len(), 2);
        assert!(matches!(els[1], PathEl::LineTo(_)));
    }

    #[test]
    fn bez_path_builder_quad_to_adds_quad_el() {
        let mut b = BezPathBuilder(BezPath::new());
        b.move_to(0.0, 0.0);
        b.quad_to(1.0, 1.0, 2.0, 0.0);
        let els: Vec<PathEl> = b.0.elements().to_vec();
        assert_eq!(els.len(), 2);
        assert!(matches!(els[1], PathEl::QuadTo(_, _)));
    }

    #[test]
    fn bez_path_builder_curve_to_adds_curve_el() {
        let mut b = BezPathBuilder(BezPath::new());
        b.move_to(0.0, 0.0);
        b.curve_to(1.0, 0.0, 2.0, 1.0, 3.0, 0.0);
        let els: Vec<PathEl> = b.0.elements().to_vec();
        assert_eq!(els.len(), 2);
        assert!(matches!(els[1], PathEl::CurveTo(_, _, _)));
    }

    #[test]
    fn bez_path_builder_close_adds_close_el() {
        let mut b = BezPathBuilder(BezPath::new());
        b.move_to(0.0, 0.0);
        b.line_to(1.0, 0.0);
        b.close();
        let els: Vec<PathEl> = b.0.elements().to_vec();
        assert!(matches!(els.last(), Some(PathEl::ClosePath)));
    }

    #[test]
    fn bez_path_builder_move_to_casts_f32_to_f64() {
        let mut b = BezPathBuilder(BezPath::new());
        b.move_to(3.5_f32, 7.25_f32);
        let els: Vec<PathEl> = b.0.elements().to_vec();
        if let PathEl::MoveTo(pt) = els[0] {
            assert!((pt.x - 3.5_f64).abs() < 1e-6);
            assert!((pt.y - 7.25_f64).abs() < 1e-6);
        } else {
            panic!("expected MoveTo");
        }
    }

    // ── KanvaGlyphRun ─────────────────────────────────────────────────────────

    #[test]
    fn to_outlined_glyphs_returns_none_for_invalid_font() {
        let run = KanvaGlyphRun {
            font: invalid_font(),
            font_size: 16.0,
            hint: false,
            brush: Brush::Solid(Color::from_rgba8(0, 0, 0, 255)),
            stroke: None,
            transform: Affine::IDENTITY,
            glyphs: vec![GlyphPos { id: 0, x: 0.0, y: 0.0 }],
        };
        assert!(run.to_outlined_glyphs().is_none());
    }

    #[test]
    fn glyph_run_render_records_one_glyph_run_in_sink() {
        let run = KanvaGlyphRun {
            font: invalid_font(),
            font_size: 14.0,
            hint: false,
            brush: Brush::Solid(Color::from_rgba8(0, 0, 0, 255)),
            stroke: None,
            transform: Affine::IDENTITY,
            glyphs: vec![GlyphPos { id: 1, x: 10.0, y: 0.0 }],
        };
        let mut sink = KanvaBuilder::new(Vec2::new(100.0, 100.0));
        run.render(Affine::IDENTITY, &mut sink);
        let kanva = sink.build();
        // No stroke → only one glyph_run call
        assert_eq!(kanva.glyph_runs.len(), 1);
    }

    #[test]
    fn glyph_run_render_with_stroke_records_two_glyph_runs() {
        let run = KanvaGlyphRun {
            font: invalid_font(),
            font_size: 14.0,
            hint: false,
            brush: Brush::Solid(Color::from_rgba8(0, 0, 0, 255)),
            stroke: Some(KanvaStroke {
                style: peniko::kurbo::Stroke::new(1.0),
                brush: Brush::Solid(Color::from_rgba8(255, 0, 0, 255)),
                transform: None,
            }),
            transform: Affine::IDENTITY,
            glyphs: vec![GlyphPos { id: 2, x: 0.0, y: 0.0 }],
        };
        let mut sink = KanvaBuilder::new(Vec2::new(100.0, 100.0));
        run.render(Affine::IDENTITY, &mut sink);
        let kanva = sink.build();
        // Fill run + stroke run
        assert_eq!(kanva.glyph_runs.len(), 2);
    }

    #[test]
    fn glyph_run_render_concatenates_transform() {
        let run_tf = Affine::translate((5.0, 0.0));
        let run = KanvaGlyphRun {
            font: invalid_font(),
            font_size: 12.0,
            hint: false,
            brush: Brush::Solid(Color::from_rgba8(0, 0, 0, 255)),
            stroke: None,
            transform: run_tf,
            glyphs: vec![],
        };
        let parent_tf = Affine::translate((10.0, 0.0));
        let mut sink = KanvaBuilder::new(Vec2::new(100.0, 100.0));
        run.render(parent_tf, &mut sink);
        let kanva = sink.build();
        let expected = parent_tf * run_tf;
        assert_eq!(kanva.glyph_runs[0].transform, expected);
    }

    // ── KanvaOutlinedGlyphs ───────────────────────────────────────────────────

    fn sample_outlined_glyphs(with_stroke: bool) -> KanvaOutlinedGlyphs {
        let mut path = BezPath::new();
        path.move_to((0.0, 0.0));
        path.line_to((5.0, 0.0));
        path.line_to((5.0, 5.0));
        path.close_path();

        let stroke = if with_stroke {
            Some(KanvaStroke {
                style: peniko::kurbo::Stroke::new(1.0),
                brush: Brush::Solid(Color::from_rgba8(0, 0, 255, 255)),
                transform: None,
            })
        } else {
            None
        };

        KanvaOutlinedGlyphs {
            brush: Brush::Solid(Color::from_rgba8(0, 0, 0, 255)),
            stroke,
            glyphs: vec![KanvaGlyph {
                path,
                transform: Affine::IDENTITY,
                fill_transform: None,
                stroke_transform: None,
            }],
        }
    }

    #[test]
    fn outlined_glyphs_render_fill_only_records_one_shape_per_glyph() {
        let og = sample_outlined_glyphs(false);
        let mut sink = KanvaBuilder::new(Vec2::new(100.0, 100.0));
        og.render(Affine::IDENTITY, &mut sink);
        let kanva = sink.build();
        assert_eq!(kanva.shapes.len(), 1);
        assert!(kanva.shapes[0].fill.is_some());
        assert!(kanva.shapes[0].stroke.is_none());
    }

    #[test]
    fn outlined_glyphs_render_with_stroke_records_two_shapes_per_glyph() {
        let og = sample_outlined_glyphs(true);
        let mut sink = KanvaBuilder::new(Vec2::new(100.0, 100.0));
        og.render(Affine::IDENTITY, &mut sink);
        let kanva = sink.build();
        // fill + stroke = 2 shapes for 1 glyph
        assert_eq!(kanva.shapes.len(), 2);
    }

    #[test]
    fn outlined_glyphs_render_empty_glyphs_records_nothing() {
        let og = KanvaOutlinedGlyphs {
            brush: Brush::Solid(Color::from_rgba8(0, 0, 0, 255)),
            stroke: None,
            glyphs: vec![],
        };
        let mut sink = KanvaBuilder::new(Vec2::new(100.0, 100.0));
        og.render(Affine::IDENTITY, &mut sink);
        let kanva = sink.build();
        assert_eq!(kanva.shapes.len(), 0);
    }

    #[test]
    fn outlined_glyphs_render_concatenates_transform() {
        let glyph_tf = Affine::translate((3.0, 0.0));
        let og = KanvaOutlinedGlyphs {
            brush: Brush::Solid(Color::from_rgba8(0, 0, 0, 255)),
            stroke: None,
            glyphs: vec![KanvaGlyph {
                path: BezPath::new(),
                transform: glyph_tf,
                fill_transform: None,
                stroke_transform: None,
            }],
        };
        let parent_tf = Affine::translate((7.0, 0.0));
        let mut sink = KanvaBuilder::new(Vec2::new(100.0, 100.0));
        og.render(parent_tf, &mut sink);
        let kanva = sink.build();
        let expected = parent_tf * glyph_tf;
        assert_eq!(kanva.shapes[0].transform, expected);
    }
}
