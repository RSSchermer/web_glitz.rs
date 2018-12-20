#[macro_use]
extern crate web_glitz;

use web_glitz::uniforms;

fn main() {
    let uniforms = uniforms! { //~ ERROR: the trait bound `&str: web_glitz::uniform::Uniform` is not satisfied
        uniform_a: 1.0,
        uniform_b: "string"
    };
}
