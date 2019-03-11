#![recursion_limit = "128"]

extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2;
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

#[proc_macro_attribute]
pub fn repr_std140(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = args.to_string();
    let mut input = input.to_string();

    assert!(
        args.starts_with("= \""),
        "`#[panics_note]` requires an argument of the form \
         `#[panics_note = \"panic note here\"]`"
    );

    // Get just the bare note string
    let panics_note = args.trim_matches(&['=', ' ', '"'][..]);

    // The input will include all docstrings regardless of where the attribute is placed,
    // so we need to find the last index before the start of the item
    let insert_idx = idx_after_last_docstring(&input);

    // And insert our `### Panics` note there so it always appears at the end of an item's docs
    input.insert_str(insert_idx, &format!("/// # Panics \n/// {}\n", panics_note));

    input.parse().unwrap()
}

fn compile_error(message: String) -> proc_macro2::TokenStream {
    quote! {
        compile_error!(#message);
    }
}
