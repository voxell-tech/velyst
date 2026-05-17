#![doc = include_str!("../README.md")]

use hashbrown::HashMap;
use imaging::kurbo::Affine;
use imaging::peniko::{BlendMode, Style};
use imaging::{
    ClipRef, Composite, FillRef, GeometryRef, GroupRef, PaintSink,
    StrokeRef,
};

pub mod builder;
pub mod modifiers;
pub mod node;

pub use imaging;
pub use modifiers::{
    GroupModEntry, GroupMods, PathModEntry, PathMods,
};
pub use node::*;

pub mod prelude {
    pub use crate::Kanva;
    pub use crate::builder::KanvaBuilder;
    pub use crate::modifiers::{
        GroupModEntry, GroupMods, PathModEntry, PathMods,
    };
    pub use crate::node::{
        Command, Group, GroupRange, KanvaClip, KanvaFill, KanvaPath,
        KanvaStroke, NodeIndex,
    };
}

/// A baked 2D graphics scene graph.
///
/// Stores paths, fills, strokes, and groups in flat parallel buffers indexed
/// by a [`Command`] buffer that encodes draw order. Groups may carry a
/// transform that is accumulated onto child paths at render time without
/// modifying stored data.
///
/// Primary data is write-once at build time. Per-field overrides are stored
/// in [`PathMods`] and [`GroupMods`] and reset via [`Self::clear_mods`].
/// Each field map is keyed by path or group index; absent = keep stored value.
/// For optional-target fields (`fill`, `stroke`, `clip`), `None` clears the field.
#[derive(Default, Debug, Clone)]
pub struct Kanva {
    commands: Vec<Command>,
    groups: Vec<Group>,
    /// Parallel to `groups`: command index range (push, pop) for each group.
    group_cmds: Vec<GroupRange>,
    paths: Vec<KanvaPath>,
    fills: Vec<KanvaFill>,
    strokes: Vec<KanvaStroke>,
    index: HashMap<Box<str>, NodeIndex>,
    /// Per-path field overrides.
    pub path_mods: PathMods,
    /// Per-group field overrides.
    pub group_mods: GroupMods,
}

impl Kanva {
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns `true` if the kanva has no commands (nothing to render).
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    /// Look up a node by label, returning its [`NodeIndex`].
    ///
    /// Labels are assigned during build via [`imaging::ContextRef`].
    pub fn query(&self, label: &str) -> Option<NodeIndex> {
        self.index.get(label).copied()
    }

    /// Look up a labeled group and return its raw index directly.
    ///
    /// Returns `None` if the label does not exist or resolves to a path.
    pub fn query_group(&self, label: &str) -> Option<usize> {
        match self.index.get(label).copied()? {
            NodeIndex::Group(i) => Some(i),
            _ => None,
        }
    }

    /// Look up a labeled path and return its raw index directly.
    ///
    /// Returns `None` if the label does not exist or resolves to a group.
    pub fn query_path(&self, label: &str) -> Option<usize> {
        match self.index.get(label).copied()? {
            NodeIndex::Path(i) => Some(i),
            _ => None,
        }
    }

    /// Returns the [`KanvaPath`] at `idx`, or `None` if out of bounds.
    pub fn get_path(&self, idx: usize) -> Option<&KanvaPath> {
        self.paths.get(idx)
    }

    /// Returns the [`KanvaFill`] at `idx`, or `None` if out of bounds.
    pub fn get_fill(&self, idx: usize) -> Option<&KanvaFill> {
        self.fills.get(idx)
    }

    /// Returns the [`KanvaStroke`] at `idx`, or `None` if out of bounds.
    pub fn get_stroke(&self, idx: usize) -> Option<&KanvaStroke> {
        self.strokes.get(idx)
    }

    /// Returns the [`Group`] at `idx`, or `None` if out of bounds.
    pub fn get_group(&self, idx: usize) -> Option<&Group> {
        self.groups.get(idx)
    }

    /// Returns the contiguous range of path indices directly owned by this group.
    ///
    /// Scans the group's inner commands (PushGroup/PopGroup excluded) for the
    /// first and last [`Command::DrawPath`] indices and returns `first..last + 1`.
    /// Returns `None` if the group index is out of bounds or the group has no paths.
    /// Use [`Self::get_path`] to access individual paths by index.
    ///
    /// ```
    /// use kanva::prelude::*;
    /// use kanva::imaging::kurbo::BezPath;
    /// use kanva::imaging::peniko::Brush;
    /// use kanva::imaging::{FillRef, GeometryRef, GroupRef, PaintSink};
    ///
    /// let mut builder = KanvaBuilder::new();
    /// let path = BezPath::new();
    /// let brush = Brush::default();
    /// builder.push_group(GroupRef::new());
    /// builder.fill(FillRef::new(GeometryRef::Path(&path), &brush));
    /// builder.fill(FillRef::new(GeometryRef::Path(&path), &brush));
    /// builder.pop_group();
    /// let kanva = builder.build();
    ///
    /// assert_eq!(kanva.get_group_path_range(0).unwrap().len(), 2);
    /// ```
    pub fn get_group_path_range(
        &self,
        group_idx: usize,
    ) -> Option<core::ops::Range<usize>> {
        let range = self.group_cmds.get(group_idx)?;
        let cmds = &self.commands[range.start + 1..range.end];
        let first = cmds.iter().find_map(|c| {
            if let Command::DrawPath(i) = c {
                Some(*i)
            } else {
                None
            }
        })?;
        let last = cmds.iter().rev().find_map(|c| {
            if let Command::DrawPath(i) = c {
                Some(*i)
            } else {
                None
            }
        })?;
        Some(first..last + 1)
    }

    /// Returns the contiguous range of path indices that appear in the command
    /// stream between two labeled marker groups (exclusive of the groups
    /// themselves). Useful when paths are placed between `<start>` and `<end>`
    /// marker boxes rather than inside a single labeled group.
    pub fn get_paths_between_groups(
        &self,
        start_group: usize,
        end_group: usize,
    ) -> Option<core::ops::Range<usize>> {
        let start_end = self.group_cmds.get(start_group)?.end;
        let end_start = self.group_cmds.get(end_group)?.start;
        let cmds = &self.commands[start_end + 1..end_start];
        let first = cmds.iter().find_map(|c| {
            if let Command::DrawPath(i) = c {
                Some(*i)
            } else {
                None
            }
        })?;
        let last = cmds.iter().rev().find_map(|c| {
            if let Command::DrawPath(i) = c {
                Some(*i)
            } else {
                None
            }
        })?;
        Some(first..last + 1)
    }

    /// Return a cursor for setting per-path field overrides at `path_idx`.
    pub fn mod_path(&mut self, path_idx: usize) -> PathModEntry<'_> {
        PathModEntry::new(&mut self.path_mods, path_idx)
    }

    /// Return a cursor for setting per-group field overrides at `group_idx`.
    pub fn mod_group(
        &mut self,
        group_idx: usize,
    ) -> GroupModEntry<'_> {
        GroupModEntry::new(&mut self.group_mods, group_idx)
    }

    /// Clear all active overrides in [`PathMods`] and [`GroupMods`].
    ///
    /// The next render will use the original stored data.
    pub fn clear_mods(&mut self) {
        self.path_mods.clear();
        self.group_mods.clear();
    }

    /// Render into any [`PaintSink`].
    ///
    /// Group transforms are accumulated and multiplied by each path's stored
    /// world transform at draw time.
    pub fn render(&self, sink: &mut impl PaintSink) {
        let mut group_tf_stack = vec![Affine::IDENTITY];

        for cmd in &self.commands {
            match *cmd {
                Command::PushGroup(idx) => {
                    let group = &self.groups[idx];
                    let parent_tf = *group_tf_stack.last().unwrap();
                    let group_tf = self
                        .group_mods
                        .transform
                        .get(&idx)
                        .copied()
                        .unwrap_or(group.transform);
                    group_tf_stack.push(group_tf * parent_tf);

                    let clip: Option<&KanvaClip> = if let Some(ov) =
                        self.group_mods.clip.get(&idx)
                    {
                        ov.as_ref()
                    } else {
                        group.clip.as_ref()
                    };
                    let clip = clip.map(|c| match &c.style {
                        Style::Fill(fill_rule) => ClipRef::Fill {
                            transform: c.transform,
                            shape: GeometryRef::Path(&c.path),
                            fill_rule: *fill_rule,
                        },
                        Style::Stroke(stroke) => ClipRef::Stroke {
                            transform: c.transform,
                            shape: GeometryRef::Path(&c.path),
                            stroke,
                        },
                    });

                    let composite = self
                        .group_mods
                        .composite
                        .get(&idx)
                        .copied()
                        .unwrap_or(group.composite);
                    let mut group_ref =
                        GroupRef::new().with_composite(composite);
                    if let Some(c) = clip {
                        group_ref = group_ref.with_clip(c);
                    }
                    sink.push_group(group_ref);
                }
                Command::PopGroup => {
                    group_tf_stack.pop();
                    sink.pop_group();
                }
                Command::DrawPath(idx) => {
                    let path = &self.paths[idx];
                    let group_tf = *group_tf_stack.last().unwrap();

                    let data = self
                        .path_mods
                        .shape
                        .get(&idx)
                        .unwrap_or(&path.path);
                    let base_tf = self
                        .path_mods
                        .transform
                        .get(&idx)
                        .copied()
                        .unwrap_or(path.transform);
                    let eff_tf = group_tf * base_tf;

                    let alpha =
                        self.path_mods.alpha.get(&idx).copied();
                    if let Some(a) = alpha {
                        sink.push_group(
                            GroupRef::new().with_composite(
                                Composite::new(
                                    BlendMode::default(),
                                    a,
                                ),
                            ),
                        );
                    }

                    let fill = if let Some(fill_mod) =
                        self.path_mods.fill.get(&idx)
                    {
                        fill_mod.as_ref()
                    } else {
                        path.fill.map(|i| &self.fills[i])
                    };
                    if let Some(fill) = fill {
                        sink.fill(FillRef {
                            transform: eff_tf,
                            fill_rule: fill.rule,
                            brush: (&fill.brush).into(),
                            brush_transform: fill.brush_transform,
                            shape: GeometryRef::Path(data),
                            composite: fill.composite,
                        });
                    }

                    let stroke_style = if let Some(stroke_mod) =
                        self.path_mods.stroke.get(&idx)
                    {
                        stroke_mod.as_ref()
                    } else {
                        path.stroke.map(|i| &self.strokes[i])
                    };
                    if let Some(stroke_style) = stroke_style {
                        sink.stroke(StrokeRef {
                            transform: eff_tf,
                            stroke: &stroke_style.stroke,
                            brush: (&stroke_style.brush).into(),
                            brush_transform: stroke_style
                                .brush_transform,
                            shape: GeometryRef::Path(data),
                            composite: stroke_style.composite,
                        });
                    }

                    if alpha.is_some() {
                        sink.pop_group();
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builder::KanvaBuilder;
    use imaging::kurbo::{Affine, BezPath, Stroke};
    use imaging::peniko::{Brush, Color};
    use imaging::record::{Command as RecCmd, Draw, Scene};
    use imaging::{
        FillRef, GeometryRef, GroupRef, PaintSink, StrokeRef,
    };

    fn build_fill(brush: &Brush, transform: Affine) -> Kanva {
        let mut b = KanvaBuilder::new();
        let path = BezPath::new();
        b.fill(
            FillRef::new(GeometryRef::Path(&path), brush)
                .transform(transform),
        );
        b.build()
    }

    fn build_stroke(transform: Affine) -> Kanva {
        let mut b = KanvaBuilder::new();
        let path = BezPath::new();
        let stroke = Stroke::default();
        let brush = Brush::default();
        b.stroke(
            StrokeRef::new(GeometryRef::Path(&path), &stroke, &brush)
                .transform(transform),
        );
        b.build()
    }

    #[test]
    fn render_fill_emits_fill() {
        let kanva = build_fill(&Brush::default(), Affine::IDENTITY);
        let mut scene = Scene::new();
        kanva.render(&mut scene);
        let cmds = scene.commands();
        assert_eq!(cmds.len(), 1);
        let RecCmd::Draw(id) = cmds[0] else {
            panic!("expected Draw")
        };
        assert!(matches!(scene.draw_op(id), Draw::Fill { .. }));
    }

    #[test]
    fn render_stroke_emits_stroke() {
        let kanva = build_stroke(Affine::IDENTITY);
        let mut scene = Scene::new();
        kanva.render(&mut scene);
        let cmds = scene.commands();
        assert_eq!(cmds.len(), 1);
        let RecCmd::Draw(id) = cmds[0] else {
            panic!("expected Draw")
        };
        assert!(matches!(scene.draw_op(id), Draw::Stroke { .. }));
    }

    #[test]
    fn render_group_pushpop() {
        let mut b = KanvaBuilder::new();
        let path = BezPath::new();
        let brush = Brush::default();
        b.push_group(GroupRef::new());
        b.fill(FillRef::new(GeometryRef::Path(&path), &brush));
        b.pop_group();
        let kanva = b.build();
        let mut scene = Scene::new();
        kanva.render(&mut scene);
        let cmds = scene.commands();
        assert_eq!(cmds.len(), 3);
        assert!(matches!(cmds[0], RecCmd::PushGroup(_)));
        assert!(matches!(cmds[1], RecCmd::Draw(_)));
        assert!(matches!(cmds[2], RecCmd::PopGroup));
    }

    #[test]
    fn render_group_transform_accumulated() {
        let scale = Affine::scale(2.0);
        let path_tf = Affine::translate((10.0, 0.0));

        let mut b = KanvaBuilder::new();
        let path = BezPath::new();
        let brush = Brush::default();
        b.push_group(GroupRef::new());
        b.fill(
            FillRef::new(GeometryRef::Path(&path), &brush)
                .transform(path_tf),
        );
        b.pop_group();
        let mut kanva = b.build();
        kanva.groups[0].transform = scale;

        let mut scene = Scene::new();
        kanva.render(&mut scene);
        let cmds = scene.commands();
        let RecCmd::Draw(id) = cmds[1] else {
            panic!("expected Draw")
        };
        let Draw::Fill { transform, .. } = scene.draw_op(id) else {
            panic!()
        };
        assert_eq!(*transform, scale * path_tf);
    }

    #[test]
    fn render_path_mod_fill_overrides() {
        let original = Brush::Solid(Color::BLACK);
        let override_brush = Brush::Solid(Color::WHITE);
        let mut kanva = build_fill(&original, Affine::IDENTITY);
        kanva.path_mods.fill(
            0,
            Some(KanvaFill {
                brush: override_brush.clone(),
                ..Default::default()
            }),
        );
        let mut scene = Scene::new();
        kanva.render(&mut scene);
        let RecCmd::Draw(id) = scene.commands()[0] else {
            panic!()
        };
        let Draw::Fill { brush, .. } = scene.draw_op(id) else {
            panic!()
        };
        assert_eq!(*brush, override_brush);
    }

    #[test]
    fn render_path_mod_alpha_wraps_group() {
        let mut kanva =
            build_fill(&Brush::default(), Affine::IDENTITY);
        kanva.path_mods.alpha(0, 0.5);
        let mut scene = Scene::new();
        kanva.render(&mut scene);
        let cmds = scene.commands();
        assert_eq!(cmds.len(), 3);
        let RecCmd::PushGroup(gid) = cmds[0] else {
            panic!("expected PushGroup")
        };
        assert_eq!(scene.group(gid).composite.alpha, 0.5);
        assert!(matches!(cmds[1], RecCmd::Draw(_)));
        assert!(matches!(cmds[2], RecCmd::PopGroup));
    }

    #[test]
    fn render_path_mod_transform() {
        let override_tf = Affine::translate((5.0, 3.0));
        let mut kanva =
            build_fill(&Brush::default(), Affine::IDENTITY);
        kanva.path_mods.transform(0, override_tf);
        let mut scene = Scene::new();
        kanva.render(&mut scene);
        let RecCmd::Draw(id) = scene.commands()[0] else {
            panic!()
        };
        let Draw::Fill { transform, .. } = scene.draw_op(id) else {
            panic!()
        };
        assert_eq!(*transform, override_tf);
    }

    #[test]
    fn render_group_mod_transform() {
        let base_tf = Affine::scale(2.0);
        let override_tf = Affine::scale(3.0);
        let path_tf = Affine::translate((1.0, 0.0));

        let mut b = KanvaBuilder::new();
        let path = BezPath::new();
        let brush = Brush::default();
        b.push_group(GroupRef::new());
        b.fill(
            FillRef::new(GeometryRef::Path(&path), &brush)
                .transform(path_tf),
        );
        b.pop_group();
        let mut kanva = b.build();
        kanva.groups[0].transform = base_tf;
        kanva.group_mods.transform(0, override_tf);

        let mut scene = Scene::new();
        kanva.render(&mut scene);
        let RecCmd::Draw(id) = scene.commands()[1] else {
            panic!()
        };
        let Draw::Fill { transform, .. } = scene.draw_op(id) else {
            panic!()
        };
        assert_eq!(*transform, override_tf * path_tf);
    }

    #[test]
    fn render_group_mod_composite() {
        let composite = Composite::new(BlendMode::default(), 0.75);
        let mut b = KanvaBuilder::new();
        let path = BezPath::new();
        let brush = Brush::default();
        b.push_group(GroupRef::new());
        b.fill(FillRef::new(GeometryRef::Path(&path), &brush));
        b.pop_group();
        let mut kanva = b.build();
        kanva.group_mods.composite(0, composite);
        let mut scene = Scene::new();
        kanva.render(&mut scene);
        let RecCmd::PushGroup(gid) = scene.commands()[0] else {
            panic!()
        };
        assert_eq!(scene.group(gid).composite, composite);
    }

    #[test]
    fn render_clear_mods_restores_original() {
        let original = Brush::Solid(Color::BLACK);
        let override_brush = Brush::Solid(Color::WHITE);
        let mut kanva = build_fill(&original, Affine::IDENTITY);
        kanva.path_mods.fill(
            0,
            Some(KanvaFill {
                brush: override_brush,
                ..Default::default()
            }),
        );

        // First render: override active.
        let mut scene = Scene::new();
        kanva.render(&mut scene);
        let RecCmd::Draw(id) = scene.commands()[0] else {
            panic!()
        };
        let Draw::Fill {
            brush: first_brush, ..
        } = scene.draw_op(id)
        else {
            panic!()
        };
        assert_eq!(*first_brush, Brush::Solid(Color::WHITE));

        kanva.clear_mods();

        // Second render: original restored.
        let mut scene = Scene::new();
        kanva.render(&mut scene);
        let RecCmd::Draw(id) = scene.commands()[0] else {
            panic!()
        };
        let Draw::Fill {
            brush: second_brush,
            ..
        } = scene.draw_op(id)
        else {
            panic!()
        };
        assert_eq!(*second_brush, original);
    }
}
