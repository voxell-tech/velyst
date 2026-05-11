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
            outlined_glyphs: Vec::new(),
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
            outlined_glyphs: Vec::new(),
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

#[cfg(test)]
mod tests {
    use imaging::{
        BlurredRoundedRect, ClipRef, Composite, ContextRef, FillRef,
        GeometryRef, GroupRef, PaintSink, StrokeRef,
    };
    use peniko::kurbo::{Affine, BezPath, Rect, Shape as _, Stroke, Vec2};
    use peniko::{BlendMode, Brush, Color, Fill};

    use crate::builder::KanvaBuilder;

    fn rect_path() -> BezPath {
        Rect::new(0.0, 0.0, 10.0, 10.0).to_path(0.1)
    }

    fn red_brush() -> Brush {
        Brush::Solid(Color::from_rgba8(255, 0, 0, 255))
    }

    // ── push_context / pop_context ─────────────────────────────────────────

    #[test]
    fn push_context_sets_pending_label() {
        let mut builder = KanvaBuilder::new(Vec2::new(100.0, 100.0));
        builder.push_context(ContextRef::new("hello", None));
        assert_eq!(
            builder.pending_label.as_deref(),
            Some("hello")
        );
    }

    #[test]
    fn pop_context_clears_pending_label() {
        let mut builder = KanvaBuilder::new(Vec2::new(100.0, 100.0));
        builder.push_context(ContextRef::new("hello", None));
        builder.pop_context();
        assert!(builder.pending_label.is_none());
    }

    #[test]
    fn push_context_twice_updates_label() {
        let mut builder = KanvaBuilder::new(Vec2::new(100.0, 100.0));
        builder.push_context(ContextRef::new("first", None));
        builder.push_context(ContextRef::new("second", None));
        assert_eq!(
            builder.pending_label.as_deref(),
            Some("second")
        );
    }

    // ── fill ──────────────────────────────────────────────────────────────

    #[test]
    fn fill_creates_shape_with_fill_only() {
        let path = rect_path();
        let brush = red_brush();
        let mut builder = KanvaBuilder::new(Vec2::new(100.0, 100.0));
        builder.fill(FillRef {
            transform: Affine::IDENTITY,
            fill_rule: Fill::NonZero,
            brush: (&brush).into(),
            brush_transform: None,
            shape: GeometryRef::Path(&path),
            composite: Composite::default(),
        });
        let kanva = builder.build();
        assert_eq!(kanva.shapes.len(), 1);
        assert!(kanva.shapes[0].fill.is_some());
        assert!(kanva.shapes[0].stroke.is_none());
    }

    #[test]
    fn fill_stores_fill_rule() {
        let path = rect_path();
        let brush = red_brush();
        let mut builder = KanvaBuilder::new(Vec2::new(100.0, 100.0));
        builder.fill(FillRef {
            transform: Affine::IDENTITY,
            fill_rule: Fill::EvenOdd,
            brush: (&brush).into(),
            brush_transform: None,
            shape: GeometryRef::Path(&path),
            composite: Composite::default(),
        });
        let kanva = builder.build();
        assert_eq!(
            kanva.shapes[0].fill.as_ref().unwrap().style,
            Fill::EvenOdd
        );
    }

    #[test]
    fn fill_stores_transform() {
        let path = rect_path();
        let brush = red_brush();
        let tf = Affine::translate((5.0, 10.0));
        let mut builder = KanvaBuilder::new(Vec2::new(100.0, 100.0));
        builder.fill(FillRef {
            transform: tf,
            fill_rule: Fill::NonZero,
            brush: (&brush).into(),
            brush_transform: None,
            shape: GeometryRef::Path(&path),
            composite: Composite::default(),
        });
        let kanva = builder.build();
        assert_eq!(kanva.shapes[0].transform, tf);
    }

    // ── stroke ────────────────────────────────────────────────────────────

    #[test]
    fn stroke_creates_shape_with_stroke_only() {
        let path = rect_path();
        let brush = red_brush();
        let stroke = Stroke::new(2.0);
        let mut builder = KanvaBuilder::new(Vec2::new(100.0, 100.0));
        builder.stroke(StrokeRef {
            transform: Affine::IDENTITY,
            stroke: &stroke,
            brush: (&brush).into(),
            brush_transform: None,
            shape: GeometryRef::Path(&path),
            composite: Composite::default(),
        });
        let kanva = builder.build();
        assert_eq!(kanva.shapes.len(), 1);
        assert!(kanva.shapes[0].stroke.is_some());
        assert!(kanva.shapes[0].fill.is_none());
    }

    #[test]
    fn stroke_stores_stroke_width() {
        let path = rect_path();
        let brush = red_brush();
        let stroke = Stroke::new(3.5);
        let mut builder = KanvaBuilder::new(Vec2::new(100.0, 100.0));
        builder.stroke(StrokeRef {
            transform: Affine::IDENTITY,
            stroke: &stroke,
            brush: (&brush).into(),
            brush_transform: None,
            shape: GeometryRef::Path(&path),
            composite: Composite::default(),
        });
        let kanva = builder.build();
        assert_eq!(
            kanva.shapes[0].stroke.as_ref().unwrap().style.width,
            3.5
        );
    }

    // ── push_clip / pop_clip ──────────────────────────────────────────────

    #[test]
    fn push_clip_creates_new_node() {
        let path = rect_path();
        let mut builder = KanvaBuilder::new(Vec2::new(100.0, 100.0));
        let clip = ClipRef::fill(GeometryRef::Path(&path))
            .with_transform(Affine::IDENTITY);
        builder.push_clip(clip);
        // A new node was added (root + clip node)
        assert_eq!(builder.kanva.nodes.len(), 2);
    }

    #[test]
    fn push_clip_node_has_clip_layer() {
        let path = rect_path();
        let mut builder = KanvaBuilder::new(Vec2::new(100.0, 100.0));
        let clip = ClipRef::fill(GeometryRef::Path(&path))
            .with_transform(Affine::IDENTITY);
        builder.push_clip(clip);
        assert!(builder.kanva.nodes[1].layer.is_some());
        assert!(
            builder.kanva.nodes[1]
                .layer
                .as_ref()
                .unwrap()
                .clip
                .is_some()
        );
    }

    #[test]
    fn pop_clip_sets_subtree_end() {
        let path = rect_path();
        let mut builder = KanvaBuilder::new(Vec2::new(100.0, 100.0));
        let clip = ClipRef::fill(GeometryRef::Path(&path))
            .with_transform(Affine::IDENTITY);
        builder.push_clip(clip);
        builder.pop_clip();
        // subtree_end should be set to current node count (2 nodes = root + clip)
        assert_eq!(builder.kanva.nodes[1].subtree_end, 2);
    }

    #[test]
    fn pop_clip_with_only_root_is_safe() {
        let mut builder = KanvaBuilder::new(Vec2::new(100.0, 100.0));
        builder.pop_clip(); // stack only has root, should not pop
        assert_eq!(builder.stack.len(), 1);
    }

    #[test]
    fn stroke_clip_creates_clip_with_stroke() {
        let path = rect_path();
        let stroke = Stroke::new(1.0);
        let mut builder = KanvaBuilder::new(Vec2::new(100.0, 100.0));
        let clip =
            ClipRef::stroke(GeometryRef::Path(&path), &stroke)
                .with_transform(Affine::IDENTITY);
        builder.push_clip(clip);
        let clip_node = &builder.kanva.nodes[1];
        let stored_clip = clip_node
            .layer
            .as_ref()
            .unwrap()
            .clip
            .as_ref()
            .unwrap();
        assert!(stored_clip.stroke.is_some());
    }

    // ── push_group / pop_group ─────────────────────────────────────────────

    #[test]
    fn push_group_creates_node_with_layer() {
        let mut builder = KanvaBuilder::new(Vec2::new(100.0, 100.0));
        builder.push_group(GroupRef {
            composite: Composite {
                blend: BlendMode::default(),
                alpha: 0.5,
            },
            clip: None,
        });
        assert_eq!(builder.kanva.nodes.len(), 2);
        let layer = builder.kanva.nodes[1].layer.as_ref().unwrap();
        assert_eq!(layer.alpha, 0.5);
    }

    #[test]
    fn push_group_blend_mode_stored() {
        let mut builder = KanvaBuilder::new(Vec2::new(100.0, 100.0));
        let blend = BlendMode {
            mix: peniko::Mix::Multiply,
            compose: peniko::Compose::SrcOver,
        };
        builder.push_group(GroupRef {
            composite: Composite {
                blend,
                alpha: 1.0,
            },
            clip: None,
        });
        let layer = builder.kanva.nodes[1].layer.as_ref().unwrap();
        assert_eq!(layer.blend_mode, blend);
    }

    #[test]
    fn pop_group_sets_subtree_end() {
        let mut builder = KanvaBuilder::new(Vec2::new(100.0, 100.0));
        builder.push_group(GroupRef {
            composite: Composite {
                blend: BlendMode::default(),
                alpha: 1.0,
            },
            clip: None,
        });
        builder.pop_group();
        assert_eq!(builder.kanva.nodes[1].subtree_end, 2);
    }

    #[test]
    fn pop_group_with_only_root_is_safe() {
        let mut builder = KanvaBuilder::new(Vec2::new(100.0, 100.0));
        builder.pop_group();
        assert_eq!(builder.stack.len(), 1);
    }

    // ── blurred_rounded_rect ──────────────────────────────────────────────

    #[test]
    fn blurred_rounded_rect_records_blurred_rect() {
        let mut builder = KanvaBuilder::new(Vec2::new(100.0, 100.0));
        builder.blurred_rounded_rect(BlurredRoundedRect {
            transform: Affine::IDENTITY,
            rect: Rect::new(0.0, 0.0, 20.0, 10.0),
            color: Color::from_rgba8(0, 0, 0, 200),
            radius: 3.0,
            std_dev: 1.5,
            composite: Composite::default(),
        });
        let kanva = builder.build();
        assert_eq!(kanva.blurred_rects.len(), 1);
    }

    #[test]
    fn blurred_rounded_rect_stores_all_fields() {
        let rect = Rect::new(5.0, 5.0, 25.0, 15.0);
        let color = Color::from_rgba8(100, 150, 200, 255);
        let tf = Affine::translate((1.0, 2.0));
        let mut builder = KanvaBuilder::new(Vec2::new(100.0, 100.0));
        builder.blurred_rounded_rect(BlurredRoundedRect {
            transform: tf,
            rect,
            color,
            radius: 5.0,
            std_dev: 2.5,
            composite: Composite::default(),
        });
        let kanva = builder.build();
        let br = &kanva.blurred_rects[0];
        assert_eq!(br.transform, tf);
        assert_eq!(br.rect, rect);
        assert_eq!(br.radius, 5.0);
        assert_eq!(br.std_dev, 2.5);
    }

    // ── context label consumed by push_clip ───────────────────────────────

    #[test]
    fn push_context_label_consumed_by_push_clip() {
        let path = rect_path();
        let mut builder = KanvaBuilder::new(Vec2::new(100.0, 100.0));
        builder.push_context(ContextRef::new("my_clip", None));
        assert!(builder.pending_label.is_some());
        let clip = ClipRef::fill(GeometryRef::Path(&path))
            .with_transform(Affine::IDENTITY);
        builder.push_clip(clip);
        // Label should now be consumed (None) and stored in the node
        assert!(builder.pending_label.is_none());
        assert_eq!(
            builder.kanva.nodes[1].label.as_deref(),
            Some("my_clip")
        );
    }

    #[test]
    fn push_context_label_consumed_by_push_group() {
        let mut builder = KanvaBuilder::new(Vec2::new(100.0, 100.0));
        builder.push_context(ContextRef::new("my_group", None));
        builder.push_group(GroupRef {
            composite: Composite {
                blend: BlendMode::default(),
                alpha: 1.0,
            },
            clip: None,
        });
        assert!(builder.pending_label.is_none());
        assert_eq!(
            builder.kanva.nodes[1].label.as_deref(),
            Some("my_group")
        );
    }
}
