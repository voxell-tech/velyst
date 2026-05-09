use peniko::kurbo;
use peniko::kurbo::{Affine, BezPath, Shape as _, Stroke};
use typst_library::layout::Transform;
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

pub fn convert_fixed_stroke(stroke: &viz::FixedStroke) -> Stroke {
    let width = stroke.thickness.to_pt();
    let join = match stroke.join {
        viz::LineJoin::Miter => kurbo::Join::Miter,
        viz::LineJoin::Round => kurbo::Join::Round,
        viz::LineJoin::Bevel => kurbo::Join::Bevel,
    };
    let cap = match stroke.cap {
        viz::LineCap::Butt => kurbo::Cap::Butt,
        viz::LineCap::Round => kurbo::Cap::Round,
        viz::LineCap::Square => kurbo::Cap::Square,
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

pub fn convert_geometry(geometry: &viz::Geometry) -> BezPath {
    match geometry {
        viz::Geometry::Line(p) => {
            kurbo::Line::new((0.0, 0.0), (p.x.to_pt(), p.y.to_pt()))
                .to_path(0.1)
        }
        viz::Geometry::Rect(size) => kurbo::Rect::from_origin_size(
            (0.0, 0.0),
            (size.x.to_pt(), size.y.to_pt()),
        )
        .to_path(0.1),
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
