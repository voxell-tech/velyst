use alloc::vec::Vec;

use peniko::kurbo::{Affine, BezPath};
use peniko::{Brush, FontData};

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

/// Glyphs as pre-decomposed bezier outlines.
/// Higher memory cost; use when you need to trace, morph, or animate individual paths.
#[derive(Debug, Clone)]
pub struct KanvaOutlinedGlyphs {
    pub brush: Brush,
    pub stroke: Option<KanvaStroke>,
    pub glyphs: Vec<KanvaGlyph>,
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
