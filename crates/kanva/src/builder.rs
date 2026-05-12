use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use peniko::kurbo::Vec2;

use crate::blur::KanvaBlurredRect;
use crate::layer::Layer;
use crate::shape::KanvaShape;
use crate::text::KanvaGlyphRun;
use crate::{Kanva, KanvaNode};

pub struct KanvaBuilder {
    pub(crate) kanva: Kanva,
    pub(crate) stack: Vec<usize>,
    /// Label set by the most recent `push_context`, consumed on the next group/clip push.
    pub(crate) pending_label: Option<String>,
}

impl KanvaBuilder {
    pub fn new(size: Vec2) -> Self {
        let mut kanva = Kanva::empty(size);
        let root = kanva.push_node(KanvaNode::new(None, None));
        Self {
            kanva,
            stack: vec![root],
            pending_label: None,
        }
    }

    /// Reuse allocations from an existing [`Kanva`], rebuilding in place.
    pub fn rebuild(mut kanva: Kanva, size: Vec2) -> Self {
        kanva.clear(size);
        let root = kanva.push_node(KanvaNode::new(None, None));
        Self {
            kanva,
            stack: vec![root],
            pending_label: None,
        }
    }

    /// Push a new group node as a child of the current node.
    pub fn begin_group(
        &mut self,
        label: Option<String>,
        layer: Option<Layer>,
    ) {
        let parent = self.current();
        let mut node = KanvaNode::new(Some(parent), label);
        node.layer = layer;
        let index = self.kanva.push_node(node);
        self.stack.push(index);
    }

    /// Pop the current group node back to its parent.
    pub fn end_group(&mut self) {
        if self.stack.len() > 1 {
            let index = self.stack.pop().unwrap();
            self.kanva.nodes[index].subtree_end =
                self.kanva.nodes.len();
        }
    }

    pub fn push_shape(&mut self, shape: KanvaShape) {
        let index = self.kanva.shapes.len();
        self.kanva.shapes.push(shape);
        self.current_node_mut().shapes.push(index);
    }

    pub fn push_glyph_run(&mut self, run: KanvaGlyphRun) {
        let index = self.kanva.glyph_runs.len();
        self.kanva.glyph_runs.push(run);
        self.current_node_mut().glyph_runs.push(index);
    }

    pub fn push_blurred_rect(&mut self, rect: KanvaBlurredRect) {
        let index = self.kanva.blurred_rects.len();
        self.kanva.blurred_rects.push(rect);
        self.current_node_mut().blurred_rects.push(index);
    }

    pub fn build(mut self) -> Kanva {
        let end = self.kanva.nodes.len();
        for &i in &self.stack {
            self.kanva.nodes[i].subtree_end = end;
        }
        self.kanva
    }

    pub(crate) fn current(&self) -> usize {
        *self.stack.last().expect("stack is never empty")
    }

    fn current_node_mut(&mut self) -> &mut KanvaNode {
        let index = self.current();
        &mut self.kanva.nodes[index]
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use peniko::kurbo::{Affine, Rect, Shape as _, Vec2};
    use peniko::{Brush, Color};

    use crate::shape::{KanvaFill, KanvaShape};

    use super::KanvaBuilder;

    fn rect_shape() -> KanvaShape {
        let path = Rect::new(0.0, 0.0, 10.0, 10.0).to_path(0.1);
        KanvaShape {
            path,
            fill: Some(KanvaFill {
                brush: Brush::Solid(Color::from_rgba8(
                    255, 0, 0, 255,
                )),
                ..Default::default()
            }),
            stroke: None,
            transform: Affine::IDENTITY,
        }
    }

    #[test]
    fn new_has_single_root_node() {
        let kanva =
            KanvaBuilder::new(Vec2::new(100.0, 100.0)).build();
        assert_eq!(kanva.nodes.len(), 1);
        assert!(kanva.nodes[0].parent.is_none());
    }

    #[test]
    fn begin_group_adds_child_node() {
        let mut builder = KanvaBuilder::new(Vec2::new(100.0, 100.0));
        builder.begin_group(None, None);
        let kanva = builder.build();
        assert_eq!(kanva.nodes.len(), 2);
        assert_eq!(kanva.nodes[1].parent, Some(0));
    }

    #[test]
    fn end_group_returns_to_parent() {
        let mut builder = KanvaBuilder::new(Vec2::new(100.0, 100.0));
        builder.begin_group(None, None);
        builder.end_group();
        builder.push_shape(rect_shape());
        let kanva = builder.build();
        assert!(kanva.nodes[0].shapes.contains(&0));
    }

    #[test]
    fn end_group_past_root_is_safe() {
        let mut builder = KanvaBuilder::new(Vec2::new(100.0, 100.0));
        builder.end_group();
        builder.end_group();
        let kanva = builder.build();
        assert_eq!(kanva.nodes.len(), 1);
    }

    #[test]
    fn push_shape_registers_in_flat_buffer_and_node() {
        let mut builder = KanvaBuilder::new(Vec2::new(100.0, 100.0));
        builder.push_shape(rect_shape());
        builder.push_shape(rect_shape());
        let kanva = builder.build();
        assert_eq!(kanva.shapes.len(), 2);
        assert_eq!(kanva.nodes[0].shapes, vec![0, 1]);
    }

    #[test]
    fn shapes_in_child_group_not_in_parent() {
        let mut builder = KanvaBuilder::new(Vec2::new(100.0, 100.0));
        builder.begin_group(None, None);
        builder.push_shape(rect_shape());
        builder.end_group();
        let kanva = builder.build();
        assert!(kanva.nodes[0].shapes.is_empty());
        assert_eq!(kanva.nodes[1].shapes, vec![0]);
    }

    #[test]
    fn nested_groups_have_correct_parents() {
        let mut builder = KanvaBuilder::new(Vec2::new(100.0, 100.0));
        builder.begin_group(None, None);
        builder.begin_group(None, None);
        let kanva = builder.build();
        assert_eq!(kanva.nodes[1].parent, Some(0));
        assert_eq!(kanva.nodes[2].parent, Some(1));
    }

    #[test]
    fn rebuild_resets_state() {
        let mut builder = KanvaBuilder::new(Vec2::new(100.0, 100.0));
        builder.push_shape(rect_shape());
        builder.begin_group(Some("g".into()), None);
        let kanva = builder.build();

        let mut builder2 =
            KanvaBuilder::rebuild(kanva, Vec2::new(200.0, 200.0));
        builder2.begin_group(Some("fresh".into()), None);
        let kanva2 = builder2.build();

        assert_eq!(kanva2.shapes.len(), 0);
        assert!(kanva2.query("g").is_empty());
        assert_eq!(kanva2.query("fresh"), &[1]);
        assert_eq!(kanva2.size(), Vec2::new(200.0, 200.0));
    }
}
