use hashbrown::HashMap;
use imaging::Composite;
use imaging::kurbo::{Affine, BezPath};

use crate::node::{KanvaClip, KanvaFill, KanvaStroke};

/// Cursor returned by [`crate::Kanva::mod_path`].
///
/// Holds a mutable reference to [`PathMods`] and a fixed path index.
/// Chain calls to set multiple field overrides at once.
pub struct PathModEntry<'a> {
    mods: &'a mut PathMods,
    idx: usize,
}

impl<'a> PathModEntry<'a> {
    pub(crate) fn new(mods: &'a mut PathMods, idx: usize) -> Self {
        Self { mods, idx }
    }

    pub fn shape(self, shape: BezPath) -> Self {
        self.mods.shape.insert(self.idx, shape);
        self
    }

    pub fn transform(self, transform: Affine) -> Self {
        self.mods.transform.insert(self.idx, transform);
        self
    }

    /// Pass `None` to clear the fill.
    pub fn fill(self, fill: Option<KanvaFill>) -> Self {
        self.mods.fill.insert(self.idx, fill);
        self
    }

    /// Pass `None` to clear the stroke.
    pub fn stroke(self, stroke: Option<KanvaStroke>) -> Self {
        self.mods.stroke.insert(self.idx, stroke);
        self
    }

    pub fn alpha(self, alpha: f32) -> Self {
        self.mods.alpha.insert(self.idx, alpha);
        self
    }
}

/// Cursor returned by [`crate::Kanva::mod_group`].
///
/// Holds a mutable reference to [`GroupMods`] and a fixed group
/// index. Chain calls to set multiple field overrides at once.
pub struct GroupModEntry<'a> {
    mods: &'a mut GroupMods,
    idx: usize,
}

impl<'a> GroupModEntry<'a> {
    pub(crate) fn new(mods: &'a mut GroupMods, idx: usize) -> Self {
        Self { mods, idx }
    }

    pub fn transform(self, transform: Affine) -> Self {
        self.mods.transform.insert(self.idx, transform);
        self
    }

    /// Pass `None` to clear the clip.
    pub fn clip(self, clip: Option<KanvaClip>) -> Self {
        self.mods.clip.insert(self.idx, clip);
        self
    }

    pub fn composite(self, composite: Composite) -> Self {
        self.mods.composite.insert(self.idx, composite);
        self
    }
}

/// Per-path field overrides, keyed by path index.
///
/// Each field has its own map. Absent entry = keep stored value.
/// For optional-target fields (`fill`, `stroke`), `None` clears the
/// field.
///
/// Methods return `&mut Self` for chaining.
#[derive(Default, Debug, Clone)]
pub struct PathMods {
    pub(crate) shape: HashMap<usize, BezPath>,
    pub(crate) transform: HashMap<usize, Affine>,
    pub(crate) fill: HashMap<usize, Option<KanvaFill>>,
    pub(crate) stroke: HashMap<usize, Option<KanvaStroke>>,
    pub(crate) alpha: HashMap<usize, f32>,
}

impl PathMods {
    pub fn new() -> Self {
        Self::default()
    }

    /// Override the geometry of the path at `idx`.
    pub fn shape(&mut self, idx: usize, shape: BezPath) -> &mut Self {
        self.shape.insert(idx, shape);
        self
    }

    /// Override the transform of the path at `idx`.
    pub fn transform(
        &mut self,
        idx: usize,
        transform: Affine,
    ) -> &mut Self {
        self.transform.insert(idx, transform);
        self
    }

    /// Override the fill of the path at `idx`.
    ///
    /// Pass `None` to clear the fill entirely.
    pub fn fill(
        &mut self,
        idx: usize,
        fill: Option<KanvaFill>,
    ) -> &mut Self {
        self.fill.insert(idx, fill);
        self
    }

    /// Override the stroke of the path at `idx`.
    ///
    /// Pass `None` to clear the stroke entirely.
    pub fn stroke(
        &mut self,
        idx: usize,
        stroke: Option<KanvaStroke>,
    ) -> &mut Self {
        self.stroke.insert(idx, stroke);
        self
    }

    /// Wrap the path at `idx` in an isolated group with this alpha.
    pub fn alpha(&mut self, idx: usize, alpha: f32) -> &mut Self {
        self.alpha.insert(idx, alpha);
        self
    }

    /// Clear all path overrides.
    pub fn clear(&mut self) {
        self.shape.clear();
        self.transform.clear();
        self.fill.clear();
        self.stroke.clear();
        self.alpha.clear();
    }
}

/// Per-group field overrides, keyed by group index.
///
/// Each field has its own map. Absent entry = keep stored value.
/// For `clip`, `None` clears the clip entirely.
///
/// Methods return `&mut Self` for chaining.
#[derive(Default, Debug, Clone)]
pub struct GroupMods {
    pub(crate) transform: HashMap<usize, Affine>,
    pub(crate) clip: HashMap<usize, Option<KanvaClip>>,
    pub(crate) composite: HashMap<usize, Composite>,
}

impl GroupMods {
    pub fn new() -> Self {
        Self::default()
    }

    /// Override the transform of the group at `idx`.
    pub fn transform(
        &mut self,
        idx: usize,
        transform: Affine,
    ) -> &mut Self {
        self.transform.insert(idx, transform);
        self
    }

    /// Override the clip of the group at `idx`.
    ///
    /// Pass `None` to clear the clip entirely.
    pub fn clip(
        &mut self,
        idx: usize,
        clip: Option<KanvaClip>,
    ) -> &mut Self {
        self.clip.insert(idx, clip);
        self
    }

    /// Override the composite mode of the group at `idx`.
    pub fn composite(
        &mut self,
        idx: usize,
        composite: Composite,
    ) -> &mut Self {
        self.composite.insert(idx, composite);
        self
    }

    /// Clear all group overrides.
    pub fn clear(&mut self) {
        self.transform.clear();
        self.clip.clear();
        self.composite.clear();
    }
}
