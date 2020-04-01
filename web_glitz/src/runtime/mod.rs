//! WebGlitz currently only provides a single threaded runtime that can run on the main WASM thread,
//! see the documentation for the [single_threaded] module for details.

mod context_options;
pub use self::context_options::{ContextOptions, ContextOptionsBuilder, PowerPreference};

mod rendering_context;
pub use self::rendering_context::{
    Connection, CreateGraphicsPipelineError, Execution, RenderingContext, ShaderCompilationError,
    UnsupportedSampleCount,
};

pub mod single_threaded;

pub mod state;

pub(crate) mod executor_job;
pub(crate) mod fenced;
pub(crate) mod index_lru;
