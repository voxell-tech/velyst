use typst::{
    diag::EcoString,
    foundations::{Content, Label, Packed},
    layout,
    loading::Readable,
    model, text, visualize,
};

macro_rules! fn_elem_empty {
    ($fn_name:ident, $elem:ty) => {
        pub fn $fn_name() -> $elem {
            <$elem>::new()
        }
    };
}

macro_rules! fn_elem {
    ($fn_name:ident, $elem:ty, $($param:ident = $in_elem:ty),+) => {
        pub fn $fn_name($($param: impl Into<$in_elem>),+) -> $elem {
            <$elem>::new($($param.into()),+)
        }
    };

    ($fn_name:ident, $elem:ty) => {
        fn_elem!($fn_name, $elem, body = ::typst::foundations::Content);
    };

    ($fn_name:ident, $elem:ty, $in_elem:ty) => {
        fn_elem!($fn_name, $elem, body = $in_elem);
    };
}

// Layout
fn_elem!(page, layout::PageElem);
fn_elem_empty!(pagebreak, layout::PagebreakElem);
fn_elem!(vertical, layout::VElem, layout::Spacing);
fn_elem!(horizontal, layout::HElem, layout::Spacing);
fn_elem_empty!(boxed, layout::BoxElem);
fn_elem_empty!(block, layout::BlockElem);
fn_elem!(stack, layout::StackElem, Vec<layout::StackChild>);
fn_elem!(grid, layout::GridElem, Vec<layout::GridChild>);
fn_elem!(column, layout::ColumnsElem);
fn_elem_empty!(colbreak, layout::ColbreakElem);
fn_elem!(place, layout::PlaceElem);
fn_elem!(align, layout::AlignElem);
fn_elem!(pad, layout::PadElem);
fn_elem!(repeat, layout::RepeatElem);
fn_elem!(moved, layout::MoveElem);
fn_elem!(scale, layout::ScaleElem);
fn_elem!(rotate, layout::RotateElem);
fn_elem!(hide, layout::HideElem);

// Model
fn_elem!(doc, model::DocumentElem, Vec<Content>);
fn_elem!(reference, model::RefElem, Label);
fn_elem!(
    link,
    model::LinkElem,
    dest = model::LinkTarget,
    body = Content
);
fn_elem_empty!(outline, model::OutlineElem);
fn_elem!(heading, model::HeadingElem);
fn_elem!(figure, model::FigureElem);
fn_elem!(footnote, model::FootnoteElem, model::FootnoteBody);
fn_elem!(quote, model::QuoteElem);
fn_elem!(cite, model::CiteElem, Label);
fn_elem!(
    bibliography,
    model::BibliographyElem,
    paths = model::BibliographyPaths,
    bibliography = model::Bibliography
);
fn_elem!(numbered_list, model::EnumElem, Vec<Packed<model::EnumItem>>);
fn_elem!(bullet_list, model::ListElem, Vec<Packed<model::ListItem>>);
fn_elem_empty!(parbreak, model::ParbreakElem);
fn_elem!(par, model::ParElem, Vec<Content>);
fn_elem!(table, model::TableElem, Vec<model::TableChild>);
fn_elem!(terms, model::TermsElem, Vec<Packed<model::TermItem>>);
fn_elem!(emph, model::EmphElem);
fn_elem!(strong, model::StrongElem);

// Text
fn_elem!(text, text::TextElem, EcoString);
fn_elem_empty!(linebreak, text::LinebreakElem);
fn_elem_empty!(smart_quote, text::SmartQuoteElem);
fn_elem!(subscript, text::SubElem);
fn_elem!(superscript, text::SuperElem);
fn_elem!(underline, text::UnderlineElem);
fn_elem!(overline, text::OverlineElem);
fn_elem!(strike, text::StrikeElem);
fn_elem!(highlight, text::HighlightElem);
fn_elem!(raw, text::RawElem, text::RawContent);

#[macro_export]
macro_rules! sequence {
    ($($native_elem:expr),*,) => {
        ::typst::foundations::SequenceElem::new(vec![
            $(::typst::foundations::Content::from($native_elem),)*
        ])
    };
}

// Visualize

fn_elem!(
    image,
    visualize::ImageElem,
    path = EcoString,
    readable = Readable
);
fn_elem_empty!(line, visualize::LineElem);
fn_elem_empty!(rect, visualize::RectElem);
fn_elem_empty!(square, visualize::SquareElem);
fn_elem_empty!(ellipse, visualize::EllipseElem);
fn_elem_empty!(circle, visualize::CircleElem);
fn_elem!(
    polygon,
    visualize::PolygonElem,
    Vec<layout::Axes<layout::Rel<layout::Length>>>
);
fn_elem!(path, visualize::PathElem, Vec<visualize::PathVertex>);

pub fn solid(color: visualize::Color) -> visualize::Paint {
    visualize::Paint::Solid(color)
}

pub fn gradient(gradient: visualize::Gradient) -> visualize::Paint {
    visualize::Paint::Gradient(gradient)
}

pub fn pattern(pattern: visualize::Pattern) -> visualize::Paint {
    visualize::Paint::Pattern(pattern)
}

pub fn stroke(paint: visualize::Paint, thickness: layout::Length) -> visualize::Stroke {
    visualize::Stroke::from_pair(paint, thickness)
}
