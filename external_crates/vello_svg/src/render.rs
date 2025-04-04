// Copyright 2024 the Vello Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::util;
use vello::kurbo::Affine;
use vello::peniko::{BlendMode, Fill};
use vello::Scene;

pub(crate) fn render_group<F: FnMut(&mut Scene, &usvg::Node)>(
    scene: &mut Scene,
    group: &usvg::Group,
    transform: Affine,
    error_handler: &mut F,
) {
    for node in group.children() {
        let transform = transform * util::to_affine(&node.abs_transform());
        match node {
            usvg::Node::Group(g) => {
                let alpha = g.opacity().get();
                let mix = match g.blend_mode() {
                    usvg::BlendMode::Normal => {
                        if alpha < 1.0 {
                            vello::peniko::Mix::Normal
                        } else {
                            vello::peniko::Mix::Clip
                        }
                    }
                    usvg::BlendMode::Multiply => vello::peniko::Mix::Multiply,
                    usvg::BlendMode::Screen => vello::peniko::Mix::Screen,
                    usvg::BlendMode::Overlay => vello::peniko::Mix::Overlay,
                    usvg::BlendMode::Darken => vello::peniko::Mix::Darken,
                    usvg::BlendMode::Lighten => vello::peniko::Mix::Lighten,
                    usvg::BlendMode::ColorDodge => vello::peniko::Mix::ColorDodge,
                    usvg::BlendMode::ColorBurn => vello::peniko::Mix::ColorBurn,
                    usvg::BlendMode::HardLight => vello::peniko::Mix::HardLight,
                    usvg::BlendMode::SoftLight => vello::peniko::Mix::SoftLight,
                    usvg::BlendMode::Difference => vello::peniko::Mix::Difference,
                    usvg::BlendMode::Exclusion => vello::peniko::Mix::Exclusion,
                    usvg::BlendMode::Hue => vello::peniko::Mix::Hue,
                    usvg::BlendMode::Saturation => vello::peniko::Mix::Saturation,
                    usvg::BlendMode::Color => vello::peniko::Mix::Color,
                    usvg::BlendMode::Luminosity => vello::peniko::Mix::Luminosity,
                };

                let clipped = match g
                    .clip_path()
                    // support clip-path with a single path
                    .and_then(|path| path.root().children().first())
                {
                    Some(usvg::Node::Path(clip_path)) => {
                        let local_path = util::to_bez_path(clip_path);
                        scene.push_layer(
                            BlendMode {
                                mix,
                                compose: vello::peniko::Compose::SrcOver,
                            },
                            alpha,
                            transform,
                            &local_path,
                        );

                        true
                    }
                    // Ignore if we're in `Mix::Clip` mode and there's nothing to clip.
                    _ if !matches!(mix, vello::peniko::Mix::Clip) => {
                        // Use bounding box as the clip path.
                        let bounding_box = g.layer_bounding_box();
                        let rect = vello::kurbo::Rect::from_origin_size(
                            (bounding_box.x(), bounding_box.y()),
                            (bounding_box.width() as f64, bounding_box.height() as f64),
                        );
                        scene.push_layer(
                            BlendMode {
                                mix,
                                compose: vello::peniko::Compose::SrcOver,
                            },
                            alpha,
                            transform,
                            &rect,
                        );

                        true
                    }
                    _ => false,
                };

                render_group(scene, g, Affine::IDENTITY, error_handler);

                if clipped {
                    scene.pop_layer();
                }
            }
            usvg::Node::Path(path) => {
                if !path.is_visible() {
                    continue;
                }
                let local_path = util::to_bez_path(path);

                let do_fill = |scene: &mut Scene, error_handler: &mut F| {
                    if let Some(fill) = &path.fill() {
                        if let Some((brush, brush_transform)) =
                            util::to_brush(fill.paint(), fill.opacity())
                        {
                            scene.fill(
                                match fill.rule() {
                                    usvg::FillRule::NonZero => Fill::NonZero,
                                    usvg::FillRule::EvenOdd => Fill::EvenOdd,
                                },
                                transform,
                                &brush,
                                Some(brush_transform),
                                &local_path,
                            );
                        } else {
                            error_handler(scene, node);
                        }
                    }
                };
                let do_stroke = |scene: &mut Scene, error_handler: &mut F| {
                    if let Some(stroke) = &path.stroke() {
                        if let Some((brush, brush_transform)) =
                            util::to_brush(stroke.paint(), stroke.opacity())
                        {
                            let conv_stroke = util::to_stroke(stroke);
                            scene.stroke(
                                &conv_stroke,
                                transform,
                                &brush,
                                Some(brush_transform),
                                &local_path,
                            );
                        } else {
                            error_handler(scene, node);
                        }
                    }
                };
                match path.paint_order() {
                    usvg::PaintOrder::FillAndStroke => {
                        do_fill(scene, error_handler);
                        do_stroke(scene, error_handler);
                    }
                    usvg::PaintOrder::StrokeAndFill => {
                        do_stroke(scene, error_handler);
                        do_fill(scene, error_handler);
                    }
                }
            }
            usvg::Node::Image(img) => {
                if !img.is_visible() {
                    continue;
                }
                match img.kind() {
                    usvg::ImageKind::JPEG(_)
                    | usvg::ImageKind::PNG(_)
                    | usvg::ImageKind::GIF(_)
                    | usvg::ImageKind::WEBP(_) => {
                        let Ok(decoded_image) = util::decode_raw_raster_image(img.kind()) else {
                            error_handler(scene, node);
                            continue;
                        };
                        let image = util::into_image(decoded_image);
                        let image_ts = util::to_affine(&img.abs_transform());
                        scene.draw_image(&image, image_ts);
                    }
                    usvg::ImageKind::SVG(svg) => {
                        render_group(scene, svg.root(), transform, error_handler);
                    }
                }
            }
            usvg::Node::Text(text) => {
                render_group(scene, text.flattened(), transform, error_handler);
            }
        }
    }
}
