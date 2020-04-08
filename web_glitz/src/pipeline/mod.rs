//! Pipelines represent GPU configurations for parallel data processing.
//!
//! # Graphics pipelines
//!
//! Graphics pipelines represent GPU configurations that take a vertex stream as their input, and
//! output image data to the framebuffer (see the [rendering] module). Graphics pipelines are
//! currently the only pipeline type supported by WebGlitz. See [graphics] for details.
//!
//! # Pipeline resources
//!
//! Pipeline resources are memory resources that shared by all pipeline invocations. This
//! distinguishes resources from inputs, where each invocation receives separate inputs. See
//! [resources] for details.

pub mod graphics;
pub mod interface_block;
pub mod resources;
