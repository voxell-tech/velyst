#![no_std]

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use hashbrown::HashMap;
use peniko::kurbo::{Affine, Vec2};
use smallvec::SmallVec;

use blur::KanvaBlurredRect;
use layer::Layer;
use shape::KanvaShape;
use text::KanvaGlyphRun;

pub mod blur;
pub mod builder;
pub mod layer;
pub mod overrides;
pub mod render;
pub mod shape;
pub mod sink;
pub mod text;

pub mod prelude {
    pub use crate::blur::KanvaBlurredRect;
    pub use crate::builder::KanvaBuilder;
    pub use crate::layer::{KanvaClip, Layer};
    pub use crate::shape::{KanvaFill, KanvaShape, KanvaStroke};
    pub use crate::text::{GlyphPos, KanvaGlyphRun};
    pub use crate::{Kanva, KanvaNode};
}

/// A flat 2D scene graph of [`KanvaNode`]s with per-type item buffers, queryable by label.
#[derive(Clone)]
pub struct Kanva {
    pub nodes: Vec<KanvaNode>,
    pub shapes: Vec<KanvaShape>,
    pub glyph_runs: Vec<KanvaGlyphRun>,
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
            blurred_rects: Vec::new(),
            label_map: HashMap::new(),
            size,
        }
    }

    pub(crate) fn clear(&mut self, size: Vec2) {
        self.nodes.clear();
        self.shapes.clear();
        self.glyph_runs.clear();
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

    /// Decompose all glyph runs into individual [`KanvaShape`]s (one per glyph).
    /// Glyph runs that fail to parse are silently dropped.
    pub fn outline_all_glyphs(&mut self) {
        // First pass: decompose every run, recording the shape index range per run.
        let mut run_to_shapes: Vec<(usize, usize)> =
            Vec::with_capacity(self.glyph_runs.len());
        for run in &self.glyph_runs {
            let start = self.shapes.len();
            self.shapes.extend(run.to_shapes());
            run_to_shapes.push((start, self.shapes.len()));
        }

        // Second pass: replace each node's glyph_run indices with shape indices.
        for node in &mut self.nodes {
            for &gi in &node.glyph_runs {
                let (start, end) = run_to_shapes[gi];
                node.shapes.extend(start..end);
            }
            node.glyph_runs.clear();
        }

        self.glyph_runs.clear();
    }
}

#[derive(Default, Clone)]
pub struct KanvaNode {
    /// Index of the node's parent.
    pub parent: Option<usize>,
    pub label: Option<String>,
    pub style: KanvaNodeStyle,
    /// Index of the first node after this node's entire subtree.
    pub subtree_end: usize,
    /// Indices into [`Kanva::shapes`].
    pub shapes: Vec<usize>,
    /// Indices into [`Kanva::glyph_runs`].
    pub glyph_runs: Vec<usize>,
    /// Indices into [`Kanva::blurred_rects`].
    pub blurred_rects: Vec<usize>,
}

impl KanvaNode {
    pub fn new(parent: Option<usize>, label: Option<String>) -> Self {
        KanvaNode {
            parent,
            label,
            ..Default::default()
        }
    }
}

#[derive(Default, Clone)]
pub struct KanvaNodeStyle {
    /// The offset transform of the node.
    ///
    /// Defaults to [`Affine::IDENTITY`] if unset.
    pub offset_transform: Option<Affine>,
    /// Optional layer of the node.
    pub layer: Option<Layer>,
}

impl KanvaNodeStyle {
    pub fn from_layer(layer: Layer) -> Self {
        Self {
            layer: Some(layer),
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::builder::KanvaBuilder;

    use super::*;

    #[test]
    fn labeled_group_is_queryable() {
        let mut builder = KanvaBuilder::new(Vec2::new(100.0, 100.0));
        builder.begin_group(Some("title".into()), None);
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
        builder.begin_group(Some("item".into()), None);
        builder.end_group();
        builder.begin_group(Some("item".into()), None);
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
