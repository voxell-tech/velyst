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
}

impl KanvaBlurredRect {
    pub fn render(&self, tf: Affine, sink: &mut impl PaintSink) {
        sink.blurred_rounded_rect(BlurredRoundedRect {
            transform: tf * self.transform,
            rect: self.rect,
            color: self.color,
            radius: self.radius,
            std_dev: self.std_dev,
            composite: Composite::default(),
        });
    }
}

#[cfg(test)]
mod tests {
    use peniko::kurbo::{Affine, Rect, Vec2};
    use peniko::Color;

    use crate::builder::KanvaBuilder;

    use super::KanvaBlurredRect;

    fn sample_blurred_rect() -> KanvaBlurredRect {
        KanvaBlurredRect {
            transform: Affine::IDENTITY,
            rect: Rect::new(0.0, 0.0, 50.0, 30.0),
            color: Color::from_rgba8(0, 0, 0, 128),
            radius: 4.0,
            std_dev: 2.0,
        }
    }

    #[test]
    fn render_records_one_blurred_rect() {
        let br = sample_blurred_rect();
        let mut sink = KanvaBuilder::new(Vec2::new(100.0, 100.0));
        br.render(Affine::IDENTITY, &mut sink);
        let kanva = sink.build();
        assert_eq!(kanva.blurred_rects.len(), 1);
    }

    #[test]
    fn render_stores_correct_rect_dimensions() {
        let br = sample_blurred_rect();
        let mut sink = KanvaBuilder::new(Vec2::new(100.0, 100.0));
        br.render(Affine::IDENTITY, &mut sink);
        let kanva = sink.build();
        assert_eq!(kanva.blurred_rects[0].rect, br.rect);
    }

    #[test]
    fn render_stores_correct_radius_and_std_dev() {
        let br = sample_blurred_rect();
        let mut sink = KanvaBuilder::new(Vec2::new(100.0, 100.0));
        br.render(Affine::IDENTITY, &mut sink);
        let kanva = sink.build();
        assert_eq!(kanva.blurred_rects[0].radius, br.radius);
        assert_eq!(kanva.blurred_rects[0].std_dev, br.std_dev);
    }

    #[test]
    fn render_concatenates_transforms() {
        let shape_tf = Affine::translate((10.0, 20.0));
        let mut br = sample_blurred_rect();
        br.transform = shape_tf;
        let parent_tf = Affine::translate((5.0, 0.0));
        let mut sink = KanvaBuilder::new(Vec2::new(100.0, 100.0));
        br.render(parent_tf, &mut sink);
        let kanva = sink.build();
        // The sink stores the transform as passed in draw.transform = tf * self.transform
        let expected = parent_tf * shape_tf;
        assert_eq!(kanva.blurred_rects[0].transform, expected);
    }

    #[test]
    fn clone_preserves_all_fields() {
        let br = sample_blurred_rect();
        let cloned = br.clone();
        assert_eq!(br.radius, cloned.radius);
        assert_eq!(br.std_dev, cloned.std_dev);
        assert_eq!(br.rect, cloned.rect);
        assert_eq!(br.color, cloned.color);
    }

    #[test]
    fn render_with_identity_parent_tf_uses_self_transform() {
        let shape_tf = Affine::translate((3.0, 7.0));
        let mut br = sample_blurred_rect();
        br.transform = shape_tf;
        let mut sink = KanvaBuilder::new(Vec2::new(100.0, 100.0));
        br.render(Affine::IDENTITY, &mut sink);
        let kanva = sink.build();
        assert_eq!(kanva.blurred_rects[0].transform, shape_tf);
    }
}
