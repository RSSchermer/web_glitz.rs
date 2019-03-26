use syn::{DeriveInput, Ident, Data};
use syn::spanned::Spanned;
use proc_macro2::{TokenStream, Span};
use quote::{quote, quote_spanned, ToTokens};

use crate::util::ErrorLog;

pub fn expand_repr_std140(input: &DeriveInput) -> Result<TokenStream, String> {
    if let Data::Struct(data) = &input.data {
        if has_other_repr(input) {
            return Err("Cannot parse another #[repr] attribute on a struct marked with #[repr_std140]".to_string());
        }

        let mod_path = quote!(_web_glitz::std140);
        let struct_name = &input.ident;
        let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

        let asserts = data.fields.iter().map(|field| {
            let ty = &field.ty;
            let span = field.span();

            quote_spanned!(span=> assert_repr_std140::<#ty>();)
        });

        let suffix = struct_name.to_string().trim_left_matches("r#").to_owned();
        let dummy_const = Ident::new(&format!("_ASSERT_STD140_FOR_{}", suffix), Span::call_site());

        let asserts = quote! {
            const #dummy_const: () = {
                fn assert_repr_std140<T: #mod_path::ReprStd140>() {}

                #(#asserts)*
            }
        };

        let impl_repr_std140 = quote! {
            #[automatically_derived]
            unsafe impl #impl_generics #mod_path::ReprStd140 for #struct_name #ty_generics #where_clause {}
        };

        let generated = quote! {
            #[repr(C, align(16))]
            #input

            #asserts

            #impl_repr_std140
        };

        Ok(generated)
    } else {
        Err("Cannot represent an enum or union as std140, only a struct.".to_string())
    }
}

fn has_other_repr(input: &DeriveInput) -> bool {
    input.attrs.iter().any(|attr| {
        attr.path.is_ident(Ident::new("repr", Span::call_site()))
    })
}