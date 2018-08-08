#![feature(plugin)]
#![plugin(phf_macros)]

extern crate phf;
#[macro_use]
extern crate stdweb;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate stdweb_derive;

pub mod core;
mod webgl_rendering_context;
