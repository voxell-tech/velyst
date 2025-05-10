use darling::ast::Data;
use darling::util::{Ignored, Override};
use darling::{Error, FromDeriveInput, FromField};
use proc_macro::TokenStream;
use quote::{ToTokens, quote};
use syn::{DeriveInput, Ident, Index, parse_macro_input};

#[derive(FromDeriveInput)]
#[darling(forward_attrs(typst_path))]
struct TypstPathArgs {
    ident: Ident,
    #[darling(with = "convert_typst_path")]
    attrs: TypstPathValue,
}

struct TypstPathValue(syn::Expr);

fn convert_typst_path(
    attrs: Vec<syn::Attribute>,
) -> darling::Result<TypstPathValue> {
    let mut filtered = attrs
        .into_iter()
        .filter(|a| a.path().is_ident("typst_path"));

    let attr = filtered
        .next()
        .ok_or(darling::Error::custom("#[typst_path] is required"))?;

    if filtered.next().is_some() {
        return Err(darling::Error::custom(
            "#[typst_path] can only occur once",
        ));
    }

    let val = attr.meta.require_name_value()?.value.clone();

    Ok(TypstPathValue(val))
}

/// Macro for deriving `TypstPath`.
///
/// Usage
/// ```rust
/// # use velyst_macros::TypstPath;
/// #[derive(TypstPath)]
/// #[typst_path = "path_to_file.typ"] // Path relative to bevy asset dir
/// struct MyTypstFile;
/// ```
#[proc_macro_derive(TypstPath, attributes(typst_path))]
pub fn derive_typst_path(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let TypstPathArgs {
        ident,
        attrs: TypstPathValue(path),
    } = match TypstPathArgs::from_derive_input(&ast) {
        Ok(t) => t,
        Err(e) => return e.write_errors().into(),
    };

    quote! {
        impl ::velyst::renderer::TypstPath for #ident {
            fn path() -> &'static str {
                #path
            }
        }
    }
    .into()
}

#[derive(FromDeriveInput)]
#[darling(supports(struct_any), attributes(typst_func))]
struct TypstFuncArgs {
    ident: Ident,
    data: Data<Ignored, TypstFuncField>,
    name: String,
    #[darling(default)]
    layer: usize,
}

#[derive(FromField)]
#[darling(attributes(typst_func))]
struct TypstFuncField {
    ident: Option<Ident>,
    named: Option<Override<String>>,
}

impl TypstFuncField {
    fn named(self) -> Option<darling::Result<String>> {
        self.named.map(|ov| {
            ov.explicit().map(Ok).unwrap_or_else(|| {
                self.ident
                    .map(|i| i.to_string())
                    .ok_or(darling::Error::custom(
                        "#[typst_func(named)] without a value is only supported on named fields",
                    ))
            })
        })
    }
}

/// Macro for deriving `TypstFunc`.
///
/// Usage
/// ```rust
/// # use velyst_macros::TypstFunc;
/// # use bevy::prelude::*;
/// #[derive(TypstFunc, Resource)]
/// #[typst_func(name = "main", layer = 0)] // layer is optional
/// struct MainFunc {
///     width: f64,
///     height: f64,
///     #[typst_func(named)] // use as a named argument
///     animate: f64,
///     #[typst_func(named = "bar")] // use as a named argument with the name "bar"
///     foo: String,
/// }
/// ```
#[proc_macro_derive(TypstFunc, attributes(typst_func))]
pub fn derive_typst_func(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let TypstFuncArgs {
        ident,
        data,
        name,
        layer,
    } = match TypstFuncArgs::from_derive_input(&ast) {
        Ok(t) => t,
        Err(e) => return e.write_errors().into(),
    };

    let fields = data.take_struct().expect("Can never be an enum");

    let mut errors = Error::accumulator();

    let field_tokens = fields
        .into_iter()
        .enumerate()
        .filter_map(|(i, field)| {
            errors.handle_in(|| {
                let ident = field
                    .ident
                    .as_ref()
                    .map(|f| f.to_token_stream())
                    .unwrap_or(Index::from(i).to_token_stream());

                let field_token = match field.named() {
                    Some(named) => {
                        let name = named?;
                        quote! {
                            args.push_named(#name, self.#ident.clone());
                        }
                    }
                    None => quote! {
                        args.push(self.#ident.clone());
                    },
                };

                Ok(field_token)
            })
        })
        .collect::<Vec<_>>();

    if let Err(e) = errors.finish() {
        return e.write_errors().into();
    }

    // velyst paths
    let foundations = quote!(::velyst::typst::foundations);
    let elem = quote!(::velyst::typst_element::elem);
    // bevy paths
    let view = quote!(::bevy::render::view);

    quote! {
        impl ::velyst::renderer::TypstFunc for #ident {
            fn func_name(&self) -> &str {
                #name
            }

            fn content(&self, func: #foundations::Func) -> #foundations::Content {
                #foundations::NativeElement::pack(
                    #elem::context(func, |args| {
                        #(#field_tokens)*
                    })
                )
            }

            fn render_layers(&self) -> #view::RenderLayers {
                #view::RenderLayers::layer(#layer)
            }

        }
    }
    .into()
}
