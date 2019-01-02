use proc_macro_hack::proc_macro_hack;

mod uniform;
pub use self::uniform::{BindingError, Uniform};

mod identifier;
pub use self::identifier::{
    IdentifierSegment, IdentifierTail, NameSegment, UniformIdentifier,
};

pub mod binding;
pub use self::binding::BindingSlot;
