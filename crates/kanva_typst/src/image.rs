use std::sync::Arc;

use kanva::imaging::Composite;
use kanva::imaging::kurbo::{Affine, Rect, Shape as _};
use kanva::imaging::peniko::{
    Blob, Brush, Fill, ImageAlphaType, ImageBrush, ImageData,
    ImageFormat,
};
use kanva::prelude::*;
use typst_imaging::RenderState;
use typst_library::layout::Size;
use typst_library::visualize::{Image, ImageKind};

pub fn render_image(
    image: &Image,
    size: Size,
    sink: &mut impl KanvaSink,
    state: RenderState,
) {
    match image.kind() {
        ImageKind::Raster(raster) => {
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

            let transform = state.transform.pre_scale_non_uniform(
                size.x.to_pt() / width as f64,
                size.y.to_pt() / height as f64,
            );

            sink.draw_path(
                Rect::new(0.0, 0.0, width as f64, height as f64)
                    .to_path(0.1),
                transform,
                Some(KanvaFill {
                    rule: Fill::NonZero,
                    brush: Brush::Image(ImageBrush::new(image_data)),
                    brush_transform: None,
                    composite: Composite::default(),
                }),
                None,
                Default::default(),
            );
        }
        ImageKind::Svg(svg) => {
            let options = usvg::Options::default();
            let Ok(tree) =
                usvg::Tree::from_data(svg.data().as_ref(), &options)
            else {
                return;
            };
            let svg_size = tree.size();
            if svg_size.width() < f32::EPSILON
                || svg_size.height() < f32::EPSILON
            {
                return;
            }
            let scale = Affine::scale_non_uniform(
                size.x.to_pt() / f64::from(svg_size.width()),
                size.y.to_pt() / f64::from(svg_size.height()),
            );
            kanva_svg::render_svg(
                &tree,
                sink,
                state.transform * scale,
            );
        }
        ImageKind::Pdf(pdf) => {
            let (w, h) = (pdf.width() as f64, pdf.height() as f64);
            if w < f64::EPSILON || h < f64::EPSILON {
                return;
            }
            let svg_str =
                hayro_svg::convert(pdf.page(), &Default::default());
            let options = usvg::Options::default();
            let Ok(tree) = usvg::Tree::from_str(&svg_str, &options)
            else {
                return;
            };
            let svg_size = tree.size();
            let scale = Affine::scale_non_uniform(
                size.x.to_pt() / f64::from(svg_size.width()),
                size.y.to_pt() / f64::from(svg_size.height()),
            );
            kanva_svg::render_svg(
                &tree,
                sink,
                state.transform * scale,
            );
        }
    }
}
