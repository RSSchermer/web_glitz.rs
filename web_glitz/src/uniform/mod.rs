use proc_macro_hack::proc_macro_hack;

mod uniform;
pub use self::uniform::{BindingError, Uniform};

mod uniform_identifier;
pub use self::uniform_identifier::{
    IdentifierTail, NameSegment, UniformIdentifier,
    IdentifierSegment,
};
