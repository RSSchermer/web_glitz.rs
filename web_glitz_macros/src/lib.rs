#![recursion_limit = "128"]

extern crate proc_macro;

use proc_macro2;
use proc_macro::TokenStream;
use proc_macro_hack::proc_macro_hack;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

mod uniforms_impl;
mod util;
mod vertex;

#[proc_macro_derive(Vertex, attributes(vertex_attribute))]
pub fn derive_vertex(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    vertex::expand_derive_vertex(&input)
        .unwrap_or_else(compile_error)
        .into()
}

#[proc_macro_hack]
pub fn uniforms(input: TokenStream) -> TokenStream {
    uniforms_impl::expand_uniforms(input.into()).into()
}

fn compile_error(message: String) -> proc_macro2::TokenStream {
    quote! {
        compile_error!(#message);
    }
}
