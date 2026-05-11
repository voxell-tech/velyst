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

#[cfg(test)]
mod tests {
    use alloc::sync::Arc;
    use alloc::vec;

    use peniko::kurbo::{Affine, Vec2};
    use peniko::{Blob, ImageAlphaType, ImageData, ImageFormat};

    use crate::builder::KanvaBuilder;

    use super::KanvaImage;

    fn tiny_image_data() -> ImageData {
        // 1x1 red RGBA pixel
        ImageData {
            data: Blob::new(Arc::new(vec![255u8, 0, 0, 255])),
            format: ImageFormat::Rgba8,
            alpha_type: ImageAlphaType::Alpha,
            width: 1,
            height: 1,
        }
    }

    #[test]
    fn render_records_one_shape() {
        let img = KanvaImage {
            data: tiny_image_data(),
            size: Vec2::new(100.0, 80.0),
            transform: Affine::IDENTITY,
        };
        let mut sink = KanvaBuilder::new(Vec2::new(200.0, 200.0));
        img.render(Affine::IDENTITY, &mut sink);
        let kanva = sink.build();
        assert_eq!(kanva.shapes.len(), 1);
    }

    #[test]
    fn render_shape_has_fill_no_stroke() {
        let img = KanvaImage {
            data: tiny_image_data(),
            size: Vec2::new(100.0, 80.0),
            transform: Affine::IDENTITY,
        };
        let mut sink = KanvaBuilder::new(Vec2::new(200.0, 200.0));
        img.render(Affine::IDENTITY, &mut sink);
        let kanva = sink.build();
        assert!(kanva.shapes[0].fill.is_some());
        assert!(kanva.shapes[0].stroke.is_none());
    }

    #[test]
    fn render_concatenates_transforms() {
        let shape_tf = Affine::translate((5.0, 10.0));
        let img = KanvaImage {
            data: tiny_image_data(),
            size: Vec2::new(50.0, 50.0),
            transform: shape_tf,
        };
        let parent_tf = Affine::translate((20.0, 30.0));
        let mut sink = KanvaBuilder::new(Vec2::new(200.0, 200.0));
        img.render(parent_tf, &mut sink);
        let kanva = sink.build();
        let expected = parent_tf * shape_tf;
        assert_eq!(kanva.shapes[0].transform, expected);
    }

    #[test]
    fn render_with_identity_transform_uses_identity() {
        let img = KanvaImage {
            data: tiny_image_data(),
            size: Vec2::new(32.0, 32.0),
            transform: Affine::IDENTITY,
        };
        let mut sink = KanvaBuilder::new(Vec2::new(200.0, 200.0));
        img.render(Affine::IDENTITY, &mut sink);
        let kanva = sink.build();
        assert_eq!(kanva.shapes[0].transform, Affine::IDENTITY);
    }

    #[test]
    fn clone_preserves_size() {
        let img = KanvaImage {
            data: tiny_image_data(),
            size: Vec2::new(64.0, 48.0),
            transform: Affine::IDENTITY,
        };
        let cloned = img.clone();
        assert_eq!(img.size, cloned.size);
    }
}
