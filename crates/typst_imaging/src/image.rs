use std::sync::Arc;

use hayro_svg::convert;
use imaging::peniko::kurbo::{Affine, Rect};
use imaging::peniko::{
    Blob, Brush, Fill, ImageAlphaType, ImageBrush, ImageData,
    ImageFormat,
};
use imaging::{Composite, FillRef, GeometryRef, PaintSink, Painter};
use svg_imaging::{RenderOptions, SvgDocument};
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
            // TODO(nixon): We should optimize this to prevent
            // re-computing and re-allocating image data.
            let rgba = raster.dynamic().to_rgba8();
            let width = rgba.width();
            let height = rgba.height();

            if width == 0 || height == 0 {
                return;
            }

            let image_data = ImageData {
                data: Blob::new(Arc::new(rgba.into_vec())),
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
                fill_rule: Fill::NonZero,
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
        ImageKind::Svg(svg) => {
            let Ok(doc) = SvgDocument::from_data(
                svg.data().as_ref(),
                &Default::default(),
            ) else {
                return;
            };

            let svg_size = doc.size();

            if svg_size.width.abs() < f64::EPSILON
                || svg_size.height.abs() < f64::EPSILON
            {
                return;
            }

            let scale = Affine::scale_non_uniform(
                size.x.to_pt() / svg_size.width,
                size.y.to_pt() / svg_size.height,
            );

            let mut painter = Painter::new(sink);
            let _ = doc.render(
                &mut painter,
                &RenderOptions {
                    transform: state.transform * scale,
                },
            );
        }
        ImageKind::Pdf(pdf) => {
            let (w, h) = (pdf.width() as f64, pdf.height() as f64);
            if w.abs() < f64::EPSILON || h.abs() < f64::EPSILON {
                return;
            }

            let svg_str = convert(pdf.page(), &Default::default());

            let Ok(doc) =
                SvgDocument::from_str(&svg_str, &Default::default())
            else {
                return;
            };

            let scale = Affine::scale_non_uniform(
                size.x.to_pt() / w,
                size.y.to_pt() / h,
            );

            let mut painter = Painter::new(sink);
            let _ = doc.render(
                &mut painter,
                &RenderOptions {
                    transform: state.transform * scale,
                },
            );
        }
    }
}
