pub mod prelude {
    pub use typst::diag::{EcoString, SourceResult};
    pub use typst::foundations::{Content, Label as TypLabel, NativeElement, Packed, Smart};
    pub use typst::layout::{Abs, Em, Length, Ratio, Rel};
    pub use typst::{foundations, layout, math, model, text, visualize as viz};

    pub use crate::extensions::{ScopeError, ScopeExt, UnitExt};
    pub use crate::{elem, sequence};
}

pub mod elem;
pub mod extensions;
