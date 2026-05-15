use imaging::kurbo::{Affine, BezPath, Stroke};
use imaging::peniko::{Brush, Fill, Style};
use imaging::{ClipRef, Composite};

/// Tolerance used when flattening curves into [`BezPath`]s.
pub(crate) const PATH_TOLERANCE: f64 = 0.1;

/// A group in the [`Kanva`][crate::Kanva] scene graph.
///
/// Groups carry a transform that is accumulated onto child path world transforms
/// at render time. They may also carry a clip shape and a composite mode.
#[derive(Default, Debug, Clone)]
pub struct Group {
    /// Transform accumulated with parent group transforms at render time.
    pub transform: Affine,
    pub clip: Option<KanvaClip>,
    pub composite: Composite,
}

/// An owned clip shape stored inside a [`Group`].
#[derive(Debug, Clone)]
pub struct KanvaClip {
    pub path: BezPath,
    pub transform: Affine,
    /// Fill rule or stroke style that defines the clip boundary.
    pub style: Style,
}

impl KanvaClip {
    /// Convert a borrowed [`ClipRef`] into an owned `KanvaClip`.
    pub fn from_ref(clip: ClipRef<'_>) -> Self {
        match clip {
            ClipRef::Fill {
                transform,
                shape,
                fill_rule,
            } => Self {
                path: shape.to_path(PATH_TOLERANCE),
                transform,
                style: Style::Fill(fill_rule),
            },
            ClipRef::Stroke {
                transform,
                shape,
                stroke,
            } => Self {
                path: shape.to_path(PATH_TOLERANCE),
                transform,
                style: Style::Stroke(stroke.clone()),
            },
        }
    }
}

/// A stored path with its world transform and optional fill/stroke indices.
#[derive(Default, Debug, Clone)]
pub struct KanvaPath {
    pub path: BezPath,
    /// Full world transform as received from [`imaging`].
    pub transform: Affine,
    /// Index into the fills buffer; retrieve via [`crate::Kanva::get_fill`].
    pub fill: Option<usize>,
    /// Index into the strokes buffer; retrieve via [`crate::Kanva::get_stroke`].
    pub stroke: Option<usize>,
}

/// A draw command in the [`Kanva`][crate::Kanva] command buffer.
#[derive(Debug, Clone, Copy)]
pub enum Command {
    /// Push a group onto the render stack (index into the groups vec).
    PushGroup(usize),
    /// Pop the current group from the render stack.
    PopGroup,
    /// Draw the path at the given index.
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

/// A reference to a node in the [`Kanva`][crate::Kanva] index.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeIndex {
    Group(usize),
    Path(usize),
}

/// Stored fill paint for a [`KanvaPath`].
#[derive(Default, Debug, Clone)]
pub struct KanvaFill {
    pub rule: Fill,
    pub brush: Brush,
    pub brush_transform: Option<Affine>,
    pub composite: Composite,
}

/// Stored stroke paint for a [`KanvaPath`].
#[derive(Default, Debug, Clone)]
pub struct KanvaStroke {
    pub stroke: Stroke,
    pub brush: Brush,
    pub brush_transform: Option<Affine>,
    pub composite: Composite,
}
