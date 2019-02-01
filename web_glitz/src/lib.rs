#![feature(unboxed_closures)]
#![feature(fn_traits)]

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
pub mod image;
pub mod render_pass;
pub mod runtime;
pub mod sampler;
pub mod std_140;
pub mod task;
pub mod uniform;
pub mod vertex_input;

mod util;
