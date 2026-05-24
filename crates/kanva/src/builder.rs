use imaging::kurbo::{Affine, BezPath};
use imaging::record::Glyph;
use ttf_parser::OutlineBuilder;

use crate::sink::{GlyphRun, KanvaSink};

use crate::{
    Command, Group, GroupRange, Kanva, KanvaFill, KanvaPath,
    KanvaStroke, NodeIndex,
};

/// Builds a [`Kanva`] by consuming a [`KanvaSink`] draw stream.
///
/// Feed any draw stream into this builder, then call [`Self::build`] to get
/// the finished `Kanva`.
/// Wrap draws with [`KanvaSink::push_context`] / [`KanvaSink::pop_context`]
/// to label nodes for later lookup via [`Kanva::query`].
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


impl KanvaSink for KanvaBuilder {
    fn push_context(&mut self, label: &str) {
        self.pending_label = Some(label.into());
    }

    fn pop_context(&mut self) {
        self.pending_label = None;
    }

    fn push_group(&mut self, group: Group) {
        self.push_group_entry(group);
    }

    fn pop_group(&mut self) {
        self.pop_group_entry();
    }

    fn draw_path(
        &mut self,
        path: BezPath,
        transform: Affine,
        fill: Option<KanvaFill>,
        stroke: Option<KanvaStroke>,
    ) {
        let fill_idx = fill.map(|f| {
            let idx = self.kanva.fills.len();
            self.kanva.fills.push(f);
            idx
        });
        let stroke_idx = stroke.map(|s| {
            let idx = self.kanva.strokes.len();
            self.kanva.strokes.push(s);
            idx
        });
        let path_idx = self.push_path(KanvaPath {
            path,
            transform,
            fill: fill_idx,
            stroke: stroke_idx,
        });
        self.kanva.commands.push(Command::DrawPath(path_idx));
    }

    fn glyph_run(
        &mut self,
        run: GlyphRun,
        fill: Option<KanvaFill>,
        stroke: Option<KanvaStroke>,
        glyphs: &mut dyn Iterator<Item = Glyph>,
    ) {
        let font_data = run.font.data.data();
        let Ok(face) =
            ttf_parser::Face::parse(font_data, run.font.index)
        else {
            return;
        };

        let Some(scale_tf) = glyph_scale_tf(&face, run.font_size)
        else {
            return;
        };

        // One shared fill and stroke entry for the whole run.
        let fill_idx = fill.map(|f| {
            let idx = self.kanva.fills.len();
            self.kanva.fills.push(f);
            idx
        });
        let stroke_idx = stroke.map(|s| {
            let idx = self.kanva.strokes.len();
            self.kanva.strokes.push(s);
            idx
        });

        self.push_group_entry(Group::default());

        for glyph in glyphs {
            let Some((path, glyph_tf)) =
                outline_glyph(&face, glyph, run.transform, scale_tf)
            else {
                continue;
            };

            let path_idx = self.push_path(KanvaPath {
                path,
                transform: glyph_tf,
                fill: fill_idx,
                stroke: stroke_idx,
            });
            self.kanva.commands.push(Command::DrawPath(path_idx));
        }

        self.pop_group_entry();
    }
}

/// Returns the Y-flipped scale transform for a glyph run, or `None` if
/// `units_per_em` is zero.
fn glyph_scale_tf(
    face: &ttf_parser::Face<'_>,
    font_size: f32,
) -> Option<Affine> {
    let units_per_em = face.units_per_em();
    if units_per_em == 0 {
        return None;
    }
    let scale = font_size as f64 / units_per_em as f64;
    Some(Affine::scale_non_uniform(scale, -scale))
}

/// Outlines a single glyph and returns its path + world transform.
fn outline_glyph(
    face: &ttf_parser::Face<'_>,
    glyph: Glyph,
    base_transform: Affine,
    scale_tf: Affine,
) -> Option<(BezPath, Affine)> {
    let glyph_id = ttf_parser::GlyphId(glyph.id as u16);
    let mut pen = GlyphPen(BezPath::new());
    face.outline_glyph(glyph_id, &mut pen)?;
    let glyph_tf = base_transform
        * Affine::translate((glyph.x as f64, glyph.y as f64))
        * scale_tf;
    Some((pen.0, glyph_tf))
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
    use crate::sink::KanvaSink;
    use crate::{KanvaClip, KanvaFill, KanvaStroke, NodeIndex};
    use imaging::kurbo::{Affine, BezPath, Stroke};
    use imaging::peniko::{Brush, Fill, Style};

    fn draw_fill(b: &mut KanvaBuilder, brush: &Brush) {
        KanvaSink::draw_path(
            b,
            BezPath::new(),
            Affine::IDENTITY,
            Some(KanvaFill {
                rule: Fill::NonZero,
                brush: brush.clone(),
                ..Default::default()
            }),
            None,
        );
    }

    fn draw_stroke(b: &mut KanvaBuilder, stroke: &Stroke, brush: &Brush) {
        KanvaSink::draw_path(
            b,
            BezPath::new(),
            Affine::IDENTITY,
            None,
            Some(KanvaStroke {
                stroke: stroke.clone(),
                brush: brush.clone(),
                ..Default::default()
            }),
        );
    }

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
        draw_fill(&mut b, &Brush::default());
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
        draw_stroke(&mut b, &Stroke::default(), &Brush::default());
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
        let brush = Brush::default();
        KanvaSink::push_group(&mut b, Group::default());
        draw_fill(&mut b, &brush);
        KanvaSink::pop_group(&mut b);
        KanvaSink::push_group(&mut b, Group::default());
        draw_fill(&mut b, &brush);
        KanvaSink::pop_group(&mut b);
        let k = b.build();
        assert_eq!(k.group_cmds[0].start, 0, "first PushGroup at index 0");
        assert_eq!(k.group_cmds[0].end, 2, "first PopGroup at index 2");
        assert_eq!(k.group_cmds[1].start, 3, "second PushGroup at index 3");
        assert_eq!(k.group_cmds[1].end, 5, "second PopGroup at index 5");
        assert!(matches!(k.commands[2], Command::PopGroup));
        assert!(matches!(k.commands[5], Command::PopGroup));
    }

    #[test]
    fn group_ends_nested() {
        // commands: [PushGroup(0), PushGroup(1), DrawPath(0), PopGroup(idx=3), PopGroup(idx=4)]
        let mut b = KanvaBuilder::new();
        let brush = Brush::default();
        KanvaSink::push_group(&mut b, Group::default());
        KanvaSink::push_group(&mut b, Group::default());
        draw_fill(&mut b, &brush);
        KanvaSink::pop_group(&mut b);
        KanvaSink::pop_group(&mut b);
        let k = b.build();
        assert_eq!(k.group_cmds[0].start, 0, "outer PushGroup at index 0");
        assert_eq!(k.group_cmds[1].start, 1, "inner PushGroup at index 1");
        assert_eq!(k.group_cmds[1].end, 3, "inner PopGroup at index 3");
        assert_eq!(k.group_cmds[0].end, 4, "outer PopGroup at index 4");
        assert!(matches!(k.commands[3], Command::PopGroup));
        assert!(matches!(k.commands[4], Command::PopGroup));
    }

    #[test]
    fn get_group_returns_group() {
        let mut b = KanvaBuilder::new();
        KanvaSink::push_group(&mut b, Group::default());
        KanvaSink::pop_group(&mut b);
        let k = b.build();
        assert!(k.get_group(0).is_some());
        assert!(k.get_group(1).is_none());
    }

    #[test]
    fn get_group_path_range_returns_range() {
        let mut b = KanvaBuilder::new();
        let brush = Brush::default();
        KanvaSink::push_group(&mut b, Group::default());
        draw_fill(&mut b, &brush);
        draw_fill(&mut b, &brush);
        KanvaSink::pop_group(&mut b);
        let k = b.build();
        assert_eq!(k.get_group_path_range(0).unwrap().len(), 2);
    }

    #[test]
    fn label_indexes_path() {
        let mut b = KanvaBuilder::new();
        let brush = Brush::default();
        KanvaSink::push_context(&mut b, "foo");
        draw_fill(&mut b, &brush);
        KanvaSink::pop_context(&mut b);
        let k = b.build();
        assert_eq!(k.query("foo"), Some(NodeIndex::Path(0)));
    }

    #[test]
    fn label_indexes_group() {
        let mut b = KanvaBuilder::new();
        KanvaSink::push_context(&mut b, "bar");
        KanvaSink::push_group(&mut b, Group::default());
        KanvaSink::pop_group(&mut b);
        KanvaSink::pop_context(&mut b);
        let k = b.build();
        assert_eq!(k.query("bar"), Some(NodeIndex::Group(0)));
    }

    #[test]
    fn clip_stored_in_group() {
        let mut b = KanvaBuilder::new();
        KanvaSink::push_group(
            &mut b,
            Group {
                clip: Some(KanvaClip {
                    path: BezPath::new(),
                    transform: Affine::IDENTITY,
                    style: Style::Fill(Fill::NonZero),
                }),
                ..Default::default()
            },
        );
        KanvaSink::pop_group(&mut b);
        let k = b.build();
        assert!(k.groups[0].clip.is_some());
    }
}
