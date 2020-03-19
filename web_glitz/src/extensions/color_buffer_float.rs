use crate::runtime::RenderingContext;
use crate::image::format::{FloatRenderable as BaseFloatRenderable, R16F, R32F, RG16F, RG32F, RGBA16F, RGBA32F, R11F_G11F_B10F};
use crate::render_target::{ColorAttachmentDescription, ColorAttachmentEncoding, ColorAttachmentEncodingContext, AsAttachableImageRef, FloatAttachment, AttachableImageRef};
use crate::render_pass::FloatBuffer;
use std::marker;
use std::ops::{Deref, DerefMut};

pub struct Extension {}

impl Extension {
    pub fn request<Rc>(context: &Rc) -> Option<Self> where Rc: RenderingContext {
        unimplemented!()
    }

    pub fn extend<I>(&self, float_attachment: FloatAttachment<I>) -> Extended<I> where I: AsAttachableImageRef, I::Format: FloatRenderable {
        Extended {
            float_attachment
        }
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
    float_attachment: FloatAttachment<I>
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

impl<I> ColorAttachmentDescription for Extended<I>
    where
        I: AsAttachableImageRef,
        I::Format: FloatRenderable,
{
    type Buffer = FloatBuffer<I::Format>;

    fn encode<'a, 'b>(
        &'a mut self,
        context: &'b mut ColorAttachmentEncodingContext,
    ) -> ColorAttachmentEncoding<'b, 'a, Self::Buffer> {
        let image = self.image.as_attachable_image_ref().into_data();

        ColorAttachmentEncoding {
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
