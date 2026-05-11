use imaging::{Composite, FillRef, GeometryRef, PaintSink};
use peniko::kurbo::{Affine, Vec2};
use peniko::{Brush, ImageBrush, ImageData};

#[derive(Debug, Clone)]
pub struct KanvaImage {
    pub data: ImageData,
    pub size: Vec2,
    pub transform: Affine,
}

impl KanvaImage {
    pub fn render(&self, tf: Affine, sink: &mut impl PaintSink) {
        let brush = Brush::Image(ImageBrush::new(self.data.clone()));
        sink.fill(FillRef {
            transform: tf * self.transform,
            fill_rule: peniko::Fill::NonZero,
            brush: (&brush).into(),
            brush_transform: None,
            shape: GeometryRef::from(
                peniko::kurbo::Rect::from_origin_size(
                    (0.0, 0.0),
                    (self.size.x, self.size.y),
                ),
            ),
            composite: Composite::default(),
        });
    }
}
