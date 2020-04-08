//! Extended functionality that may not be available in all contexts.
//!
//! Each extension provides its functionality through a main [Extension] object that may be
//! obtained for a rendering context by calling [RenderingContext::get_extension]. This returns an
//! extension object if the extension is available on the context, or `None` otherwise:
//!
//! ```
//! # use web_glitz::runtime::RenderingContext;
//! # fn wrapper<Rc>(context: &Rc) where Rc: RenderingContext {
//! use web_glitz::extensions::color_buffer_float::Extension as ColorBufferFloatExtension;
//!
//! let extension: Option<ColorBufferFloatExtension> = context.get_extension();
//!
//! if let Some(extension) = extension {
//!     // Do something with extension...
//! }
//! # }
//! ```
//!
//! Here `context` is a [RenderingContext].
use crate::runtime::Connection;

pub mod color_buffer_float;
pub mod texture_float_linear;

/// Trait implemented for extension objects, used by [RenderingContext::get_extension] to
/// initialize the extension.
pub trait Extension: Sized {
    /// Attempts to initialize the extension on the current [Connection] and return the extension
    /// object, or returns `None` if it fails.
    fn try_init(connection: &mut Connection, context_id: usize) -> Option<Self>;
}
