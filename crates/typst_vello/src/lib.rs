//! # Typst Vello
//!
//! A Vello scene drawer for Typst's frames.

use std::f32::consts::TAU;

use bevy_utils::{prelude::*, HashMap};
use bevy_vello_graphics::{
    bevy_vello::vello::{self, kurbo, peniko},
    prelude::*,
};
use typst::{
    foundations::Label,
    layout::{
        Abs, Frame, FrameItem, FrameKind, GroupItem, Point, Quadrant, Ratio, Size, Transform,
    },
    visualize as viz,
};

/// Contextual information for rendering.
#[derive(Default, Clone, Copy)]
pub struct State {
    /// The transform of the current item.
    transform: Transform,
    /// The size of the first hard frame in the hierarchy.
    size: Size,
}

impl State {
    fn new(size: Size, transform: Transform) -> Self {
        Self { size, transform }
    }

    /// Pre translate the current item's transform.
    fn pre_translate(self, pos: Point) -> Self {
        self.pre_concat(Transform::translate(pos.x, pos.y))
    }

    /// Pre concat the current item's transform.
    fn pre_concat(self, transform: Transform) -> Self {
        Self {
            transform: self.transform.pre_concat(transform),
            ..self
        }
    }

    /// Sets the size of the first hard frame in the hierarchy.
    fn with_size(self, size: Size) -> Self {
        Self { size, ..self }
    }

    /// Sets the current item's transform.
    fn with_transform(self, transform: Transform) -> Self {
        Self { transform, ..self }
    }
}

// #[derive(Default)]
// pub enum GroupParent {
//     #[default]
//     Root,
//     Parent(usize),
// }

// TODO: Add layers.
/// Every group is layouted in a flat list.
/// Each group will have a parent index associated with it.
#[derive(Default)]
pub struct TypstScene {
    pub groups: Vec<GroupPaths>,
    pub group_map: HashMap<Label, Vec<usize>>,
}

impl TypstScene {
    pub fn append_group(&mut self, group: GroupPaths) {
        if let Some(label) = group.label {
            let index = self.groups.len();
            match self.group_map.get_mut(&label) {
                Some(map) => {
                    map.push(index);
                }
                None => {
                    self.group_map.insert(label, vec![index]);
                }
            }
        }
        self.groups.push(group);
    }

    pub fn from_frame(&mut self, frame: &Frame) {
        self.from_frame_impl(0, State::default(), Transform::default(), frame);
    }

    fn from_frame_impl(&mut self, layer: usize, state: State, transform: Transform, frame: &Frame) {
        for (pos, item) in frame.items() {
            let pos = *pos;
            match item {
                FrameItem::Group(group) => {
                    self.render_group(layer, state.pre_translate(pos), group);
                }
                FrameItem::Text(_) => todo!(),
                FrameItem::Shape(shape, _) => {
                    GroupPaths::from_shape_scene(render_shape(state.pre_translate(pos), shape));
                }
                FrameItem::Image(_, _, _) => todo!(),
                FrameItem::Link(_, _) => todo!(),
                FrameItem::Tag(_) => todo!(),
            }
        }
    }

    fn render_group(&mut self, layer: usize, state: State, group: &GroupItem) {
        let state = match group.frame.kind() {
            FrameKind::Soft => state.pre_concat(group.transform),
            FrameKind::Hard => state
                .with_transform(Transform::identity())
                .with_size(group.frame.size()),
        };

        // group.

        // GroupPaths {

        // }

        // GroupScene {
        //     label: group.label,
        //     clip_path: group.clip_path.as_ref().map(|path| convert_path(path)),
        //     paths: render_frame(state, group.transform, &group.frame),
        // }
    }
}

pub fn render_frame(state: State, transform: Transform, frame: &Frame) {
    let mut typst_scene = TypstScene::default();

    for (pos, item) in frame.items() {
        let pos = *pos;
        match item {
            FrameItem::Group(group) => {
                render_group(state.pre_translate(pos), group);
            }
            FrameItem::Text(_) => todo!(),
            FrameItem::Shape(shape, _) => {
                GroupPaths::from_shape_scene(render_shape(state.pre_translate(pos), shape));
            }
            FrameItem::Image(_, _, _) => todo!(),
            FrameItem::Link(_, _) => todo!(),
            FrameItem::Tag(_) => todo!(),
        }
    }
}

#[derive(Default)]
pub struct GroupPaths {
    pub transform: kurbo::Affine,
    pub shapes: Vec<ShapeScene>,
    pub layer: usize,
    pub clip_path: Option<kurbo::BezPath>,
    pub label: Option<Label>,
}

impl GroupPaths {
    /// Create [`GroupPaths`] from a single [`ShapeScene`].
    pub fn from_shape_scene(shape_scene: ShapeScene) -> Self {
        Self {
            shapes: vec![shape_scene],
            ..default()
        }
    }

    pub fn with_transform(mut self, transform: kurbo::Affine) -> Self {
        self.transform = transform;
        self
    }

    pub fn clipped(mut self, path: kurbo::BezPath) -> Self {
        self.clip_path = Some(path);
        self
    }

    pub fn labelled(mut self, label: Label) -> Self {
        self.label = Some(label);
        self
    }

    pub fn render(&self) -> vello::Scene {
        let mut scene = vello::Scene::new();

        for shape in self.shapes.iter() {
            scene.append(&shape.render(), None);
        }

        scene
    }
}

#[derive(Default)]
pub struct ShapeScene {
    pub path: kurbo::BezPath,
    pub transform: kurbo::Affine,
    pub fill: Option<Fill>,
    pub stroke: Option<Stroke>,
}

impl ShapeScene {
    pub fn render(&self) -> vello::Scene {
        let mut scene = vello::Scene::new();

        if let Some(fill) = &self.fill {
            scene.fill(
                fill.style,
                self.transform,
                &fill.brush.value,
                Some(fill.brush.transform),
                &self.path,
            );
        }

        if let Some(stroke) = &self.stroke {
            scene.stroke(
                &stroke.style,
                self.transform,
                &stroke.brush.value,
                Some(stroke.brush.transform),
                &self.path,
            );
        }

        scene
    }
}

pub fn render_group(state: State, group: &GroupItem) {
    let state = match group.frame.kind() {
        FrameKind::Soft => state.pre_concat(group.transform),
        FrameKind::Hard => state
            .with_transform(Transform::identity())
            .with_size(group.frame.size()),
    };

    // GroupScene {
    //     label: group.label,
    //     clip_path: group.clip_path.as_ref().map(|path| convert_path(path)),
    //     paths: render_frame(state, group.transform, &group.frame),
    // }
}

pub fn render_shape(state: State, shape: &viz::Shape) -> ShapeScene {
    ShapeScene {
        path: convert_geometry_to_path(&shape.geometry),
        fill: shape.fill.as_ref().map(|paint| {
            let transform = shape_paint_transform(state, paint, shape);
            let size = shape_fill_size(state, paint, shape);
            let brush = convert_paint_to_brush(paint, size);

            Fill {
                style: match shape.fill_rule {
                    viz::FillRule::NonZero => peniko::Fill::NonZero,
                    viz::FillRule::EvenOdd => peniko::Fill::EvenOdd,
                },
                brush: Brush::from_brush(brush).with_transform(convert_transform(transform)),
            }
        }),
        stroke: shape.stroke.as_ref().map(|stroke| {
            let transform = shape_paint_transform(state, &stroke.paint, shape);
            let size = shape_fill_size(state, &stroke.paint, shape);
            let brush = convert_paint_to_brush(&stroke.paint, size);

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

            let mut kurbo_stroke = kurbo::Stroke {
                width: stroke.thickness.to_pt(),
                join,
                miter_limit: stroke.miter_limit.get(),
                start_cap: cap,
                end_cap: cap,
                ..default()
            };

            if let Some(dash) = &stroke.dash {
                kurbo_stroke.dash_pattern = dash.array.iter().map(|dash| dash.to_pt()).collect();
                kurbo_stroke.dash_offset = dash.phase.to_pt();
            }

            Stroke {
                style: kurbo_stroke,
                brush: Brush::from_brush(brush).with_transform(convert_transform(transform)),
            }
        }),
        ..default()
    }
}

pub fn convert_paint_to_brush(paint: &viz::Paint, size: Size) -> peniko::Brush {
    match paint {
        viz::Paint::Solid(solid) => {
            let channels = solid.to_vec4_u8();
            peniko::Brush::Solid(peniko::Color::rgba8(
                channels[0],
                channels[1],
                channels[2],
                channels[3],
            ))
        }
        viz::Paint::Gradient(gradient) => {
            let ratio = size.aspect_ratio();

            let stops = gradient
                .stops_ref()
                .iter()
                .map(|(color, ratio)| peniko::ColorStop {
                    offset: ratio.get() as f32,
                    color: convert_color(color),
                })
                .collect::<Vec<_>>();

            let gradient = match gradient {
                viz::Gradient::Linear(linear) => {
                    let angle = viz::Gradient::correct_aspect_ratio(linear.angle, ratio);
                    let (sin, cos) = (angle.sin(), angle.cos());
                    let length = sin.abs() + cos.abs();
                    let (start, end) = match angle.quadrant() {
                        Quadrant::First => ((0.0, 0.0), (cos * length, sin * length)),
                        Quadrant::Second => ((1.0, 0.0), (cos * length + 1.0, sin * length)),
                        Quadrant::Third => ((1.0, 1.0), (cos * length + 1.0, sin * length + 1.0)),
                        Quadrant::Fourth => ((0.0, 1.0), (cos * length, sin * length + 1.0)),
                    };
                    peniko::Gradient::new_linear(start, end).with_stops(stops.as_slice())
                }
                viz::Gradient::Radial(radial) => {
                    let start_center = (radial.focal_center.x.get(), radial.focal_center.y.get());
                    let start_radius = radial.focal_radius.get() as f32;
                    let end_center = (radial.center.x.get(), radial.center.y.get());
                    let end_radius = radial.radius.get() as f32;

                    peniko::Gradient::new_two_point_radial(
                        start_center,
                        start_radius,
                        end_center,
                        end_radius,
                    )
                    .with_stops(stops.as_slice())
                }
                viz::Gradient::Conic(conic) => {
                    let angle: f32 = -(viz::Gradient::correct_aspect_ratio(conic.angle, ratio)
                        .to_rad() as f32)
                        .rem_euclid(TAU);
                    let center = (conic.center.x.get(), conic.center.y.get());

                    peniko::Gradient::new_sweep(center, angle, TAU - angle)
                        .with_stops(stops.as_slice())
                }
            };
            peniko::Brush::Gradient(gradient)
        }
        // TODO: Support pattern.
        viz::Paint::Pattern(_) => peniko::Brush::Solid(peniko::Color::RED),
    }
}

/// Calculate the transform of the shape's fill or stroke.
pub fn shape_paint_transform(state: State, paint: &viz::Paint, shape: &viz::Shape) -> Transform {
    let mut shape_size = shape.geometry.bbox_size();
    // Edge cases for strokes.
    if shape_size.x.to_pt() == 0.0 {
        shape_size.x = Abs::pt(1.0);
    }

    if shape_size.y.to_pt() == 0.0 {
        shape_size.y = Abs::pt(1.0);
    }

    if let viz::Paint::Gradient(gradient) = paint {
        match gradient.unwrap_relative(false) {
            viz::RelativeTo::Self_ => Transform::scale(
                Ratio::new(shape_size.x.to_pt()),
                Ratio::new(shape_size.y.to_pt()),
            ),
            viz::RelativeTo::Parent => Transform::scale(
                Ratio::new(state.size.x.to_pt()),
                Ratio::new(state.size.y.to_pt()),
            )
            .post_concat(state.transform.invert().unwrap()),
        }
    } else if let viz::Paint::Pattern(pattern) = paint {
        match pattern.unwrap_relative(false) {
            viz::RelativeTo::Self_ => Transform::identity(),
            viz::RelativeTo::Parent => state.transform.invert().unwrap(),
        }
    } else {
        Transform::identity()
    }
}

/// Calculate the size of the shape's fill.
fn shape_fill_size(state: State, paint: &viz::Paint, shape: &viz::Shape) -> Size {
    let mut shape_size = shape.geometry.bbox_size();
    // Edge cases for strokes.
    if shape_size.x.to_pt() == 0.0 {
        shape_size.x = Abs::pt(1.0);
    }

    if shape_size.y.to_pt() == 0.0 {
        shape_size.y = Abs::pt(1.0);
    }

    if let viz::Paint::Gradient(gradient) = paint {
        match gradient.unwrap_relative(false) {
            viz::RelativeTo::Self_ => shape_size,
            viz::RelativeTo::Parent => state.size,
        }
    } else {
        shape_size
    }
}

pub fn convert_color(color: &viz::Color) -> peniko::Color {
    let channels = color.to_vec4_u8();
    peniko::Color::rgba8(channels[0], channels[1], channels[2], channels[3])
}

pub fn convert_geometry_to_path(geometry: &viz::Geometry) -> kurbo::BezPath {
    match geometry {
        viz::Geometry::Line(p) => kurbo::Shape::to_path(
            &kurbo::Line::new((0.0, 0.0), (p.x.to_pt(), p.y.to_pt())),
            0.1,
        ),
        viz::Geometry::Rect(rect) => kurbo::Shape::to_path(
            &kurbo::Rect::from_origin_size((0.0, 0.0), (rect.x.to_pt(), rect.y.to_pt())),
            0.1,
        ),

        viz::Geometry::Path(p) => convert_path(p),
    }
}

pub fn convert_path(path: &viz::Path) -> kurbo::BezPath {
    let mut bezpath = kurbo::BezPath::new();

    for item in &path.0 {
        match item {
            viz::PathItem::MoveTo(p) => bezpath.move_to((p.x.to_pt(), p.y.to_pt())),
            viz::PathItem::LineTo(p) => bezpath.line_to((p.x.to_pt(), p.y.to_pt())),
            viz::PathItem::CubicTo(p1, p2, p3) => bezpath.curve_to(
                (p1.x.to_pt(), p1.y.to_pt()),
                (p2.x.to_pt(), p2.y.to_pt()),
                (p3.x.to_pt(), p3.y.to_pt()),
            ),
            viz::PathItem::ClosePath => bezpath.close_path(),
        }
    }
    bezpath
}

pub fn convert_transform(transform: Transform) -> kurbo::Affine {
    kurbo::Affine::new([
        transform.sx.get(),
        transform.ky.get(),
        transform.kx.get(),
        transform.sy.get(),
        transform.tx.to_pt(),
        transform.ty.to_pt(),
    ])
}
