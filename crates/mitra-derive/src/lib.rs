//! `#[derive(Observe)]`: a value written as Mitra writes it, for the
//! rows `mitra::check_eq!` prints. A struct with named fields is
//! `Name { a: 1, b: true }`; a unit struct, `Name`; an enum's variant,
//! `V`, or `V(1, 2)` with its values. A tuple struct, or a variant with
//! named fields, has no Mitra form: a compile error says so.

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, parse_quote, Data, DeriveInput, Fields};

#[proc_macro_derive(Observe)]
pub fn derive_observe(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match observe(input) {
        Ok(tokens) => tokens.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

fn observe(mut input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let name = &input.ident;
    let shown = name.to_string();
    let body = match &input.data {
        Data::Struct(s) => match &s.fields {
            Fields::Named(fields) => {
                let names: Vec<_> = fields.named.iter().map(|f| f.ident.clone().unwrap()).collect();
                let labels: Vec<String> = names.iter().map(|n| n.to_string()).collect();
                quote! {
                    let fields: ::std::vec::Vec<::std::string::String> = ::std::vec![
                        #(::std::format!("{}: {}", #labels, ::mitra::Observe::observe(&self.#names))),*
                    ];
                    ::std::format!("{} {{ {} }}", #shown, fields.join(", "))
                }
            }
            Fields::Unit => quote! { ::std::string::String::from(#shown) },
            Fields::Unnamed(_) => {
                return Err(syn::Error::new_spanned(
                    &input.ident,
                    "Observe: a tuple struct has no Mitra form; name its fields, as a Mitra record does",
                ))
            }
        },
        Data::Enum(e) => {
            let mut arms = Vec::new();
            for v in &e.variants {
                let ident = &v.ident;
                let label = ident.to_string();
                arms.push(match &v.fields {
                    Fields::Unit => quote! { Self::#ident => ::std::string::String::from(#label) },
                    Fields::Unnamed(f) => {
                        let holds: Vec<_> = (0..f.unnamed.len()).map(|i| format_ident!("v{i}")).collect();
                        quote! {
                            Self::#ident(#(#holds),*) => {
                                let holds: ::std::vec::Vec<::std::string::String> = ::std::vec![
                                    #(::mitra::Observe::observe(#holds)),*
                                ];
                                ::std::format!("{}({})", #label, holds.join(", "))
                            }
                        }
                    }
                    Fields::Named(_) => {
                        return Err(syn::Error::new_spanned(
                            ident,
                            "Observe: a variant with named fields has no Mitra form; give it its values in order, `V(a, b)`",
                        ))
                    }
                });
            }
            quote! { match self { #(#arms),* } }
        }
        Data::Union(_) => {
            return Err(syn::Error::new_spanned(
                &input.ident,
                "Observe: a union has no Mitra form",
            ))
        }
    };
    let params: Vec<_> = input
        .generics
        .type_params()
        .map(|p| p.ident.clone())
        .collect();
    let clause = input.generics.make_where_clause();
    for p in params {
        clause.predicates.push(parse_quote!(#p: ::mitra::Observe));
    }
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();
    Ok(quote! {
        impl #impl_generics ::mitra::Observe for #name #type_generics #where_clause {
            fn observe(&self) -> ::std::string::String {
                #body
            }
        }
    })
}
