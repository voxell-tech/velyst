use peniko::{Color, kurbo::{Affine, Rect}};

#[derive(Debug, Clone)]
pub struct KanvaBlurredRect {
    pub transform: Affine,
    pub rect: Rect,
    pub color: Color,
    pub radius: f64,
    pub std_dev: f64,
}
