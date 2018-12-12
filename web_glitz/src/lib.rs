#![feature(plugin)]
#![feature(concat_idents)]
#![feature(nll)]
#![feature(optin_builtin_traits)]
#![feature(try_from)]
#![feature(unboxed_closures)]
#![feature(fn_traits)]
//#![plugin(phf_macros)]

//extern crate phf;
extern crate futures;
extern crate wasm_bindgen;
extern crate web_sys;

#[allow(unused_imports)]
#[macro_use]
extern crate web_glitz_derive;

pub use web_glitz_derive::*;

pub mod buffer;
pub mod framebuffer;
pub mod image_format;
pub mod image_region;
pub mod program;
pub mod renderbuffer;
pub mod rendering_context;
pub mod task;
pub mod texture;
//pub mod texture_old;
pub mod uniform;
pub mod vertex_input;

mod util;
