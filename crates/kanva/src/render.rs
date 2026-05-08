use alloc::vec::Vec;

use imaging::record::Glyph;
use imaging::{
    BlurredRoundedRect, ClipRef, Composite, ContextRef, FillRef,
    GeometryRef, GlyphRunRef, PaintSink, StrokeRef,
};
use peniko::kurbo::Affine;
use peniko::{Brush, ImageBrush, Style};

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

            if let Some(layer) = &node.layer {
                if let Some(clip) = &layer.clip {
                    let clip_ref = match &clip.stroke {
                        None => ClipRef::fill(GeometryRef::Path(
                            &clip.path,
                        ))
                        .with_transform(tf),
                        Some(stroke) => ClipRef::stroke(
                            GeometryRef::Path(&clip.path),
                            stroke,
                        )
                        .with_transform(tf),
                    };
                    sink.push_clip(clip_ref);
                    clip_stack.push(node.subtree_end);
                }
            }

            for &si in &node.shapes {
                let shape = &self.shapes[si];
                let item_tf = tf * shape.transform;
                if let Some(fill) = &shape.fill {
                    sink.fill(FillRef {
                        transform: item_tf,
                        fill_rule: fill.style,
                        brush: (&fill.brush).into(),
                        brush_transform: fill.transform,
                        shape: GeometryRef::Path(&shape.path),
                        composite: Composite::default(),
                    });
                }
                if let Some(stroke) = &shape.stroke {
                    sink.stroke(StrokeRef {
                        transform: item_tf,
                        stroke: &stroke.style,
                        brush: (&stroke.brush).into(),
                        brush_transform: stroke.transform,
                        shape: GeometryRef::Path(&shape.path),
                        composite: Composite::default(),
                    });
                }
            }

            for &gi in &node.glyph_runs {
                let run = &self.glyph_runs[gi];
                let run_tf = tf * run.transform;
                let fill_style = Style::Fill(peniko::Fill::NonZero);
                sink.glyph_run(
                    GlyphRunRef {
                        font: &run.font,
                        transform: run_tf,
                        glyph_transform: None,
                        font_size: run.font_size,
                        font_embolden: peniko::kurbo::Vec2::ZERO,
                        hint: run.hint,
                        normalized_coords: &[],
                        style: &fill_style,
                        brush: (&run.brush).into(),
                        composite: Composite::default(),
                    },
                    &mut run.glyphs.iter().map(|g| Glyph {
                        id: g.id,
                        x: g.x,
                        y: g.y,
                    }),
                );
                if let Some(stroke) = &run.stroke {
                    let stroke_style =
                        Style::Stroke(stroke.style.clone());
                    sink.glyph_run(
                        GlyphRunRef {
                            font: &run.font,
                            transform: run_tf,
                            glyph_transform: None,
                            font_size: run.font_size,
                            font_embolden: peniko::kurbo::Vec2::ZERO,
                            hint: run.hint,
                            normalized_coords: &[],
                            style: &stroke_style,
                            brush: (&stroke.brush).into(),
                            composite: Composite::default(),
                        },
                        &mut run.glyphs.iter().map(|g| Glyph {
                            id: g.id,
                            x: g.x,
                            y: g.y,
                        }),
                    );
                }
            }

            for &oi in &node.outlined_texts {
                let text = &self.outlined_texts[oi];
                for glyph in &text.glyphs {
                    let glyph_tf = tf * glyph.transform;
                    if let Some(fill_tf) = glyph.fill_transform {
                        sink.fill(FillRef {
                            transform: glyph_tf,
                            fill_rule: peniko::Fill::NonZero,
                            brush: (&text.brush).into(),
                            brush_transform: Some(fill_tf),
                            shape: GeometryRef::Path(&glyph.path),
                            composite: Composite::default(),
                        });
                    } else {
                        sink.fill(FillRef {
                            transform: glyph_tf,
                            fill_rule: peniko::Fill::NonZero,
                            brush: (&text.brush).into(),
                            brush_transform: None,
                            shape: GeometryRef::Path(&glyph.path),
                            composite: Composite::default(),
                        });
                    }
                    if let Some(stroke) = &text.stroke {
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

            for &ii in &node.images {
                let image = &self.images[ii];
                let item_tf = tf * image.transform;
                let brush =
                    Brush::Image(ImageBrush::new(image.data.clone()));
                sink.fill(FillRef {
                    transform: item_tf,
                    fill_rule: peniko::Fill::NonZero,
                    brush: (&brush).into(),
                    brush_transform: None,
                    shape: GeometryRef::from(
                        peniko::kurbo::Rect::from_origin_size(
                            (0.0, 0.0),
                            (image.size.x, image.size.y),
                        ),
                    ),
                    composite: Composite::default(),
                });
            }

            for &bi in &node.blurred_rects {
                let b = &self.blurred_rects[bi];
                sink.blurred_rounded_rect(BlurredRoundedRect {
                    transform: tf * b.transform,
                    rect: b.rect,
                    color: b.color,
                    radius: b.radius,
                    std_dev: b.std_dev,
                    composite: Composite::default(),
                });
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
