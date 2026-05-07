use std::sync::Arc;

use imaging::{FillRef, GeometryRef, PaintSink};
use peniko::kurbo::Rect;
use peniko::{Blob, Brush, ImageAlphaType, ImageBrush, ImageData, ImageFormat};
use typst_library::layout::Size;
use typst_library::visualize::{Image, ImageKind};

use crate::RenderState;

pub(crate) fn render_image(
    image: &Image,
    size: Size,
    sink: &mut impl PaintSink,
    state: RenderState,
) {
    match image.kind() {
        ImageKind::Raster(raster) => {
            let rgba = raster.dynamic().to_rgba8();
            let width = rgba.width();
            let height = rgba.height();

            let image_data = ImageData {
                data: Blob::new(Arc::new(rgba.into_vec())),
                format: ImageFormat::Rgba8,
                alpha_type: ImageAlphaType::Alpha,
                width,
                height,
            };

            let brush = Brush::Image(ImageBrush::new(image_data));
            let transform = state
                .transform
                .pre_scale_non_uniform(
                    size.x.to_pt() / width as f64,
                    size.y.to_pt() / height as f64,
                );

            sink.fill(FillRef {
                transform,
                fill_rule: peniko::Fill::NonZero,
                brush: (&brush).into(),
                brush_transform: None,
                shape: GeometryRef::Rect(Rect::new(
                    0.0,
                    0.0,
                    width as f64,
                    height as f64,
                )),
                composite: imaging::Composite::default(),
            });
        }
        // TODO: SVG and PDF image rendering.
        ImageKind::Svg(_) | ImageKind::Pdf(_) => {}
    }
}
