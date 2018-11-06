#![feature(plugin)]
#![feature(concat_idents)]
#![feature(nll)]
#![feature(optin_builtin_traits)]
#![feature(try_from)]
//#![plugin(phf_macros)]

//extern crate phf;
extern crate futures;
extern crate wasm_bindgen;
extern crate web_sys;

//#[macro_use] extern crate failure;

pub mod buffer;
pub mod task;
pub mod rendering_context;
pub mod uniforms;
pub mod vertex_input_binding;
//pub mod vertex_stream;

mod util;