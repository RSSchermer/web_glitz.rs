//! WebGlitz is a low level graphics framework for the web build on top of WebGL 2 through
//! [wasm-bindgen]. It intends to be easier to use by providing a more declarative interface (as
//! opposed to WebGL's procedural and stateful interface) that is memory safe and does not expose
//! undefined behaviour.
//!
//! # Execution model
//!
//! WebGlitz uses a stateless computation-as-data model, where rather than calling functions that
//! directly cause an effect, you instead construct "task" values. Only when this task is submitted
//! will it produce its effects. See the documentation for the [task] module for details on how
//! tasks are constructed by combining commands and on how tasks are submitted to a runtime.
//!
//! # Runtimes
//!
//! WebGlitz currently only implements a single threaded runtime, see [runtime::single_threaded].
//! With this runtime, tasks may only be submitted from the same thread as the thread in which the
//! runtime was initialized. Submitting a task to the single threaded runtime typically does not
//! result in dynamic dispatch (unless the task involves waiting on a GPU fence), which should
//! mostly make the task model a zero/low cost abstraction when used with this runtime.
//!
//! The task model should also make is possible to implement a thread-safe runtime, where tasks
//! may also be submitted from other threads/(web-)workers. This is not yet implemented pending
//! further details on WASM threads/workers and their implementation in wasm-bindgen. Submitting a
//! task to this runtime will always involve dynamic dispatch, which may impose a (slight)
//! performance cost on a task. However, it may also allow you to spread the work involved in task
//! construction across multiple threads. How this trade-off impacts overall performance will depend
//! on your application.
//!
//! [wasm-bindgen]: https://github.com/rustwasm/wasm-bindgen

// For const_generics warning
#![allow(incomplete_features)]
#![feature(
    coerce_unsized,
    const_fn,
    const_generics,
    fn_traits,
    get_mut_unchecked,
    slice_index_methods,
    specialization,
    unboxed_closures,
    unsize,
    optin_builtin_traits
)]

pub mod derive {
    pub use web_glitz_macros::{InterfaceBlock, Resources, TransformFeedback, Vertex};
}

pub mod buffer;
pub mod extensions;
pub mod image;
pub mod pipeline;
pub mod rendering;
pub mod runtime;
pub mod task;

mod util;

// Re-export `memoffset::offset_of` so we can use in proc macros without users having to include a
// dependency on `memoffset`.
#[doc(hidden)]
pub use memoffset::offset_of;

/// Placeholder type for builders to mark unspecified or not yet specified attributes.
pub struct Unspecified {}
