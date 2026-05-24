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

/// Warnings emitted during SVG rendering for unsupported features.
///
/// Collected and returned by [`render_svg`]. An empty vec means the
/// SVG rendered without any degradation.
#[derive(Debug, Clone)]
pub enum SvgWarning {
    /// SVG filters are not supported; the group's filters were
    /// ignored.
    FiltersUnsupported { id: Box<str> },
    /// SVG masks are not supported; the group's mask was ignored.
    MaskUnsupported { id: Box<str> },
    /// A clip path was too complex to represent as a single
    /// `KanvaClip` and was skipped.
    ComplexClipPathSkipped,
    /// SVG pattern paint is not supported; the path was drawn
    /// without fill or stroke.
    PatternPaintUnsupported,
}

/// Walk a usvg [`usvg::Tree`] and emit all draw commands into `sink`.
///
/// `transform` is the root world transform applied to all paths.
/// Returns any warnings for unsupported SVG features encountered
/// during the walk.
pub fn render_svg(
    tree: &usvg::Tree,
    sink: &mut impl KanvaSink,
    transform: Affine,
) -> Vec<SvgWarning> {
    let mut warnings = Vec::new();
    render_group(tree.root(), sink, transform, &mut warnings);
    warnings
}

pub fn render_node(
    node: &Node,
    sink: &mut impl KanvaSink,
    transform: Affine,
    warnings: &mut Vec<SvgWarning>,
) {
    match node {
        Node::Group(group) => {
            render_group(group, sink, transform, warnings)
        }
        Node::Path(path) => {
            render_path(path, sink, transform, warnings)
        }
        Node::Image(image) => {
            render_image(image, sink, transform, warnings)
        }
        Node::Text(text) => {
            render_group(text.flattened(), sink, transform, warnings)
        }
    }
}

pub fn render_group(
    group: &usvg::Group,
    sink: &mut impl KanvaSink,
    transform: Affine,
    warnings: &mut Vec<SvgWarning>,
) {
    let group_id = group.id();
    if !group.filters().is_empty() {
        warnings.push(SvgWarning::FiltersUnsupported {
            id: group_id.into(),
        });
    }
    if group.mask().is_some() {
        warnings.push(SvgWarning::MaskUnsupported {
            id: group_id.into(),
        });
    }

    let has_id = !group_id.is_empty();
    if has_id {
        sink.push_context(group_id);
    }

    let composite = Composite::new(
        convert_blend_mode(group.blend_mode()),
        group.opacity().get(),
    );
    let clip =
        merge_clip_chain(group.clip_path(), transform, warnings);

    sink.push_group(Group {
        clip,
        composite,
        ..Default::default()
    });

    for child in group.children() {
        render_node(child, sink, transform, warnings);
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
    warnings: &mut Vec<SvgWarning>,
) {
    if !path.is_visible() {
        return;
    }

    let geometry = convert_path(path.data());
    let transform =
        transform * convert_transform(path.abs_transform());
    let fill = convert_fill(path.fill(), warnings);
    let stroke = convert_stroke(path.stroke(), warnings);
    let paint_order = convert_paint_order(path.paint_order());

    sink.draw_path(geometry, transform, fill, stroke, paint_order);
}

pub fn render_image(
    image: &usvg::Image,
    sink: &mut impl KanvaSink,
    transform: Affine,
    warnings: &mut Vec<SvgWarning>,
) {
    if !image.is_visible() {
        return;
    }
    match image.kind() {
        usvg::ImageKind::SVG(tree) => {
            render_group(
                tree.root(),
                sink,
                transform * convert_transform(image.abs_transform()),
                warnings,
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

    let display_width = image.size().width() as f64;
    let display_height = image.size().height() as f64;

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
