use imaging::kurbo::{Affine, BezPath};
use imaging::peniko::Style;
use imaging::record::Glyph;
use imaging::{
    BlurredRoundedRect, ClipRef, ContextRef, FillRef, GlyphRunRef,
    GroupRef, PaintSink, StrokeRef,
};
use ttf_parser::OutlineBuilder;

use crate::{
    Command, Group, GroupRange, Kanva, KanvaClip, KanvaFill,
    KanvaPath, KanvaStroke, NodeIndex,
};

/// Builds a [`Kanva`] by consuming an [`imaging::PaintSink`] draw stream.
///
/// Feed any draw stream (e.g. a Typst frame rendered via `typst_imaging`) into
/// this builder, then call [`Self::build`] to get the finished `Kanva`.
/// Wrap draws with [`imaging::ContextRef`] push/pop to label nodes for later
/// lookup via [`Kanva::query`].
pub struct KanvaBuilder {
    kanva: Kanva,
    group_stack: Vec<usize>,
    pending_label: Option<Box<str>>,
}

impl KanvaBuilder {
    pub fn new() -> Self {
        Self {
            kanva: Kanva::new(),
            group_stack: Vec::new(),
            pending_label: None,
        }
    }

    /// Finish building and return the [`Kanva`].
    ///
    /// Panics in debug builds if any groups were left unclosed.
    pub fn build(self) -> Kanva {
        debug_assert!(
            self.group_stack.is_empty(),
            "unclosed groups in KanvaBuilder"
        );
        self.kanva
    }

    fn push_path(&mut self, path: KanvaPath) -> usize {
        let idx = self.kanva.paths.len();
        if let Some(label) = self.pending_label.take() {
            self.kanva.index.insert(label, NodeIndex::Path(idx));
        }
        self.kanva.paths.push(path);
        idx
    }

    fn push_group_entry(&mut self, group: Group) {
        let idx = self.kanva.groups.len();
        if let Some(label) = self.pending_label.take() {
            self.kanva.index.insert(label, NodeIndex::Group(idx));
        }
        let cmd_start = self.kanva.commands.len();
        self.group_stack.push(idx);
        self.kanva.commands.push(Command::PushGroup(idx));
        self.kanva.groups.push(group);
        self.kanva.group_cmds.push(GroupRange {
            start: cmd_start,
            end: cmd_start,
        });
    }

    fn pop_group_entry(&mut self) {
        if let Some(idx) = self.group_stack.pop() {
            self.kanva.group_cmds[idx].end =
                self.kanva.commands.len();
        }
        self.kanva.commands.push(Command::PopGroup);
    }
}

impl Default for KanvaBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl PaintSink for KanvaBuilder {
    fn push_context(&mut self, context: ContextRef<'_>) {
        self.pending_label = Some(context.label.into());
    }

    fn pop_context(&mut self) {
        self.pending_label = None;
    }

    fn push_clip(&mut self, clip: ClipRef<'_>) {
        self.push_group_entry(Group {
            clip: Some(KanvaClip::from_ref(clip)),
            ..Default::default()
        });
    }

    fn pop_clip(&mut self) {
        self.pop_group_entry();
    }

    fn push_group(&mut self, group: GroupRef<'_>) {
        self.push_group_entry(Group {
            clip: group.clip.map(KanvaClip::from_ref),
            composite: group.composite,
            ..Default::default()
        });
    }

    fn pop_group(&mut self) {
        self.pop_group_entry();
    }

    fn fill(&mut self, draw: FillRef<'_>) {
        let fill_idx = self.kanva.fills.len();
        self.kanva.fills.push(KanvaFill {
            rule: draw.fill_rule,
            brush: draw.brush.to_owned(),
            brush_transform: draw.brush_transform,
            composite: draw.composite,
        });
        let path = KanvaPath {
            path: draw.shape.to_path(crate::node::PATH_TOLERANCE),
            transform: draw.transform,
            fill: Some(fill_idx),
            ..Default::default()
        };
        let idx = self.push_path(path);
        self.kanva.commands.push(Command::DrawPath(idx));
    }

    fn stroke(&mut self, draw: StrokeRef<'_>) {
        let stroke_idx = self.kanva.strokes.len();
        self.kanva.strokes.push(KanvaStroke {
            stroke: draw.stroke.clone(),
            brush: draw.brush.to_owned(),
            brush_transform: draw.brush_transform,
            composite: draw.composite,
        });
        let path = KanvaPath {
            path: draw.shape.to_path(crate::node::PATH_TOLERANCE),
            transform: draw.transform,
            stroke: Some(stroke_idx),
            ..Default::default()
        };
        let idx = self.push_path(path);
        self.kanva.commands.push(Command::DrawPath(idx));
    }

    fn glyph_run(
        &mut self,
        draw: GlyphRunRef<'_>,
        glyphs: &mut dyn Iterator<Item = Glyph>,
    ) {
        let font_data = draw.font.data.data();
        let Ok(face) =
            ttf_parser::Face::parse(font_data, draw.font.index)
        else {
            return;
        };

        let units_per_em = face.units_per_em();
        if units_per_em == 0 {
            return;
        }
        let scale = draw.font_size as f64 / units_per_em as f64;
        let scale_tf = Affine::scale_non_uniform(scale, -scale);

        self.push_group_entry(Group {
            composite: draw.composite,
            ..Default::default()
        });

        for glyph in glyphs {
            let glyph_id = ttf_parser::GlyphId(glyph.id as u16);
            let mut pen = GlyphPen(BezPath::new());
            if face.outline_glyph(glyph_id, &mut pen).is_none() {
                continue;
            }

            let glyph_tf = draw.transform
                * Affine::translate((glyph.x as f64, glyph.y as f64))
                * scale_tf;

            match &draw.style {
                Style::Fill(fill_rule) => {
                    let fill_idx = self.kanva.fills.len();
                    self.kanva.fills.push(KanvaFill {
                        rule: *fill_rule,
                        brush: draw.brush.to_owned(),
                        ..Default::default()
                    });
                    let path_idx = self.push_path(KanvaPath {
                        path: pen.0,
                        transform: glyph_tf,
                        fill: Some(fill_idx),
                        ..Default::default()
                    });
                    self.kanva
                        .commands
                        .push(Command::DrawPath(path_idx));
                }
                Style::Stroke(stroke) => {
                    let stroke_idx = self.kanva.strokes.len();
                    self.kanva.strokes.push(KanvaStroke {
                        stroke: stroke.clone(),
                        brush: draw.brush.to_owned(),
                        ..Default::default()
                    });
                    let path_idx = self.push_path(KanvaPath {
                        path: pen.0,
                        transform: glyph_tf,
                        stroke: Some(stroke_idx),
                        ..Default::default()
                    });
                    self.kanva
                        .commands
                        .push(Command::DrawPath(path_idx));
                }
            }
        }

        self.pop_group_entry();
    }

    fn blurred_rounded_rect(&mut self, _draw: BlurredRoundedRect) {
        // Not supported in Kanva.
    }
}

struct GlyphPen(BezPath);

impl OutlineBuilder for GlyphPen {
    fn move_to(&mut self, x: f32, y: f32) {
        self.0.move_to((x as f64, y as f64));
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.0.line_to((x as f64, y as f64));
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.0.quad_to((x1 as f64, y1 as f64), (x as f64, y as f64));
    }

    fn curve_to(
        &mut self,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        x: f32,
        y: f32,
    ) {
        self.0.curve_to(
            (x1 as f64, y1 as f64),
            (x2 as f64, y2 as f64),
            (x as f64, y as f64),
        );
    }

    fn close(&mut self) {
        self.0.close_path();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::NodeIndex;
    use imaging::kurbo::{BezPath, Stroke};
    use imaging::peniko::Brush;
    use imaging::{
        ClipRef, ContextRef, FillRef, GeometryRef, GroupRef,
        StrokeRef,
    };

    #[test]
    fn empty_build() {
        let k = KanvaBuilder::new().build();
        assert!(k.commands.is_empty());
        assert!(k.groups.is_empty());
        assert!(k.paths.is_empty());
        assert!(k.index.is_empty());
    }

    #[test]
    fn fill_creates_path_and_command() {
        let mut b = KanvaBuilder::new();
        let path = BezPath::new();
        let brush = Brush::default();
        b.fill(FillRef::new(GeometryRef::Path(&path), &brush));
        let k = b.build();
        assert_eq!(k.paths.len(), 1);
        assert_eq!(k.commands.len(), 1);
        assert!(matches!(k.commands[0], Command::DrawPath(0)));
        assert!(k.paths[0].fill.is_some());
        assert!(k.paths[0].stroke.is_none());
    }

    #[test]
    fn stroke_creates_path_and_command() {
        let mut b = KanvaBuilder::new();
        let path = BezPath::new();
        let stroke = Stroke::default();
        let brush = Brush::default();
        b.stroke(StrokeRef::new(
            GeometryRef::Path(&path),
            &stroke,
            &brush,
        ));
        let k = b.build();
        assert_eq!(k.paths.len(), 1);
        assert_eq!(k.commands.len(), 1);
        assert!(matches!(k.commands[0], Command::DrawPath(0)));
        assert!(k.paths[0].stroke.is_some());
        assert!(k.paths[0].fill.is_none());
    }

    #[test]
    fn group_ends_siblings() {
        // commands: [PushGroup(0), DrawPath(0), PopGroup(idx=2), PushGroup(1), DrawPath(1), PopGroup(idx=5)]
        let mut b = KanvaBuilder::new();
        let path = BezPath::new();
        let brush = Brush::default();
        b.push_group(GroupRef::new());
        b.fill(FillRef::new(GeometryRef::Path(&path), &brush));
        b.pop_group();
        b.push_group(GroupRef::new());
        b.fill(FillRef::new(GeometryRef::Path(&path), &brush));
        b.pop_group();
        let k = b.build();
        assert_eq!(
            k.group_cmds[0].start, 0,
            "first PushGroup at index 0"
        );
        assert_eq!(
            k.group_cmds[0].end, 2,
            "first PopGroup at index 2"
        );
        assert_eq!(
            k.group_cmds[1].start, 3,
            "second PushGroup at index 3"
        );
        assert_eq!(
            k.group_cmds[1].end, 5,
            "second PopGroup at index 5"
        );
        assert!(matches!(k.commands[2], Command::PopGroup));
        assert!(matches!(k.commands[5], Command::PopGroup));
    }

    #[test]
    fn group_ends_nested() {
        // commands: [PushGroup(0), PushGroup(1), DrawPath(0), PopGroup(idx=3), PopGroup(idx=4)]
        let mut b = KanvaBuilder::new();
        let path = BezPath::new();
        let brush = Brush::default();
        b.push_group(GroupRef::new());
        b.push_group(GroupRef::new());
        b.fill(FillRef::new(GeometryRef::Path(&path), &brush));
        b.pop_group();
        b.pop_group();
        let k = b.build();
        assert_eq!(
            k.group_cmds[0].start, 0,
            "outer PushGroup at index 0"
        );
        assert_eq!(
            k.group_cmds[1].start, 1,
            "inner PushGroup at index 1"
        );
        assert_eq!(
            k.group_cmds[1].end, 3,
            "inner PopGroup at index 3"
        );
        assert_eq!(
            k.group_cmds[0].end, 4,
            "outer PopGroup at index 4"
        );
        assert!(matches!(k.commands[3], Command::PopGroup));
        assert!(matches!(k.commands[4], Command::PopGroup));
    }

    #[test]
    fn get_group_returns_group() {
        let mut b = KanvaBuilder::new();
        b.push_group(GroupRef::new());
        b.pop_group();
        let k = b.build();
        assert!(k.get_group(0).is_some());
        assert!(k.get_group(1).is_none());
    }

    #[test]
    fn get_group_shapes_returns_slice() {
        let mut b = KanvaBuilder::new();
        let path = BezPath::new();
        let brush = Brush::default();
        b.push_group(GroupRef::new());
        b.fill(FillRef::new(GeometryRef::Path(&path), &brush));
        b.fill(FillRef::new(GeometryRef::Path(&path), &brush));
        b.pop_group();
        let k = b.build();
        assert_eq!(k.get_group_shapes(0).unwrap().len(), 2);
    }

    #[test]
    fn label_indexes_path() {
        let mut b = KanvaBuilder::new();
        let path = BezPath::new();
        let brush = Brush::default();
        b.push_context(ContextRef::new("foo", None));
        b.fill(FillRef::new(GeometryRef::Path(&path), &brush));
        b.pop_context();
        let k = b.build();
        assert_eq!(k.query("foo"), Some(NodeIndex::Path(0)));
    }

    #[test]
    fn label_indexes_group() {
        let mut b = KanvaBuilder::new();
        b.push_context(ContextRef::new("bar", None));
        b.push_group(GroupRef::new());
        b.pop_group();
        b.pop_context();
        let k = b.build();
        assert_eq!(k.query("bar"), Some(NodeIndex::Group(0)));
    }

    #[test]
    fn clip_stored_in_group() {
        let mut b = KanvaBuilder::new();
        let path = BezPath::new();
        b.push_clip(ClipRef::fill(GeometryRef::Path(&path)));
        b.pop_clip();
        let k = b.build();
        assert!(k.groups[0].clip.is_some());
    }
}
