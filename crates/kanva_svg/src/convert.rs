use kanva::imaging::Composite;
use kanva::imaging::kurbo::{
    Affine, BezPath, Cap, Join, Point, Stroke,
};
use kanva::imaging::peniko::{
    BlendMode, Brush, ColorStop, Extend, Fill, Gradient, ImageBrush,
    ImageQuality, Mix, Style,
};
use kanva::prelude::*;
use usvg::Node;

pub(crate) fn merge_clip_chain(
    clip_path: Option<&usvg::ClipPath>,
    transform: Affine,
) -> Option<KanvaClip> {
    let clip_path = clip_path?;
    let mut merged = BezPath::new();
    let mut fill_rule = Fill::NonZero;
    collect_clip_chain(
        clip_path,
        transform,
        &mut merged,
        &mut fill_rule,
    );
    if merged.is_empty() {
        return None;
    }
    Some(KanvaClip {
        path: merged,
        transform: Affine::IDENTITY,
        style: Style::Fill(fill_rule),
    })
}

fn collect_clip_chain(
    clip_path: &usvg::ClipPath,
    transform: Affine,
    out: &mut BezPath,
    fill_rule: &mut Fill,
) {
    if let Some(parent) = clip_path.clip_path() {
        collect_clip_chain(parent, transform, out, fill_rule);
    }
    if let Some(clip) = convert_clip(clip_path, transform) {
        if let Style::Fill(rule) = clip.style {
            *fill_rule = rule;
        }
        out.extend(clip.path.elements().iter().copied());
    }
}

fn convert_clip(
    clip_path: &usvg::ClipPath,
    transform: Affine,
) -> Option<KanvaClip> {
    let mut path = BezPath::new();
    let mut fill_rule: Option<Fill> = None;

    if !collect_clip_group(
        clip_path.root(),
        transform,
        &mut path,
        &mut fill_rule,
    ) || path.is_empty()
    {
        eprintln!("kanva_svg: complex clip path skipped");
        return None;
    }

    Some(KanvaClip {
        path,
        transform: Affine::IDENTITY,
        style: Style::Fill(fill_rule.unwrap_or(Fill::NonZero)),
    })
}

fn collect_clip_group(
    group: &usvg::Group,
    transform: Affine,
    out: &mut BezPath,
    fill_rule: &mut Option<Fill>,
) -> bool {
    if group.clip_path().is_some()
        || group.mask().is_some()
        || !group.filters().is_empty()
    {
        return false;
    }
    for child in group.children() {
        if !collect_clip_node(child, transform, out, fill_rule) {
            return false;
        }
    }
    true
}

fn collect_clip_node(
    node: &Node,
    transform: Affine,
    out: &mut BezPath,
    fill_rule: &mut Option<Fill>,
) -> bool {
    match node {
        Node::Path(path) => {
            if !path.is_visible() {
                return true;
            }
            let rule = path
                .fill()
                .map(|fill| match fill.rule() {
                    usvg::FillRule::NonZero => Fill::NonZero,
                    usvg::FillRule::EvenOdd => Fill::EvenOdd,
                })
                .unwrap_or(Fill::NonZero);
            if let Some(existing) = *fill_rule {
                if existing != rule {
                    return false;
                }
            } else {
                *fill_rule = Some(rule);
            }
            let mut clip_path_bez = convert_path(path.data());
            clip_path_bez.apply_affine(
                transform * convert_transform(path.abs_transform()),
            );
            out.extend(clip_path_bez.elements().iter().copied());
            true
        }
        Node::Text(text) => collect_clip_group(
            text.flattened(),
            transform,
            out,
            fill_rule,
        ),
        Node::Group(group) => {
            collect_clip_group(group, transform, out, fill_rule)
        }
        Node::Image(_) => false,
    }
}

pub(crate) fn convert_fill(
    fill: Option<&usvg::Fill>,
) -> Option<KanvaFill> {
    let fill = fill?;
    let (brush, brush_transform) =
        convert_brush(fill.paint(), fill.opacity().get())?;
    Some(KanvaFill {
        rule: match fill.rule() {
            usvg::FillRule::NonZero => Fill::NonZero,
            usvg::FillRule::EvenOdd => Fill::EvenOdd,
        },
        brush,
        brush_transform,
        composite: Composite::default(),
    })
}

pub(crate) fn convert_stroke(
    stroke: Option<&usvg::Stroke>,
) -> Option<KanvaStroke> {
    let stroke = stroke?;
    let (brush, brush_transform) =
        convert_brush(stroke.paint(), stroke.opacity().get())?;
    Some(KanvaStroke {
        stroke: convert_stroke_style(stroke),
        brush,
        brush_transform,
        composite: Composite::default(),
    })
}

fn convert_brush(
    paint: &usvg::Paint,
    opacity: f32,
) -> Option<(Brush, Option<Affine>)> {
    match paint {
        usvg::Paint::Color(color) => {
            Some((convert_color(*color, opacity), None))
        }
        usvg::Paint::LinearGradient(g) => Some((
            Brush::Gradient(
                convert_linear_gradient(g).multiply_alpha(opacity),
            ),
            affine_if_non_identity(g.transform()),
        )),
        usvg::Paint::RadialGradient(g) => Some((
            Brush::Gradient(
                convert_radial_gradient(g).multiply_alpha(opacity),
            ),
            affine_if_non_identity(g.transform()),
        )),
        usvg::Paint::Pattern(_) => {
            eprintln!("kanva_svg: pattern paint unsupported");
            None
        }
    }
}

pub(crate) fn convert_path(
    path: &usvg::tiny_skia_path::Path,
) -> BezPath {
    let mut bez = BezPath::new();
    for segment in path.segments() {
        match segment {
            usvg::tiny_skia_path::PathSegment::MoveTo(p) => {
                bez.move_to(Point::new(
                    f64::from(p.x),
                    f64::from(p.y),
                ));
            }
            usvg::tiny_skia_path::PathSegment::LineTo(p) => {
                bez.line_to(Point::new(
                    f64::from(p.x),
                    f64::from(p.y),
                ));
            }
            usvg::tiny_skia_path::PathSegment::QuadTo(p1, p2) => {
                bez.quad_to(
                    Point::new(f64::from(p1.x), f64::from(p1.y)),
                    Point::new(f64::from(p2.x), f64::from(p2.y)),
                );
            }
            usvg::tiny_skia_path::PathSegment::CubicTo(
                p1,
                p2,
                p3,
            ) => {
                bez.curve_to(
                    Point::new(f64::from(p1.x), f64::from(p1.y)),
                    Point::new(f64::from(p2.x), f64::from(p2.y)),
                    Point::new(f64::from(p3.x), f64::from(p3.y)),
                );
            }
            usvg::tiny_skia_path::PathSegment::Close => {
                bez.close_path()
            }
        }
    }
    bez
}

pub(crate) fn convert_transform(t: usvg::Transform) -> Affine {
    Affine::new([
        f64::from(t.sx),
        f64::from(t.ky),
        f64::from(t.kx),
        f64::from(t.sy),
        f64::from(t.tx),
        f64::from(t.ty),
    ])
}

fn affine_if_non_identity(t: usvg::Transform) -> Option<Affine> {
    (!t.is_identity()).then(|| convert_transform(t))
}

fn convert_color(color: usvg::Color, opacity: f32) -> Brush {
    let alpha = opacity_to_u8(opacity);
    Brush::Solid(kanva::imaging::peniko::Color::from_rgba8(
        color.red,
        color.green,
        color.blue,
        alpha,
    ))
}

fn convert_stroke_style(stroke: &usvg::Stroke) -> Stroke {
    let mut s = Stroke::new(f64::from(stroke.width().get()))
        .with_join(match stroke.linejoin() {
            usvg::LineJoin::Miter | usvg::LineJoin::MiterClip => {
                Join::Miter
            }
            usvg::LineJoin::Round => Join::Round,
            usvg::LineJoin::Bevel => Join::Bevel,
        })
        .with_miter_limit(f64::from(stroke.miterlimit().get()))
        .with_caps(match stroke.linecap() {
            usvg::LineCap::Butt => Cap::Butt,
            usvg::LineCap::Square => Cap::Square,
            usvg::LineCap::Round => Cap::Round,
        });
    if let Some(dashes) = stroke.dasharray() {
        s = s.with_dashes(
            f64::from(stroke.dashoffset()),
            dashes.iter().copied().map(f64::from),
        );
    }
    s
}

fn convert_linear_gradient(g: &usvg::LinearGradient) -> Gradient {
    Gradient::new_linear((g.x1(), g.y1()), (g.x2(), g.y2()))
        .with_extend(convert_extend(g.spread_method()))
        .with_stops(convert_stops(g.stops()).as_slice())
}

fn convert_radial_gradient(g: &usvg::RadialGradient) -> Gradient {
    Gradient::new_two_point_radial(
        (g.fx(), g.fy()),
        0.0,
        (g.cx(), g.cy()),
        g.r().get(),
    )
    .with_extend(convert_extend(g.spread_method()))
    .with_stops(convert_stops(g.stops()).as_slice())
}

fn convert_stops(stops: &[usvg::Stop]) -> Vec<ColorStop> {
    stops
        .iter()
        .map(|s| ColorStop {
            offset: s.offset().get(),
            color: kanva::imaging::peniko::color::DynamicColor::from_alpha_color(
                kanva::imaging::peniko::Color::from_rgba8(
                    s.color().red,
                    s.color().green,
                    s.color().blue,
                    opacity_to_u8(s.opacity().get()),
                ),
            ),
        })
        .collect()
}

fn convert_extend(spread: usvg::SpreadMethod) -> Extend {
    match spread {
        usvg::SpreadMethod::Pad => Extend::Pad,
        usvg::SpreadMethod::Reflect => Extend::Reflect,
        usvg::SpreadMethod::Repeat => Extend::Repeat,
    }
}

pub(crate) fn map_blend_mode(mode: usvg::BlendMode) -> BlendMode {
    match mode {
        usvg::BlendMode::Normal => BlendMode::default(),
        usvg::BlendMode::Multiply => BlendMode::from(Mix::Multiply),
        usvg::BlendMode::Screen => BlendMode::from(Mix::Screen),
        usvg::BlendMode::Overlay => BlendMode::from(Mix::Overlay),
        usvg::BlendMode::Darken => BlendMode::from(Mix::Darken),
        usvg::BlendMode::Lighten => BlendMode::from(Mix::Lighten),
        usvg::BlendMode::ColorDodge => {
            BlendMode::from(Mix::ColorDodge)
        }
        usvg::BlendMode::ColorBurn => BlendMode::from(Mix::ColorBurn),
        usvg::BlendMode::HardLight => BlendMode::from(Mix::HardLight),
        usvg::BlendMode::SoftLight => BlendMode::from(Mix::SoftLight),
        usvg::BlendMode::Difference => {
            BlendMode::from(Mix::Difference)
        }
        usvg::BlendMode::Exclusion => BlendMode::from(Mix::Exclusion),
        usvg::BlendMode::Hue => BlendMode::from(Mix::Hue),
        usvg::BlendMode::Saturation => {
            BlendMode::from(Mix::Saturation)
        }
        usvg::BlendMode::Color => BlendMode::from(Mix::Color),
        usvg::BlendMode::Luminosity => {
            BlendMode::from(Mix::Luminosity)
        }
    }
}

pub(crate) fn convert_paint_order(
    order: usvg::PaintOrder,
) -> PaintOrder {
    match order {
        usvg::PaintOrder::FillAndStroke => PaintOrder::FillStroke,
        usvg::PaintOrder::StrokeAndFill => PaintOrder::StrokeFill,
    }
}

pub(crate) fn with_rendering_quality(
    brush: ImageBrush,
    mode: usvg::ImageRendering,
) -> ImageBrush {
    let quality = match mode {
        usvg::ImageRendering::OptimizeSpeed
        | usvg::ImageRendering::CrispEdges
        | usvg::ImageRendering::Pixelated => ImageQuality::Low,
        usvg::ImageRendering::OptimizeQuality
        | usvg::ImageRendering::HighQuality => ImageQuality::High,
        usvg::ImageRendering::Smooth => ImageQuality::Medium,
    };
    brush.with_quality(quality)
}

#[allow(
    clippy::cast_possible_truncation,
    reason = "opacity clamped to 0..=1 and scaled to 0..=255 before cast"
)]
fn opacity_to_u8(opacity: f32) -> u8 {
    (opacity.clamp(0.0, 1.0) * 255.0).round() as u8
}
