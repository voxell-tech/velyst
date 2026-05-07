use peniko::{BlendMode, kurbo::{BezPath, Stroke}};

#[derive(Debug, Clone)]
pub struct Layer {
    pub blend_mode: BlendMode,
    pub alpha: f32,
    pub clip: Option<KanvaClip>,
}

impl Default for Layer {
    fn default() -> Self {
        Self {
            blend_mode: BlendMode::default(),
            alpha: 1.0,
            clip: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct KanvaClip {
    pub path: BezPath,
    pub stroke: Option<Stroke>,
}
