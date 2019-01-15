use task::GpuTask;
use runtime::Connection;
use task::Progress;
use runtime::dynamic_state::DepthStencilAttachmentDescriptor;
use runtime::dynamic_state::AttachmentSet;
use util::slice_make_mut;
use util::JsId;
use texture::CubeFace;
use renderbuffer::RenderbufferHandle;
use image_format::InternalFormat;
use image_format::ColorFloatRenderable;
use image_format::ColorIntegerRenderable;
use image_format::ColorUnsignedIntegerRenderable;
use image_format::DepthStencilRenderable;
use image_format::DepthRenderable;
use image_format::StencilRenderable;

use js_sys::Uint32Array;
use web_sys::WebGl2RenderingContext as Gl;
use std::hash::Hash;
use std::hash::Hasher;
use runtime::dynamic_state::DrawBuffer;
use renderbuffer::RenderbufferFormat;

pub struct RenderPass<Fb, F> {
    render_target_encoding: RenderTargetEncoding<Fb>,
    f: F
}

pub struct RenderPassContext {
    connection: *mut Connection
}

impl<Fb, F, T> GpuTask<Connection> for RenderPass<Fb, F> where F: Fn(&mut Fb) -> T, T: GpuTask<RenderPassContext> {
    type Output = ();

    fn progress(&mut self, connection: &mut Connection) -> Progress<Self::Output> {
        let Connection(gl, state) = connection;

        state.framebuffer_cache_mut().get_or_create(&self.render_target_encoding, gl).set_draw_buffers(self.render_target_encoding.draw_buffers());

        let connection_ptr = connection as *mut _;
        let Connection(gl, _) = connection;
        let RenderTargetEncoding { framebuffer, data } = &mut self.render_target_encoding;

        for i in 0..data.color_count {
            data.load_ops[i].perform(gl);
        }

        if data.depth_stencil_attachment != DepthStencilAttachmentDescriptor::None {
            data.load_ops[16].perform(gl);
        }

        (self.f)(framebuffer).progress(&mut RenderPassContext {
            connection: connection_ptr
        });

        let mut invalidate_buffers = [0;17];
        let mut invalidate_counter = 0;

        for i in 0..data.color_count {
            if data.store_ops[i] == StoreOp::DontCare {
                invalidate_buffers[invalidate_counter] = Gl::COLOR_ATTACHMENT0 + i as u32;

                invalidate_counter += 1;
            }
        }

        if let Some(buffer_id) = match data.depth_stencil_attachment {
            DepthStencilAttachmentDescriptor::DepthStencil(_) => {
                Some(Gl::DEPTH_STENCIL_ATTACHMENT)
            },
            DepthStencilAttachmentDescriptor::Depth(_) => {
                Some(Gl::DEPTH_ATTACHMENT)
            },
            DepthStencilAttachmentDescriptor::Stencil(_) => {
                Some(Gl::STENCIL_ATTACHMENT)
            },
            DepthStencilAttachmentDescriptor::None => None
        } {
            if data.store_ops[16] == StoreOp::DontCare {
                invalidate_buffers[invalidate_counter] = buffer_id;

                invalidate_counter += 1;
            }
        }

        if invalidate_counter > 0 {
            let array = unsafe { Uint32Array::view(&invalidate_buffers[0..invalidate_counter]) };

            gl.invalidate_framebuffer(Gl::DRAW_FRAMEBUFFER, array.as_ref()).unwrap();
        }

        Progress::Finished(())
    }
}

pub trait AttachableImage {
    type Format: InternalFormat;

    fn as_attachment_descriptor(&self) -> AttachableImageDescriptor;
}

impl<F> AttachableImage for RenderbufferHandle<F> where F: RenderbufferFormat + 'static {
    type Format = F;

    fn as_attachment_descriptor(&self) -> AttachableImageDescriptor {
        AttachableImageDescriptor {
            internal: AttachableImageDescriptorInternal::Renderbuffer { id: self.id().unwrap() }
        }
    }
}

#[derive(Hash, Clone, Copy, PartialEq)]
pub struct AttachableImageDescriptor {
    internal: AttachableImageDescriptorInternal
}

#[derive(Hash, Clone, Copy, PartialEq)]
enum AttachableImageDescriptorInternal {
    Texture2DLevel { id: JsId, level: u8 },
    LayeredTextureLevelLayer { id: JsId, level: u8, layer: u16 },
    TextureCubeLevelFace { id: JsId, level: u8, face: CubeFace },
    Renderbuffer { id: JsId },
    None
}

impl AttachableImageDescriptor {
    pub(crate) fn none() -> Self {
        AttachableImageDescriptor {
            internal: AttachableImageDescriptorInternal::None
        }
    }

    pub(crate) fn id(&self) -> JsId {
        match self.internal {
            AttachableImageDescriptorInternal::Texture2DLevel { id, .. } => id,
            AttachableImageDescriptorInternal::LayeredTextureLevelLayer { id, .. } => id,
            AttachableImageDescriptorInternal::TextureCubeLevelFace { id, .. } => id,
            AttachableImageDescriptorInternal::Renderbuffer { id } => id,
            AttachableImageDescriptorInternal::None => panic!("Should not be None")
        }
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
                AttachableImageDescriptorInternal::LayeredTextureLevelLayer { id, level, layer } => {
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
                },
                AttachableImageDescriptorInternal::None => panic!("Should not be None")
            }
        }
    }
}

pub trait RenderTargetDescriptor {
    type Framebuffer;

    fn encode_render_target(&self) -> RenderTargetEncoding<Self::Framebuffer>;
}

pub struct FloatAttachment<'a, I> where I: AttachableImage, I::Format: ColorFloatRenderable {
    pub image: &'a mut I,
    pub load_op: LoadOp<[f32;4]>,
    pub store_op: StoreOp
}

pub struct IntegerAttachment<'a, I> where I: AttachableImage, I::Format: ColorIntegerRenderable {
    pub image: &'a mut I,
    pub load_op: LoadOp<[i32;4]>,
    pub store_op: StoreOp
}

pub struct UnsignedIntegerAttachment<'a, I> where I: AttachableImage, I::Format: ColorUnsignedIntegerRenderable {
    pub image: &'a mut I,
    pub load_op: LoadOp<[u32;4]>,
    pub store_op: StoreOp
}

pub struct DepthStencilAttachment<'a, I> where I: AttachableImage, I::Format: DepthStencilRenderable {
    pub image: &'a mut I,
    pub load_op: LoadOp<(f32, i32)>,
    pub store_op: StoreOp
}

pub struct DepthAttachment<'a, I> where I: AttachableImage, I::Format: DepthRenderable {
    pub image: &'a mut I,
    pub load_op: LoadOp<f32>,
    pub store_op: StoreOp
}

pub struct StencilAttachment<'a, I> where I: AttachableImage, I::Format: StencilRenderable {
    pub image: &'a mut I,
    pub load_op: LoadOp<i32>,
    pub store_op: StoreOp
}

#[derive(Clone, Copy, PartialEq)]
pub enum LoadOp<T> {
    Load,
    Clear(T)
}

impl LoadOp<[f32;4]> {
    fn as_instance(&self, index: i32) -> LoadOpInstance {
        match self {
            LoadOp::Load => LoadOpInstance::Load,
            LoadOp::Clear(value) => LoadOpInstance::ClearColorFloat(index, *value)
        }
    }
}

impl LoadOp<[i32;4]> {
    fn as_instance(&self, index: i32) -> LoadOpInstance {
        match self {
            LoadOp::Load => LoadOpInstance::Load,
            LoadOp::Clear(value) => LoadOpInstance::ClearColorInteger(index, *value)
        }
    }
}

impl LoadOp<[u32;4]> {
    fn as_instance(&self, index: i32) -> LoadOpInstance {
        match self {
            LoadOp::Load => LoadOpInstance::Load,
            LoadOp::Clear(value) => LoadOpInstance::ClearColorUnsignedInteger(index, *value)
        }
    }
}

impl LoadOp<(f32, i32)> {
    fn as_instance(&self) -> LoadOpInstance {
        match self {
            LoadOp::Load => LoadOpInstance::Load,
            LoadOp::Clear((depth, stencil)) => LoadOpInstance::ClearDepthStencil(*depth, *stencil)
        }
    }
}

impl LoadOp<f32> {
    fn as_instance(&self) -> LoadOpInstance {
        match self {
            LoadOp::Load => LoadOpInstance::Load,
            LoadOp::Clear(depth) => LoadOpInstance::ClearDepth(*depth)
        }
    }
}

impl LoadOp<i32> {
    fn as_instance(&self) -> LoadOpInstance {
        match self {
            LoadOp::Load => LoadOpInstance::Load,
            LoadOp::Clear(stencil) => LoadOpInstance::ClearStencil(*stencil)
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
    ClearStencil(i32)
}

impl LoadOpInstance {
    fn perform(&self, gl: &Gl) {
        match self {
            LoadOpInstance::Load => (),
            LoadOpInstance::ClearColorFloat(index, value) => {
                gl.clear_bufferfv_with_f32_array(Gl::COLOR, *index, unsafe { slice_make_mut(value) })
            },
            LoadOpInstance::ClearColorInteger(index, value) => {
                gl.clear_bufferiv_with_i32_array(Gl::COLOR, *index, unsafe { slice_make_mut(value) })
            },
            LoadOpInstance::ClearColorUnsignedInteger(index, value) => {
                gl.clear_bufferuiv_with_u32_array(Gl::COLOR, *index, unsafe { slice_make_mut(value) })
            },
            LoadOpInstance::ClearDepthStencil(depth, stencil) => {
                gl.clear_bufferfi(Gl::DEPTH_STENCIL, 0, *depth, *stencil)
            },
            LoadOpInstance::ClearDepth(value) => {
                gl.clear_bufferfv_with_f32_array(Gl::DEPTH, 0, unsafe { slice_make_mut(&[*value]) })
            },
            LoadOpInstance::ClearStencil(value) => {
                gl.clear_bufferiv_with_i32_array(Gl::STENCIL, 0, unsafe { slice_make_mut(&[*value]) })
            },
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum StoreOp {
    Store,
    DontCare
}

pub struct ColorFloatBuffer {}

impl ColorFloatBuffer {
    fn clear_task(&mut self, value: [f32;4]) {}
}

pub struct ColorIntegerBuffer {}

impl ColorIntegerBuffer {
    fn clear_task(&mut self, value: [i32;4]) {}
}

pub struct ColorUnsignedIntegerBuffer {}

impl ColorUnsignedIntegerBuffer {
    fn clear_task(&mut self, value: [u32;4]) {}
}

pub struct DepthBuffer {}

impl DepthBuffer {
    fn clear_task(&mut self, value: f32) {}
}

pub struct StencilBuffer {}

impl StencilBuffer {
    fn clear_task(&mut self, value: u8) {}
}

pub struct DepthStencilBuffer {}

impl DepthBuffer {
    fn clear_both_task(&mut self, depth: f32, stencil: u8) {}

    fn clear_depth_task(&mut self, depth: f32) {}

    fn clear_stencil_task(&mut self, stencil: u8) {}
}

pub struct RenderTarget<C, Ds> {
    color: C,
    depth_stencil: Ds
}

impl Default for RenderTarget<(), ()> {
    fn default() -> Self {
        RenderTarget {
            color: (),
            depth_stencil: ()
        }
    }
}

impl<C0> RenderTargetDescriptor for RenderTarget<C0, ()> where C0: ColorAttachable {
    type Framebuffer = Framebuffer<C0::Buffer, ()>;

    fn encode_render_target(&self) -> RenderTargetEncoding<Self::Framebuffer> {
        let encoder = RenderTargetEncoder::new();
        let encoder = self.color.attach(encoder).unwrap();

        encoder.finish()
    }
}

impl<C0, Ds> RenderTargetDescriptor for RenderTarget<C0, Ds> where C0: ColorAttachable, Ds: DepthStencilAttachable {
    type Framebuffer = Framebuffer<C0::Buffer, Ds::Buffer>;

    fn encode_render_target(&self) -> RenderTargetEncoding<Self::Framebuffer> {
        let encoder = RenderTargetEncoder::new();
        let encoder = self.color.attach(encoder).unwrap();
        let encoder = self.depth_stencil.attach(encoder);

        encoder.finish()
    }
}

macro_rules! impl_render_target_descriptor {
    (($($location:tt),*) <$($C:ident),*>) => {
        impl<$($C),*> RenderTargetDescriptor for RenderTarget<($($C),*), ()> where $($C: ColorAttachable),* {
            type Framebuffer = Framebuffer<($($C::Buffer),*), ()>;

            fn encode_render_target(&self) -> RenderTargetEncoding<Self::Framebuffer> {
                let encoder = RenderTargetEncoder::new();

                $(
                    let encoder = self.color.$location.attach(encoder).unwrap();
                )*

                encoder.finish()
            }
        }

        impl<$($C),*, Ds> RenderTargetDescriptor for RenderTarget<($($C),*), Ds> where $($C: ColorAttachable),*, Ds: DepthStencilAttachable {
            type Framebuffer = Framebuffer<($($C::Buffer),*), Ds::Buffer>;

            fn encode_render_target(&self) -> RenderTargetEncoding<Self::Framebuffer> {
                let encoder = RenderTargetEncoder::new();

                $(
                    let encoder = self.color.$location.attach(encoder).unwrap();
                )*

                let encoder = self.depth_stencil.attach(encoder);

                encoder.finish()
            }
        }
    }
}

impl_render_target_descriptor!{
    (0, 1)
    <C0, C1>
}

//impl_render_target_descriptor!{
//    (0, 1, 2)
//    <C0, C1, C2>
//}
//
//impl_render_target_descriptor!{
//    (0, 1, 2, 3)
//    <C0, C1, C2, C3>
//}
//
//impl_render_target_descriptor!{
//    (0, 1, 2, 3, 4)
//    <C0, C1, C2, C3, C4>
//}
//
//impl_render_target_descriptor!{
//    (0, 1, 2, 3, 4, 5)
//    <C0, C1, C2, C3, C4, C5>
//}
//
//impl_render_target_descriptor!{
//    (0, 1, 2, 3, 4, 5, 6)
//    <C0, C1, C2, C3, C4, C5, C6>
//}
//
//impl_render_target_descriptor!{
//    (0, 1, 2, 3, 4, 5, 6, 7)
//    <C0, C1, C2, C3, C4, C5, C6, C7>
//}
//
//impl_render_target_descriptor!{
//    (0, 1, 2, 3, 4, 5, 6, 7, 8)
//    <C0, C1, C2, C3, C4, C5, C6, C7, C8>
//}
//
//impl_render_target_descriptor!{
//    (0, 1, 2, 3, 4, 5, 6, 7, 8, 9)
//    <C0, C1, C2, C3, C4, C5, C6, C7, C8, C9>
//}
//
//impl_render_target_descriptor!{
//    (0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10)
//    <C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10>
//}
//
//impl_render_target_descriptor!{
//    (0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11)
//    <C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11>
//}
//
//impl_render_target_descriptor!{
//    (0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12)
//    <C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12>
//}
//
//impl_render_target_descriptor!{
//    (0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13)
//    <C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12, C13>
//}
//
//impl_render_target_descriptor!{
//    (0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14)
//    <C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12, C13, C14>
//}
//
//impl_render_target_descriptor!{
//    (0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15)
//    <C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12, C13, C14, C15>
//}

pub trait ColorAttachable {
    type Buffer: Buffer;

    fn attach<C, Ds>(&self, render_target_encoder: RenderTargetEncoder<C, Ds>) -> Result<RenderTargetEncoder<(C, Self::Buffer), Ds>, MaxColorAttachmentsExceeded>;
}

impl<'a, I> ColorAttachable for FloatAttachment<'a, I> where I: AttachableImage, I::Format: ColorFloatRenderable {
    type Buffer = ColorFloatBuffer;

    fn attach<C, Ds>(&self, render_target_encoder: RenderTargetEncoder<C, Ds>) -> Result<RenderTargetEncoder<(C, Self::Buffer), Ds>, MaxColorAttachmentsExceeded> {
        render_target_encoder.add_color_float_buffer(self)
    }
}

impl<'a, I> ColorAttachable for IntegerAttachment<'a, I> where I: AttachableImage, I::Format: ColorIntegerRenderable {
    type Buffer = ColorIntegerBuffer;

    fn attach<C, Ds>(&self, render_target_encoder: RenderTargetEncoder<C, Ds>) -> Result<RenderTargetEncoder<(C, Self::Buffer), Ds>, MaxColorAttachmentsExceeded> {
        render_target_encoder.add_color_integer_buffer(self)
    }
}

impl<'a, I> ColorAttachable for UnsignedIntegerAttachment<'a, I> where I: AttachableImage, I::Format: ColorUnsignedIntegerRenderable {
    type Buffer = ColorUnsignedIntegerBuffer;

    fn attach<C, Ds>(&self, render_target_encoder: RenderTargetEncoder<C, Ds>) -> Result<RenderTargetEncoder<(C, Self::Buffer), Ds>, MaxColorAttachmentsExceeded> {
        render_target_encoder.add_color_unsigned_integer_buffer(self)
    }
}

pub trait DepthStencilAttachable {
    type Buffer: Buffer;

    fn attach<C, Ds>(&self, render_target_encoder: RenderTargetEncoder<C, Ds>) -> RenderTargetEncoder<C, Self::Buffer>;
}

impl<'a, I> DepthStencilAttachable for DepthStencilAttachment<'a, I> where I: AttachableImage, I::Format: DepthStencilRenderable {
    type Buffer = DepthStencilBuffer;

    fn attach<C, Ds>(&self, render_target_encoder: RenderTargetEncoder<C, Ds>) -> RenderTargetEncoder<C, Self::Buffer> {
        render_target_encoder.set_depth_stencil_buffer(self)
    }
}

impl<'a, I> DepthStencilAttachable for DepthAttachment<'a, I> where I: AttachableImage, I::Format: DepthRenderable {
    type Buffer = DepthBuffer;

    fn attach<C, Ds>(&self, render_target_encoder: RenderTargetEncoder<C, Ds>) -> RenderTargetEncoder<C, Self::Buffer> {
        render_target_encoder.set_depth_stencil_depth_buffer(self)
    }
}

impl<'a, I> DepthStencilAttachable for StencilAttachment<'a, I> where I: AttachableImage, I::Format: StencilRenderable {
    type Buffer = StencilBuffer;

    fn attach<C, Ds>(&self, render_target_encoder: RenderTargetEncoder<C, Ds>) -> RenderTargetEncoder<C, Self::Buffer> {
        render_target_encoder.set_depth_stencil_stencil_buffer(self)
    }
}

#[derive(Debug)]
pub struct MaxColorAttachmentsExceeded;

pub struct RenderTargetEncoder<C, Ds> {
    color: C,
    depth_stencil: Ds,
    data: RenderTargetEncoderData
}

struct RenderTargetEncoderData {
    load_ops: [LoadOpInstance; 17],
    store_ops: [StoreOp; 17],
    color_count: usize,
    color_attachments: [AttachableImageDescriptor; 16],
    depth_stencil_attachment: DepthStencilAttachmentDescriptor,
}

impl RenderTargetEncoder<(), ()> {
    pub fn new() -> Self {
        RenderTargetEncoder {
            color: (),
            depth_stencil: (),
            data: RenderTargetEncoderData {
                load_ops: [LoadOpInstance::Load; 17],
                store_ops: [StoreOp::Store; 17],
                color_count: 0,
                color_attachments: [AttachableImageDescriptor::none(); 16],
                depth_stencil_attachment: DepthStencilAttachmentDescriptor::None
            }
        }
    }
}

impl<C, Ds> RenderTargetEncoder<C, Ds> {
    pub fn add_color_float_buffer<I>(mut self, attachment: &FloatAttachment<I>) -> Result<RenderTargetEncoder<(C, ColorFloatBuffer), Ds>, MaxColorAttachmentsExceeded> where I: AttachableImage, I::Format: ColorFloatRenderable {
        let c = self.data.color_count;

        if c > 15 {
            Err(MaxColorAttachmentsExceeded)
        } else {
            self.data.color_attachments[c] = attachment.image.as_attachment_descriptor();
            self.data.load_ops[c] = attachment.load_op.as_instance(c as i32);
            self.data.store_ops[c] = attachment.store_op;
            self.data.color_count += 1;

            Ok(RenderTargetEncoder {
                color: (self.color, ColorFloatBuffer {}),
                depth_stencil: self.depth_stencil,
                data: self.data
            })
        }
    }

    pub fn add_color_integer_buffer<I>(mut self, attachment: &IntegerAttachment<I>) -> Result<RenderTargetEncoder<(C, ColorIntegerBuffer), Ds>, MaxColorAttachmentsExceeded> where I: AttachableImage, I::Format: ColorIntegerRenderable {
        let c = self.data.color_count;

        if c > 15 {
            Err(MaxColorAttachmentsExceeded)
        } else {
            self.data.color_attachments[c] = attachment.image.as_attachment_descriptor();
            self.data.load_ops[c] = attachment.load_op.as_instance(c as i32);
            self.data.store_ops[c] = attachment.store_op;
            self.data.color_count += 1;

            Ok(RenderTargetEncoder {
                color: (self.color, ColorIntegerBuffer {}),
                depth_stencil: self.depth_stencil,
                data: self.data
            })
        }
    }

    pub fn add_color_unsigned_integer_buffer<I>(mut self, attachment: &UnsignedIntegerAttachment<I>) -> Result<RenderTargetEncoder<(C, ColorUnsignedIntegerBuffer), Ds>, MaxColorAttachmentsExceeded> where I: AttachableImage, I::Format: ColorUnsignedIntegerRenderable {
        let c = self.data.color_count;

        if c > 15 {
            Err(MaxColorAttachmentsExceeded)
        } else {
            self.data.color_attachments[c] = attachment.image.as_attachment_descriptor();
            self.data.load_ops[c] = attachment.load_op.as_instance(c as i32);
            self.data.store_ops[c] = attachment.store_op;
            self.data.color_count += 1;

            Ok(RenderTargetEncoder {
                color: (self.color, ColorUnsignedIntegerBuffer {}),
                depth_stencil: self.depth_stencil,
                data: self.data
            })
        }
    }

    pub fn set_depth_stencil_buffer<I>(mut self, attachment: &DepthStencilAttachment<I>) -> RenderTargetEncoder<C, DepthStencilBuffer> where I: AttachableImage, I::Format: DepthStencilRenderable {
        self.data.load_ops[16] = attachment.load_op.as_instance();
        self.data.store_ops[16] = attachment.store_op;
        self.data.depth_stencil_attachment = DepthStencilAttachmentDescriptor::DepthStencil(attachment.image.as_attachment_descriptor());

        RenderTargetEncoder {
            color: self.color,
            depth_stencil: DepthStencilBuffer {},
            data: self.data
        }
    }

    pub fn set_depth_stencil_depth_buffer<I>(mut self, attachment: &DepthAttachment<I>) -> RenderTargetEncoder<C, DepthBuffer> where I: AttachableImage, I::Format: DepthRenderable {
        self.data.load_ops[16] = attachment.load_op.as_instance();
        self.data.store_ops[16] = attachment.store_op;
        self.data.depth_stencil_attachment = DepthStencilAttachmentDescriptor::Depth(attachment.image.as_attachment_descriptor());

        RenderTargetEncoder {
            color: self.color,
            depth_stencil: DepthBuffer {},
            data: self.data
        }
    }

    pub fn set_depth_stencil_stencil_buffer<I>(mut self, attachment: &StencilAttachment<I>) -> RenderTargetEncoder<C, StencilBuffer> where I: AttachableImage, I::Format: StencilRenderable {
        self.data.load_ops[16] = attachment.load_op.as_instance();
        self.data.store_ops[16] = attachment.store_op;
        self.data.depth_stencil_attachment = DepthStencilAttachmentDescriptor::Stencil(attachment.image.as_attachment_descriptor());

        RenderTargetEncoder {
            color: self.color,
            depth_stencil: StencilBuffer {},
            data: self.data
        }
    }
}

pub trait Buffer {}

impl Buffer for ColorFloatBuffer {}
impl Buffer for ColorIntegerBuffer {}
impl Buffer for ColorUnsignedIntegerBuffer {}
impl Buffer for DepthStencilBuffer {}
impl Buffer for DepthBuffer {}
impl Buffer for StencilBuffer {}

impl<C0> RenderTargetEncoder<((), C0), ()> where C0: Buffer {
    pub fn finish(self) -> RenderTargetEncoding<Framebuffer<(C0), ()>> {
        RenderTargetEncoding {
            framebuffer: Framebuffer {
                color: (self.color.1),
                depth_stencil: (),
                _private: ()
            },
            data: self.data
        }
    }
}

impl<C0, Ds> RenderTargetEncoder<((), C0), Ds> where C0: Buffer, Ds: Buffer {
    pub fn finish(self) -> RenderTargetEncoding<Framebuffer<(C0), Ds>> {
        RenderTargetEncoding {
            framebuffer: Framebuffer {
                color: (self.color.1),
                depth_stencil: self.depth_stencil,
                _private: ()
            },
            data: self.data
        }
    }
}

impl<C0, C1> RenderTargetEncoder<(((), C0), C1), ()> where C0: Buffer, C1: Buffer {
    pub fn finish(self) -> RenderTargetEncoding<Framebuffer<(C0, C1), ()>> {
        let ((_, c0), c1) = self.color;

        RenderTargetEncoding {
            framebuffer: Framebuffer {
                color: (c0, c1),
                depth_stencil: (),
                _private: ()
            },
            data: self.data
        }
    }
}

impl<C0, C1, Ds> RenderTargetEncoder<(((), C0), C1), Ds> where C0: Buffer, C1: Buffer, Ds: Buffer {
    pub fn finish(self) -> RenderTargetEncoding<Framebuffer<(C0, C1), Ds>> {
        let ((_, c0), c1) = self.color;

        RenderTargetEncoding {
            framebuffer: Framebuffer {
                color: (c0, c1),
                depth_stencil: self.depth_stencil,
                _private: ()
            },
            data: self.data
        }
    }
}

pub struct RenderTargetEncoding<F> {
    framebuffer: F,
    data: RenderTargetEncoderData
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
    fn hash<H>(&self, hasher: &mut H) where H: Hasher {
        self.color_attachments().hash(hasher);
        self.depth_stencil_attachment().hash(hasher);
    }
}

impl<F> AttachmentSet for RenderTargetEncoding<F> {
    fn color_attachments(&self) -> &[AttachableImageDescriptor] {
        &self.data.color_attachments[0..self.data.color_count]
    }

    fn depth_stencil_attachment(&self) -> &DepthStencilAttachmentDescriptor {
        &self.data.depth_stencil_attachment
    }
}

pub struct Framebuffer<C, Ds> {
    pub color: C,
    pub depth_stencil: Ds,
    _private: () // Should make it impossible to instantiate Framebuffer outside of this module
}
