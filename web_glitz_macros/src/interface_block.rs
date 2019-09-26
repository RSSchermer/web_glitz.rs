use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned, ToTokens};
use syn::spanned::Spanned;
use syn::{Data, DeriveInput, Ident, Field};

pub fn expand_derive_interface_block(input: &DeriveInput) -> Result<TokenStream, String> {
    if let Data::Struct(data) = &input.data {
        let mod_path = quote!(_web_glitz::pipeline::interface_block);
        let struct_name = &input.ident;

        let chain = data.fields.iter().enumerate().map(|(position, field)| {
            let ty = &field.ty;
            let ident = field.ident.clone().map(|i| i.into_token_stream()).unwrap_or(position.into_token_stream());
            let span = field.span();

            quote_spanned! {span=>
                let offset = offset_of!(#struct_name, #ident);
                let memory_units = <#ty as #mod_path::InterfaceBlockComponent>::MEMORY_UNITS;
                let offset_memory_units = #mod_path::OffsetMemoryUnits::new(memory_units, offset);
                let chain = #mod_path::Chain::new(chain, offset_memory_units);
            }
        });

        let chain_type = {
            fn chain_recursive<'a, I>(mod_path: &TokenStream, init: TokenStream, iter: &mut I) -> TokenStream where I: Iterator<Item=&'a Field> {
                if let Some(field) = iter.next() {
                    let ty = &field.ty;
                    let res = quote! {
                        #mod_path::OffsetMemoryUnits<<#ty as #mod_path::InterfaceBlockComponent>::MemoryUnits>
                    };

                    let remainder = chain_recursive(mod_path, res, iter);

                    quote! {
                        #mod_path::Chain<#init, #remainder>
                    }
                } else {
                    init
                }
            }

            let mut iter = data.fields.iter();

            chain_recursive(&mod_path, quote! {
                std::iter::Empty<#mod_path::MemoryUnit>
            }, &mut iter)
        };

        let suffix = struct_name.to_string().trim_start_matches("r#").to_owned();
        let dummy_const = Ident::new(
            &format!("_IMPL_INTERFACE_BLOCK_FOR_{}", suffix),
            Span::call_site(),
        );

        let offset_of = quote! {
            macro_rules! offset_of {
                ($parent:path, $field:ident) => (unsafe {
                    let base_ptr = std::ptr::null::<$parent>();
                    let field_ptr = &raw const base_ptr.$field;

                    (field_ptr as usize).wrapping_sub(base_ptr as usize)
                });
            }
        };

        let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

        let impl_block = quote! {
            #[automatically_derived]
            unsafe impl #impl_generics #mod_path::InterfaceBlock for #struct_name #ty_generics #where_clause {
                type MemoryUnits = #chain_type;

                const MEMORY_UNITS: Self::MemoryUnits = {
                    let chain = std::iter::empty();

                    #(#chain)*

                    chain
                };
            }
        };

        let generated = quote! {
            #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
            const #dummy_const: () = {
                #[allow(unknown_lints)]
                #[cfg_attr(feature = "cargo-clippy", allow(useless_attribute))]
                #[allow(rust_2018_idioms)]
                extern crate web_glitz as _web_glitz;

                #offset_of

                #impl_block
            };
        };

        Ok(generated)
    } else {
        Err("`InterfaceBlock` can only be derived for a struct.".to_string())
    }
}
