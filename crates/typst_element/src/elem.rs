use typst::diag::EcoString;
use typst::foundations::{self, Bytes, Content, Derived, IntoValue, Label, Packed};
use typst::loading::DataSource;
use typst::syntax::{Span, Spanned};
use typst::{layout, math, model, text, visualize};
use unicode_math_class::MathClass;

macro_rules! fn_elem_empty {
    ($fn_name:ident, $elem:ty) => {
        #[doc = concat!("[", stringify!($elem), "]")]
        pub fn $fn_name() -> $elem {
            <$elem>::new()
        }
    };
}

macro_rules! fn_elem {
    ($fn_name:ident, $elem:ty, $($param:ident = $in_elem:ty),+) => {
        #[doc = concat!("[", stringify!($elem), "]")]
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

// Foundations
/// [foundations::SequenceElem]
#[macro_export]
macro_rules! sequence {
    ($($native_elem:expr),*,) => {
        ::typst::foundations::SequenceElem::new(vec![
            $(::typst::foundations::Content::from($native_elem),)*
        ])
    };
}

/// [foundations::ContextElem]
pub fn context(
    func: foundations::Func,
    apply_args: impl Fn(&mut SpannedArgs),
) -> foundations::ContextElem {
    let mut spanned_args = SpannedArgs::new(func.span());
    apply_args(&mut spanned_args);
    foundations::ContextElem::new(func.with(&mut spanned_args.args))
}

pub struct SpannedArgs {
    span: Span,
    args: foundations::Args,
}

impl SpannedArgs {
    pub fn new(span: Span) -> Self {
        Self {
            span,
            args: foundations::Args {
                span,
                items: [].into(),
            },
        }
    }

    pub fn push(&mut self, value: impl IntoValue) {
        self.args.push(self.span, value.into_value());
    }

    pub fn push_named(&mut self, name: &str, value: impl IntoValue) {
        self.args.items.push(foundations::Arg {
            span: self.span,
            name: Some(name.into()),
            value: Spanned::new(value.into_value(), self.span),
        })
    }
}

// Layout
fn_elem_empty!(page, layout::PageElem);
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
fn_elem_empty!(doc, model::DocumentElem);
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
// fn_elem!(
//     bibliography,
//     model::BibliographyElem,
//     paths = model::BibliographyPaths,
//     bibliography = model::Bibliography
// );
fn_elem!(numbered_list, model::EnumElem, Vec<Packed<model::EnumItem>>);
fn_elem!(bullet_list, model::ListElem, Vec<Packed<model::ListItem>>);
fn_elem_empty!(parbreak, model::ParbreakElem);
fn_elem!(par, model::ParElem, Content);
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

// Symbols
pub fn symbol(c: char) -> foundations::Symbol {
    foundations::Symbol::single(c)
}

// Math
fn_elem!(equation, math::EquationElem);
fn_elem!(lr, math::LrElem);
fn_elem!(mid, math::MidElem);
fn_elem!(attach, math::AttachElem);
fn_elem!(scripts, math::ScriptsElem);
fn_elem!(limits, math::LimitsElem);
fn_elem!(
    accent,
    math::AccentElem,
    body = Content,
    accent = math::Accent
);
fn_elem!(math_underline, math::UnderlineElem);
fn_elem!(math_overline, math::OverlineElem);
fn_elem!(underbrace, math::UnderbraceElem);
fn_elem!(overbrace, math::OverbraceElem);
fn_elem!(underbracket, math::UnderbracketElem);
fn_elem!(overbracket, math::OverbracketElem);
fn_elem!(cancel, math::CancelElem);
fn_elem!(frac, math::FracElem, num = Content, denom = Content);
fn_elem!(binom, math::BinomElem, upper = Content, lower = Vec<Content>);
fn_elem!(vec, math::VecElem, Vec<Content>);
fn_elem!(mat, math::MatElem, Vec<Vec<Content>>);
fn_elem!(cases, math::CasesElem, Vec<Content>);
fn_elem!(root, math::RootElem);
fn_elem!(class, math::ClassElem, class = MathClass, body = Content);
fn_elem!(op, math::OpElem);
fn_elem!(primes, math::PrimesElem, usize);

// Visualize
fn_elem!(
    image,
    visualize::ImageElem,
    source = Derived<DataSource, Bytes>
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

/// [visualize::Paint::Solid]
pub fn solid(color: impl Into<visualize::Color>) -> visualize::Paint {
    visualize::Paint::Solid(color.into())
}

/// [visualize::Paint::Gradient]
pub fn gradient(gradient: impl Into<visualize::Gradient>) -> visualize::Paint {
    visualize::Paint::Gradient(gradient.into())
}

/// [visualize::Paint::Tiling]
pub fn tiling(tiling: impl Into<visualize::Tiling>) -> visualize::Paint {
    visualize::Paint::Tiling(tiling.into())
}

/// [visualize::Stroke]
pub fn stroke(paint: visualize::Paint, thickness: layout::Length) -> visualize::Stroke {
    visualize::Stroke::from_pair(paint, thickness)
}
