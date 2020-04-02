use std::ops::{Deref, DerefMut};

use crate::image::format::{R11F_G11F_B10F, R16F, R32F, RG16F, RG32F, RGBA16F, RGBA32F};
use crate::rendering::render_target::{AttachColorFloat, AttachMultisampleColorFloat};
use crate::rendering::{AsAttachment, AsMultisampleAttachment, Attachment, MultisampleAttachment};
use crate::runtime::{Connection, RenderingContext};

#[derive(Clone, Debug)]
pub struct Extension {
    context_id: usize,
}

impl Extension {
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

pub unsafe trait FloatRenderable {}

unsafe impl FloatRenderable for R16F {}
unsafe impl FloatRenderable for R32F {}
unsafe impl FloatRenderable for RG16F {}
unsafe impl FloatRenderable for RG32F {}
unsafe impl FloatRenderable for RGBA16F {}
unsafe impl FloatRenderable for RGBA32F {}
unsafe impl FloatRenderable for R11F_G11F_B10F {}

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
