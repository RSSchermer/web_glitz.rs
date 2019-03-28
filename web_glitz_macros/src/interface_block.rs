use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned, ToTokens};
use syn::spanned::Spanned;
use syn::{Attribute, Data, DeriveInput, Field, Ident, Lit, Meta, NestedMeta, Type};

use crate::util::ErrorLog;

pub fn expand_derive_interface_block(input: &DeriveInput) -> Result<TokenStream, String> {
    if let Data::Struct(data) = &input.data {
        let mod_path = quote!(_web_glitz::pipeline::interface_block);
        let struct_name = &input.ident;

        let recurse = data.fields.iter().enumerate().map(|(position, field)| {
            let ty = &field.ty;
            let ident = field.ident.clone().map(|i| i.into_token_stream()).unwrap_or(position.into_token_stream());
            let span = field.span();

            quote_spanned! {span=>
                let offset = offset_of!(#struct_name, #ident);

                match <#ty as #mod_path::InterfaceBlockComponent>::check_compatibility(offset, &mut remainder) {
                    #mod_path::CheckCompatibility::Finished => {
                        return Ok(());
                    },
                    #mod_path::CheckCompatibility::Incompatible(err) => {
                        return Err(err);
                    },
                    #mod_path::CheckCompatibility::Continue => ()
                }
            }
        });

        let suffix = struct_name.to_string().trim_left_matches("r#").to_owned();
        let dummy_const = Ident::new(
            &format!("_IMPL_INTERFACE_BLOCK_COMPONENT_FOR_{}", suffix),
            Span::call_site(),
        );

        // Modified from the memoffset crate (https://github.com/Gilnaa/memoffset)
        // TODO: replace with std::mem::offset_of when it becomes available
        let offset_of = quote! {
            macro_rules! offset_of {
                ($father:ty, $($field:tt)+) => ({
                    #[allow(unused_unsafe)]
                    let root: $father = unsafe { std::mem::uninitialized() };

                    let base = &root as *const _ as usize;

                    // Future error: borrow of packed field requires unsafe function or block (error E0133)
                    #[allow(unused_unsafe)]
                    let member =  unsafe { &root.$($field)* as *const _ as usize };

                    std::mem::forget(root);

                    member - base
                });
            }
        };

        let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

        let impl_block = quote! {
            #[automatically_derived]
            unsafe impl #impl_generics #mod_path::InterfaceBlock for #struct_name #ty_generics #where_clause {
                fn compatibility(memory_layout: &[#mod_path::MemoryUnitDescriptor]) -> Result<(), #mod_path::Incompatible> {
                    let mut remainder = memory_layout.iter();

                    #(#recurse)*

                    if let Some(unit) = remainder.next() {
                        Err(#mod_path::Incompatible::MissingUnit(unit.clone()))
                    } else {
                        Ok(())
                    }
                }
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
