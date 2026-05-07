use peniko::{Brush, Fill, kurbo::{Affine, BezPath, Stroke}};

#[derive(Debug, Clone)]
pub struct KanvaShape {
    pub path: BezPath,
    pub fill: Option<KanvaFill>,
    pub stroke: Option<KanvaStroke>,
    pub transform: Affine,
}

#[derive(Debug, Clone)]
pub struct KanvaFill {
    pub style: Fill,
    pub brush: Brush,
    pub transform: Option<Affine>,
}

#[derive(Debug, Clone)]
pub struct KanvaStroke {
    pub style: Stroke,
    pub brush: Brush,
    pub transform: Option<Affine>,
}
