use std::marker;

use web_sys::WebGl2RenderingContext as Gl;

use crate::image::format::{
    DepthRenderable, DepthStencilRenderable, FloatRenderable, IntegerRenderable, StencilRenderable,
    UnsignedIntegerRenderable,
};
use crate::render_pass::{
    DepthBuffer, DepthStencilBuffer, FloatBuffer, IntegerBuffer, RenderingOutputBuffer,
    StencilBuffer, UnsignedIntegerBuffer,
};
use crate::render_target::attachable_image_ref::AttachableImageData;
use crate::render_target::AsAttachableImageRef;
use crate::runtime::state::DepthStencilAttachmentDescriptor;

/// Trait implemented by types that describe a color image attachment for a [RenderTarget].
///
/// See [RenderTarget] for details.
pub trait ColorAttachmentDescription {
    /// The type of [RenderBuffer] that is allocated in the framebuffer to buffer modifications to
    /// the attached image.
    type Buffer: RenderingOutputBuffer;

    /// Returns an encoding of the information needed by a [RenderPass] to load data from the
    /// attached image into the framebuffer before the render pass, and to store data from the
    /// framebuffer back into the attached image after the render pass.
    fn encode<'a, 'b>(
        &'a mut self,
        context: &'b mut ColorAttachmentEncodingContext,
    ) -> ColorAttachmentEncoding<'b, 'a, Self::Buffer>;
}

/// Provides the context for encoding a [ColorAttachmentDescription].
///
/// See [ColorAttachmentDescription::encode].
pub struct ColorAttachmentEncodingContext {
    pub(crate) render_pass_id: usize,
    pub(crate) buffer_index: i32,
}

/// An encoding of the information needed by a [RenderPass] to load data from an attached image
/// into the framebuffer before the render pass, and to store data from the framebuffer back into
/// the attached image after the render pass.
pub struct ColorAttachmentEncoding<'a, 'b, B> {
    pub(crate) buffer: B,
    pub(crate) load_action: LoadAction,
    pub(crate) store_op: StoreOp,
    pub(crate) image: AttachableImageData,
    pub(crate) _context: &'a mut ColorAttachmentEncodingContext,
    pub(crate) _image_ref: marker::PhantomData<&'b ()>,
}

impl<'a, 'b, F> ColorAttachmentEncoding<'a, 'b, FloatBuffer<F>>
where
    F: FloatRenderable,
{
    pub fn float_attachment<I>(
        context: &'a mut ColorAttachmentEncodingContext,
        image: &'b mut I,
        load_op: LoadOp<[f32; 4]>,
        store_op: StoreOp,
    ) -> Self
    where
        I: AsAttachableImageRef<Format = F>,
    {
        let image = image.as_attachable_image_ref().into_data();

        ColorAttachmentEncoding {
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

impl<'a, 'b, F> ColorAttachmentEncoding<'a, 'b, IntegerBuffer<F>>
where
    F: IntegerRenderable,
{
    pub fn integer_attachment<I>(
        context: &'a mut ColorAttachmentEncodingContext,
        image: &'b mut I,
        load_op: LoadOp<[i32; 4]>,
        store_op: StoreOp,
    ) -> Self
    where
        I: AsAttachableImageRef<Format = F>,
    {
        let image = image.as_attachable_image_ref().into_data();

        ColorAttachmentEncoding {
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

impl<'a, 'b, F> ColorAttachmentEncoding<'a, 'b, UnsignedIntegerBuffer<F>>
where
    F: UnsignedIntegerRenderable,
{
    pub fn unsigned_integer_attachment<I>(
        context: &'a mut ColorAttachmentEncodingContext,
        image: &'b mut I,
        load_op: LoadOp<[u32; 4]>,
        store_op: StoreOp,
    ) -> Self
    where
        I: AsAttachableImageRef<Format = F>,
    {
        let image = image.as_attachable_image_ref().into_data();

        ColorAttachmentEncoding {
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

pub trait DepthStencilAttachmentDescription {
    type Buffer: RenderingOutputBuffer;

    fn encode<'a, 'b>(
        &'a mut self,
        context: &'b mut DepthStencilAttachmentEncodingContext,
    ) -> DepthStencilAttachmentEncoding<'b, 'a, Self::Buffer>;
}

pub struct DepthStencilAttachmentEncodingContext {
    pub(crate) render_pass_id: usize,
}

pub struct DepthStencilAttachmentEncoding<'a, 'b, B> {
    pub(crate) buffer: B,
    pub(crate) load_action: LoadAction,
    pub(crate) store_op: StoreOp,
    pub(crate) depth_stencil_type: DepthStencilAttachmentType,
    pub(crate) image: AttachableImageData,
    _context: &'a mut DepthStencilAttachmentEncodingContext,
    _image_ref: marker::PhantomData<&'b ()>,
}

pub(crate) enum DepthStencilAttachmentType {
    DepthStencil,
    Depth,
    Stencil,
}

impl DepthStencilAttachmentType {
    pub(crate) fn descriptor(
        &self,
        image: AttachableImageData,
    ) -> DepthStencilAttachmentDescriptor {
        match self {
            DepthStencilAttachmentType::DepthStencil => {
                DepthStencilAttachmentDescriptor::DepthStencil(image)
            }
            DepthStencilAttachmentType::Depth => DepthStencilAttachmentDescriptor::Depth(image),
            DepthStencilAttachmentType::Stencil => DepthStencilAttachmentDescriptor::Stencil(image),
        }
    }
}

impl<'a, 'b, F> DepthStencilAttachmentEncoding<'a, 'b, DepthStencilBuffer<F>>
where
    F: DepthStencilRenderable,
{
    pub fn depth_stencil_attachment<I>(
        context: &'a mut DepthStencilAttachmentEncodingContext,
        image: &'b mut I,
        load_op: LoadOp<(f32, i32)>,
        store_op: StoreOp,
    ) -> Self
    where
        I: AsAttachableImageRef<Format = F>,
    {
        let image = image.as_attachable_image_ref().into_data();

        DepthStencilAttachmentEncoding {
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

impl<'a, 'b, F> DepthStencilAttachmentEncoding<'a, 'b, DepthBuffer<F>>
where
    F: DepthRenderable,
{
    pub fn depth_attachment<I>(
        context: &'a mut DepthStencilAttachmentEncodingContext,
        image: &'b mut I,
        load_op: LoadOp<f32>,
        store_op: StoreOp,
    ) -> Self
    where
        I: AsAttachableImageRef<Format = F>,
    {
        let image = image.as_attachable_image_ref().into_data();

        DepthStencilAttachmentEncoding {
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

impl<'a, 'b, F> DepthStencilAttachmentEncoding<'a, 'b, StencilBuffer<F>>
where
    F: StencilRenderable,
{
    pub fn stencil_attachment<I>(
        context: &'a mut DepthStencilAttachmentEncodingContext,
        image: &'b mut I,
        load_op: LoadOp<i32>,
        store_op: StoreOp,
    ) -> Self
    where
        I: AsAttachableImageRef<Format = F>,
    {
        let image = image.as_attachable_image_ref().into_data();

        DepthStencilAttachmentEncoding {
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

#[derive(Clone, Copy, PartialEq)]
pub enum LoadOp<T> {
    Load,
    Clear(T),
}

impl LoadOp<[f32; 4]> {
    pub(crate) fn as_load_float_action(&self, index: i32) -> LoadAction {
        match self {
            LoadOp::Load => LoadAction::Load,
            LoadOp::Clear(value) => LoadAction::ClearColorFloat(index, *value),
        }
    }
}

impl LoadOp<[i32; 4]> {
    pub(crate) fn as_load_integer_action(&self, index: i32) -> LoadAction {
        match self {
            LoadOp::Load => LoadAction::Load,
            LoadOp::Clear(value) => LoadAction::ClearColorInteger(index, *value),
        }
    }
}

impl LoadOp<[u32; 4]> {
    pub(crate) fn as_load_unsigned_integer_action(&self, index: i32) -> LoadAction {
        match self {
            LoadOp::Load => LoadAction::Load,
            LoadOp::Clear(value) => LoadAction::ClearColorUnsignedInteger(index, *value),
        }
    }
}

impl LoadOp<(f32, i32)> {
    pub(crate) fn as_load_depth_stencil_action(&self) -> LoadAction {
        match self {
            LoadOp::Load => LoadAction::Load,
            LoadOp::Clear((depth, stencil)) => LoadAction::ClearDepthStencil(*depth, *stencil),
        }
    }
}

impl LoadOp<f32> {
    pub(crate) fn as_load_depth_action(&self) -> LoadAction {
        match self {
            LoadOp::Load => LoadAction::Load,
            LoadOp::Clear(depth) => LoadAction::ClearDepth(*depth),
        }
    }
}

impl LoadOp<i32> {
    pub(crate) fn as_load_stencil_action(&self) -> LoadAction {
        match self {
            LoadOp::Load => LoadAction::Load,
            LoadOp::Clear(stencil) => LoadAction::ClearStencil(*stencil),
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) enum LoadAction {
    Load,
    ClearColorFloat(i32, [f32; 4]),
    ClearColorInteger(i32, [i32; 4]),
    ClearColorUnsignedInteger(i32, [u32; 4]),
    ClearDepthStencil(f32, i32),
    ClearDepth(f32),
    ClearStencil(i32),
}

impl LoadAction {
    pub(crate) fn perform(&self, gl: &Gl) {
        match self {
            LoadAction::Load => (),
            LoadAction::ClearColorFloat(index, value) => {
                gl.clear_bufferfv_with_f32_array(Gl::COLOR, *index, value)
            }
            LoadAction::ClearColorInteger(index, value) => {
                gl.clear_bufferiv_with_i32_array(Gl::COLOR, *index, value)
            }
            LoadAction::ClearColorUnsignedInteger(index, value) => {
                gl.clear_bufferuiv_with_u32_array(Gl::COLOR, *index, value)
            }
            LoadAction::ClearDepthStencil(depth, stencil) => {
                gl.clear_bufferfi(Gl::DEPTH_STENCIL, 0, *depth, *stencil)
            }
            LoadAction::ClearDepth(value) => {
                gl.clear_bufferfv_with_f32_array(Gl::DEPTH, 0, &mut [*value])
            }
            LoadAction::ClearStencil(value) => {
                gl.clear_bufferiv_with_i32_array(Gl::STENCIL, 0, &mut [*value])
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum StoreOp {
    Store,
    DontCare,
}

pub struct FloatAttachment<I>
{
    pub image: I,
    pub load_op: LoadOp<[f32; 4]>,
    pub store_op: StoreOp,
}

impl<I> ColorAttachmentDescription for FloatAttachment<I>
where
    I: AsAttachableImageRef,
    I::Format: FloatRenderable,
{
    type Buffer = FloatBuffer<I::Format>;

    fn encode<'a, 'b>(
        &'a mut self,
        context: &'b mut ColorAttachmentEncodingContext,
    ) -> ColorAttachmentEncoding<'b, 'a, Self::Buffer> {
        ColorAttachmentEncoding::float_attachment(
            context,
            &mut self.image,
            self.load_op,
            self.store_op,
        )
    }
}

pub struct IntegerAttachment<I>
{
    pub image: I,
    pub load_op: LoadOp<[i32; 4]>,
    pub store_op: StoreOp,
}

impl<I> ColorAttachmentDescription for IntegerAttachment<I>
where
    I: AsAttachableImageRef,
    I::Format: IntegerRenderable,
{
    type Buffer = IntegerBuffer<I::Format>;

    fn encode<'a, 'b>(
        &'a mut self,
        context: &'b mut ColorAttachmentEncodingContext,
    ) -> ColorAttachmentEncoding<'b, 'a, Self::Buffer> {
        ColorAttachmentEncoding::integer_attachment(
            context,
            &mut self.image,
            self.load_op,
            self.store_op,
        )
    }
}

pub struct UnsignedIntegerAttachment<I>
{
    pub image: I,
    pub load_op: LoadOp<[u32; 4]>,
    pub store_op: StoreOp,
}

impl<I> ColorAttachmentDescription for UnsignedIntegerAttachment<I>
where
    I: AsAttachableImageRef,
    I::Format: UnsignedIntegerRenderable,
{
    type Buffer = UnsignedIntegerBuffer<I::Format>;

    fn encode<'a, 'b>(
        &'a mut self,
        context: &'b mut ColorAttachmentEncodingContext,
    ) -> ColorAttachmentEncoding<'b, 'a, Self::Buffer> {
        ColorAttachmentEncoding::unsigned_integer_attachment(
            context,
            &mut self.image,
            self.load_op,
            self.store_op,
        )
    }
}

pub struct DepthStencilAttachment<I>
{
    pub image: I,
    pub load_op: LoadOp<(f32, i32)>,
    pub store_op: StoreOp,
}

impl<I> DepthStencilAttachmentDescription for DepthStencilAttachment<I>
where
    I: AsAttachableImageRef,
    I::Format: DepthStencilRenderable,
{
    type Buffer = DepthStencilBuffer<I::Format>;

    fn encode<'a, 'b>(
        &'a mut self,
        context: &'b mut DepthStencilAttachmentEncodingContext,
    ) -> DepthStencilAttachmentEncoding<'b, 'a, Self::Buffer> {
        DepthStencilAttachmentEncoding::depth_stencil_attachment(
            context,
            &mut self.image,
            self.load_op,
            self.store_op,
        )
    }
}

pub struct DepthAttachment<I>

{
    pub image: I,
    pub load_op: LoadOp<f32>,
    pub store_op: StoreOp,
}

impl<I> DepthStencilAttachmentDescription for DepthAttachment<I>
where
    I: AsAttachableImageRef,
    I::Format: DepthRenderable,
{
    type Buffer = DepthBuffer<I::Format>;

    fn encode<'a, 'b>(
        &'a mut self,
        context: &'b mut DepthStencilAttachmentEncodingContext,
    ) -> DepthStencilAttachmentEncoding<'b, 'a, Self::Buffer> {
        DepthStencilAttachmentEncoding::depth_attachment(
            context,
            &mut self.image,
            self.load_op,
            self.store_op,
        )
    }
}

pub struct StencilAttachment<I>
{
    pub image: I,
    pub load_op: LoadOp<i32>,
    pub store_op: StoreOp,
}

impl<I> DepthStencilAttachmentDescription for StencilAttachment<I>
where
    I: AsAttachableImageRef,
    I::Format: StencilRenderable,
{
    type Buffer = StencilBuffer<I::Format>;

    fn encode<'a, 'b>(
        &'a mut self,
        context: &'b mut DepthStencilAttachmentEncodingContext,
    ) -> DepthStencilAttachmentEncoding<'b, 'a, Self::Buffer> {
        DepthStencilAttachmentEncoding::stencil_attachment(
            context,
            &mut self.image,
            self.load_op,
            self.store_op,
        )
    }
}
