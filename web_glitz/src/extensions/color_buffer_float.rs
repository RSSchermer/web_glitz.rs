use std::marker;
use std::ops::{Deref, DerefMut};

use crate::image::format::{R11F_G11F_B10F, R16F, R32F, RG16F, RG32F, RGBA16F, RGBA32F};
use crate::rendering::{
    AsAttachment, ColorBufferEncoding, ColorBufferEncodingContext, EncodeColorBuffer,
    FloatAttachment, FloatBuffer,
};
use crate::runtime::{Connection, RenderingContext};

#[derive(Clone, Debug)]
pub struct Extension {
    context_id: usize,
}

impl Extension {
    pub fn extend<I>(&self, mut float_attachment: FloatAttachment<I>) -> Extended<I>
    where
        I: AsAttachment,
        I::Format: FloatRenderable,
    {
        if float_attachment
            .image
            .as_attachment()
            .into_data()
            .context_id
            != self.context_id
        {
            panic!("Attachment image belongs to a different context than this extension.");
        }

        Extended { float_attachment }
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
    float_attachment: FloatAttachment<I>,
}

impl<I> Deref for Extended<I> {
    type Target = FloatAttachment<I>;

    fn deref(&self) -> &Self::Target {
        &self.float_attachment
    }
}

impl<I> DerefMut for Extended<I> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.float_attachment
    }
}

impl<I> EncodeColorBuffer for Extended<I>
where
    I: AsAttachment,
    I::Format: FloatRenderable,
{
    type Buffer = FloatBuffer<I::Format>;

    fn encode_color_buffer<'a, 'b>(
        &'a mut self,
        context: &'b mut ColorBufferEncodingContext,
    ) -> ColorBufferEncoding<'b, 'a, Self::Buffer> {
        let image = self.image.as_attachment().into_data();

        ColorBufferEncoding {
            buffer: FloatBuffer::new(
                context.render_pass_id,
                context.buffer_index,
                image.width,
                image.height,
            ),
            load_action: self.load_op.as_load_float_action(context.buffer_index),
            store_op: self.store_op,
            image,
            _context: context,
            _image_ref: marker::PhantomData,
        }
    }
}
