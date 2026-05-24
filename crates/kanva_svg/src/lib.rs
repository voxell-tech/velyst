use convert::*;
use kanva::imaging::Composite;
use kanva::imaging::kurbo::{Affine, Shape as _};
use kanva::imaging::peniko::{
    Blob, Brush, Fill, ImageAlphaType, ImageBrush, ImageData,
    ImageFormat,
};
use kanva::prelude::*;
use usvg::Node;

mod convert;

/// Walk a usvg [`usvg::Tree`] and emit all draw commands into `sink`.
///
/// `transform` is the root world transform applied to all paths.
pub fn render_svg(
    tree: &usvg::Tree,
    sink: &mut impl KanvaSink,
    transform: Affine,
) {
    render_group(tree.root(), sink, transform);
}

pub fn render_node(
    node: &Node,
    sink: &mut impl KanvaSink,
    transform: Affine,
) {
    match node {
        Node::Group(group) => render_group(group, sink, transform),
        Node::Path(path) => render_path(path, sink, transform),
        Node::Image(image) => render_image(image, sink, transform),
        Node::Text(text) => {
            render_group(text.flattened(), sink, transform)
        }
    }
}

pub fn render_group(
    group: &usvg::Group,
    sink: &mut impl KanvaSink,
    transform: Affine,
) {
    if !group.filters().is_empty() {
        eprintln!(
            "kanva_svg: filters unsupported (id={:?})",
            group.id()
        );
    }
    if group.mask().is_some() {
        eprintln!(
            "kanva_svg: masks unsupported (id={:?})",
            group.id()
        );
    }

    let has_id = !group.id().is_empty();
    if has_id {
        sink.push_context(group.id());
    }

    let composite = Composite::new(
        map_blend_mode(group.blend_mode()),
        group.opacity().get(),
    );
    let clip = merge_clip_chain(group.clip_path(), transform);

    sink.push_group(Group {
        clip,
        composite,
        ..Default::default()
    });

    for child in group.children() {
        render_node(child, sink, transform);
    }

    sink.pop_group();

    if has_id {
        sink.pop_context();
    }
}

pub fn render_path(
    path: &usvg::Path,
    sink: &mut impl KanvaSink,
    transform: Affine,
) {
    if !path.is_visible() {
        return;
    }

    let geometry = convert_path(path.data());
    let transform =
        transform * convert_transform(path.abs_transform());
    let fill = convert_fill(path.fill());
    let stroke = convert_stroke(path.stroke());
    let paint_order = convert_paint_order(path.paint_order());

    sink.draw_path(geometry, transform, fill, stroke, paint_order);
}

pub fn render_image(
    image: &usvg::Image,
    sink: &mut impl KanvaSink,
    transform: Affine,
) {
    if !image.is_visible() {
        return;
    }
    match image.kind() {
        usvg::ImageKind::SVG(tree) => {
            render_svg(
                tree,
                sink,
                transform * convert_transform(image.abs_transform()),
            );
        }
        usvg::ImageKind::PNG(data) => {
            render_raster(
                data,
                image::ImageFormat::Png,
                image,
                sink,
                transform,
            );
        }
        usvg::ImageKind::JPEG(data) => {
            render_raster(
                data,
                image::ImageFormat::Jpeg,
                image,
                sink,
                transform,
            );
        }
        usvg::ImageKind::GIF(data) => {
            render_raster(
                data,
                image::ImageFormat::Gif,
                image,
                sink,
                transform,
            );
        }
        usvg::ImageKind::WEBP(data) => {
            render_raster(
                data,
                image::ImageFormat::WebP,
                image,
                sink,
                transform,
            );
        }
    }
}

pub fn render_raster(
    data: &[u8],
    format: image::ImageFormat,
    image: &usvg::Image,
    sink: &mut impl KanvaSink,
    transform: Affine,
) {
    let Ok(decoded) =
        image::load_from_memory_with_format(data, format)
    else {
        return;
    };
    let rgba = decoded.into_rgba8();
    let (pixel_width, pixel_height) = rgba.dimensions();
    if pixel_width == 0 || pixel_height == 0 {
        return;
    }

    let display_width = f64::from(image.size().width());
    let display_height = f64::from(image.size().height());

    let image_data = ImageData {
        data: Blob::new(std::sync::Arc::new(rgba.into_vec())),
        format: ImageFormat::Rgba8,
        alpha_type: ImageAlphaType::Alpha,
        width: pixel_width,
        height: pixel_height,
    };
    let image_brush = with_rendering_quality(
        ImageBrush::new(image_data),
        image.rendering_mode(),
    );

    let transform = transform
        * convert_transform(image.abs_transform())
        * Affine::scale_non_uniform(
            display_width / pixel_width as f64,
            display_height / pixel_height as f64,
        );
    let rect = kanva::imaging::kurbo::Rect::new(
        0.0,
        0.0,
        pixel_width as f64,
        pixel_height as f64,
    )
    .to_path(0.1);

    sink.draw_path(
        rect,
        transform,
        Some(KanvaFill {
            rule: Fill::NonZero,
            brush: Brush::Image(image_brush),
            brush_transform: None,
            composite: Composite::default(),
        }),
        None,
        Default::default(),
    );
}
