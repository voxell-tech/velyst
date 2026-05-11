#![no_std]

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use hashbrown::HashMap;
use peniko::kurbo::Vec2;
use smallvec::SmallVec;

use blur::KanvaBlurredRect;
use image::KanvaImage;
use layer::Layer;
use shape::KanvaShape;
use text::{KanvaGlyphRun, KanvaOutlinedGlyphs};

pub mod blur;
pub mod builder;
pub mod image;
pub mod layer;
pub mod render;
pub mod shape;
pub mod sink;
pub mod text;

pub mod prelude {
    pub use crate::blur::KanvaBlurredRect;
    pub use crate::builder::KanvaBuilder;
    pub use crate::image::KanvaImage;
    pub use crate::layer::{KanvaClip, Layer};
    pub use crate::shape::{KanvaFill, KanvaShape, KanvaStroke};
    pub use crate::text::{
        GlyphPos, KanvaGlyph, KanvaGlyphRun, KanvaOutlinedGlyphs,
    };
    pub use crate::{Kanva, KanvaNode};
}

/// A flat 2D scene graph of [`KanvaNode`]s with per-type item buffers, queryable by label.
pub struct Kanva {
    pub nodes: Vec<KanvaNode>,
    pub shapes: Vec<KanvaShape>,
    pub glyph_runs: Vec<KanvaGlyphRun>,
    pub outlined_glyphs: Vec<KanvaOutlinedGlyphs>,
    pub images: Vec<KanvaImage>,
    pub blurred_rects: Vec<KanvaBlurredRect>,
    label_map: HashMap<String, SmallVec<[usize; 1]>>,
    size: Vec2,
}

impl Kanva {
    pub(crate) fn empty(size: Vec2) -> Self {
        Self {
            nodes: Vec::new(),
            shapes: Vec::new(),
            glyph_runs: Vec::new(),
            outlined_glyphs: Vec::new(),
            images: Vec::new(),
            blurred_rects: Vec::new(),
            label_map: HashMap::new(),
            size,
        }
    }

    pub(crate) fn clear(&mut self, size: Vec2) {
        self.nodes.clear();
        self.shapes.clear();
        self.glyph_runs.clear();
        self.outlined_glyphs.clear();
        self.images.clear();
        self.blurred_rects.clear();
        self.label_map.clear();
        self.size = size;
    }

    pub(crate) fn push_node(&mut self, node: KanvaNode) -> usize {
        let index = self.nodes.len();
        if let Some(label) = &node.label {
            self.label_map
                .entry(label.clone())
                .or_default()
                .push(index);
        }
        self.nodes.push(node);
        index
    }

    /// Returns the indices of all nodes with the given label.
    pub fn query(&self, label: &str) -> &[usize] {
        self.label_map
            .get(label)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    pub fn size(&self) -> Vec2 {
        self.size
    }

    /// Convert all glyph runs in the scene to outlined glyphs.
    /// Glyph runs that fail to parse are silently dropped.
    pub fn outline_all_glyphs(&mut self) {
        // First pass: decompose every glyph run, recording which run index
        // maps to which outlined-glyphs index (None if decomposition failed).
        let mut run_to_outlined: Vec<Option<usize>> =
            Vec::with_capacity(self.glyph_runs.len());
        for run in &self.glyph_runs {
            if let Some(outlined) = run.to_outlined_glyphs() {
                run_to_outlined
                    .push(Some(self.outlined_glyphs.len()));
                self.outlined_glyphs.push(outlined);
            } else {
                run_to_outlined.push(None);
            }
        }

        // Second pass: update each node's index lists.
        for node in &mut self.nodes {
            let new: Vec<usize> = node
                .glyph_runs
                .iter()
                .filter_map(|&gi| run_to_outlined[gi])
                .collect();
            node.outlined_glyphs.extend(new);
            node.glyph_runs.clear();
        }

        self.glyph_runs.clear();
    }
}

pub struct KanvaNode {
    pub parent: Option<usize>,
    pub label: Option<String>,
    pub transform: peniko::kurbo::Affine,
    pub layer: Option<Layer>,
    /// Index of the first node after this node's entire subtree.
    pub subtree_end: usize,
    /// Indices into [`Kanva::shapes`].
    pub shapes: Vec<usize>,
    /// Indices into [`Kanva::glyph_runs`].
    pub glyph_runs: Vec<usize>,
    /// Indices into [`Kanva::outlined_glyphs`].
    pub outlined_glyphs: Vec<usize>,
    /// Indices into [`Kanva::images`].
    pub images: Vec<usize>,
    /// Indices into [`Kanva::blurred_rects`].
    pub blurred_rects: Vec<usize>,
}

#[cfg(test)]
mod tests {
    use alloc::sync::Arc;
    use alloc::vec;

    use peniko::kurbo::{Affine, Vec2};
    use peniko::{Blob, Brush, Color, FontData};

    use crate::builder::KanvaBuilder;
    use crate::text::{GlyphPos, KanvaGlyphRun};

    fn invalid_font() -> FontData {
        FontData::new(
            Blob::new(Arc::new(vec![0u8, 1, 2, 3])),
            0,
        )
    }

    fn glyph_run_with_invalid_font() -> KanvaGlyphRun {
        KanvaGlyphRun {
            font: invalid_font(),
            font_size: 16.0,
            hint: false,
            brush: Brush::Solid(Color::from_rgba8(0, 0, 0, 255)),
            stroke: None,
            transform: Affine::IDENTITY,
            glyphs: vec![GlyphPos { id: 0, x: 0.0, y: 0.0 }],
        }
    }

    // ── outline_all_glyphs ────────────────────────────────────────────────

    #[test]
    fn outline_all_glyphs_with_invalid_font_drops_run_silently() {
        let mut builder = KanvaBuilder::new(Vec2::new(100.0, 100.0));
        builder.push_glyph_run(glyph_run_with_invalid_font());
        let mut kanva = builder.build();

        kanva.outline_all_glyphs();

        // The invalid glyph run should be silently dropped
        assert_eq!(kanva.glyph_runs.len(), 0);
        assert_eq!(kanva.outlined_glyphs.len(), 0);
    }

    #[test]
    fn outline_all_glyphs_clears_glyph_runs() {
        let mut builder = KanvaBuilder::new(Vec2::new(100.0, 100.0));
        builder.push_glyph_run(glyph_run_with_invalid_font());
        builder.push_glyph_run(glyph_run_with_invalid_font());
        let mut kanva = builder.build();

        kanva.outline_all_glyphs();

        assert_eq!(kanva.glyph_runs.len(), 0);
    }

    #[test]
    fn outline_all_glyphs_clears_glyph_run_indices_in_nodes() {
        let mut builder = KanvaBuilder::new(Vec2::new(100.0, 100.0));
        builder.push_glyph_run(glyph_run_with_invalid_font());
        let mut kanva = builder.build();

        assert_eq!(kanva.nodes[0].glyph_runs.len(), 1);
        kanva.outline_all_glyphs();
        assert_eq!(kanva.nodes[0].glyph_runs.len(), 0);
    }

    #[test]
    fn outline_all_glyphs_on_empty_scene_is_safe() {
        let mut kanva = KanvaBuilder::new(Vec2::new(50.0, 50.0)).build();
        // Should not panic
        kanva.outline_all_glyphs();
        assert_eq!(kanva.glyph_runs.len(), 0);
        assert_eq!(kanva.outlined_glyphs.len(), 0);
    }

    #[test]
    fn outline_all_glyphs_with_no_runs_leaves_outlined_glyphs_unchanged() {
        let mut builder = KanvaBuilder::new(Vec2::new(100.0, 100.0));
        // Push some outlined glyphs directly (pre-existing)
        use crate::shape::KanvaStroke;
        use crate::text::{KanvaGlyph, KanvaOutlinedGlyphs};
        use peniko::kurbo::BezPath;
        builder.push_outlined_glyphs(KanvaOutlinedGlyphs {
            brush: Brush::Solid(Color::from_rgba8(0, 0, 0, 255)),
            stroke: None,
            glyphs: vec![KanvaGlyph {
                path: BezPath::new(),
                transform: Affine::IDENTITY,
                fill_transform: None,
                stroke_transform: None,
            }],
        });
        let mut kanva = builder.build();
        let pre_count = kanva.outlined_glyphs.len();

        kanva.outline_all_glyphs();

        // Existing outlined_glyphs should be unchanged (no new ones added for invalid runs)
        assert_eq!(kanva.outlined_glyphs.len(), pre_count);
    }

    #[test]
    fn labeled_group_is_queryable() {
        let mut builder = KanvaBuilder::new(Vec2::new(100.0, 100.0));
        builder.begin_group(
            Some("title".into()),
            Default::default(),
            None,
        );
        let kanva = builder.build();
        let hits = kanva.query("title");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0], 1);
    }

    #[test]
    fn query_unknown_label_returns_empty() {
        let kanva =
            KanvaBuilder::new(Vec2::new(100.0, 100.0)).build();
        assert!(kanva.query("nope").is_empty());
    }

    #[test]
    fn multiple_groups_share_label() {
        let mut builder = KanvaBuilder::new(Vec2::new(100.0, 100.0));
        builder.begin_group(
            Some("item".into()),
            Default::default(),
            None,
        );
        builder.end_group();
        builder.begin_group(
            Some("item".into()),
            Default::default(),
            None,
        );
        let kanva = builder.build();
        assert_eq!(kanva.query("item"), &[1, 2]);
    }

    #[test]
    fn size_is_stored() {
        let size = Vec2::new(800.0, 600.0);
        let kanva = KanvaBuilder::new(size).build();
        assert_eq!(kanva.size(), size);
    }
}
