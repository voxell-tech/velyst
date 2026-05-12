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
