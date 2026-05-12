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

#[derive(Default, Debug, Clone)]
pub struct KanvaFill {
    pub style: Fill,
    pub brush: Brush,
    pub transform: Option<Affine>,
    pub composite: Composite,
}

#[derive(Default, Debug, Clone)]
pub struct KanvaStroke {
    pub style: Stroke,
    pub brush: Brush,
    pub transform: Option<Affine>,
    pub composite: Composite,
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
                composite: fill.composite,
            });
        }
        if let Some(stroke) = &self.stroke {
            sink.stroke(StrokeRef {
                transform: item_tf,
                stroke: &stroke.style,
                brush: (&stroke.brush).into(),
                brush_transform: stroke.transform,
                shape: GeometryRef::Path(&self.path),
                composite: stroke.composite,
            });
        }
    }
}
