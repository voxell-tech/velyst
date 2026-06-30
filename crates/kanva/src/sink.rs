use imaging::kurbo::{Affine, BezPath};
use imaging::peniko::FontData;
use imaging::record::Glyph;

use crate::{Group, KanvaFill, KanvaStroke, PaintOrder};

/// Metadata for a positioned glyph run passed to
/// [`KanvaSink::glyph_run`].
///
/// Unlike [`imaging::GlyphRunRef`], this carries no style or brush -
/// fill and stroke are passed separately so a single combined
/// [`crate::KanvaPath`] can be emitted per glyph.
pub struct GlyphRun {
    pub font: FontData,
    pub transform: Affine,
    pub glyph_transform: Option<Affine>,
    pub font_size: f32,
}

/// A draw sink that accepts **combined fill+stroke** per element.
///
/// Implementations store draw commands and produce a [`crate::Kanva`]
/// scene where each path carries both an optional fill and an
/// optional stroke index, eliminating the path-doubling that occurs
/// with [`imaging::PaintSink`]'s fill-XOR-stroke model.
pub trait KanvaSink {
    fn push_context(&mut self, label: &str);
    fn pop_context(&mut self);
    fn push_group(&mut self, group: Group);
    fn pop_group(&mut self);
    fn draw_path(
        &mut self,
        path: BezPath,
        transform: Affine,
        fill: Option<KanvaFill>,
        stroke: Option<KanvaStroke>,
        paint_order: PaintOrder,
    );
    fn glyph_run(
        &mut self,
        run: GlyphRun,
        fill: Option<KanvaFill>,
        stroke: Option<KanvaStroke>,
        glyphs: &mut dyn Iterator<Item = Glyph>,
    );
}
