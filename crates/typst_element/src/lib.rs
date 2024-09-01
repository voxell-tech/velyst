use foundations::{SequenceElem, Style};
use prelude::*;

pub mod prelude {
    pub use typst::{
        diag::EcoString,
        foundations::{self, Content, Label as TypLabel, NativeElement, Packed, Smart},
        layout::{self, Abs, Em, Length, Ratio, Rel},
        math, model, text, visualize as viz,
    };

    pub use crate::extensions::{ScopeExt, UnitExt};
    pub use crate::{elem::*, sequence};
    pub use crate::{DocWriter, SimpleWriter};
}

pub mod elem;
pub mod extensions;

pub trait DocWriter: Sized {
    /// A immutable reference to all contents within the writer.
    fn contents(&self) -> &Vec<Content>;
    /// A mutable reference to all contents within the writer.
    fn contents_mut(&mut self) -> &mut Vec<Content>;
    /// Moves all contents out from the writer and drop the writer.
    fn take_contents(self) -> Vec<Content>;

    /// Pack all contents within the writer into a single [`Content`].
    fn pack(self) -> Content {
        SequenceElem::new(self.take_contents()).pack()
    }

    /// Adds a new [`Content`] into the writer while returning
    /// a [`ContentMut`], mutably referencing the newly added content.
    fn add_content(&mut self, content: impl Into<Content>) -> ContentMut {
        self.contents_mut().push(content.into());
        ContentMut(self.contents_mut().last_mut().unwrap())
    }
}

#[derive(Default, Clone)]
pub struct SimpleWriter(pub Vec<Content>);

impl DocWriter for SimpleWriter {
    fn contents(&self) -> &Vec<Content> {
        &self.0
    }

    fn contents_mut(&mut self) -> &mut Vec<Content> {
        &mut self.0
    }

    fn take_contents(self) -> Vec<Content> {
        self.0
    }
}

impl SimpleWriter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn blank_page(&mut self, writer: impl FnOnce(&mut Self)) -> ContentMut {
        let mut doc = Self::default();
        writer(&mut doc);

        let page_elem = page(doc.pack())
            .with_width(Smart::Auto)
            .with_height(Smart::Auto)
            .with_margin(layout::Margin::splat(Some(Abs::zero().smart_rel())))
            .with_fill(Smart::Custom(None));

        self.add_content(page_elem.pack())
    }
}

pub struct ContentMut<'a>(&'a mut Content);

impl<'a> ContentMut<'a> {
    pub fn style(self, style: impl Into<Style>) -> Self {
        let content_value = std::mem::take(self.0);
        *self.0 = content_value.styled(style);
        self
    }

    pub fn as_content(self) -> &'a mut Content {
        self.0
    }
}
