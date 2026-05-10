use std::f32::consts::TAU;

use peniko::kurbo::Affine;
use peniko::{Brush, Color, GradientKind};
use typst_library::layout::{Abs, Quadrant, Size};
use typst_library::visualize as viz;

use crate::RenderState;

pub fn convert_color(color: &viz::Color) -> Color {
    let [r, g, b, a] = color.to_vec4_u8();
    Color::from_rgba8(r, g, b, a)
}

/// Convert a typst [`viz::Paint`] to a [`peniko::Brush`] for a
/// [`viz::Shape`] fill and stroke.
pub(crate) fn shape_paint(
    paint: &viz::Paint,
    shape: &viz::Shape,
    state: &RenderState,
) -> (Brush, Option<Affine>) {
    let shape_size =
        shape.geometry.bbox_size().max(Size::splat(Abs::pt(1.0)));

    let brush_transform = match paint {
        viz::Paint::Gradient(gradient) => {
            let mut affine = match gradient.unwrap_relative(false) {
                // Gradient maps unit square -> shape bbox size (pt).
                viz::RelativeTo::Self_ => {
                    Some(Affine::scale_non_uniform(
                        shape_size.x.to_pt(),
                        shape_size.y.to_pt(),
                    ))
                }
                // Gradient maps unit square -> container size (pt), then
                // container transform moves it into canvas space. The inverse
                // is passed as brush_transform so vello un-applies it during
                // sampling.
                viz::RelativeTo::Parent => {
                    let inv = state.container_transform.inverse();
                    Some(
                        inv * Affine::scale_non_uniform(
                            state.container_size.x.to_pt(),
                            state.container_size.y.to_pt(),
                        ),
                    )
                }
            };

            // Rotate brush for conic angle, around the gradient center.
            if let viz::Gradient::Conic(conic) = gradient {
                let rad = conic.angle.to_rad();
                let center = peniko::kurbo::Vec2::new(
                    conic.center.x.get(),
                    conic.center.y.get(),
                );
                let rotation = Affine::translate(center)
                    * Affine::rotate(rad)
                    * Affine::translate(-center);
                if let Some(affine) = affine.as_mut() {
                    *affine *= rotation;
                } else {
                    affine = Some(rotation);
                }
            }

            affine
        }
        viz::Paint::Tiling(tiling) => {
            match tiling.unwrap_relative(false) {
                viz::RelativeTo::Self_ => None,
                viz::RelativeTo::Parent => {
                    Some(state.container_transform.inverse())
                }
            }
        }
        viz::Paint::Solid(_) => None,
    };

    let brush_size = match paint {
        viz::Paint::Gradient(gradient)
            if matches!(
                gradient.unwrap_relative(false),
                viz::RelativeTo::Self_
            ) =>
        {
            shape_size
        }
        _ => state.container_size,
    };

    (build_brush(paint, brush_size), brush_transform)
}

/// Convert a typst [`viz::Paint`] to a [`peniko ::Brush`] for a
/// text glyph run, baking the brush transform directly into gradient
/// control points.
pub(crate) fn text_paint(
    paint: &viz::Paint,
    state: &RenderState,
    last_glyph_x: f64,
) -> Brush {
    let mut brush = build_brush(paint, state.container_size);

    // TODO(nixon): Glyph runs does not support brush transform right
    // now. So we have to apply the transform on our own.
    if let peniko::Brush::Gradient(gradient) = &mut brush {
        let w = state.container_size.x.to_pt();
        let h = state.container_size.y.to_pt();

        // The brush lives in the last glyph's actual transform space,
        // which includes vello's internal Y-flip matrix [1, 0, 0, -1].
        //
        // Factor that in so the brush correctly maps gradient unit to
        // container canvas space without needing glyph_transform to
        // cancel the flip (which would make glyphs upside down).
        let glyph_last_xform = state.transform
            * Affine::new([1.0, 0.0, 0.0, -1.0, last_glyph_x, 0.0]);
        let local_to_container =
            glyph_last_xform.inverse() * state.container_transform;
        let brush_transform =
            local_to_container.pre_scale_non_uniform(w, h);

        apply_transform_to_text_gradient(gradient, brush_transform);
    }

    brush
}

/// Build the base brush in normalized *unit square* gradient space.
pub fn build_brush(paint: &viz::Paint, size: Size) -> Brush {
    match paint {
        viz::Paint::Solid(c) => Brush::Solid(convert_color(c)),
        viz::Paint::Gradient(gradient) => {
            let ratio = size.aspect_ratio();
            let stops: Vec<_> = gradient
                .stops_ref()
                .iter()
                .map(|(color, ratio)| peniko::ColorStop {
                    offset: ratio.get() as f32,
                    color: convert_color(color).into(),
                })
                .collect();

            let gradient = match gradient {
                viz::Gradient::Linear(linear) => {
                    let angle = viz::Gradient::correct_aspect_ratio(
                        linear.angle,
                        ratio,
                    );
                    let (sin, cos) = (angle.sin(), angle.cos());
                    let length = sin.abs() + cos.abs();
                    let (start, end) = match angle.quadrant() {
                        Quadrant::First => {
                            ((0.0, 0.0), (cos * length, sin * length))
                        }
                        Quadrant::Second => (
                            (1.0, 0.0),
                            (cos * length + 1.0, sin * length),
                        ),
                        Quadrant::Third => (
                            (1.0, 1.0),
                            (cos * length + 1.0, sin * length + 1.0),
                        ),
                        Quadrant::Fourth => (
                            (0.0, 1.0),
                            (cos * length, sin * length + 1.0),
                        ),
                    };
                    peniko::Gradient::new_linear(start, end)
                        .with_stops(stops.as_slice())
                }
                viz::Gradient::Radial(radial) => {
                    let start_center = (
                        radial.focal_center.x.get(),
                        radial.focal_center.y.get(),
                    );
                    let end_center = (
                        radial.center.x.get(),
                        radial.center.y.get(),
                    );
                    peniko::Gradient::new_two_point_radial(
                        start_center,
                        radial.focal_radius.get() as f32,
                        end_center,
                        radial.radius.get() as f32,
                    )
                    .with_stops(stops.as_slice())
                }
                viz::Gradient::Conic(conic) => {
                    let center =
                        (conic.center.x.get(), conic.center.y.get());

                    // TODO: Will Typst support start + end angle?

                    // Typst's conic gradient is always a full circle,
                    // angle will be handled via brush transform.
                    //
                    // This means that angle will not be supported by
                    // text yet since `imaging` does not support
                    // glyph's brush transform yet.
                    peniko::Gradient::new_sweep(center, 0.0, TAU)
                        .with_stops(stops.as_slice())
                }
            };
            Brush::Gradient(gradient)
        }
        viz::Paint::Tiling(_) => {
            // TODO: tiling/pattern support
            Brush::Solid(Color::TRANSPARENT)
        }
    }
}

/// Bake an affine transform into a gradient control points.
pub fn apply_transform_to_text_gradient(
    gradient: &mut peniko::Gradient,
    transform: Affine,
) {
    match &mut gradient.kind {
        GradientKind::Linear(pos) => {
            pos.start = transform * pos.start;
            pos.end = transform * pos.end;
        }
        GradientKind::Radial(pos) => {
            pos.start_center = transform * pos.start_center;
            pos.end_center = transform * pos.end_center;
            let [a, b, ..] = transform.as_coeffs();
            let scale = a.hypot(b) as f32;
            pos.start_radius *= scale;
            pos.end_radius *= scale;
        }
        GradientKind::Sweep(pos) => {
            pos.center = transform * pos.center;
            // Compensate for inverted text glyph matrix.
            pos.start_angle = TAU;
            pos.end_angle = 0.0;
        }
    }
}
