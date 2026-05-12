use alloc::vec::Vec;
use imaging::{Composite, Filter};
use peniko::kurbo::{BezPath, Stroke};

#[derive(Default, Debug, Clone)]
pub struct Layer {
    pub composite: Composite,
    pub clip: Option<KanvaClip>,
    pub filters: Vec<Filter>,
}

#[derive(Debug, Clone)]
pub struct KanvaClip {
    pub path: BezPath,
    pub stroke: Option<Stroke>,
}
