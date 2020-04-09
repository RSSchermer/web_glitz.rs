//! Allows images that use a floating point internal format to be attached to render targets.
//!
//! Allows images that use the following internal formats to be attached to a
//! [RenderTargetDescriptor] or [MultisampleRenderTargetDescriptor]:
//!
//! - [R16F]
//! - [R32F]
//! - [RG16F]
//! - [RG32F]
//! - [RGBA16F]
//! - [RGBA32F]
//! - [R11F_G11F_B10F]
//!
//! This extension uses an [Extended] wrapper type to act as a type proof for the availability of
//! this extension without requiring additional runtime checks when attaching extended images.
//!
//! # Example
//!
//! ```
//! # use web_glitz::runtime::RenderingContext;
//! # fn wrapper<Rc>(context: &Rc) where Rc: RenderingContext {
//! use web_glitz::extensions::color_buffer_float::Extension as ColorBufferFloatExtension;
//! use web_glitz::image::MipmapLevels;
//! use web_glitz::image::format::RGBA32F;
//! use web_glitz::image::texture_2d::Texture2DDescriptor;
//! use web_glitz::rendering::{RenderTargetDescriptor, LoadOp, StoreOp};
//!
//! let extension: Option<ColorBufferFloatExtension> = context.get_extension();
//!
//! if let Some(extension) = extension {
//!     let mut texture = context.try_create_texture_2d(&Texture2DDescriptor{
//!         format: RGBA32F,
//!         width: 500,
//!         height: 500,
//!         levels: MipmapLevels::Partial(1)
//!     }).unwrap();
//!
//!     let render_target_descriptor = RenderTargetDescriptor::new()
//!         .attach_color_float(
//!             extension.extend(texture.base_level_mut()), // Extend the image reference
//!             LoadOp::Load,
//!             StoreOp::Store
//!         );
//!
//!     let render_target = context.create_render_target(render_target_descriptor);
//! }
//! # }
//! ```
//!
//! Here `context` is a [RenderingContext].
use std::ops::{Deref, DerefMut};

use crate::image::format::{R11F_G11F_B10F, R16F, R32F, RG16F, RG32F, RGBA16F, RGBA32F};
use crate::rendering::render_target::{AttachColorFloat, AttachMultisampleColorFloat};
use crate::rendering::{AsAttachment, AsMultisampleAttachment, Attachment, MultisampleAttachment};
use crate::runtime::Connection;

/// Extension object for the [color_buffer_float] extension.
///
/// See the [color_buffer_float] module documentation for details.
#[derive(Clone, Debug)]
pub struct Extension {
    context_id: usize,
}

impl Extension {
    /// Wraps an attachable floating point `image` in a type that can be attached to a
    /// [RenderTargetDescriptor] without causing a type error.
    ///
    /// The image's internal format must implement [FloatRenderable].
    ///
    /// # Panics
    ///
    /// Panics if the image belongs to a different context than the extension.
    pub fn extend<I>(&self, mut image: I) -> Extended<I>
    where
        I: AsAttachment,
        I::Format: FloatRenderable,
    {
        if image.as_attachment().into_data().context_id != self.context_id {
            panic!("Attachment image belongs to a different context than this extension.");
        }

        Extended { image }
    }

    /// Wraps an attachable multisample floating point `image` in a type that can be attached to a
    /// [MultisampleRenderTargetDescriptor] without causing a type error.
    ///
    /// The image's internal format must implement [FloatRenderable].
    ///
    /// # Panics
    ///
    /// Panics if the image belongs to a different context than the extension.
    pub fn extend_multisample<I>(&self, mut image: I) -> Extended<I>
    where
        I: AsMultisampleAttachment,
        I::SampleFormat: FloatRenderable,
    {
        if image.as_multisample_attachment().into_data().context_id != self.context_id {
            panic!("Attachment image belongs to a different context than this extension.");
        }

        Extended { image }
    }
}

impl super::Extension for Extension {
    fn try_init(connection: &mut Connection, context_id: usize) -> Option<Self> {
        let (gl, _) = unsafe { connection.unpack() };

        gl.get_extension("EXT_color_buffer_float")
            .ok()
            .flatten()
            .map(|_| Extension { context_id })
    }
}

/// Marker trait for internal image format types for which this extension enables rendering.
pub unsafe trait FloatRenderable {}

unsafe impl FloatRenderable for R16F {}
unsafe impl FloatRenderable for R32F {}
unsafe impl FloatRenderable for RG16F {}
unsafe impl FloatRenderable for RG32F {}
unsafe impl FloatRenderable for RGBA16F {}
unsafe impl FloatRenderable for RGBA32F {}
unsafe impl FloatRenderable for R11F_G11F_B10F {}

/// Wrapper type for attachable images that acts as a type proof for the availability of this
/// extension, allowing the attachment of images that use a floating point internal format.
pub struct Extended<I> {
    image: I,
}

impl<I> Deref for Extended<I> {
    type Target = I;

    fn deref(&self) -> &Self::Target {
        &self.image
    }
}

impl<I> DerefMut for Extended<I> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.image
    }
}

impl<I> AsAttachment for Extended<I>
where
    I: AsAttachment,
{
    type Format = I::Format;

    fn as_attachment(&mut self) -> Attachment<Self::Format> {
        self.image.as_attachment()
    }
}

impl<I> AsMultisampleAttachment for Extended<I>
where
    I: AsMultisampleAttachment,
{
    type SampleFormat = I::SampleFormat;

    fn as_multisample_attachment(&mut self) -> MultisampleAttachment<Self::SampleFormat> {
        self.image.as_multisample_attachment()
    }
}

unsafe impl<I> AttachColorFloat for Extended<I>
where
    I: AsAttachment,
    I::Format: FloatRenderable,
{
}

unsafe impl<I> AttachMultisampleColorFloat for Extended<I>
where
    I: AsMultisampleAttachment,
    I::SampleFormat: FloatRenderable,
{
}
