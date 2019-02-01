use std::hash::{Hash, Hasher};

use fnv::FnvHasher;
use proc_macro::TokenStream;
use proc_macro2;
use quote::{quote, quote_spanned};
use syn::{Expr, Ident};
//use syn::parse::{Parse, ParseStream, Result as ParseResult};
use proc_macro2::Span;
use syn::parse::Parse;
use syn::parse::ParseStream;
use syn::parse::Result as ParseResult;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{parse_macro_input, Token};

struct Uniforms {
    fields: Punctuated<UniformFieldValue, Token![,]>,
}

impl Parse for Uniforms {
    fn parse(input: ParseStream<'_>) -> ParseResult<Self> {
        Ok(Uniforms {
            fields: input.parse_terminated(UniformFieldValue::parse)?,
        })
    }
}

struct UniformFieldValue {
    name: Ident,
    value: Expr,
}

impl Parse for UniformFieldValue {
    fn parse(input: ParseStream<'_>) -> ParseResult<Self> {
        let name = input.parse()?;

        input.parse::<Token![:]>()?;

        let value = input.parse()?;

        Ok(UniformFieldValue { name, value })
    }
}

pub fn expand_uniforms(tokens: TokenStream) -> TokenStream {
    let input = parse_macro_input!(tokens as Uniforms);
    let mod_path = quote!(_web_glitz::uniform);

    let generics = input
        .fields
        .iter()
        .enumerate()
        .map(|(index, _)| Ident::new(&format!("T{}", index), Span::call_site()));

    let generics = quote!(#(#generics),*);

    let constraints = input
        .fields
        .iter()
        .enumerate()
        .map(|(index, _)| {
            let type_ident = Ident::new(&format!("T{}", index), Span::call_site());

            quote!(#type_ident: #mod_path::Uniform)
        })
        .collect::<Vec<proc_macro2::TokenStream>>();

    let constraints = quote!(#(#constraints),*);

    let struct_fields = input.fields.iter().enumerate().map(|(index, field)| {
        let field_name = &field.name;
        let span = field_name.span();
        let type_ident = Ident::new(&format!("T{}", index), Span::call_site());

        quote_spanned!(span=> #field_name: #type_ident)
    });

    let struct_block = quote! {
        struct CustomUniforms<#generics> {
            #(#struct_fields),*
        }
    };

    let match_arms = input.fields.iter().map(|field| {
        let field_name = &field.name;
        let mut hasher = FnvHasher::default();

        field_name.to_string().hash(&mut hasher);

        let name_hash = hasher.finish();

        quote! {
            #name_hash => self.#field_name.bind(tail, slot)
        }
    });

    let impl_block = quote! {
        impl<#generics> #mod_path::Uniform for CustomUniforms<#generics> where #constraints {
            fn bind(
                &self,
                identifier: #mod_path::IdentifierTail,
                slot: &mut _web_glitz::uniform::BindingSlot
            ) -> Result<(), #mod_path::BindingError> {
                if let Some(#mod_path::IdentifierSegment::Name(segment)) = identifier.head() {
                    let tail = identifier.tail();

                    match segment.name_hash_fnv64() {
                        #(#match_arms,)*
                        _ => Err(#mod_path::BindingError::NotFound),
                    }
                } else {
                    Err(#mod_path::BindingError::NotFound)
                }
            }
        }
    };

    let assignments = input.fields.iter().map(|field| {
        let field_name = &field.name;
        let field_value = &field.value;

        quote!(let #field_name = #field_value;)
    });

    let assignment_block = quote!(#(#assignments)*);

    let asserts = input.fields.iter().map(|field| {
        let field_name = &field.name;
        let span = field.value.span();

        quote_spanned!(span=> assert_uniform(&#field_name);)
    });

    let assert_block = quote! {
        fn assert_uniform<T>(value: &T) where T: #mod_path::Uniform {}

        #(#asserts)*
    };

    let field_names = input.fields.iter().map(|field| &field.name);

    let instantiation_block = quote! {
        CustomUniforms {
            #(#field_names),*
        }
    };

    let output = quote! {
        {
            extern crate web_glitz as _web_glitz;

            #struct_block
            #impl_block
            #assignment_block
            #assert_block
            #instantiation_block
        }
    };

    output.into()
}
