use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{Data, DeriveInput, Ident};

// TODO: investigate whether the presence of #[repr(C)] should be required. Currently this can only
// be safely derived for structs for which all fields are 4 byte aligned, which may be enough to
// guarantee defined behaviour, but I've not yet explicitly verified that it is.

pub fn expand_derive_transform_feedback(input: &DeriveInput) -> TokenStream {
    if let Data::Struct(data) = &input.data {
        let struct_name = &input.ident;
        let mod_path = quote!(_web_glitz::pipeline::graphics);

        let recurse = data.fields.iter().map(|field| {
            let name = field.ident.clone().expect("`TransformFeedback` can only be derived for a struct with named fields.").to_string();
            let ty = &field.ty;
            let span = field.span();

            quote_spanned!(span=> {
                #mod_path::TransformFeedbackAttributeDescriptor {
                    ident: #mod_path::TransformFeedbackAttributeIdentifier::Static(#name),
                    attribute_type: <#ty as #mod_path::TransformFeedbackAttribute>::TYPE,
                    size: <#ty as #mod_path::TransformFeedbackAttribute>::SIZE
                }
            })
        });

        let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

        let impl_block = quote! {
            #[automatically_derived]
            unsafe impl #impl_generics #mod_path::TransformFeedback for #struct_name #ty_generics #where_clause {
                const ATTRIBUTE_DESCRIPTORS: &'static [#mod_path::TransformFeedbackAttributeDescriptor] =
                    &[
                        #(#recurse),*
                    ];
            }
        };

        let suffix = struct_name.to_string().trim_start_matches("r#").to_owned();
        let dummy_const = Ident::new(&format!("_IMPL_TRANSFORM_FEEDBACK_FOR_{}", suffix), Span::call_site());

        let generated = quote! {
            #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
            const #dummy_const: () = {
                #[allow(unknown_lints)]
                #[cfg_attr(feature = "cargo-clippy", allow(useless_attribute))]
                #[allow(rust_2018_idioms)]
                extern crate web_glitz as _web_glitz;

                #impl_block
            };
        };

        generated
    } else {
        panic!("`TransformFeedback` can only be derived for a struct.");
    }
}
