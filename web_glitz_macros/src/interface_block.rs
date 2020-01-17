use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned, ToTokens};
use syn::spanned::Spanned;
use syn::{Data, DeriveInput, Ident};

pub fn expand_derive_interface_block(input: &DeriveInput) -> Result<TokenStream, String> {
    if let Data::Struct(data) = &input.data {
        let mod_path = quote!(web_glitz::pipeline::interface_block);
        let struct_name = &input.ident;

        let recurse_len = data.fields.iter().map(|field| {
            let ty = &field.ty;
            let span = field.span();

            quote_spanned! {span=>
                <#ty as #mod_path::InterfaceBlockComponent>::MEMORY_UNITS.len()
            }
        });

        let recurse_array = data.fields.iter().enumerate().map(|(position, field)| {
            let ty = &field.ty;
            let ident = field
                .ident
                .clone()
                .map(|i| i.into_token_stream())
                .unwrap_or(position.into_token_stream());
            let span = field.span();

            quote_spanned! {span=>
                let base_offset = web_glitz::offset_of!(#struct_name, #ident);
                let memory_units = <#ty as #mod_path::InterfaceBlockComponent>::MEMORY_UNITS;
                let mut j = 0;

                while j < memory_units.len() {
                    let memory_unit = memory_units[j];

                    array[i] = #mod_path::MemoryUnit {
                        offset: base_offset + memory_unit.offset,
                        layout: memory_unit.layout
                    };

                    i += 1;
                    j += 1;
                }
            }
        });

        let suffix = struct_name.to_string().trim_start_matches("r#").to_owned();
        let dummy_const = Ident::new(
            &format!("_IMPL_INTERFACE_BLOCK_FOR_{}", suffix),
            Span::call_site(),
        );

        let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

        let impl_block = quote! {
            #[automatically_derived]
            unsafe impl #impl_generics #mod_path::InterfaceBlock for #struct_name #ty_generics #where_clause {
                const MEMORY_UNITS: &'static [#mod_path::MemoryUnit] = &{
                    const LEN: usize = #(#recurse_len)+*;

                    // Initialize array with temporary values;
                    let mut array = [#mod_path::MemoryUnit {
                        offset: 0,
                        layout: #mod_path::UnitLayout::Float
                    }; LEN];

                    let mut i = 0;

                    #(#recurse_array)*

                    array
                };
            }
        };

        let generated = quote! {
            #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
            const #dummy_const: () = {
                #[allow(unknown_lints)]
                #[cfg_attr(feature = "cargo-clippy", allow(useless_attribute))]
                #[allow(rust_2018_idioms)]

                #impl_block
            };
        };

        Ok(generated)
    } else {
        Err("`InterfaceBlock` can only be derived for a struct.".to_string())
    }
}
