use image_format::ColorFloatRenderable;
use image_format::ColorIntegerRenderable;
use image_format::ColorUnsignedIntegerRenderable;
use image_format::DepthRenderable;
use image_format::DepthStencilRenderable;
use image_format::InternalFormat;
use image_format::StencilRenderable;
use renderbuffer::RenderbufferHandle;
use runtime::dynamic_state::AttachmentSet;
use runtime::dynamic_state::DepthStencilAttachmentDescriptor;
use runtime::Connection;
use task::GpuTask;
use task::Progress;
use texture::CubeFace;
use util::slice_make_mut;
use util::JsId;

use js_sys::Uint32Array;
use renderbuffer::RenderbufferFormat;
use runtime::dynamic_state::DrawBuffer;
use runtime::dynamic_state::DynamicState;
use std::hash::Hash;
use std::hash::Hasher;
use web_sys::WebGl2RenderingContext as Gl;

use crate::render_pass::framebuffer::{
    Buffer, ColorFloatBuffer, ColorIntegerBuffer, ColorUnsignedIntegerBuffer, DepthBuffer,
    DepthStencilBuffer, Framebuffer, StencilBuffer,
};

pub struct RenderPass<Fb, F> {
    id: usize,
    context_id: usize,
    render_target_encoding: RenderTargetEncoding<Fb>,
    f: Option<F>,
}

pub struct RenderPassContext<'a> {
    connection: &'a mut Connection,
    render_pass_id: usize
}

impl<'a> RenderPassContext<'a> {
    pub fn render_pass_id(&self) -> usize {
        self.render_pass_id
    }

    pub unsafe fn unpack(&self) -> (&Gl, &DynamicState) {
        unsafe { self.connection.unpack() }
    }

    pub unsafe fn unpack_mut(&mut self) -> (&mut Gl, &mut DynamicState) {
        unsafe { self.connection.unpack_mut() }
    }
}

pub struct RenderPassMismatch;

impl<Fb, F, T, O> GpuTask<Connection> for RenderPass<Fb, F>
where
    F: FnOnce(&mut Fb) -> T,
    for<'a> T: GpuTask<RenderPassContext<'a>, Output=O>,
{
    type Output = O;

    fn progress(&mut self, connection: &mut Connection) -> Progress<Self::Output> {
        let (gl, state) = unsafe { connection.unpack_mut() };

        state
            .framebuffer_cache_mut()
            .bind_or_create(&self.render_target_encoding, gl)
            .set_draw_buffers(self.render_target_encoding.draw_buffers());

        let RenderTargetEncoding { framebuffer, data } = &mut self.render_target_encoding;

        for i in 0..data.color_count {
            data.load_ops[i].perform(gl);
        }

        if data.depth_stencil_attachment != DepthStencilAttachmentDescriptor::None {
            data.load_ops[16].perform(gl);
        }

        let f = self.f.take().expect("Can only execute render pass once");
        let output = f(framebuffer).progress(&mut RenderPassContext { connection, render_pass_id: self.id });

        let mut invalidate_buffers = [0; 17];
        let mut invalidate_counter = 0;

        for i in 0..data.color_count {
            if data.store_ops[i] == StoreOp::DontCare {
                invalidate_buffers[invalidate_counter] = Gl::COLOR_ATTACHMENT0 + i as u32;

                invalidate_counter += 1;
            }
        }

        if let Some(buffer_id) = match data.depth_stencil_attachment {
            DepthStencilAttachmentDescriptor::DepthStencil(_) => Some(Gl::DEPTH_STENCIL_ATTACHMENT),
            DepthStencilAttachmentDescriptor::Depth(_) => Some(Gl::DEPTH_ATTACHMENT),
            DepthStencilAttachmentDescriptor::Stencil(_) => Some(Gl::STENCIL_ATTACHMENT),
            DepthStencilAttachmentDescriptor::None => None,
        } {
            if data.store_ops[16] == StoreOp::DontCare {
                invalidate_buffers[invalidate_counter] = buffer_id;

                invalidate_counter += 1;
            }
        }

        if invalidate_counter > 0 {
            let (gl, _) = unsafe { connection.unpack() };
            let array = unsafe { Uint32Array::view(&invalidate_buffers[0..invalidate_counter]) };

            gl.invalidate_framebuffer(Gl::DRAW_FRAMEBUFFER, array.as_ref())
                .unwrap();
        }

        output
    }
}

pub trait AttachableImage {
    type Format: InternalFormat;

    fn descriptor(&self) -> AttachableImageDescriptor;
}

impl<F> AttachableImage for RenderbufferHandle<F>
where
    F: RenderbufferFormat + 'static,
{
    type Format = F;

    fn descriptor(&self) -> AttachableImageDescriptor {
        AttachableImageDescriptor {
            internal: AttachableImageDescriptorInternal::Renderbuffer {
                id: self.id().unwrap(),
            },
            width: self.width(),
            height: self.height()
        }
    }
}

#[derive(Hash, Clone, Copy, PartialEq)]
pub struct AttachableImageDescriptor {
    internal: AttachableImageDescriptorInternal,
    width: u32,
    height: u32,
}

#[derive(Hash, Clone, Copy, PartialEq)]
enum AttachableImageDescriptorInternal {
    Texture2DLevel { id: JsId, level: u8 },
    LayeredTextureLevelLayer { id: JsId, level: u8, layer: u16 },
    TextureCubeLevelFace { id: JsId, level: u8, face: CubeFace },
    Renderbuffer { id: JsId },
}

impl AttachableImageDescriptor {
    pub(crate) fn id(&self) -> JsId {
        match self.internal {
            AttachableImageDescriptorInternal::Texture2DLevel { id, .. } => id,
            AttachableImageDescriptorInternal::LayeredTextureLevelLayer { id, .. } => id,
            AttachableImageDescriptorInternal::TextureCubeLevelFace { id, .. } => id,
            AttachableImageDescriptorInternal::Renderbuffer { id } => id,
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub(crate) fn attach(&self, gl: &Gl, target: u32, slot: u32) {
        unsafe {
            match self.internal {
                AttachableImageDescriptorInternal::Texture2DLevel { id, level } => {
                    id.with_value_unchecked(|texture_object| {
                        gl.framebuffer_texture_2d(
                            target,
                            slot,
                            Gl::TEXTURE_2D,
                            Some(&texture_object),
                            level as i32,
                        );
                    });
                }
                AttachableImageDescriptorInternal::LayeredTextureLevelLayer {
                    id,
                    level,
                    layer,
                } => {
                    id.with_value_unchecked(|texture_object| {
                        gl.framebuffer_texture_layer(
                            target,
                            slot,
                            Some(&texture_object),
                            level as i32,
                            layer as i32,
                        );
                    });
                }
                AttachableImageDescriptorInternal::TextureCubeLevelFace { id, level, face } => {
                    id.with_value_unchecked(|texture_object| {
                        gl.framebuffer_texture_2d(
                            target,
                            slot,
                            face.id(),
                            Some(&texture_object),
                            level as i32,
                        );
                    });
                }
                AttachableImageDescriptorInternal::Renderbuffer { id } => {
                    id.with_value_unchecked(|renderbuffer_object| {
                        gl.framebuffer_renderbuffer(
                            target,
                            slot,
                            Gl::RENDERBUFFER,
                            Some(&renderbuffer_object),
                        );
                    });
                }
            }
        }
    }
}

pub trait RenderTargetDescriptor {
    type Framebuffer;

    fn encode_render_target(&self, context: &mut EncodingContext) -> RenderTargetEncoding<Self::Framebuffer>;
}

pub struct FloatAttachment<'a, I>
where
    I: AttachableImage,
    I::Format: ColorFloatRenderable,
{
    pub image: &'a mut I,
    pub load_op: LoadOp<[f32; 4]>,
    pub store_op: StoreOp,
}

pub struct IntegerAttachment<'a, I>
where
    I: AttachableImage,
    I::Format: ColorIntegerRenderable,
{
    pub image: &'a mut I,
    pub load_op: LoadOp<[i32; 4]>,
    pub store_op: StoreOp,
}

pub struct UnsignedIntegerAttachment<'a, I>
where
    I: AttachableImage,
    I::Format: ColorUnsignedIntegerRenderable,
{
    pub image: &'a mut I,
    pub load_op: LoadOp<[u32; 4]>,
    pub store_op: StoreOp,
}

pub struct DepthStencilAttachment<'a, I>
where
    I: AttachableImage,
    I::Format: DepthStencilRenderable,
{
    pub image: &'a mut I,
    pub load_op: LoadOp<(f32, i32)>,
    pub store_op: StoreOp,
}

pub struct DepthAttachment<'a, I>
where
    I: AttachableImage,
    I::Format: DepthRenderable,
{
    pub image: &'a mut I,
    pub load_op: LoadOp<f32>,
    pub store_op: StoreOp,
}

pub struct StencilAttachment<'a, I>
where
    I: AttachableImage,
    I::Format: StencilRenderable,
{
    pub image: &'a mut I,
    pub load_op: LoadOp<i32>,
    pub store_op: StoreOp,
}

#[derive(Clone, Copy, PartialEq)]
pub enum LoadOp<T> {
    Load,
    Clear(T),
}

impl LoadOp<[f32; 4]> {
    fn as_instance(&self, index: i32) -> LoadOpInstance {
        match self {
            LoadOp::Load => LoadOpInstance::Load,
            LoadOp::Clear(value) => LoadOpInstance::ClearColorFloat(index, *value),
        }
    }
}

impl LoadOp<[i32; 4]> {
    fn as_instance(&self, index: i32) -> LoadOpInstance {
        match self {
            LoadOp::Load => LoadOpInstance::Load,
            LoadOp::Clear(value) => LoadOpInstance::ClearColorInteger(index, *value),
        }
    }
}

impl LoadOp<[u32; 4]> {
    fn as_instance(&self, index: i32) -> LoadOpInstance {
        match self {
            LoadOp::Load => LoadOpInstance::Load,
            LoadOp::Clear(value) => LoadOpInstance::ClearColorUnsignedInteger(index, *value),
        }
    }
}

impl LoadOp<(f32, i32)> {
    fn as_instance(&self) -> LoadOpInstance {
        match self {
            LoadOp::Load => LoadOpInstance::Load,
            LoadOp::Clear((depth, stencil)) => LoadOpInstance::ClearDepthStencil(*depth, *stencil),
        }
    }
}

impl LoadOp<f32> {
    fn as_instance(&self) -> LoadOpInstance {
        match self {
            LoadOp::Load => LoadOpInstance::Load,
            LoadOp::Clear(depth) => LoadOpInstance::ClearDepth(*depth),
        }
    }
}

impl LoadOp<i32> {
    fn as_instance(&self) -> LoadOpInstance {
        match self {
            LoadOp::Load => LoadOpInstance::Load,
            LoadOp::Clear(stencil) => LoadOpInstance::ClearStencil(*stencil),
        }
    }
}

#[derive(Clone, Copy)]
enum LoadOpInstance {
    Load,
    ClearColorFloat(i32, [f32; 4]),
    ClearColorInteger(i32, [i32; 4]),
    ClearColorUnsignedInteger(i32, [u32; 4]),
    ClearDepthStencil(f32, i32),
    ClearDepth(f32),
    ClearStencil(i32),
}

impl LoadOpInstance {
    fn perform(&self, gl: &Gl) {
        match self {
            LoadOpInstance::Load => (),
            LoadOpInstance::ClearColorFloat(index, value) => {
                gl.clear_bufferfv_with_f32_array(Gl::COLOR, *index, unsafe {
                    slice_make_mut(value)
                })
            }
            LoadOpInstance::ClearColorInteger(index, value) => {
                gl.clear_bufferiv_with_i32_array(Gl::COLOR, *index, unsafe {
                    slice_make_mut(value)
                })
            }
            LoadOpInstance::ClearColorUnsignedInteger(index, value) => gl
                .clear_bufferuiv_with_u32_array(Gl::COLOR, *index, unsafe {
                    slice_make_mut(value)
                }),
            LoadOpInstance::ClearDepthStencil(depth, stencil) => {
                gl.clear_bufferfi(Gl::DEPTH_STENCIL, 0, *depth, *stencil)
            }
            LoadOpInstance::ClearDepth(value) => {
                gl.clear_bufferfv_with_f32_array(Gl::DEPTH, 0, unsafe { slice_make_mut(&[*value]) })
            }
            LoadOpInstance::ClearStencil(value) => {
                gl.clear_bufferiv_with_i32_array(Gl::STENCIL, 0, unsafe {
                    slice_make_mut(&[*value])
                })
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum StoreOp {
    Store,
    DontCare,
}

pub struct RenderTarget<C, Ds> {
    color: C,
    depth_stencil: Ds,
}

impl Default for RenderTarget<(), ()> {
    fn default() -> Self {
        RenderTarget {
            color: (),
            depth_stencil: (),
        }
    }
}

impl<C0> RenderTargetDescriptor for RenderTarget<C0, ()>
where
    C0: ColorAttachable,
{
    type Framebuffer = Framebuffer<C0::Buffer, ()>;

    fn encode_render_target(&self, context: &mut EncodingContext) -> RenderTargetEncoding<Self::Framebuffer> {
        let encoder = RenderTargetEncoder::new(context);
        let encoder = self.color.attach(encoder).unwrap();

        encoder.finish()
    }
}

impl<C0, Ds> RenderTargetDescriptor for RenderTarget<C0, Ds>
where
    C0: ColorAttachable,
    Ds: DepthStencilAttachable,
{
    type Framebuffer = Framebuffer<C0::Buffer, Ds::Buffer>;

    fn encode_render_target(&self, context: &mut EncodingContext) -> RenderTargetEncoding<Self::Framebuffer> {
        let encoder = RenderTargetEncoder::new(context);
        let encoder = self.color.attach(encoder).unwrap();
        let encoder = self.depth_stencil.attach(encoder);

        encoder.finish()
    }
}

macro_rules! impl_render_target_descriptor {
    ($($location:tt: $C:ident),*) => {
        impl<$($C),*> RenderTargetDescriptor for RenderTarget<($($C),*), ()> where $($C: ColorAttachable),* {
            type Framebuffer = Framebuffer<($($C::Buffer),*), ()>;

            fn encode_render_target(&self, context: &mut EncodingContext) -> RenderTargetEncoding<Self::Framebuffer> {
                let encoder = RenderTargetEncoder::new(context);

                $(
                    let encoder = self.color.$location.attach(encoder).unwrap();
                )*

                encoder.finish()
            }
        }

        impl<$($C),*, Ds> RenderTargetDescriptor for RenderTarget<($($C),*), Ds> where $($C: ColorAttachable),*, Ds: DepthStencilAttachable {
            type Framebuffer = Framebuffer<($($C::Buffer),*), Ds::Buffer>;

            fn encode_render_target(&self, context: &mut EncodingContext) -> RenderTargetEncoding<Self::Framebuffer> {
                let encoder = RenderTargetEncoder::new(context);

                $(
                    let encoder = self.color.$location.attach(encoder).unwrap();
                )*

                let encoder = self.depth_stencil.attach(encoder);

                encoder.finish()
            }
        }
    }
}

impl_render_target_descriptor!(0: C0, 1: C1);
impl_render_target_descriptor!(0: C0, 1: C1, 2: C2);
impl_render_target_descriptor!(0: C0, 1: C1, 2: C2, 3: C3);
impl_render_target_descriptor!(0: C0, 1: C1, 2: C2, 3: C3, 4: C4);
impl_render_target_descriptor!(0: C0, 1: C1, 2: C2, 3: C3, 4: C4, 5: C5);
impl_render_target_descriptor!(0: C0, 1: C1, 2: C2, 3: C3, 4: C4, 5: C5, 6: C6);
impl_render_target_descriptor!(0: C0, 1: C1, 2: C2, 3: C3, 4: C4, 5: C5, 6: C6, 7: C7);
impl_render_target_descriptor!(
    0: C0,
    1: C1,
    2: C2,
    3: C3,
    4: C4,
    5: C5,
    6: C6,
    7: C7,
    8: C8
);
impl_render_target_descriptor!(
    0: C0,
    1: C1,
    2: C2,
    3: C3,
    4: C4,
    5: C5,
    6: C6,
    7: C7,
    8: C8,
    9: C9
);
impl_render_target_descriptor!(
    0: C0,
    1: C1,
    2: C2,
    3: C3,
    4: C4,
    5: C5,
    6: C6,
    7: C7,
    8: C8,
    9: C9,
    10: C10
);
impl_render_target_descriptor!(
    0: C0,
    1: C1,
    2: C2,
    3: C3,
    4: C4,
    5: C5,
    6: C6,
    7: C7,
    8: C8,
    9: C9,
    10: C10,
    11: C11
);
impl_render_target_descriptor!(
    0: C0,
    1: C1,
    2: C2,
    3: C3,
    4: C4,
    5: C5,
    6: C6,
    7: C7,
    8: C8,
    9: C9,
    10: C10,
    11: C11,
    12: C12
);
impl_render_target_descriptor!(
    0: C0,
    1: C1,
    2: C2,
    3: C3,
    4: C4,
    5: C5,
    6: C6,
    7: C7,
    8: C8,
    9: C9,
    10: C10,
    11: C11,
    12: C12,
    13: C13
);
impl_render_target_descriptor!(
    0: C0,
    1: C1,
    2: C2,
    3: C3,
    4: C4,
    5: C5,
    6: C6,
    7: C7,
    8: C8,
    9: C9,
    10: C10,
    11: C11,
    12: C12,
    13: C13,
    14: C14
);
impl_render_target_descriptor!(
    0: C0,
    1: C1,
    2: C2,
    3: C3,
    4: C4,
    5: C5,
    6: C6,
    7: C7,
    8: C8,
    9: C9,
    10: C10,
    11: C11,
    12: C12,
    13: C13,
    14: C14,
    15: C15
);

pub trait ColorAttachable {
    type Buffer: Buffer;

    fn attach<C, Ds>(
        &self,
        render_target_encoder: RenderTargetEncoder<C, Ds>,
    ) -> Result<RenderTargetEncoder<(Self::Buffer, C), Ds>, MaxColorAttachmentsExceeded>;
}

impl<'a, I> ColorAttachable for FloatAttachment<'a, I>
where
    I: AttachableImage,
    I::Format: ColorFloatRenderable,
{
    type Buffer = ColorFloatBuffer<I::Format>;

    fn attach<C, Ds>(
        &self,
        render_target_encoder: RenderTargetEncoder<C, Ds>,
    ) -> Result<RenderTargetEncoder<(Self::Buffer, C), Ds>, MaxColorAttachmentsExceeded> {
        render_target_encoder.add_color_float_buffer(self)
    }
}

impl<'a, I> ColorAttachable for IntegerAttachment<'a, I>
where
    I: AttachableImage,
    I::Format: ColorIntegerRenderable,
{
    type Buffer = ColorIntegerBuffer<I::Format>;

    fn attach<C, Ds>(
        &self,
        render_target_encoder: RenderTargetEncoder<C, Ds>,
    ) -> Result<RenderTargetEncoder<(Self::Buffer, C), Ds>, MaxColorAttachmentsExceeded> {
        render_target_encoder.add_color_integer_buffer(self)
    }
}

impl<'a, I> ColorAttachable for UnsignedIntegerAttachment<'a, I>
where
    I: AttachableImage,
    I::Format: ColorUnsignedIntegerRenderable,
{
    type Buffer = ColorUnsignedIntegerBuffer<I::Format>;

    fn attach<C, Ds>(
        &self,
        render_target_encoder: RenderTargetEncoder<C, Ds>,
    ) -> Result<RenderTargetEncoder<(Self::Buffer, C), Ds>, MaxColorAttachmentsExceeded> {
        render_target_encoder.add_color_unsigned_integer_buffer(self)
    }
}

pub trait DepthStencilAttachable {
    type Buffer: Buffer;

    fn attach<C, Ds>(
        &self,
        render_target_encoder: RenderTargetEncoder<C, Ds>,
    ) -> RenderTargetEncoder<C, Self::Buffer>;
}

impl<'a, I> DepthStencilAttachable for DepthStencilAttachment<'a, I>
where
    I: AttachableImage,
    I::Format: DepthStencilRenderable,
{
    type Buffer = DepthStencilBuffer<I::Format>;

    fn attach<C, Ds>(
        &self,
        render_target_encoder: RenderTargetEncoder<C, Ds>,
    ) -> RenderTargetEncoder<C, Self::Buffer> {
        render_target_encoder.set_depth_stencil_buffer(self)
    }
}

impl<'a, I> DepthStencilAttachable for DepthAttachment<'a, I>
where
    I: AttachableImage,
    I::Format: DepthRenderable,
{
    type Buffer = DepthBuffer<I::Format>;

    fn attach<C, Ds>(
        &self,
        render_target_encoder: RenderTargetEncoder<C, Ds>,
    ) -> RenderTargetEncoder<C, Self::Buffer> {
        render_target_encoder.set_depth_stencil_depth_buffer(self)
    }
}

impl<'a, I> DepthStencilAttachable for StencilAttachment<'a, I>
where
    I: AttachableImage,
    I::Format: StencilRenderable,
{
    type Buffer = StencilBuffer<I::Format>;

    fn attach<C, Ds>(
        &self,
        render_target_encoder: RenderTargetEncoder<C, Ds>,
    ) -> RenderTargetEncoder<C, Self::Buffer> {
        render_target_encoder.set_depth_stencil_stencil_buffer(self)
    }
}

#[derive(Debug)]
pub struct MaxColorAttachmentsExceeded;

pub struct EncodingContext {
    render_pass_id: usize
}

pub struct RenderTargetEncoder<C, Ds> {
    color: C,
    depth_stencil: Ds,
    data: RenderTargetEncoderData,
}

struct RenderTargetEncoderData {
    render_pass_id: usize,
    load_ops: [LoadOpInstance; 17],
    store_ops: [StoreOp; 17],
    color_count: usize,
    color_attachments: [Option<AttachableImageDescriptor>; 16],
    depth_stencil_attachment: DepthStencilAttachmentDescriptor,
}

impl RenderTargetEncoder<(), ()> {
    pub fn new(context: &mut EncodingContext) -> Self {
        RenderTargetEncoder {
            color: (),
            depth_stencil: (),
            data: RenderTargetEncoderData {
                render_pass_id: context.render_pass_id,
                load_ops: [LoadOpInstance::Load; 17],
                store_ops: [StoreOp::Store; 17],
                color_count: 0,
                color_attachments: [None; 16],
                depth_stencil_attachment: DepthStencilAttachmentDescriptor::None,
            },
        }
    }
}

impl<C, Ds> RenderTargetEncoder<C, Ds> {
    pub fn add_color_float_buffer<I>(
        mut self,
        attachment: &FloatAttachment<I>,
    ) -> Result<RenderTargetEncoder<(ColorFloatBuffer<I::Format>, C), Ds>, MaxColorAttachmentsExceeded>
    where
        I: AttachableImage,
        I::Format: ColorFloatRenderable,
    {
        let c = self.data.color_count;

        if c > 15 {
            Err(MaxColorAttachmentsExceeded)
        } else {
            let image_descriptor = attachment.image.descriptor();
            let width = image_descriptor.width();
            let height = image_descriptor.height();

            self.data.color_attachments[c] = Some(image_descriptor);
            self.data.load_ops[c] = attachment.load_op.as_instance(c as i32);
            self.data.store_ops[c] = attachment.store_op;
            self.data.color_count += 1;

            Ok(RenderTargetEncoder {
                color: (ColorFloatBuffer::new(self.data.render_pass_id, c as i32, width, height), self.color),
                depth_stencil: self.depth_stencil,
                data: self.data,
            })
        }
    }

    pub fn add_color_integer_buffer<I>(
        mut self,
        attachment: &IntegerAttachment<I>,
    ) -> Result<RenderTargetEncoder<(ColorIntegerBuffer<I::Format>, C), Ds>, MaxColorAttachmentsExceeded>
    where
        I: AttachableImage,
        I::Format: ColorIntegerRenderable,
    {
        let c = self.data.color_count;

        if c > 15 {
            Err(MaxColorAttachmentsExceeded)
        } else {
            let image_descriptor = attachment.image.descriptor();
            let width = image_descriptor.width();
            let height = image_descriptor.height();

            self.data.color_attachments[c] = Some(image_descriptor);
            self.data.load_ops[c] = attachment.load_op.as_instance(c as i32);
            self.data.store_ops[c] = attachment.store_op;
            self.data.color_count += 1;

            Ok(RenderTargetEncoder {
                color: (ColorIntegerBuffer::new(self.data.render_pass_id, c as i32, width, height), self.color),
                depth_stencil: self.depth_stencil,
                data: self.data,
            })
        }
    }

    pub fn add_color_unsigned_integer_buffer<I>(
        mut self,
        attachment: &UnsignedIntegerAttachment<I>,
    ) -> Result<RenderTargetEncoder<(ColorUnsignedIntegerBuffer<I::Format>, C), Ds>, MaxColorAttachmentsExceeded>
    where
        I: AttachableImage,
        I::Format: ColorUnsignedIntegerRenderable,
    {
        let c = self.data.color_count;

        if c > 15 {
            Err(MaxColorAttachmentsExceeded)
        } else {
            let image_descriptor = attachment.image.descriptor();
            let width = image_descriptor.width();
            let height = image_descriptor.height();

            self.data.color_attachments[c] = Some(image_descriptor);
            self.data.load_ops[c] = attachment.load_op.as_instance(c as i32);
            self.data.store_ops[c] = attachment.store_op;
            self.data.color_count += 1;

            Ok(RenderTargetEncoder {
                color: (ColorUnsignedIntegerBuffer::new(self.data.render_pass_id, c as i32, width, height), self.color),
                depth_stencil: self.depth_stencil,
                data: self.data,
            })
        }
    }

    pub fn set_depth_stencil_buffer<I>(
        mut self,
        attachment: &DepthStencilAttachment<I>,
    ) -> RenderTargetEncoder<C, DepthStencilBuffer<I::Format>>
    where
        I: AttachableImage,
        I::Format: DepthStencilRenderable,
    {
        let image_descriptor = attachment.image.descriptor();
        let width = image_descriptor.width();
        let height = image_descriptor.height();

        self.data.load_ops[16] = attachment.load_op.as_instance();
        self.data.store_ops[16] = attachment.store_op;
        self.data.depth_stencil_attachment = DepthStencilAttachmentDescriptor::DepthStencil(
            image_descriptor,
        );

        RenderTargetEncoder {
            color: self.color,
            depth_stencil: DepthStencilBuffer::new(self.data.render_pass_id, width, height),
            data: self.data,
        }
    }

    pub fn set_depth_stencil_depth_buffer<I>(
        mut self,
        attachment: &DepthAttachment<I>,
    ) -> RenderTargetEncoder<C, DepthBuffer<I::Format>>
    where
        I: AttachableImage,
        I::Format: DepthRenderable,
    {
        let image_descriptor = attachment.image.descriptor();
        let width = image_descriptor.width();
        let height = image_descriptor.height();

        self.data.load_ops[16] = attachment.load_op.as_instance();
        self.data.store_ops[16] = attachment.store_op;
        self.data.depth_stencil_attachment =
            DepthStencilAttachmentDescriptor::Depth(image_descriptor);

        RenderTargetEncoder {
            color: self.color,
            depth_stencil: DepthBuffer::new(self.data.render_pass_id, width, height),
            data: self.data,
        }
    }

    pub fn set_depth_stencil_stencil_buffer<I>(
        mut self,
        attachment: &StencilAttachment<I>,
    ) -> RenderTargetEncoder<C, StencilBuffer<I::Format>>
    where
        I: AttachableImage,
        I::Format: StencilRenderable,
    {
        let image_descriptor = attachment.image.descriptor();
        let width = image_descriptor.width();
        let height = image_descriptor.height();

        self.data.load_ops[16] = attachment.load_op.as_instance();
        self.data.store_ops[16] = attachment.store_op;
        self.data.depth_stencil_attachment =
            DepthStencilAttachmentDescriptor::Stencil(image_descriptor);

        RenderTargetEncoder {
            color: self.color,
            depth_stencil: StencilBuffer::new(self.data.render_pass_id, width, height),
            data: self.data,
        }
    }
}

macro_rules! nest_pairs {
    ($head:tt) => ($head);
    ($head:tt, $($tail:tt),*) => (($head, nest_pairs!($($tail),*)));
}

macro_rules! nest_pairs_reverse {
    ([$head:tt] $($reverse:tt)*) => (nest_pairs!($head, $($reverse),*));
    ([$head:tt, $($tail:tt),*] $($reverse:tt)*) => {
        nest_pairs_reverse!([$($tail),*] $head $($reverse)*)
    }
}

macro_rules! generate_encoder_finish {
    ($($C:ident),*) => {
        impl<$($C),*> RenderTargetEncoder<nest_pairs_reverse!([(), $($C),*]), ()>
            where $($C: Buffer),*
        {
            pub fn finish(self) -> RenderTargetEncoding<Framebuffer<($($C),*), ()>> {
                #[allow(non_snake_case)]
                let nest_pairs_reverse!([_, $($C),*]) = self.color;

                RenderTargetEncoding {
                    framebuffer: Framebuffer {
                        color: ($($C),*),
                        depth_stencil: (),
                        render_pass_id: self.data.render_pass_id,
                    },
                    data: self.data,
                }
            }
        }

        impl<$($C),*, Ds> RenderTargetEncoder<nest_pairs_reverse!([(), $($C),*]), Ds>
        where
            $($C: Buffer),*,
            Ds: Buffer
        {
            pub fn finish(self) -> RenderTargetEncoding<Framebuffer<($($C),*), Ds>> {
                #[allow(non_snake_case)]
                let nest_pairs_reverse!([_, $($C),*]) = self.color;

                RenderTargetEncoding {
                    framebuffer: Framebuffer {
                        color: ($($C),*),
                        depth_stencil: self.depth_stencil,
                        render_pass_id: self.data.render_pass_id,
                    },
                    data: self.data,
                }
            }
        }
    }
}

generate_encoder_finish!(C0);
generate_encoder_finish!(C0, C1);
generate_encoder_finish!(C0, C1, C2);
generate_encoder_finish!(C0, C1, C2, C3);
generate_encoder_finish!(C0, C1, C2, C3, C4);
generate_encoder_finish!(C0, C1, C2, C3, C4, C5);
generate_encoder_finish!(C0, C1, C2, C3, C4, C5, C6);
generate_encoder_finish!(C0, C1, C2, C3, C4, C5, C6, C7);
generate_encoder_finish!(C0, C1, C2, C3, C4, C5, C6, C7, C8);
generate_encoder_finish!(C0, C1, C2, C3, C4, C5, C6, C7, C8, C9);
generate_encoder_finish!(C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10);
generate_encoder_finish!(C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11);
generate_encoder_finish!(C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12);
generate_encoder_finish!(C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12, C13);
generate_encoder_finish!(C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12, C13, C14);
generate_encoder_finish!(C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12, C13, C14, C15);

pub struct RenderTargetEncoding<F> {
    framebuffer: F,
    data: RenderTargetEncoderData,
}

impl<F> RenderTargetEncoding<F> {
    fn draw_buffers(&self) -> &[DrawBuffer] {
        const DRAW_BUFFERS_SEQUENTIAL: [DrawBuffer; 16] = [
            DrawBuffer::Color0,
            DrawBuffer::Color1,
            DrawBuffer::Color2,
            DrawBuffer::Color3,
            DrawBuffer::Color4,
            DrawBuffer::Color5,
            DrawBuffer::Color6,
            DrawBuffer::Color7,
            DrawBuffer::Color8,
            DrawBuffer::Color9,
            DrawBuffer::Color10,
            DrawBuffer::Color11,
            DrawBuffer::Color12,
            DrawBuffer::Color13,
            DrawBuffer::Color14,
            DrawBuffer::Color15,
        ];

        &DRAW_BUFFERS_SEQUENTIAL[0..self.data.color_count]
    }
}

impl<F> Hash for RenderTargetEncoding<F> {
    fn hash<H>(&self, hasher: &mut H)
    where
        H: Hasher,
    {
        self.color_attachments().hash(hasher);
        self.depth_stencil_attachment().hash(hasher);
    }
}

impl<F> AttachmentSet for RenderTargetEncoding<F> {
    fn color_attachments(&self) -> &[Option<AttachableImageDescriptor>] {
        &self.data.color_attachments[0..self.data.color_count]
    }

    fn depth_stencil_attachment(&self) -> &DepthStencilAttachmentDescriptor {
        &self.data.depth_stencil_attachment
    }
}
