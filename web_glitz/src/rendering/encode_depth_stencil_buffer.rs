use std::marker;

use crate::image::format::{InternalFormat, Multisamplable};
use crate::rendering::attachment::AttachmentData;
use crate::rendering::load_op::LoadAction;
use crate::rendering::{
    AsAttachment, AsMultisampleAttachment, DepthBuffer, DepthStencilBuffer, LoadOp,
    RenderingOutputBuffer, StencilBuffer, StoreOp,
};
use crate::runtime::state::DepthStencilAttachmentDescriptor;

/// Helper trait implemented by types that describe a depth-stencil image attachment for a
/// [RenderTarget].
pub trait EncodeDepthStencilBuffer {
    /// The type of [RenderingOutputBuffer] that is allocated in the framebuffer to buffer
    /// modifications to the attached image.
    type Buffer: RenderingOutputBuffer;

    /// Returns an encoding of the information needed by a [RenderPass] to load data from the
    /// attached image into the framebuffer before the render pass, and to store data from the
    /// framebuffer back into the attached image after the render pass.
    fn encode_depth_stencil_buffer<'a, 'b>(
        &'a mut self,
        context: &'b mut DepthStencilBufferEncodingContext,
    ) -> DepthStencilBufferEncoding<'b, 'a, Self::Buffer>;
}

/// Helper trait implemented by types that describe a multisampledepth-stencil image attachment for
/// a [MultisampleRenderTarget].
pub trait EncodeMultisampleDepthStencilBuffer {
    /// The type of [RenderingOutputBuffer] that is allocated in the framebuffer to buffer
    /// modifications to the attached image.
    type Buffer: RenderingOutputBuffer;

    /// Returns an encoding of the information needed by a [RenderPass] to load data from the
    /// attached image into the framebuffer before the render pass, and to store data from the
    /// framebuffer back into the attached image after the render pass.
    fn encode_multisample_depth_stencil_buffer<'a, 'b>(
        &'a mut self,
        context: &'b mut DepthStencilBufferEncodingContext,
    ) -> DepthStencilBufferEncoding<'b, 'a, Self::Buffer>;
}

pub struct DepthStencilBufferEncodingContext {
    pub(crate) render_pass_id: u64,
}

pub struct DepthStencilBufferEncoding<'a, 'b, B> {
    pub(crate) buffer: B,
    pub(crate) load_action: LoadAction,
    pub(crate) store_op: StoreOp,
    pub(crate) depth_stencil_type: DepthStencilAttachmentType,
    pub(crate) image: AttachmentData,
    _context: &'a mut DepthStencilBufferEncodingContext,
    _image_ref: marker::PhantomData<&'b ()>,
}

impl<'a, 'b, F> DepthStencilBufferEncoding<'a, 'b, DepthStencilBuffer<F>>
where
    F: InternalFormat,
{
    pub fn depth_stencil_attachment<I>(
        context: &'a mut DepthStencilBufferEncodingContext,
        image: &'b mut I,
        load_op: LoadOp<(f32, i32)>,
        store_op: StoreOp,
    ) -> Self
    where
        I: AsAttachment<Format = F>,
    {
        let image = image.as_attachment().into_data();

        DepthStencilBufferEncoding {
            buffer: DepthStencilBuffer::new(context.render_pass_id, image.width, image.height),
            load_action: load_op.as_load_depth_stencil_action(),
            store_op,
            depth_stencil_type: DepthStencilAttachmentType::DepthStencil,
            image,
            _context: context,
            _image_ref: marker::PhantomData,
        }
    }

    pub fn multisample_depth_stencil_attachment<I>(
        context: &'a mut DepthStencilBufferEncodingContext,
        image: &'b mut I,
        load_op: LoadOp<(f32, i32)>,
        store_op: StoreOp,
    ) -> Self
    where
        I: AsMultisampleAttachment<SampleFormat = F>,
        F: Multisamplable
    {
        let image = image.as_multisample_attachment().into_data();

        DepthStencilBufferEncoding {
            buffer: DepthStencilBuffer::new(context.render_pass_id, image.width, image.height),
            load_action: load_op.as_load_depth_stencil_action(),
            store_op,
            depth_stencil_type: DepthStencilAttachmentType::DepthStencil,
            image,
            _context: context,
            _image_ref: marker::PhantomData,
        }
    }
}

impl<'a, 'b, F> DepthStencilBufferEncoding<'a, 'b, DepthBuffer<F>>
where
    F: InternalFormat,
{
    pub fn depth_attachment<I>(
        context: &'a mut DepthStencilBufferEncodingContext,
        image: &'b mut I,
        load_op: LoadOp<f32>,
        store_op: StoreOp,
    ) -> Self
    where
        I: AsAttachment<Format = F>,
    {
        let image = image.as_attachment().into_data();

        DepthStencilBufferEncoding {
            buffer: DepthBuffer::new(context.render_pass_id, image.width, image.height),
            load_action: load_op.as_load_depth_action(),
            store_op,
            depth_stencil_type: DepthStencilAttachmentType::Depth,
            image,
            _context: context,
            _image_ref: marker::PhantomData,
        }
    }

    pub fn multisample_depth_attachment<I>(
        context: &'a mut DepthStencilBufferEncodingContext,
        image: &'b mut I,
        load_op: LoadOp<f32>,
        store_op: StoreOp,
    ) -> Self
    where
        I: AsMultisampleAttachment<SampleFormat = F>,
        F: Multisamplable
    {
        let image = image.as_multisample_attachment().into_data();

        DepthStencilBufferEncoding {
            buffer: DepthBuffer::new(context.render_pass_id, image.width, image.height),
            load_action: load_op.as_load_depth_action(),
            store_op,
            depth_stencil_type: DepthStencilAttachmentType::Depth,
            image,
            _context: context,
            _image_ref: marker::PhantomData,
        }
    }
}

impl<'a, 'b, F> DepthStencilBufferEncoding<'a, 'b, StencilBuffer<F>>
where
    F: InternalFormat,
{
    pub fn stencil_attachment<I>(
        context: &'a mut DepthStencilBufferEncodingContext,
        image: &'b mut I,
        load_op: LoadOp<i32>,
        store_op: StoreOp,
    ) -> Self
    where
        I: AsAttachment<Format = F>,
    {
        let image = image.as_attachment().into_data();

        DepthStencilBufferEncoding {
            buffer: StencilBuffer::new(context.render_pass_id, image.width, image.height),
            load_action: load_op.as_load_stencil_action(),
            store_op,
            depth_stencil_type: DepthStencilAttachmentType::Stencil,
            image,
            _context: context,
            _image_ref: marker::PhantomData,
        }
    }
}

pub struct DepthStencilAttachment<I> {
    pub(crate) image: I,
    pub(crate) load_op: LoadOp<(f32, i32)>,
    pub(crate) store_op: StoreOp,
}

impl<I> EncodeDepthStencilBuffer for DepthStencilAttachment<I>
where
    I: AsAttachment,
{
    type Buffer = DepthStencilBuffer<I::Format>;

    fn encode_depth_stencil_buffer<'a, 'b>(
        &'a mut self,
        context: &'b mut DepthStencilBufferEncodingContext,
    ) -> DepthStencilBufferEncoding<'b, 'a, Self::Buffer> {
        DepthStencilBufferEncoding::depth_stencil_attachment(
            context,
            &mut self.image,
            self.load_op,
            self.store_op,
        )
    }
}

impl<I> EncodeMultisampleDepthStencilBuffer for DepthStencilAttachment<I>
where
    I: AsMultisampleAttachment,
{
    type Buffer = DepthStencilBuffer<I::SampleFormat>;

    fn encode_multisample_depth_stencil_buffer<'a, 'b>(
        &'a mut self,
        context: &'b mut DepthStencilBufferEncodingContext,
    ) -> DepthStencilBufferEncoding<'b, 'a, Self::Buffer> {
        DepthStencilBufferEncoding::multisample_depth_stencil_attachment(
            context,
            &mut self.image,
            self.load_op,
            self.store_op,
        )
    }
}

pub struct DepthAttachment<I> {
    pub(crate) image: I,
    pub(crate) load_op: LoadOp<f32>,
    pub(crate) store_op: StoreOp,
}

impl<I> EncodeDepthStencilBuffer for DepthAttachment<I>
where
    I: AsAttachment,
{
    type Buffer = DepthBuffer<I::Format>;

    fn encode_depth_stencil_buffer<'a, 'b>(
        &'a mut self,
        context: &'b mut DepthStencilBufferEncodingContext,
    ) -> DepthStencilBufferEncoding<'b, 'a, Self::Buffer> {
        DepthStencilBufferEncoding::depth_attachment(
            context,
            &mut self.image,
            self.load_op,
            self.store_op,
        )
    }
}

impl<I> EncodeMultisampleDepthStencilBuffer for DepthAttachment<I>
where
    I: AsMultisampleAttachment,
{
    type Buffer = DepthBuffer<I::SampleFormat>;

    fn encode_multisample_depth_stencil_buffer<'a, 'b>(
        &'a mut self,
        context: &'b mut DepthStencilBufferEncodingContext,
    ) -> DepthStencilBufferEncoding<'b, 'a, Self::Buffer> {
        DepthStencilBufferEncoding::multisample_depth_attachment(
            context,
            &mut self.image,
            self.load_op,
            self.store_op,
        )
    }
}

pub struct StencilAttachment<I> {
    pub(crate) image: I,
    pub(crate) load_op: LoadOp<i32>,
    pub(crate) store_op: StoreOp,
}

impl<I> EncodeDepthStencilBuffer for StencilAttachment<I>
where
    I: AsAttachment,
{
    type Buffer = StencilBuffer<I::Format>;

    fn encode_depth_stencil_buffer<'a, 'b>(
        &'a mut self,
        context: &'b mut DepthStencilBufferEncodingContext,
    ) -> DepthStencilBufferEncoding<'b, 'a, Self::Buffer> {
        DepthStencilBufferEncoding::stencil_attachment(
            context,
            &mut self.image,
            self.load_op,
            self.store_op,
        )
    }
}

pub(crate) enum DepthStencilAttachmentType {
    DepthStencil,
    Depth,
    Stencil,
}

impl DepthStencilAttachmentType {
    pub(crate) fn descriptor(&self, image: AttachmentData) -> DepthStencilAttachmentDescriptor {
        match self {
            DepthStencilAttachmentType::DepthStencil => {
                DepthStencilAttachmentDescriptor::DepthStencil(image)
            }
            DepthStencilAttachmentType::Depth => DepthStencilAttachmentDescriptor::Depth(image),
            DepthStencilAttachmentType::Stencil => DepthStencilAttachmentDescriptor::Stencil(image),
        }
    }
}
