use crate::doc::DocExpr;
use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, parse_macro_input};

pub fn derive_as_document(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let (impl_generics, ty_generics, _where_clause) = input.generics.split_for_impl();

    let mut custom_doc = None;
    for attr in &input.attrs {
        if attr.path().is_ident("oak") {
            let _ = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("doc") {
                    let value = meta.value()?;
                    custom_doc = Some(value.parse::<DocExpr>()?);
                }
                Ok(())
            });
        }
    }

    let body = match &input.data {
        Data::Struct(data) => {
            if let Some(expr) = custom_doc {
                quote! {
                    let _self = self;
                    let _config = config;
                    #expr
                }
            }
            else if data.fields.len() == 1 {
                let field = data.fields.iter().next().unwrap();
                let field_name = if let Some(ident) = &field.ident {
                    quote! { self.#ident }
                }
                else {
                    quote! { self.0 }
                };
                quote! {
                    {
                        #[allow(unused_imports)]
                        use ::oak_pretty_print::AsDocument;
                        #field_name.as_document(&())
                    }
                }
            }
            else {
                quote! { ::oak_pretty_print::Doc::Nil }
            }
        }
        Data::Enum(data) => {
            let arms = data.variants.iter().map(|variant| {
                let variant_name = &variant.ident;

                // Check for variant-level attributes
                let mut variant_doc = None;
                for attr in &variant.attrs {
                    if attr.path().is_ident("oak") {
                        let _ = attr.parse_nested_meta(|meta| {
                            if meta.path.is_ident("doc") {
                                let value = meta.value()?;
                                variant_doc = Some(value.parse::<DocExpr>()?);
                            }
                            Ok(())
                        });
                    }
                }

                if let Some(expr) = variant_doc {
                    if variant.fields.len() == 1 {
                        let field = variant.fields.iter().next().unwrap();
                        if let Some(ident) = &field.ident {
                            quote! {
                                #name::#variant_name { #ident, .. } => {
                                    let _self = #ident;
                                    let _config = config;
                                    #expr
                                },
                            }
                        }
                        else {
                            quote! {
                                #name::#variant_name(_v) => {
                                    let _self = _v;
                                    let _config = config;
                                    #expr
                                },
                            }
                        }
                    }
                    else {
                        quote! {
                            #name::#variant_name { .. } => {
                                let _config = config;
                                #expr
                            },
                        }
                    }
                }
                else if variant.fields.len() == 1 {
                    quote! {
                        #name::#variant_name(v) => {
                            #[allow(unused_imports)]
                            use ::oak_pretty_print::AsDocument;
                            v.as_document(&())
                        },
                    }
                }
                else if variant.fields.is_empty() {
                    quote! {
                        #name::#variant_name => ::oak_pretty_print::Doc::Nil,
                    }
                }
                else {
                    quote! {
                        #name::#variant_name(..) => ::oak_pretty_print::Doc::Nil,
                    }
                }
            });
            quote! {
                match self {
                    #(#arms)*
                }
            }
        }
        _ => panic!("AsDocument only supports structs and enums"),
    };

    let expanded = quote! {
        impl #impl_generics ::oak_pretty_print::AsDocument for #name #ty_generics {
            fn as_document(&self, config: &Self::Config) -> ::oak_pretty_print::Doc<'_> {
                #body
            }
        }
    };

    TokenStream::from(expanded)
}
