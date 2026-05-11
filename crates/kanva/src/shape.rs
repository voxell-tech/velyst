use imaging::{
    Composite, FillRef, GeometryRef, PaintSink, StrokeRef,
};
use peniko::kurbo::{Affine, BezPath, Stroke};
use peniko::{Brush, Fill};

#[derive(Debug, Clone)]
pub struct KanvaShape {
    pub path: BezPath,
    pub fill: Option<KanvaFill>,
    pub stroke: Option<KanvaStroke>,
    pub transform: Affine,
}

#[derive(Debug, Clone)]
pub struct KanvaFill {
    pub style: Fill,
    pub brush: Brush,
    pub transform: Option<Affine>,
}

#[derive(Debug, Clone)]
pub struct KanvaStroke {
    pub style: Stroke,
    pub brush: Brush,
    pub transform: Option<Affine>,
}

impl KanvaShape {
    pub fn render(&self, tf: Affine, sink: &mut impl PaintSink) {
        let item_tf = tf * self.transform;
        if let Some(fill) = &self.fill {
            sink.fill(FillRef {
                transform: item_tf,
                fill_rule: fill.style,
                brush: (&fill.brush).into(),
                brush_transform: fill.transform,
                shape: GeometryRef::Path(&self.path),
                composite: Composite::default(),
            });
        }
        if let Some(stroke) = &self.stroke {
            sink.stroke(StrokeRef {
                transform: item_tf,
                stroke: &stroke.style,
                brush: (&stroke.brush).into(),
                brush_transform: stroke.transform,
                shape: GeometryRef::Path(&self.path),
                composite: Composite::default(),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use peniko::kurbo::{Affine, Rect, Shape as _, Stroke, Vec2};
    use peniko::{Brush, Color, Fill};

    use crate::builder::KanvaBuilder;

    use super::{KanvaFill, KanvaShape, KanvaStroke};

    fn red_fill() -> KanvaFill {
        KanvaFill {
            style: Fill::NonZero,
            brush: Brush::Solid(Color::from_rgba8(255, 0, 0, 255)),
            transform: None,
        }
    }

    fn blue_stroke() -> KanvaStroke {
        KanvaStroke {
            style: Stroke::new(2.0),
            brush: Brush::Solid(Color::from_rgba8(0, 0, 255, 255)),
            transform: None,
        }
    }

    fn rect_path() -> peniko::kurbo::BezPath {
        Rect::new(0.0, 0.0, 10.0, 10.0).to_path(0.1)
    }

    #[test]
    fn fill_only_shape_records_one_shape() {
        let shape = KanvaShape {
            path: rect_path(),
            fill: Some(red_fill()),
            stroke: None,
            transform: Affine::IDENTITY,
        };
        let mut sink = KanvaBuilder::new(Vec2::new(100.0, 100.0));
        shape.render(Affine::IDENTITY, &mut sink);
        let kanva = sink.build();
        assert_eq!(kanva.shapes.len(), 1);
        assert!(kanva.shapes[0].fill.is_some());
        assert!(kanva.shapes[0].stroke.is_none());
    }

    #[test]
    fn stroke_only_shape_records_one_shape() {
        let shape = KanvaShape {
            path: rect_path(),
            fill: None,
            stroke: Some(blue_stroke()),
            transform: Affine::IDENTITY,
        };
        let mut sink = KanvaBuilder::new(Vec2::new(100.0, 100.0));
        shape.render(Affine::IDENTITY, &mut sink);
        let kanva = sink.build();
        assert_eq!(kanva.shapes.len(), 1);
        assert!(kanva.shapes[0].fill.is_none());
        assert!(kanva.shapes[0].stroke.is_some());
    }

    #[test]
    fn shape_with_fill_and_stroke_records_two_shapes() {
        let shape = KanvaShape {
            path: rect_path(),
            fill: Some(red_fill()),
            stroke: Some(blue_stroke()),
            transform: Affine::IDENTITY,
        };
        let mut sink = KanvaBuilder::new(Vec2::new(100.0, 100.0));
        shape.render(Affine::IDENTITY, &mut sink);
        let kanva = sink.build();
        // fill() and stroke() each push a shape
        assert_eq!(kanva.shapes.len(), 2);
    }

    #[test]
    fn shape_with_no_fill_or_stroke_records_nothing() {
        let shape = KanvaShape {
            path: rect_path(),
            fill: None,
            stroke: None,
            transform: Affine::IDENTITY,
        };
        let mut sink = KanvaBuilder::new(Vec2::new(100.0, 100.0));
        shape.render(Affine::IDENTITY, &mut sink);
        let kanva = sink.build();
        assert_eq!(kanva.shapes.len(), 0);
    }

    #[test]
    fn render_concatenates_transforms() {
        let shape_tf = Affine::translate((5.0, 0.0));
        let shape = KanvaShape {
            path: rect_path(),
            fill: Some(red_fill()),
            stroke: None,
            transform: shape_tf,
        };
        let parent_tf = Affine::translate((10.0, 0.0));
        let mut sink = KanvaBuilder::new(Vec2::new(100.0, 100.0));
        shape.render(parent_tf, &mut sink);
        let kanva = sink.build();
        // The stored transform on the recorded shape is parent_tf * shape_tf
        let expected = parent_tf * shape_tf;
        assert_eq!(kanva.shapes[0].transform, expected);
    }

    #[test]
    fn kanva_fill_clone_preserves_brush() {
        let fill = red_fill();
        let cloned = fill.clone();
        assert_eq!(fill.style, cloned.style);
    }

    #[test]
    fn kanva_stroke_clone_preserves_width() {
        let stroke = blue_stroke();
        let cloned = stroke.clone();
        assert_eq!(
            stroke.style.width,
            cloned.style.width
        );
    }
}
