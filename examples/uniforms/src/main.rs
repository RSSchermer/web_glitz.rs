extern crate web_glitz;

use web_glitz::uniforms;

//mod public {
//    mod private {
//        pub(super) struct SomeStruct {
//            pub(super) pub_field: u32,
//            _private_field: u32
//        }
//
//        impl Default for SomeStruct {
//            fn default() -> Self {
//                SomeStruct {
//                    pub_field: 0,
//                    _private_field: 0
//                }
//            }
//        }
//    }
//
//    use self::private::SomeStruct;
//
//    impl SomeStruct {
//        pub fn new() -> Self {
//            SomeStruct {
//                pub_field: 1,
//                ..Default::default()
//            }
//        }
//    }
//}

fn main() {
    let _ = uniforms! {
        uniform_a: 1.0,
        uniform_b: [1.0, 2.0]
    };
}
