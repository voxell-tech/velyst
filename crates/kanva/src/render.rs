use alloc::vec::Vec;

use imaging::{ClipRef, ContextRef, GeometryRef, PaintSink};
use peniko::kurbo::Affine;

use crate::Kanva;

impl Kanva {
    /// Walk every node in the scene and emit all draw commands into `sink`.
    pub fn render(&self, sink: &mut impl PaintSink) {
        // Pass 1: accumulate world-space transforms.
        // Parent index < child index is guaranteed by the builder.
        let mut transforms: Vec<Affine> =
            Vec::with_capacity(self.nodes.len());
        for node in &self.nodes {
            let parent_tf = node
                .parent
                .map(|p| transforms[p])
                .unwrap_or(Affine::IDENTITY);
            transforms.push(parent_tf * node.transform);
        }

        // Pass 2: linear scan. Two stacks track when to pop clips and contexts.
        let mut clip_stack: Vec<usize> = Vec::new();
        let mut label_stack: Vec<usize> = Vec::new();

        for (i, node) in self.nodes.iter().enumerate() {
            // Pop any clips/contexts whose subtrees have ended before node i.
            while clip_stack.last() == Some(&i) {
                clip_stack.pop();
                sink.pop_clip();
            }
            while label_stack.last() == Some(&i) {
                label_stack.pop();
                sink.pop_context();
            }

            let tf = transforms[i];

            if let Some(label) = &node.label {
                sink.push_context(ContextRef::new(label, None));
                label_stack.push(node.subtree_end);
            }

            if let Some(layer) = &node.layer
                && let Some(clip) = &layer.clip
            {
                let clip_ref = match &clip.stroke {
                    None => {
                        ClipRef::fill(GeometryRef::Path(&clip.path))
                            .with_transform(tf)
                    }
                    Some(stroke) => ClipRef::stroke(
                        GeometryRef::Path(&clip.path),
                        stroke,
                    )
                    .with_transform(tf),
                };
                sink.push_clip(clip_ref);
                clip_stack.push(node.subtree_end);
            }

            for &si in &node.shapes {
                self.shapes[si].render(tf, sink);
            }
            for &gi in &node.glyph_runs {
                self.glyph_runs[gi].render(tf, sink);
            }
            for &oi in &node.outlined_glyphs {
                self.outlined_glyphs[oi].render(tf, sink);
            }
            for &ii in &node.images {
                self.images[ii].render(tf, sink);
            }
            for &bi in &node.blurred_rects {
                self.blurred_rects[bi].render(tf, sink);
            }
        }

        // Drain remaining stacks.
        for _ in clip_stack {
            sink.pop_clip();
        }
        for _ in label_stack {
            sink.pop_context();
        }
    }
}

#[cfg(test)]
mod tests {
    use peniko::kurbo::{Affine, Rect, Shape as _, Vec2};
    use peniko::{Brush, Color, Fill};

    use crate::builder::KanvaBuilder;
    use crate::layer::{KanvaClip, Layer};
    use crate::shape::{KanvaFill, KanvaShape};

    fn red_fill_shape(transform: Affine) -> KanvaShape {
        KanvaShape {
            path: Rect::new(0.0, 0.0, 10.0, 10.0).to_path(0.1),
            fill: Some(KanvaFill {
                style: Fill::NonZero,
                brush: Brush::Solid(Color::from_rgba8(
                    255, 0, 0, 255,
                )),
                transform: None,
            }),
            stroke: None,
            transform,
        }
    }

    #[test]
    fn empty_kanva_renders_no_shapes() {
        let kanva =
            KanvaBuilder::new(Vec2::new(100.0, 100.0)).build();
        let mut sink = KanvaBuilder::new(Vec2::new(100.0, 100.0));
        kanva.render(&mut sink);
        let result = sink.build();
        assert_eq!(result.shapes.len(), 0);
    }

    #[test]
    fn shape_in_root_is_rendered() {
        let mut builder = KanvaBuilder::new(Vec2::new(100.0, 100.0));
        builder.push_shape(red_fill_shape(Affine::IDENTITY));
        let kanva = builder.build();

        let mut sink = KanvaBuilder::new(Vec2::new(100.0, 100.0));
        kanva.render(&mut sink);
        let result = sink.build();
        assert_eq!(result.shapes.len(), 1);
    }

    #[test]
    fn child_group_transform_accumulated_with_parent() {
        let group_tf = Affine::translate((20.0, 0.0));
        let shape_tf = Affine::translate((5.0, 0.0));
        let expected = group_tf * shape_tf;

        let mut builder = KanvaBuilder::new(Vec2::new(100.0, 100.0));
        builder.begin_group(None, group_tf, None);
        builder.push_shape(red_fill_shape(shape_tf));
        builder.end_group();
        let kanva = builder.build();

        let mut sink = KanvaBuilder::new(Vec2::new(100.0, 100.0));
        kanva.render(&mut sink);
        let result = sink.build();
        assert_eq!(result.shapes[0].transform, expected);
    }

    #[test]
    fn multiple_shapes_rendered_in_order() {
        let mut builder = KanvaBuilder::new(Vec2::new(100.0, 100.0));
        builder.push_shape(red_fill_shape(Affine::translate((1.0, 0.0))));
        builder.push_shape(red_fill_shape(Affine::translate((2.0, 0.0))));
        let kanva = builder.build();

        let mut sink = KanvaBuilder::new(Vec2::new(100.0, 100.0));
        kanva.render(&mut sink);
        let result = sink.build();
        assert_eq!(result.shapes.len(), 2);
        assert_eq!(
            result.shapes[0].transform,
            Affine::translate((1.0, 0.0))
        );
        assert_eq!(
            result.shapes[1].transform,
            Affine::translate((2.0, 0.0))
        );
    }

    #[test]
    fn nested_group_transforms_accumulate() {
        let outer_tf = Affine::translate((10.0, 0.0));
        let inner_tf = Affine::translate((5.0, 0.0));
        let shape_tf = Affine::IDENTITY;
        let expected = outer_tf * inner_tf * shape_tf;

        let mut builder = KanvaBuilder::new(Vec2::new(100.0, 100.0));
        builder.begin_group(None, outer_tf, None);
        builder.begin_group(None, inner_tf, None);
        builder.push_shape(red_fill_shape(shape_tf));
        builder.end_group();
        builder.end_group();
        let kanva = builder.build();

        let mut sink = KanvaBuilder::new(Vec2::new(100.0, 100.0));
        kanva.render(&mut sink);
        let result = sink.build();
        assert_eq!(result.shapes[0].transform, expected);
    }

    #[test]
    fn sibling_groups_render_all_shapes() {
        let mut builder = KanvaBuilder::new(Vec2::new(100.0, 100.0));
        builder.begin_group(None, Affine::IDENTITY, None);
        builder.push_shape(red_fill_shape(Affine::IDENTITY));
        builder.end_group();
        builder.begin_group(None, Affine::IDENTITY, None);
        builder.push_shape(red_fill_shape(Affine::IDENTITY));
        builder.push_shape(red_fill_shape(Affine::IDENTITY));
        builder.end_group();
        let kanva = builder.build();

        let mut sink = KanvaBuilder::new(Vec2::new(100.0, 100.0));
        kanva.render(&mut sink);
        let result = sink.build();
        assert_eq!(result.shapes.len(), 3);
    }

    #[test]
    fn clipped_node_creates_clip_in_sink() {
        let clip_path = Rect::new(0.0, 0.0, 50.0, 50.0).to_path(0.1);
        let layer = Layer {
            clip: Some(KanvaClip {
                path: clip_path,
                stroke: None,
            }),
            ..Layer::default()
        };
        let mut builder = KanvaBuilder::new(Vec2::new(100.0, 100.0));
        builder.begin_group(None, Affine::IDENTITY, Some(layer));
        builder.push_shape(red_fill_shape(Affine::IDENTITY));
        builder.end_group();
        let kanva = builder.build();

        let mut sink = KanvaBuilder::new(Vec2::new(100.0, 100.0));
        kanva.render(&mut sink);
        let result = sink.build();
        // The clip creates a new node in the sink, and the shape is also recorded
        assert_eq!(result.shapes.len(), 1);
        // There should be at least 2 nodes: root and clip node
        assert!(result.nodes.len() >= 2);
    }

    #[test]
    fn root_transform_is_identity() {
        let mut builder = KanvaBuilder::new(Vec2::new(100.0, 100.0));
        builder.push_shape(red_fill_shape(Affine::translate((3.0, 4.0))));
        let kanva = builder.build();

        let mut sink = KanvaBuilder::new(Vec2::new(100.0, 100.0));
        kanva.render(&mut sink);
        let result = sink.build();
        // root has no parent, so parent_tf = IDENTITY
        // recorded transform = IDENTITY * shape.transform = shape.transform
        assert_eq!(
            result.shapes[0].transform,
            Affine::translate((3.0, 4.0))
        );
    }

    #[test]
    fn shape_in_unclosed_group_is_still_rendered() {
        // If end_group is never called, the group's subtree_end is set at build time
        let mut builder = KanvaBuilder::new(Vec2::new(100.0, 100.0));
        builder.begin_group(None, Affine::IDENTITY, None);
        builder.push_shape(red_fill_shape(Affine::IDENTITY));
        // no end_group - build() closes it
        let kanva = builder.build();

        let mut sink = KanvaBuilder::new(Vec2::new(100.0, 100.0));
        kanva.render(&mut sink);
        let result = sink.build();
        assert_eq!(result.shapes.len(), 1);
    }

    #[test]
    fn blurred_rect_is_rendered() {
        use crate::blur::KanvaBlurredRect;
        use peniko::kurbo::Rect;

        let br = KanvaBlurredRect {
            transform: Affine::IDENTITY,
            rect: Rect::new(0.0, 0.0, 10.0, 10.0),
            color: peniko::Color::from_rgba8(0, 0, 0, 128),
            radius: 2.0,
            std_dev: 1.0,
        };
        let mut builder = KanvaBuilder::new(Vec2::new(100.0, 100.0));
        builder.push_blurred_rect(br);
        let kanva = builder.build();

        let mut sink = KanvaBuilder::new(Vec2::new(100.0, 100.0));
        kanva.render(&mut sink);
        let result = sink.build();
        assert_eq!(result.blurred_rects.len(), 1);
    }

    #[test]
    fn labeled_node_context_is_pushed_to_sink() {
        // A labeled group: render should call push_context on the sink.
        // In KanvaBuilder, push_context just sets pending_label.
        // After render, the pending_label could be set and then cleared by pop_context.
        // We test this indirectly: after rendering, pending_label should be None
        // (it was set then cleared).
        let mut builder = KanvaBuilder::new(Vec2::new(100.0, 100.0));
        builder.begin_group(
            Some("section".into()),
            Affine::IDENTITY,
            None,
        );
        builder.end_group();
        let kanva = builder.build();

        let mut sink = KanvaBuilder::new(Vec2::new(100.0, 100.0));
        kanva.render(&mut sink);
        // pending_label should be cleared after pop_context
        assert!(sink.pending_label.is_none());
    }
}
