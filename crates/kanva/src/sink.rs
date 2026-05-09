use alloc::string::ToString;
use alloc::vec::Vec;

use imaging::record::Glyph;
use imaging::{
    BlurredRoundedRect, ClipRef, ContextRef, FillRef, GlyphRunRef,
    GroupRef, PaintSink, StrokeRef,
};
use peniko::Style;

use crate::KanvaNode;
use crate::blur::KanvaBlurredRect;
use crate::builder::KanvaBuilder;
use crate::layer::{KanvaClip, Layer};
use crate::shape::{KanvaFill, KanvaShape, KanvaStroke};
use crate::text::{GlyphPos, KanvaGlyphRun};

impl PaintSink for KanvaBuilder {
    fn push_context(&mut self, context: ContextRef<'_>) {
        self.pending_label = Some(context.label.to_string());
    }

    fn pop_context(&mut self) {
        self.pending_label = None;
    }

    fn push_clip(&mut self, clip: ClipRef<'_>) {
        let parent = self.current();
        let label = self.pending_label.take();
        let index = self.kanva.push_node(KanvaNode {
            parent: Some(parent),
            label,
            transform: peniko::kurbo::Affine::IDENTITY,
            layer: Some(Layer {
                clip: Some(clip_to_kanva(clip)),
                ..Layer::default()
            }),
            subtree_end: 0,
            shapes: Vec::new(),
            glyph_runs: Vec::new(),
            outlined_texts: Vec::new(),
            images: Vec::new(),
            blurred_rects: Vec::new(),
        });
        self.stack.push(index);
    }

    fn pop_clip(&mut self) {
        if self.stack.len() > 1 {
            let index = self.stack.pop().unwrap();
            self.kanva.nodes[index].subtree_end =
                self.kanva.nodes.len();
        }
    }

    fn push_group(&mut self, group: GroupRef<'_>) {
        let parent = self.current();
        let label = self.pending_label.take();
        let index = self.kanva.push_node(KanvaNode {
            parent: Some(parent),
            label,
            transform: peniko::kurbo::Affine::IDENTITY,
            layer: Some(Layer {
                blend_mode: group.composite.blend,
                alpha: group.composite.alpha,
                clip: group.clip.map(clip_to_kanva),
            }),
            subtree_end: 0,
            shapes: Vec::new(),
            glyph_runs: Vec::new(),
            outlined_texts: Vec::new(),
            images: Vec::new(),
            blurred_rects: Vec::new(),
        });
        self.stack.push(index);
    }

    fn pop_group(&mut self) {
        if self.stack.len() > 1 {
            let index = self.stack.pop().unwrap();
            self.kanva.nodes[index].subtree_end =
                self.kanva.nodes.len();
        }
    }

    fn fill(&mut self, draw: FillRef<'_>) {
        self.push_shape(KanvaShape {
            path: draw.shape.to_path(0.1),
            fill: Some(KanvaFill {
                style: draw.fill_rule,
                brush: draw.brush.to_owned(),
                transform: draw.brush_transform,
            }),
            stroke: None,
            transform: draw.transform,
        });
    }

    fn stroke(&mut self, draw: StrokeRef<'_>) {
        self.push_shape(KanvaShape {
            path: draw.shape.to_path(0.1),
            fill: None,
            stroke: Some(KanvaStroke {
                style: draw.stroke.clone(),
                brush: draw.brush.to_owned(),
                transform: draw.brush_transform,
            }),
            transform: draw.transform,
        });
    }

    fn glyph_run(
        &mut self,
        draw: GlyphRunRef<'_>,
        glyphs: &mut dyn Iterator<Item = Glyph>,
    ) {
        let stroke = match draw.style {
            Style::Stroke(s) => Some(KanvaStroke {
                style: s.clone(),
                brush: draw.brush.to_owned(),
                transform: None,
            }),
            Style::Fill(_) => None,
        };

        self.push_glyph_run(KanvaGlyphRun {
            font: draw.font.clone(),
            font_size: draw.font_size,
            hint: draw.hint,
            brush: draw.brush.to_owned(),
            stroke,
            transform: draw.transform,
            glyphs: glyphs
                .map(|g| GlyphPos {
                    id: g.id,
                    x: g.x,
                    y: g.y,
                })
                .collect(),
        });
    }

    fn blurred_rounded_rect(&mut self, draw: BlurredRoundedRect) {
        self.push_blurred_rect(KanvaBlurredRect {
            transform: draw.transform,
            rect: draw.rect,
            color: draw.color,
            radius: draw.radius,
            std_dev: draw.std_dev,
        });
    }
}

fn clip_to_kanva(clip: ClipRef<'_>) -> KanvaClip {
    match clip {
        ClipRef::Fill { shape, .. } => KanvaClip {
            path: shape.to_path(0.1),
            stroke: None,
        },
        ClipRef::Stroke { shape, stroke, .. } => KanvaClip {
            path: shape.to_path(0.1),
            stroke: Some(stroke.clone()),
        },
    }
}
