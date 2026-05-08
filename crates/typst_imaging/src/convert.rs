use std::f32::consts::TAU;

use peniko::kurbo::{Affine, BezPath, Shape as _, Stroke};
use peniko::{Brush, Color};
use typst_library::layout::{Abs, Quadrant, Size, Transform};
use typst_library::visualize as viz;

pub fn convert_transform(t: Transform) -> Affine {
    Affine::new([
        t.sx.get(),
        t.ky.get(),
        t.kx.get(),
        t.sy.get(),
        t.tx.to_pt(),
        t.ty.to_pt(),
    ])
}

pub fn convert_color(color: &viz::Color) -> Color {
    let [r, g, b, a] = color.to_vec4_u8();
    Color::from_rgba8(r, g, b, a)
}

pub fn convert_fixed_stroke(stroke: &viz::FixedStroke) -> Stroke {
    let width = stroke.thickness.to_pt();
    let join = match stroke.join {
        viz::LineJoin::Miter => peniko::kurbo::Join::Miter,
        viz::LineJoin::Round => peniko::kurbo::Join::Round,
        viz::LineJoin::Bevel => peniko::kurbo::Join::Bevel,
    };
    let cap = match stroke.cap {
        viz::LineCap::Butt => peniko::kurbo::Cap::Butt,
        viz::LineCap::Round => peniko::kurbo::Cap::Round,
        viz::LineCap::Square => peniko::kurbo::Cap::Square,
    };
    let mut s = Stroke {
        width,
        join,
        miter_limit: stroke.miter_limit.get(),
        start_cap: cap,
        end_cap: cap,
        ..Default::default()
    };
    if let Some(dash) = &stroke.dash {
        s.dash_pattern =
            dash.array.iter().map(|d| d.to_pt()).collect();
        s.dash_offset = dash.phase.to_pt();
    }
    s
}

/// Convert a typst [`viz::Paint`] to a peniko [`Brush`].
///
/// `size` is the hard-frame size (for gradient `relative: parent`).
/// `brush_transform` is returned separately because imaging requires it as an optional
/// affine on the draw call, not baked into the brush itself.
pub fn convert_paint(
    paint: &viz::Paint,
    size: Size,
    container_transform: Affine,
) -> (Brush, Option<Affine>) {
    match paint {
        viz::Paint::Solid(c) => {
            (Brush::Solid(convert_color(c)), None)
        }
        viz::Paint::Gradient(gradient) => {
            let ratio = size.aspect_ratio();
            let stops: Vec<_> = gradient
                .stops_ref()
                .iter()
                .map(|(color, ratio)| peniko::ColorStop {
                    offset: ratio.get() as f32,
                    color:
                        peniko::color::DynamicColor::from_alpha_color(
                            convert_color(color),
                        ),
                })
                .collect();

            let brush_transform =
                match gradient.unwrap_relative(false) {
                    viz::RelativeTo::Self_ => None,
                    viz::RelativeTo::Parent => {
                        let inv = container_transform.inverse();
                        Some(
                            inv * Affine::scale_non_uniform(
                                size.x.to_pt(),
                                size.y.to_pt(),
                            ),
                        )
                    }
                };

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
                    let angle =
                        -(viz::Gradient::correct_aspect_ratio(
                            conic.angle,
                            ratio,
                        )
                        .to_rad() as f32)
                            .rem_euclid(TAU);
                    let center =
                        (conic.center.x.get(), conic.center.y.get());
                    peniko::Gradient::new_sweep(
                        center,
                        angle,
                        TAU - angle,
                    )
                    .with_stops(stops.as_slice())
                }
            };
            (Brush::Gradient(gradient), brush_transform)
        }
        viz::Paint::Tiling(_) => {
            // TODO: tiling/pattern support
            (Brush::Solid(Color::TRANSPARENT), None)
        }
    }
}

pub fn convert_geometry(geometry: &viz::Geometry) -> BezPath {
    match geometry {
        viz::Geometry::Line(p) => peniko::kurbo::Line::new(
            (0.0, 0.0),
            (p.x.to_pt(), p.y.to_pt()),
        )
        .to_path(0.1),
        viz::Geometry::Rect(size) => {
            peniko::kurbo::Rect::from_origin_size(
                (0.0, 0.0),
                (size.x.to_pt(), size.y.to_pt()),
            )
            .to_path(0.1)
        }
        viz::Geometry::Curve(curve) => convert_curve(curve),
    }
}

pub fn convert_curve(curve: &viz::Curve) -> BezPath {
    let mut path = BezPath::new();
    for item in &curve.0 {
        match item {
            viz::CurveItem::Move(p) => {
                path.move_to((p.x.to_pt(), p.y.to_pt()))
            }
            viz::CurveItem::Line(p) => {
                path.line_to((p.x.to_pt(), p.y.to_pt()))
            }
            viz::CurveItem::Cubic(p1, p2, p3) => path.curve_to(
                (p1.x.to_pt(), p1.y.to_pt()),
                (p2.x.to_pt(), p2.y.to_pt()),
                (p3.x.to_pt(), p3.y.to_pt()),
            ),
            viz::CurveItem::Close => path.close_path(),
        }
    }
    path
}

pub fn shape_brush_transform(
    paint: &viz::Paint,
    shape: &viz::Shape,
    container_transform: Affine,
    size: Size,
) -> Option<Affine> {
    let mut shape_size = shape.geometry.bbox_size();
    if shape_size.x.to_pt() == 0.0 {
        shape_size.x = Abs::pt(1.0);
    }
    if shape_size.y.to_pt() == 0.0 {
        shape_size.y = Abs::pt(1.0);
    }

    match paint {
        viz::Paint::Gradient(gradient) => {
            match gradient.unwrap_relative(false) {
                viz::RelativeTo::Self_ => {
                    Some(Affine::scale_non_uniform(
                        shape_size.x.to_pt(),
                        shape_size.y.to_pt(),
                    ))
                }
                viz::RelativeTo::Parent => {
                    let inv = container_transform.inverse();
                    Some(
                        inv * Affine::scale_non_uniform(
                            size.x.to_pt(),
                            size.y.to_pt(),
                        ),
                    )
                }
            }
        }
        viz::Paint::Tiling(tiling) => {
            match tiling.unwrap_relative(false) {
                viz::RelativeTo::Self_ => None,
                viz::RelativeTo::Parent => {
                    Some(container_transform.inverse())
                }
            }
        }
        viz::Paint::Solid(_) => None,
    }
}
