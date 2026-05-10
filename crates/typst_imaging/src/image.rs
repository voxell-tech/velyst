use std::sync::Arc;

use imaging::{Composite, FillRef, GeometryRef, PaintSink};
use peniko::kurbo::Rect;
use peniko::{
    Blob, Brush, ImageAlphaType, ImageBrush, ImageData, ImageFormat,
};
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
            // TODO(nixon): This is a hack to share the same id for blobs
            // pointing to the same data. We shud find a better way here.
            let id = raster.data().as_ptr() as u64;

            // TODO(nixon): We should optimize this to prevent
            // re-computing and re-allocating image data.
            let rgba = raster.dynamic().to_rgba8();
            let width = rgba.width();
            let height = rgba.height();

            let image_data = ImageData {
                data: Blob::from_raw_parts(
                    Arc::new(rgba.into_vec()),
                    id,
                ),
                format: ImageFormat::Rgba8,
                alpha_type: ImageAlphaType::Alpha,
                width,
                height,
            };

            let brush = Brush::Image(ImageBrush::new(image_data));
            let transform = state.transform.pre_scale_non_uniform(
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
                composite: Composite::default(),
            });
        }
        // TODO: SVG and PDF image rendering.
        ImageKind::Svg(_) | ImageKind::Pdf(_) => {}
    }
}
