use proc_macro_hack::proc_macro_hack;

mod uniform;
pub use self::uniform::{Uniform, Uniforms, UniformBindingError};

mod uniform_identifier;
pub use self::uniform_identifier::{UniformValueIdentifier, ValueIdentifierTail, ValueIdentifierSegment, NameSegment, UniformBlockIdentifier};

mod uniform_block_slot;
pub use self::uniform_block_slot::UniformBlockSlot;

mod uniform_value_slot;
pub use self::uniform_value_slot::UniformValueSlot;

