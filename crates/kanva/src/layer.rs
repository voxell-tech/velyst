use peniko::BlendMode;
use peniko::kurbo::{BezPath, Stroke};

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

#[cfg(test)]
mod tests {
    use peniko::kurbo::{BezPath, Rect, Shape as _, Stroke};

    use super::{KanvaClip, Layer};

    #[test]
    fn layer_default_alpha_is_one() {
        let layer = Layer::default();
        assert_eq!(layer.alpha, 1.0);
    }

    #[test]
    fn layer_default_has_no_clip() {
        let layer = Layer::default();
        assert!(layer.clip.is_none());
    }

    #[test]
    fn layer_default_blend_mode_matches_peniko_default() {
        let layer = Layer::default();
        assert_eq!(layer.blend_mode, peniko::BlendMode::default());
    }

    #[test]
    fn layer_clone_preserves_alpha() {
        let mut layer = Layer::default();
        layer.alpha = 0.5;
        let cloned = layer.clone();
        assert_eq!(cloned.alpha, 0.5);
    }

    #[test]
    fn layer_with_clip_stores_path() {
        let path = Rect::new(0.0, 0.0, 10.0, 10.0).to_path(0.1);
        let clip = KanvaClip {
            path: path.clone(),
            stroke: None,
        };
        let layer = Layer {
            clip: Some(clip),
            ..Layer::default()
        };
        assert!(layer.clip.is_some());
        assert!(layer.clip.as_ref().unwrap().stroke.is_none());
    }

    #[test]
    fn kanva_clip_with_stroke() {
        let path = BezPath::new();
        let stroke = Stroke::new(2.0);
        let clip = KanvaClip {
            path,
            stroke: Some(stroke.clone()),
        };
        assert!(clip.stroke.is_some());
        assert_eq!(clip.stroke.as_ref().unwrap().width, stroke.width);
    }

    #[test]
    fn kanva_clip_without_stroke_is_fill_clip() {
        let path = BezPath::new();
        let clip = KanvaClip { path, stroke: None };
        assert!(clip.stroke.is_none());
    }
}
