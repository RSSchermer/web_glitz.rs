use std::marker;

use crate::image::format::InternalFormat;
use crate::rendering::{FloatBuffer, IntegerBuffer, RenderingOutputBuffer, UnsignedIntegerBuffer, StoreOp, LoadOp};
use crate::rendering::attachment::{AttachmentData, AsMultisampleAttachment};
use crate::rendering::AsAttachment;
use crate::rendering::load_op::LoadAction;

/// Helper trait implemented by types that describe a color image attachment for a [RenderTarget].
pub trait EncodeColorBuffer {
    /// The type of [RenderingOutputBuffer] that is allocated in the framebuffer to buffer
    /// modifications to the attached image.
    type Buffer: RenderingOutputBuffer;

    /// Returns an encoding of the information needed by a [RenderPass] to load data from the
    /// attached image into the framebuffer before the render pass, and to store data from the
    /// framebuffer back into the attached image after the render pass.
    fn encode_color_buffer<'a, 'b>(
        &'a mut self,
        context: &'b mut ColorBufferEncodingContext,
    ) -> ColorBufferEncoding<'b, 'a, Self::Buffer>;
}

/// Helper trait implemented by types that describe a multisample color image attachment for a
/// [MultisampleRenderTarget].
pub trait EncodeMultisampleColorBuffer {
    /// The type of [RenderingOutputBuffer] that is allocated in the framebuffer to buffer
    /// modifications to the attached image.
    type Buffer: RenderingOutputBuffer;

    /// Returns an encoding of the information needed by a [RenderPass] to load data from the
    /// attached image into the framebuffer before the render pass, and to store data from the
    /// framebuffer back into the attached image after the render pass.
    fn encode_multisample_color_buffer<'a, 'b>(
        &'a mut self,
        context: &'b mut ColorBufferEncodingContext,
    ) -> ColorBufferEncoding<'b, 'a, Self::Buffer>;
}

/// Provides the context for encoding a [ColorAttachmentDescription].
///
/// See [ColorAttachmentDescription::encode].
pub struct ColorBufferEncodingContext {
    pub(crate) render_pass_id: usize,
    pub(crate) buffer_index: i32,
}

/// An encoding of the information needed by a [RenderPass] to load data from an attached image
/// into the framebuffer before the render pass, and to store data from the framebuffer back into
/// the attached image after the render pass.
pub struct ColorBufferEncoding<'a, 'b, B> {
    pub(crate) buffer: B,
    pub(crate) load_action: LoadAction,
    pub(crate) store_op: StoreOp,
    pub(crate) image: AttachmentData,
    pub(crate) _context: &'a mut ColorBufferEncodingContext,
    pub(crate) _image_ref: marker::PhantomData<&'b ()>,
}

impl<'a, 'b, F> ColorBufferEncoding<'a, 'b, FloatBuffer<F>> where F: InternalFormat
{
    pub fn float_attachment<I>(
        context: &'a mut ColorBufferEncodingContext,
        image: &'b mut I,
        load_op: LoadOp<[f32; 4]>,
        store_op: StoreOp,
    ) -> Self
    where
        I: AsAttachment<Format = F>,
    {
        let image = image.as_attachment().into_data();

        ColorBufferEncoding {
            buffer: FloatBuffer::new(
                context.render_pass_id,
                context.buffer_index,
                image.width,
                image.height,
            ),
            load_action: load_op.as_load_float_action(context.buffer_index),
            store_op,
            image,
            _context: context,
            _image_ref: marker::PhantomData,
        }
    }

    pub fn multisample_float_attachment<I>(
        context: &'a mut ColorBufferEncodingContext,
        image: &'b mut I,
        load_op: LoadOp<[f32; 4]>,
        store_op: StoreOp,
    ) -> Self
        where
            I: AsMultisampleAttachment<SampleFormat = F>,
    {
        let image = image.as_multisample_attachment().into_data();

        ColorBufferEncoding {
            buffer: FloatBuffer::new(
                context.render_pass_id,
                context.buffer_index,
                image.width,
                image.height,
            ),
            load_action: load_op.as_load_float_action(context.buffer_index),
            store_op,
            image,
            _context: context,
            _image_ref: marker::PhantomData,
        }
    }
}

impl<'a, 'b, F> ColorBufferEncoding<'a, 'b, IntegerBuffer<F>> where F: InternalFormat
{
    pub fn integer_attachment<I>(
        context: &'a mut ColorBufferEncodingContext,
        image: &'b mut I,
        load_op: LoadOp<[i32; 4]>,
        store_op: StoreOp,
    ) -> Self
    where
        I: AsAttachment<Format = F>,
    {
        let image = image.as_attachment().into_data();

        ColorBufferEncoding {
            buffer: IntegerBuffer::new(
                context.render_pass_id,
                context.buffer_index,
                image.width,
                image.height,
            ),
            load_action: load_op.as_load_integer_action(context.buffer_index),
            store_op,
            image,
            _context: context,
            _image_ref: marker::PhantomData,
        }
    }
}

impl<'a, 'b, F> ColorBufferEncoding<'a, 'b, UnsignedIntegerBuffer<F>> where F: InternalFormat
{
    pub fn unsigned_integer_attachment<I>(
        context: &'a mut ColorBufferEncodingContext,
        image: &'b mut I,
        load_op: LoadOp<[u32; 4]>,
        store_op: StoreOp,
    ) -> Self
    where
        I: AsAttachment<Format = F>,
    {
        let image = image.as_attachment().into_data();

        ColorBufferEncoding {
            buffer: UnsignedIntegerBuffer::new(
                context.render_pass_id,
                context.buffer_index,
                image.width,
                image.height,
            ),
            load_action: load_op.as_load_unsigned_integer_action(context.buffer_index),
            store_op,
            image,
            _context: context,
            _image_ref: marker::PhantomData,
        }
    }
}

pub struct FloatAttachment<I> {
    pub(crate) image: I,
    pub(crate) load_op: LoadOp<[f32; 4]>,
    pub(crate) store_op: StoreOp,
}

impl<I> EncodeColorBuffer for FloatAttachment<I>
where
    I: AsAttachment
{
    type Buffer = FloatBuffer<I::Format>;

    fn encode_color_buffer<'a, 'b>(
        &'a mut self,
        context: &'b mut ColorBufferEncodingContext,
    ) -> ColorBufferEncoding<'b, 'a, Self::Buffer> {
        ColorBufferEncoding::float_attachment(
            context,
            &mut self.image,
            self.load_op,
            self.store_op,
        )
    }
}

impl<I> EncodeMultisampleColorBuffer for FloatAttachment<I>
    where
        I: AsMultisampleAttachment
{
    type Buffer = FloatBuffer<I::SampleFormat>;

    fn encode_multisample_color_buffer<'a, 'b>(
        &'a mut self,
        context: &'b mut ColorBufferEncodingContext,
    ) -> ColorBufferEncoding<'b, 'a, Self::Buffer> {
        ColorBufferEncoding::multisample_float_attachment(
            context,
            &mut self.image,
            self.load_op,
            self.store_op,
        )
    }
}

pub struct IntegerAttachment<I> {
    pub(crate) image: I,
    pub(crate) load_op: LoadOp<[i32; 4]>,
    pub(crate) store_op: StoreOp,
}

impl<I> EncodeColorBuffer for IntegerAttachment<I>
where
    I: AsAttachment
{
    type Buffer = IntegerBuffer<I::Format>;

    fn encode_color_buffer<'a, 'b>(
        &'a mut self,
        context: &'b mut ColorBufferEncodingContext,
    ) -> ColorBufferEncoding<'b, 'a, Self::Buffer> {
        ColorBufferEncoding::integer_attachment(
            context,
            &mut self.image,
            self.load_op,
            self.store_op,
        )
    }
}

pub struct UnsignedIntegerAttachment<I> {
    pub(crate) image: I,
    pub(crate) load_op: LoadOp<[u32; 4]>,
    pub(crate) store_op: StoreOp,
}

impl<I> EncodeColorBuffer for UnsignedIntegerAttachment<I>
where
    I: AsAttachment
{
    type Buffer = UnsignedIntegerBuffer<I::Format>;

    fn encode_color_buffer<'a, 'b>(
        &'a mut self,
        context: &'b mut ColorBufferEncodingContext,
    ) -> ColorBufferEncoding<'b, 'a, Self::Buffer> {
        ColorBufferEncoding::unsigned_integer_attachment(
            context,
            &mut self.image,
            self.load_op,
            self.store_op,
        )
    }
}
