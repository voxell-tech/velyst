use std::f32::consts::TAU;
use std::f64::consts::PI;

use imaging::peniko::kurbo::Affine;
use imaging::peniko::{Brush, Color};
use imaging::{kurbo, peniko};
use typst_library::layout::{Abs, Quadrant, Size};
use typst_library::visualize as viz;

use crate::RenderState;

pub fn convert_color(color: &viz::Color) -> Color {
    let [r, g, b, a] = color.to_vec4_u8();
    Color::from_rgba8(r, g, b, a)
}

/// Convert a typst [`viz::Paint`] to a [`Brush`] for a [`viz::Shape`]
/// fill and stroke.
pub fn shape_paint(
    paint: &viz::Paint,
    shape: &viz::Shape,
    state: &RenderState,
) -> (Brush, Option<Affine>) {
    let shape_size =
        shape.bbox(true).size().max(Size::splat(Abs::pt(1.0)));

    let brush_transform = match paint {
        viz::Paint::Gradient(gradient) => {
            let relative = gradient.unwrap_relative(false);
            let is_conic =
                matches!(gradient, viz::Gradient::Conic(_));

            let (w, h) = match relative {
                viz::RelativeTo::Self_ => {
                    (shape_size.x.to_pt(), shape_size.y.to_pt())
                }
                viz::RelativeTo::Parent => (
                    state.container_size.x.to_pt(),
                    state.container_size.y.to_pt(),
                ),
            };
            let base = match relative {
                viz::RelativeTo::Self_ => Affine::IDENTITY,
                // brush_transform = T⁻¹ · C · scale(w,h)
                // so vello samples: scale(1/w,1/h) · C⁻¹ · (px,py)
                // = container-local coords, normalized by (w,h).
                viz::RelativeTo::Parent => {
                    state.transform.inverse()
                        * state.container_transform
                }
            };

            // Conic uses uniform scale(w, w) so angles stay circular
            // in screen space. All other gradients use scale(w, h).
            let mut affine = Some(if is_conic {
                base * Affine::scale(w)
            } else {
                base * Affine::scale_non_uniform(w, h)
            });

            // Rotate conic brush around the (aspect-corrected)
            // center. Typst's t=0 is at angle
            // `conic.angle - PI` (LEFT when
            // angle=0); peniko's sweep t=0 is at 0 (RIGHT), so we add
            // PI to close the gap. The center y is divided by the
            // aspect ratio because the gradient spec uses the
            // independently-normalized space but we scale uniformly.
            if let viz::Gradient::Conic(conic) = gradient {
                let ratio = w / h;
                let rad = conic.angle.to_rad() + PI;
                let center = kurbo::Vec2::new(
                    conic.center.x.get(),
                    conic.center.y.get() / ratio,
                );
                let rotation = Affine::translate(center)
                    * Affine::rotate(rad)
                    * Affine::translate(-center);
                if let Some(a) = affine.as_mut() {
                    *a *= rotation;
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

/// Convert a typst [`viz::Paint`] to a [`Brush`] and brush transform
/// for a text glyph run.
pub fn text_paint(
    paint: &viz::Paint,
    state: &RenderState,
) -> (Brush, Option<Affine>) {
    let brush_transform = match paint {
        viz::Paint::Gradient(gradient) => {
            let w = state.container_size.x.to_pt();
            let h = state.container_size.y.to_pt();
            // Same formula as RelativeTo::Parent in shape_paint.
            let base =
                state.transform.inverse() * state.container_transform;

            // Conic uses uniform scale(w, w); others use scale(w, h).
            let mut affine = Some(
                if matches!(gradient, viz::Gradient::Conic(_)) {
                    base * Affine::scale(w)
                } else {
                    base * Affine::scale_non_uniform(w, h)
                },
            );

            if let viz::Gradient::Conic(conic) = gradient {
                let ratio = w / h;
                let rad = conic.angle.to_rad() + PI;
                let center = kurbo::Vec2::new(
                    conic.center.x.get(),
                    conic.center.y.get() / ratio,
                );
                let rotation = Affine::translate(center)
                    * Affine::rotate(rad)
                    * Affine::translate(-center);
                if let Some(a) = affine.as_mut() {
                    *a *= rotation;
                }
            }

            affine
        }
        _ => None,
    };

    (build_brush(paint, state.container_size), brush_transform)
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
                    color: convert_color(&color.to_process_space(
                        viz::ProcessColorSpace::Srgb,
                    ))
                    .into(),
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
                    // The conic brush_transform uses uniform scale(w,
                    // w) (not scale(w, h)) so
                    // angles are circular in screen
                    // space. To land at the correct screen position
                    // after that uniform scale,
                    // divide cy by the aspect ratio.
                    let center = (
                        conic.center.x.get(),
                        conic.center.y.get() / ratio.get(),
                    );
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
