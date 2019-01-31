#![feature(plugin)]
#![feature(concat_idents)]
#![feature(nll)]
#![feature(optin_builtin_traits)]
#![feature(try_from)]
#![feature(unboxed_closures)]
#![feature(fn_traits)]
#![feature(specialization)]
#![feature(log_syntax)]
#![feature(trace_macros)]
//#![plugin(phf_macros)]

//extern crate phf;
extern crate fnv;
extern crate futures;
extern crate js_sys;
#[macro_use]
extern crate proc_macro_hack;
extern crate wasm_bindgen;
extern crate web_sys;

#[allow(unused_imports)]
#[macro_use]
extern crate web_glitz_macros;

#[proc_macro_hack]
pub use web_glitz_macros::uniforms;
pub use web_glitz_macros::*;

pub mod buffer;
pub mod image_format;
pub mod image_region;
pub mod program;
pub mod render_pass;
pub mod renderbuffer;
pub mod runtime;
pub mod sampler;
pub mod std_140;
pub mod task;
pub mod texture;
pub mod uniform;
pub mod vertex_input;

mod util;
