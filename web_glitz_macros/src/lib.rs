#![recursion_limit = "128"]

extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2;
use proc_macro_hack::proc_macro_hack;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

mod interface_block;
mod interface_block_component;
mod repr_std140;
mod uniforms_impl;
mod util;
mod vertex;
//
//#[proc_macro_hack]
//pub fn uniforms(input: TokenStream) -> TokenStream {
//    uniforms_impl::expand_uniforms(input.into()).into()
//}

#[proc_macro_attribute]
pub fn repr_std140(args: TokenStream, input: TokenStream) -> TokenStream {
    assert!(args.is_empty(), "#[repr_std140] does not take arguments.");

    let input = parse_macro_input!(input as DeriveInput);

    repr_std140::expand_repr_std140(&input)
        .unwrap_or_else(compile_error)
        .into()
}

#[proc_macro_derive(InterfaceBlock, attributes(vertex_attribute))]
pub fn derive_interface_block(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    interface_block::expand_derive_interface_block(&input)
        .unwrap_or_else(compile_error)
        .into()
}

#[proc_macro_derive(InterfaceBlockComponent, attributes(vertex_attribute))]
pub fn derive_interface_block_component(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    interface_block_component::expand_derive_interface_block_component(&input)
        .unwrap_or_else(compile_error)
        .into()
}

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
