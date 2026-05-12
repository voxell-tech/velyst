use hashbrown::HashMap;

use crate::KanvaNodeStyle;
use crate::blur::KanvaBlurredRect;
use crate::shape::KanvaShape;
use crate::text::KanvaGlyphRun;

#[derive(Default)]
pub struct KanvaOverrides {
    pub nodes: HashMap<usize, KanvaNodeStyle>,
    pub shapes: HashMap<usize, KanvaShape>,
    pub glyph_runs: HashMap<usize, KanvaGlyphRun>,
    pub blurred_rects: HashMap<usize, KanvaBlurredRect>,
}
