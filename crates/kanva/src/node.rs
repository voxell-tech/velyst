use imaging::kurbo::{Affine, BezPath, Stroke};
use imaging::peniko::{Brush, Fill, Style};
use imaging::{ClipRef, Composite};

#[derive(Default, Debug, Clone)]
pub struct Group {
    pub transform: Affine,
    pub clip: Option<KanvaClip>,
    pub composite: Composite,
}

#[derive(Debug, Clone)]
pub struct KanvaClip {
    pub path: BezPath,
    pub transform: Affine,
    pub style: Style,
}

impl KanvaClip {
    pub fn from_ref(clip: ClipRef<'_>) -> Self {
        match clip {
            ClipRef::Fill {
                transform,
                shape,
                fill_rule,
            } => Self {
                path: shape.to_path(0.1),
                transform,
                style: Style::Fill(fill_rule),
            },
            ClipRef::Stroke {
                transform,
                shape,
                stroke,
            } => Self {
                path: shape.to_path(0.1),
                transform,
                style: Style::Stroke(stroke.clone()),
            },
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct KanvaPath {
    pub path: BezPath,
    /// Full world transform as received from [`imaging`].
    pub transform: Affine,
    /// Index into [`crate::Kanva::fills`].
    pub fill: Option<usize>,
    /// Index into [`crate::Kanva::strokes`].
    pub stroke: Option<usize>,
}

#[derive(Debug, Clone, Copy)]
pub enum Command {
    PushGroup(usize),
    PopGroup,
    DrawPath(usize),
}

/// Command index range for a group (parallel to [`crate::Kanva`]'s `groups` vec).
///
/// The inner commands are at `start + 1..end`: `start` is the `PushGroup`
/// and `end` is the `PopGroup`, both excluded when iterating contents.
#[derive(Debug, Clone, Copy)]
pub struct GroupRange {
    /// Index of the `PushGroup` command for this group.
    pub start: usize,
    /// Index of the `PopGroup` command for this group.
    pub end: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeIndex {
    Group(usize),
    Path(usize),
}

#[derive(Default, Debug, Clone)]
pub struct KanvaFill {
    pub rule: Fill,
    pub brush: Brush,
    pub brush_transform: Option<Affine>,
    pub composite: Composite,
}

#[derive(Default, Debug, Clone)]
pub struct KanvaStroke {
    pub stroke: Stroke,
    pub brush: Brush,
    pub brush_transform: Option<Affine>,
    pub composite: Composite,
}

#[derive(Default, Debug, Clone)]
pub struct PathModifier {
    pub path: Option<BezPath>,
    /// Replaces `path.transform` before group animation transforms are applied.
    pub transform: Option<Affine>,
    pub fill: Option<KanvaFill>,
    pub stroke: Option<KanvaStroke>,
    /// Per-path alpha multiplier; wraps the path's draws in an isolated group.
    pub alpha: Option<f32>,
}

#[derive(Default, Debug, Clone)]
pub struct GroupModifier {
    pub transform: Option<Affine>,
    pub clip: Option<KanvaClip>,
    pub composite: Option<Composite>,
}
