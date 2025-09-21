#![doc = include_str!("../README.md")]

use proc_macro::TokenStream;
use proc_macro2 as pm2;
use quote::{format_ident, quote, quote_spanned};
use syn::{Fields, ItemEnum, parse::Parse, punctuated::Punctuated, spanned::Spanned, token::Comma};

struct PunctedNamedFields(Punctuated<syn::Field, Comma>);
struct PunctedUnnamedFields(Punctuated<syn::Field, Comma>);

impl std::ops::Deref for PunctedNamedFields {
    type Target = Punctuated<syn::Field, Comma>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Parse for PunctedNamedFields {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse_terminated(syn::Field::parse_named, Comma)
            .map(Self)
    }
}

impl Parse for PunctedUnnamedFields {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse_terminated(syn::Field::parse_unnamed, Comma)
            .map(Self)
    }
}

/// Insert fields for each variant, and generate getter for each fields
///
/// # Example
///
/// ```
/// #[enum_struct::fields {
///     id: u64,
/// }]
/// #[derive(Debug, PartialEq)]
/// enum Foo {
///     Named(String),
///     Complex { name: String, age: u32, level: u16 },
///     Empty,
/// }
///
/// let named = Foo::Named(2, "jack".into());
/// let complex = Foo::Complex { id: 3, name: "john".into(), age: 22, level: 4 };
/// let empty = Foo::Empty { id: 4 };
///
/// assert_eq!(named.id(), &2);
/// assert_eq!(complex.id(), &3);
/// assert_eq!(empty.id(), &4);
///
/// let mut named = named;
///
/// *named.id_mut() = 8;
/// assert_eq!(named.id(), &8);
/// assert_eq!(named, Foo::Named(8, "jack".into()));
/// ```
#[proc_macro_attribute]
pub fn fields(attr: TokenStream, adt: TokenStream) -> TokenStream {
    let mut item_enum = match syn::parse::<ItemEnum>(adt) {
        Ok(x) => x,
        Err(e) => return e.into_compile_error().into(),
    };
    let fields = match syn::parse::<PunctedNamedFields>(attr.clone()) {
        Ok(it) => it,
        Err(err) => return err.into_compile_error().into(),
    };
    item_enum.variants.iter_mut().for_each(|variant| {
        add_fields(&mut variant.fields, &fields);
    });

    let ItemEnum {
        attrs,
        vis,
        enum_token,
        ident,
        generics,
        brace_token: _,
        variants,
    } = item_enum;

    let (impl_generics,
         type_generics,
         where_clause) = generics.split_for_impl();

    let methods = generate_methods(&vis, &fields, &variants);

    quote! {
        #(#attrs)*
        #vis #enum_token #ident #generics {
            #variants
        }
        impl #impl_generics #ident #type_generics #where_clause {
            #(#methods)*
        }
    }.into()
}

fn generate_methods(
    vis: &syn::Visibility,
    fields: &PunctedNamedFields,
    variants: &Punctuated<syn::Variant, Comma>,
) -> Vec<pm2::TokenStream> {
    fields.pairs()
        .map(|pair| pair.into_value())
        .enumerate()
        .map(|(i, field)| {
            let i_field = pm2::Literal::usize_unsuffixed(i);
            let name = field.ident.as_ref().expect("empty field");
            let colon = field.colon_token.as_ref().expect("empty colon token");
            let ty = &field.ty;

            let attrs = field.attrs.iter()
                .filter(allowed_field_attr)
                .collect::<Vec<_>>();

            let field_name = lose_span(name);
            let method_span = colon.span.span();

            let immutable_getter = format_ident!("{field_name}", span = method_span);
            let mutable_getter = format_ident!("{field_name}_mut", span = method_span);
            let owned_getter = format_ident!("into_{field_name}", span = method_span);

            let variants_pat = variants.iter()
                .map(|it| {
                    let body = match it.fields {
                        Fields::Named(_) => quote! {
                            { #field_name, .. }
                        },
                        Fields::Unnamed(_) => quote! {
                            { #i_field: #field_name, .. }
                        },
                        Fields::Unit => quote! {},
                    };
                    let variant_name = lose_span(&it.ident);
                    quote! {
                        Self::#variant_name #body
                    }
                })
                .collect::<Vec<_>>();
            let match_arms = if variants_pat.is_empty() {
                quote! {
                    _ => loop {}
                }
            } else {
                quote! {
                    #(| #variants_pat)*
                    => #field_name,
                }
            };

            quote! {
                #(#attrs)*
                #[allow(unused)]
                #vis fn #immutable_getter(&self) -> &#ty {
                    match self {
                        #match_arms
                    }
                }
                #(#attrs)*
                #[allow(unused)]
                #vis fn #mutable_getter(&mut self) -> &mut #ty {
                    match self {
                        #match_arms
                    }
                }
                #(#attrs)*
                #[allow(unused)]
                #vis fn #owned_getter(self) -> #ty {
                    match self {
                        #match_arms
                    }
                }
            }
        })
        .collect()
}

fn allowed_field_attr(attr: &&syn::Attribute) -> bool {
    attr.path().is_ident("doc") && attr.meta.require_name_value().is_ok()
        || attr.path().is_ident("cfg") && attr.meta.require_list().is_ok()
}

fn lose_span(ident: &pm2::Ident) -> pm2::Ident {
    pm2::Ident::new(&ident.to_string(), pm2::Span::call_site())
}

fn add_fields(variant_fields: &mut Fields, fields: &PunctedNamedFields) {
    let needs_comma = !fields.trailing_punct() && !fields.is_empty();
    match variant_fields {
        Fields::Unit => {
            let mut tokens = pm2::Group::new(pm2::Delimiter::Brace, pm2::TokenStream::new());
            tokens.set_span(variant_fields.span());
            *variant_fields = Fields::Named(syn::parse2(pm2::TokenTree::from(tokens).into()).unwrap());
            add_fields(variant_fields, fields)
        },
        Fields::Named(syn::FieldsNamed { named, .. }) => {
            let fields_iter = fields.pairs();
            let tokens = if needs_comma {
                quote_spanned! { fields.span() => #(#fields_iter)* , #named }
            } else {
                quote_spanned! { fields.span() => #(#fields_iter)*   #named }
            };
            *named = syn::parse2::<PunctedNamedFields>(tokens).unwrap().0;
        },
        Fields::Unnamed(syn::FieldsUnnamed { unnamed, .. }) => {
            let fields_iter = fields.0.clone().into_pairs().map(|mut pair| {
                pair.value_mut().ident.take();
                pair.value_mut().colon_token.take();
                pair
            });
            let tokens = if needs_comma {
                quote_spanned! { fields.span() => #(#fields_iter)* , #unnamed }
            } else {
                quote_spanned! { fields.span() => #(#fields_iter)*   #unnamed }
            };
            *unnamed = syn::parse2::<PunctedUnnamedFields>(tokens).unwrap().0;
        },
    }
}
