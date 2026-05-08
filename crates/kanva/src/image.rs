use peniko::{
    ImageData,
    kurbo::{Affine, Vec2},
};

#[derive(Debug, Clone)]
pub struct KanvaImage {
    pub data: ImageData,
    pub size: Vec2,
    pub transform: Affine,
}
