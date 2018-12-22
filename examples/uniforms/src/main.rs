extern crate web_glitz;

use web_glitz::uniforms;

fn main() {
    let _ = uniforms! {
        uniform_a: 1.0,
        uniform_b: [1.0, 2.0]
    };
}
