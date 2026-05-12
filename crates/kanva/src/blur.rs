use imaging::{BlurredRoundedRect, Composite, PaintSink};
use peniko::Color;
use peniko::kurbo::{Affine, Rect};

#[derive(Debug, Clone)]
pub struct KanvaBlurredRect {
    pub transform: Affine,
    pub rect: Rect,
    pub color: Color,
    pub radius: f64,
    pub std_dev: f64,
    pub composite: Composite,
}

impl KanvaBlurredRect {
    pub fn render(&self, tf: Affine, sink: &mut impl PaintSink) {
        sink.blurred_rounded_rect(BlurredRoundedRect {
            transform: tf * self.transform,
            rect: self.rect,
            color: self.color,
            radius: self.radius,
            std_dev: self.std_dev,
            composite: self.composite,
        });
    }
}
