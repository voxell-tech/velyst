use hashbrown::HashMap;
use imaging::kurbo::Affine;
use imaging::peniko::{BlendMode, Style};
use imaging::{
    ClipRef, Composite, FillRef, GeometryRef, GroupRef, PaintSink,
    StrokeRef,
};

pub mod builder;
pub mod node;

pub use node::*;

pub mod prelude {
    pub use crate::{
        Kanva,
        builder::KanvaBuilder,
        node::{
            Command, Group, GroupModifier, GroupRange, KanvaClip,
            KanvaFill, KanvaPath, KanvaStroke, NodeIndex,
            PathModifier,
        },
    };
}

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
    path_mods: HashMap<usize, PathModifier>,
    group_mods: HashMap<usize, GroupModifier>,
}

impl Kanva {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn query(&self, label: &str) -> Option<NodeIndex> {
        self.index.get(label).copied()
    }

    pub fn query_group(&self, label: &str) -> Option<usize> {
        match self.index.get(label).copied()? {
            NodeIndex::Group(i) => Some(i),
            _ => None,
        }
    }

    pub fn query_path(&self, label: &str) -> Option<usize> {
        match self.index.get(label).copied()? {
            NodeIndex::Path(i) => Some(i),
            _ => None,
        }
    }

    pub fn get_path(&self, idx: usize) -> Option<&KanvaPath> {
        self.paths.get(idx)
    }

    pub fn get_fill(&self, idx: usize) -> Option<&KanvaFill> {
        self.fills.get(idx)
    }

    pub fn get_stroke(&self, idx: usize) -> Option<&KanvaStroke> {
        self.strokes.get(idx)
    }

    pub fn get_group(&self, idx: usize) -> Option<&Group> {
        self.groups.get(idx)
    }

    /// Returns the contiguous slice of [`KanvaPath`]s directly owned by this group.
    ///
    /// Scans the group's inner commands (PushGroup/PopGroup excluded) for the
    /// first and last [`Command::DrawPath`] indices, then returns that path slice.
    /// Returns `None` if the group index is out of bounds or the group has no paths.
    pub fn get_group_shapes(
        &self,
        group_idx: usize,
    ) -> Option<&[KanvaPath]> {
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
        self.paths.get(first..=last)
    }

    pub fn set_path_mod(
        &mut self,
        path_idx: usize,
        modifier: PathModifier,
    ) {
        self.path_mods.insert(path_idx, modifier);
    }

    pub fn set_group_mod(
        &mut self,
        group_idx: usize,
        modifier: GroupModifier,
    ) {
        self.group_mods.insert(group_idx, modifier);
    }

    pub fn clear_mods(&mut self) {
        self.path_mods.clear();
        self.group_mods.clear();
    }

    /// Render into any [`PaintSink`].
    ///
    /// Group transforms are accumulated as animation deltas and multiplied by
    /// each path's stored world transform at draw time.
    pub fn render(&self, sink: &mut impl PaintSink) {
        // Tracks the cumulative product of group animation transforms.
        let mut group_tf_stack = vec![Affine::IDENTITY];

        for cmd in &self.commands {
            match *cmd {
                Command::PushGroup(idx) => {
                    let group = &self.groups[idx];
                    let modifier = self.group_mods.get(&idx);
                    let parent_tf = *group_tf_stack.last().unwrap();
                    let group_tf = modifier
                        .and_then(|m| m.transform)
                        .unwrap_or(group.transform);
                    group_tf_stack.push(group_tf * parent_tf);

                    let clip = modifier
                        .and_then(|m| m.clip.as_ref())
                        .or(group.clip.as_ref())
                        .map(|c| match &c.style {
                            Style::Fill(fill_rule) => ClipRef::Fill {
                                transform: c.transform,
                                shape: GeometryRef::Path(&c.path),
                                fill_rule: *fill_rule,
                            },
                            Style::Stroke(stroke) => {
                                ClipRef::Stroke {
                                    transform: c.transform,
                                    shape: GeometryRef::Path(&c.path),
                                    stroke,
                                }
                            }
                        });

                    let composite = modifier
                        .and_then(|m| m.composite)
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
                    let modifier = self.path_mods.get(&idx);

                    let data = modifier
                        .and_then(|m| m.path.as_ref())
                        .unwrap_or(&path.path);
                    let base_tf = modifier
                        .and_then(|m| m.transform)
                        .unwrap_or(path.transform);
                    let eff_tf = group_tf * base_tf;

                    let alpha = modifier.and_then(|m| m.alpha);
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

                    let fill = modifier
                        .and_then(|m| m.fill.as_ref())
                        .or_else(|| {
                            path.fill.map(|i| &self.fills[i])
                        });
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

                    let stroke_style = modifier
                        .and_then(|m| m.stroke.as_ref())
                        .or_else(|| {
                            path.stroke.map(|i| &self.strokes[i])
                        });
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
        kanva.set_path_mod(
            0,
            PathModifier {
                fill: Some(KanvaFill {
                    brush: override_brush.clone(),
                    ..Default::default()
                }),
                ..Default::default()
            },
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
        kanva.set_path_mod(
            0,
            PathModifier {
                alpha: Some(0.5),
                ..Default::default()
            },
        );
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
        kanva.set_path_mod(
            0,
            PathModifier {
                transform: Some(override_tf),
                ..Default::default()
            },
        );
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
        kanva.set_group_mod(
            0,
            GroupModifier {
                transform: Some(override_tf),
                ..Default::default()
            },
        );

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
        kanva.set_group_mod(
            0,
            GroupModifier {
                composite: Some(composite),
                ..Default::default()
            },
        );
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
        kanva.set_path_mod(
            0,
            PathModifier {
                fill: Some(KanvaFill {
                    brush: override_brush,
                    ..Default::default()
                }),
                ..Default::default()
            },
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
