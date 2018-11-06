extern crate proc_macro;
extern crate proc_macro2;
extern crate quote;
extern crate syn;

use proc_macro::TokenStream;
use quote::quote;
use syn::{ DeriveInput, parse_macro_input };

mod vertex;
mod util;

#[proc_macro_derive(Vertex, attributes(vertex_attribute))]
pub fn derive_vertex(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    vertex::expand_derive_vertex(&input)
        .unwrap_or_else(compile_error)
        .into()
}

fn compile_error(message: String) -> proc_macro2::TokenStream {
    quote! {
        compile_error!(#message);
    }
}
