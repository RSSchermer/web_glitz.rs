use std::hash::{Hash, Hasher};

use js_sys::Uint32Array;
use web_sys::WebGl2RenderingContext as Gl;

use crate::image::format::{
    DepthRenderable, DepthStencilRenderable, FloatRenderable, IntegerRenderable, InternalFormat,
    RenderbufferFormat, StencilRenderable, UnsignedIntegerRenderable,
};
use crate::image::renderbuffer::Renderbuffer;
use crate::image::texture_cube::CubeFace;
use crate::render_pass::framebuffer::{
    Buffer, DepthBuffer, DepthStencilBuffer, FloatBuffer, Framebuffer, IntegerBuffer,
    StencilBuffer, UnsignedIntegerBuffer,
};
use crate::runtime::dynamic_state::AttachmentSet;
use crate::runtime::dynamic_state::{DepthStencilAttachmentDescriptor, DrawBuffer, DynamicState};
use crate::runtime::Connection;
use crate::task::{GpuTask, Progress};
use crate::util::{slice_make_mut, JsId};

pub struct RenderPass<Fb, F> {
    id: usize,
    context_id: usize,
    render_target_encoding: RenderTargetEncoding<Fb>,
    f: Option<F>,
}

pub struct RenderPassContext<'a> {
    connection: &'a mut Connection,
    render_pass_id: usize,
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
    F: FnOnce(&Fb) -> T,
    for<'a> T: GpuTask<RenderPassContext<'a>, Output = O>,
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
        let output = f(framebuffer).progress(&mut RenderPassContext {
            connection,
            render_pass_id: self.id,
        });

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

pub trait IntoAttachment {
    type Format: InternalFormat;

    fn into_attachment(self) -> Attachment;
}

impl<'a, F> IntoAttachment for &'a mut Renderbuffer<F>
where
    F: RenderbufferFormat + 'static,
{
    type Format = F;

    fn into_attachment(self) -> Attachment {
        Attachment {
            internal: AttachableImageDescriptorInternal::Renderbuffer {
                id: self.id().unwrap(),
            },
            width: self.width(),
            height: self.height(),
        }
    }
}

#[derive(Hash, PartialEq)]
pub struct Attachment {
    internal: AttachableImageDescriptorInternal,
    width: u32,
    height: u32,
}

#[derive(Hash, PartialEq)]
enum AttachableImageDescriptorInternal {
    Texture2DLevel { id: JsId, level: u8 },
    LayeredTextureLevelLayer { id: JsId, level: u8, layer: u16 },
    TextureCubeLevelFace { id: JsId, level: u8, face: CubeFace },
    Renderbuffer { id: JsId },
}

impl Attachment {
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

pub trait RenderTargetDescription {
    type Framebuffer;

    fn into_encoding(
        self,
        context: &mut EncodingContext,
    ) -> RenderTargetEncoding<Self::Framebuffer>;
}

pub struct FloatTarget<I>
where
    I: IntoAttachment,
    I::Format: FloatRenderable,
{
    pub image: I,
    pub load_op: LoadOp<[f32; 4]>,
    pub store_op: StoreOp,
}

pub struct IntegerTarget<I>
where
    I: IntoAttachment,
    I::Format: IntegerRenderable,
{
    pub image: I,
    pub load_op: LoadOp<[i32; 4]>,
    pub store_op: StoreOp,
}

pub struct UnsignedIntegerTarget<I>
where
    I: IntoAttachment,
    I::Format: UnsignedIntegerRenderable,
{
    pub image: I,
    pub load_op: LoadOp<[u32; 4]>,
    pub store_op: StoreOp,
}

pub struct DepthStencilTarget<I>
where
    I: IntoAttachment,
    I::Format: DepthStencilRenderable,
{
    pub image: I,
    pub load_op: LoadOp<(f32, i32)>,
    pub store_op: StoreOp,
}

pub struct DepthTarget<I>
where
    I: IntoAttachment,
    I::Format: DepthRenderable,
{
    pub image: I,
    pub load_op: LoadOp<f32>,
    pub store_op: StoreOp,
}

pub struct StencilTarget<I>
where
    I: IntoAttachment,
    I::Format: StencilRenderable,
{
    pub image: I,
    pub load_op: LoadOp<i32>,
    pub store_op: StoreOp,
}

#[derive(Clone, Copy, PartialEq)]
pub enum LoadOp<T> {
    Load,
    Clear(T),
}

impl LoadOp<[f32; 4]> {
    fn as_instance(&self, index: i32) -> LoadAction {
        match self {
            LoadOp::Load => LoadAction::Load,
            LoadOp::Clear(value) => LoadAction::ClearColorFloat(index, *value),
        }
    }
}

impl LoadOp<[i32; 4]> {
    fn as_instance(&self, index: i32) -> LoadAction {
        match self {
            LoadOp::Load => LoadAction::Load,
            LoadOp::Clear(value) => LoadAction::ClearColorInteger(index, *value),
        }
    }
}

impl LoadOp<[u32; 4]> {
    fn as_instance(&self, index: i32) -> LoadAction {
        match self {
            LoadOp::Load => LoadAction::Load,
            LoadOp::Clear(value) => LoadAction::ClearColorUnsignedInteger(index, *value),
        }
    }
}

impl LoadOp<(f32, i32)> {
    fn as_instance(&self) -> LoadAction {
        match self {
            LoadOp::Load => LoadAction::Load,
            LoadOp::Clear((depth, stencil)) => LoadAction::ClearDepthStencil(*depth, *stencil),
        }
    }
}

impl LoadOp<f32> {
    fn as_instance(&self) -> LoadAction {
        match self {
            LoadOp::Load => LoadAction::Load,
            LoadOp::Clear(depth) => LoadAction::ClearDepth(*depth),
        }
    }
}

impl LoadOp<i32> {
    fn as_action(&self) -> LoadAction {
        match self {
            LoadOp::Load => LoadAction::Load,
            LoadOp::Clear(stencil) => LoadAction::ClearStencil(*stencil),
        }
    }
}

#[derive(Clone, Copy)]
enum LoadAction {
    Load,
    ClearColorFloat(i32, [f32; 4]),
    ClearColorInteger(i32, [i32; 4]),
    ClearColorUnsignedInteger(i32, [u32; 4]),
    ClearDepthStencil(f32, i32),
    ClearDepth(f32),
    ClearStencil(i32),
}

impl LoadAction {
    fn perform(&self, gl: &Gl) {
        match self {
            LoadAction::Load => (),
            LoadAction::ClearColorFloat(index, value) => {
                gl.clear_bufferfv_with_f32_array(Gl::COLOR, *index, unsafe {
                    slice_make_mut(value)
                })
            }
            LoadAction::ClearColorInteger(index, value) => {
                gl.clear_bufferiv_with_i32_array(Gl::COLOR, *index, unsafe {
                    slice_make_mut(value)
                })
            }
            LoadAction::ClearColorUnsignedInteger(index, value) => gl
                .clear_bufferuiv_with_u32_array(Gl::COLOR, *index, unsafe {
                    slice_make_mut(value)
                }),
            LoadAction::ClearDepthStencil(depth, stencil) => {
                gl.clear_bufferfi(Gl::DEPTH_STENCIL, 0, *depth, *stencil)
            }
            LoadAction::ClearDepth(value) => {
                gl.clear_bufferfv_with_f32_array(Gl::DEPTH, 0, unsafe { slice_make_mut(&[*value]) })
            }
            LoadAction::ClearStencil(value) => {
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

//impl<'a, 'b, C, I> RenderTargetDescription for RenderTarget<C, ()> where C: IntoIterator<Item=&'a FloatAttachment<'b, I>>, I: AttachableImage, I::Format: ColorFloatRenderable {
//    type Framebuffer = Framebuffer<Vec<ColorFloatBuffer<I::Format>>, ()>;
//
//    fn encode_render_target(&self, context: &mut EncodingContext) -> RenderTargetEncoding<Self::Framebuffer> {
//        RenderTargetEncoding::from_float_attachments(context,self.color)
//    }
//}

//impl<'a, 'b, I, F> RenderTargetDescription for RenderTarget<&'a [FloatAttachment<'b, I>], ()>
//where
//    I: AttachableImageRef<Format = F>,
//    F: ColorFloatRenderable,
//{
//    type Framebuffer = Framebuffer<Vec<ColorFloatBuffer<F>>, ()>;
//
//    fn encode_render_target(
//        &self,
//        context: &mut EncodingContext,
//    ) -> RenderTargetEncoding<Self::Framebuffer> {
//        RenderTargetEncoding::from_float_colors(context, self.color)
//    }
//}
//
//impl<'a, 'b, 'c, I0, I1, F0, F1> RenderTargetDescription
//    for RenderTarget<&'a [FloatAttachment<'b, I0>], DepthStencilAttachment<'c, I1>>
//where
//    I0: AttachableImageRef<Format = F0>,
//    I1: AttachableImageRef<Format = F1>,
//    F0: ColorFloatRenderable,
//    F1: DepthStencilRenderable,
//{
//    type Framebuffer = Framebuffer<Vec<ColorFloatBuffer<F0>>, DepthStencilBuffer<F1>>;
//
//    fn encode_render_target(
//        &self,
//        context: &mut EncodingContext,
//    ) -> RenderTargetEncoding<Self::Framebuffer> {
//        RenderTargetEncoding::from_float_colors_depth_stencil(
//            context,
//            self.color,
//            &self.depth_stencil,
//        )
//    }
//}
//
//impl<'a, 'b, 'c, I0, I1, F0, F1> RenderTargetDescription
//for RenderTarget<&'a [FloatAttachment<'b, I0>], DepthAttachment<'c, I1>>
//    where
//        I0: AttachableImageRef<Format = F0>,
//        I1: AttachableImageRef<Format = F1>,
//        F0: ColorFloatRenderable,
//        F1: DepthRenderable,
//{
//    type Framebuffer = Framebuffer<Vec<ColorFloatBuffer<F0>>, DepthBuffer<F1>>;
//
//    fn encode_render_target(
//        &self,
//        context: &mut EncodingContext,
//    ) -> RenderTargetEncoding<Self::Framebuffer> {
//        RenderTargetEncoding::from_float_colors_depth(
//            context,
//            self.color,
//            &self.depth_stencil,
//        )
//    }
//}
//
//impl<'a, 'b, 'c, I0, I1, F0, F1> RenderTargetDescription
//for RenderTarget<&'a [FloatAttachment<'b, I0>], StencilAttachment<'c, I1>>
//    where
//        I0: AttachableImageRef<Format = F0>,
//        I1: AttachableImageRef<Format = F1>,
//        F0: ColorFloatRenderable,
//        F1: StencilRenderable,
//{
//    type Framebuffer = Framebuffer<Vec<ColorFloatBuffer<F0>>, StencilBuffer<F1>>;
//
//    fn encode_render_target(
//        &self,
//        context: &mut EncodingContext,
//    ) -> RenderTargetEncoding<Self::Framebuffer> {
//        RenderTargetEncoding::from_float_colors_stencil(
//            context,
//            self.color,
//            &self.depth_stencil,
//        )
//    }
//}
//
//impl<'a, 'b, I, F> RenderTargetDescription for RenderTarget<&'a [IntegerAttachment<'b, I>], ()>
//    where
//        I: AttachableImageRef<Format = F>,
//        F: ColorIntegerRenderable,
//{
//    type Framebuffer = Framebuffer<Vec<ColorIntegerBuffer<F>>, ()>;
//
//    fn encode_render_target(
//        &self,
//        context: &mut EncodingContext,
//    ) -> RenderTargetEncoding<Self::Framebuffer> {
//        RenderTargetEncoding::from_integer_colors(context, self.color)
//    }
//}
//
//impl<'a, 'b, 'c, I0, I1, F0, F1> RenderTargetDescription
//for RenderTarget<&'a [IntegerAttachment<'b, I0>], DepthStencilAttachment<'c, I1>>
//    where
//        I0: AttachableImageRef<Format = F0>,
//        I1: AttachableImageRef<Format = F1>,
//        F0: ColorIntegerRenderable,
//        F1: DepthStencilRenderable,
//{
//    type Framebuffer = Framebuffer<Vec<ColorIntegerBuffer<F0>>, DepthStencilBuffer<F1>>;
//
//    fn encode_render_target(
//        &self,
//        context: &mut EncodingContext,
//    ) -> RenderTargetEncoding<Self::Framebuffer> {
//        RenderTargetEncoding::from_integer_colors_depth_stencil(
//            context,
//            self.color,
//            &self.depth_stencil,
//        )
//    }
//}
//
//impl<'a, 'b, 'c, I0, I1, F0, F1> RenderTargetDescription
//for RenderTarget<&'a [IntegerAttachment<'b, I0>], DepthAttachment<'c, I1>>
//    where
//        I0: AttachableImageRef<Format = F0>,
//        I1: AttachableImageRef<Format = F1>,
//        F0: ColorIntegerRenderable,
//        F1: DepthRenderable,
//{
//    type Framebuffer = Framebuffer<Vec<ColorIntegerBuffer<F0>>, DepthBuffer<F1>>;
//
//    fn encode_render_target(
//        &self,
//        context: &mut EncodingContext,
//    ) -> RenderTargetEncoding<Self::Framebuffer> {
//        RenderTargetEncoding::from_integer_colors_depth(
//            context,
//            self.color,
//            &self.depth_stencil,
//        )
//    }
//}
//
//impl<'a, 'b, 'c, I0, I1, F0, F1> RenderTargetDescription
//for RenderTarget<&'a [IntegerAttachment<'b, I0>], StencilAttachment<'c, I1>>
//    where
//        I0: AttachableImageRef<Format = F0>,
//        I1: AttachableImageRef<Format = F1>,
//        F0: ColorIntegerRenderable,
//        F1: StencilRenderable,
//{
//    type Framebuffer = Framebuffer<Vec<ColorIntegerBuffer<F0>>, StencilBuffer<F1>>;
//
//    fn encode_render_target(
//        &self,
//        context: &mut EncodingContext,
//    ) -> RenderTargetEncoding<Self::Framebuffer> {
//        RenderTargetEncoding::from_integer_colors_stencil(
//            context,
//            self.color,
//            &self.depth_stencil,
//        )
//    }
//}
//
//impl<'a, 'b, I, F> RenderTargetDescription for RenderTarget<&'a [UnsignedIntegerAttachment<'b, I>], ()>
//    where
//        I: AttachableImageRef<Format = F>,
//        F: ColorUnsignedIntegerRenderable,
//{
//    type Framebuffer = Framebuffer<Vec<ColorUnsignedIntegerBuffer<F>>, ()>;
//
//    fn encode_render_target(
//        &self,
//        context: &mut EncodingContext,
//    ) -> RenderTargetEncoding<Self::Framebuffer> {
//        RenderTargetEncoding::from_unsigned_integer_colors(context, self.color)
//    }
//}
//
//impl<'a, 'b, 'c, I0, I1, F0, F1> RenderTargetDescription
//for RenderTarget<&'a [UnsignedIntegerAttachment<'b, I0>], DepthStencilAttachment<'c, I1>>
//    where
//        I0: AttachableImageRef<Format = F0>,
//        I1: AttachableImageRef<Format = F1>,
//        F0: ColorUnsignedIntegerRenderable,
//        F1: DepthStencilRenderable,
//{
//    type Framebuffer = Framebuffer<Vec<ColorUnsignedIntegerBuffer<F0>>, DepthStencilBuffer<F1>>;
//
//    fn encode_render_target(
//        &self,
//        context: &mut EncodingContext,
//    ) -> RenderTargetEncoding<Self::Framebuffer> {
//        RenderTargetEncoding::from_unsigned_integer_colors_depth_stencil(
//            context,
//            self.color,
//            &self.depth_stencil,
//        )
//    }
//}
//
//impl<'a, 'b, 'c, I0, I1, F0, F1> RenderTargetDescription
//for RenderTarget<&'a [UnsignedIntegerAttachment<'b, I0>], DepthAttachment<'c, I1>>
//    where
//        I0: AttachableImageRef<Format = F0>,
//        I1: AttachableImageRef<Format = F1>,
//        F0: ColorUnsignedIntegerRenderable,
//        F1: DepthRenderable,
//{
//    type Framebuffer = Framebuffer<Vec<ColorUnsignedIntegerBuffer<F0>>, DepthBuffer<F1>>;
//
//    fn encode_render_target(
//        &self,
//        context: &mut EncodingContext,
//    ) -> RenderTargetEncoding<Self::Framebuffer> {
//        RenderTargetEncoding::from_unsigned_integer_colors_depth(
//            context,
//            self.color,
//            &self.depth_stencil,
//        )
//    }
//}
//
//impl<'a, 'b, 'c, I0, I1, F0, F1> RenderTargetDescription
//for RenderTarget<&'a [UnsignedIntegerAttachment<'b, I0>], StencilAttachment<'c, I1>>
//    where
//        I0: AttachableImageRef<Format = F0>,
//        I1: AttachableImageRef<Format = F1>,
//        F0: ColorUnsignedIntegerRenderable,
//        F1: StencilRenderable,
//{
//    type Framebuffer = Framebuffer<Vec<ColorUnsignedIntegerBuffer<F0>>, StencilBuffer<F1>>;
//
//    fn encode_render_target(
//        &self,
//        context: &mut EncodingContext,
//    ) -> RenderTargetEncoding<Self::Framebuffer> {
//        RenderTargetEncoding::from_unsigned_integer_colors_stencil(
//            context,
//            self.color,
//            &self.depth_stencil,
//        )
//    }
//}

impl<C0> RenderTargetDescription for RenderTarget<C0, ()>
where
    C0: ColorTargetDescription,
{
    type Framebuffer = Framebuffer<C0::Buffer, ()>;

    fn into_encoding(
        self,
        context: &mut EncodingContext,
    ) -> RenderTargetEncoding<Self::Framebuffer> {
        let encoder = RenderTargetEncoder::new(context);
        let encoder = self.color.encode(encoder).unwrap();

        encoder.finish()
    }
}

impl<C0, Ds> RenderTargetDescription for RenderTarget<C0, Ds>
where
    C0: ColorTargetDescription,
    Ds: DepthStencilTargetDescription,
{
    type Framebuffer = Framebuffer<C0::Buffer, Ds::Buffer>;

    fn into_encoding(
        self,
        context: &mut EncodingContext,
    ) -> RenderTargetEncoding<Self::Framebuffer> {
        let encoder = RenderTargetEncoder::new(context);
        let encoder = self.color.encode(encoder).unwrap();
        let encoder = self.depth_stencil.encode(encoder);

        encoder.finish()
    }
}

macro_rules! impl_render_target_description {
    ($($C:ident),*) => {
        impl<$($C),*> RenderTargetDescription for RenderTarget<($($C),*), ()>
        where
            $($C: ColorTargetDescription),*
        {
            type Framebuffer = Framebuffer<($($C::Buffer),*), ()>;

            fn into_encoding(self, context: &mut EncodingContext) -> RenderTargetEncoding<Self::Framebuffer> {
                let encoder = RenderTargetEncoder::new(context);

                #[allow(non_snake_case)]
                let ($($C),*) = self.color;

                $(
                    let encoder = $C.encode(encoder).unwrap();
                )*

                encoder.finish()
            }
        }

        impl<$($C),*, Ds> RenderTargetDescription for RenderTarget<($($C),*), Ds>
        where
            $($C: ColorTargetDescription),*,
            Ds: DepthStencilTargetDescription
        {
            type Framebuffer = Framebuffer<($($C::Buffer),*), Ds::Buffer>;

            fn into_encoding(self, context: &mut EncodingContext) -> RenderTargetEncoding<Self::Framebuffer> {
                let encoder = RenderTargetEncoder::new(context);

                #[allow(non_snake_case)]
                let ($($C),*) = self.color;

                $(
                    let encoder = $C.encode(encoder).unwrap();
                )*

                let encoder = self.depth_stencil.encode(encoder);

                encoder.finish()
            }
        }
    }
}

impl_render_target_description!(C0, C1);
impl_render_target_description!(C0, C1, C2);
impl_render_target_description!(C0, C1, C2, C3);
impl_render_target_description!(C0, C1, C2, C3, C4);
impl_render_target_description!(C0, C1, C2, C3, C4, C5);
impl_render_target_description!(C0, C1, C2, C3, C4, C5, C6);
impl_render_target_description!(C0, C1, C2, C3, C4, C5, C6, C7);
impl_render_target_description!(C0, C1, C2, C3, C4, C5, C6, C7, C8);
impl_render_target_description!(C0, C1, C2, C3, C4, C5, C6, C7, C8, C9);
impl_render_target_description!(C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10);
impl_render_target_description!(C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11);
impl_render_target_description!(C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12);
impl_render_target_description!(C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12, C13);
impl_render_target_description!(C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12, C13, C14);
impl_render_target_description!(
    C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12, C13, C14, C15
);

pub trait ColorTargetDescription {
    type Buffer: Buffer;

    fn encode<C, Ds>(
        self,
        render_target_encoder: RenderTargetEncoder<C, Ds>,
    ) -> Result<RenderTargetEncoder<(Self::Buffer, C), Ds>, MaxColorAttachmentsExceeded>;
}

impl<I> ColorTargetDescription for FloatTarget<I>
where
    I: IntoAttachment,
    I::Format: FloatRenderable,
{
    type Buffer = FloatBuffer<I::Format>;

    fn encode<C, Ds>(
        self,
        render_target_encoder: RenderTargetEncoder<C, Ds>,
    ) -> Result<RenderTargetEncoder<(Self::Buffer, C), Ds>, MaxColorAttachmentsExceeded> {
        render_target_encoder.add_color_float_buffer(self)
    }
}

impl<I> ColorTargetDescription for IntegerTarget<I>
where
    I: IntoAttachment,
    I::Format: IntegerRenderable,
{
    type Buffer = IntegerBuffer<I::Format>;

    fn encode<C, Ds>(
        self,
        render_target_encoder: RenderTargetEncoder<C, Ds>,
    ) -> Result<RenderTargetEncoder<(Self::Buffer, C), Ds>, MaxColorAttachmentsExceeded> {
        render_target_encoder.add_color_integer_buffer(self)
    }
}

impl<I> ColorTargetDescription for UnsignedIntegerTarget<I>
where
    I: IntoAttachment,
    I::Format: UnsignedIntegerRenderable,
{
    type Buffer = UnsignedIntegerBuffer<I::Format>;

    fn encode<C, Ds>(
        self,
        render_target_encoder: RenderTargetEncoder<C, Ds>,
    ) -> Result<RenderTargetEncoder<(Self::Buffer, C), Ds>, MaxColorAttachmentsExceeded> {
        render_target_encoder.add_color_unsigned_integer_buffer(self)
    }
}

pub trait DepthStencilTargetDescription {
    type Buffer: Buffer;

    fn encode<C, Ds>(
        self,
        render_target_encoder: RenderTargetEncoder<C, Ds>,
    ) -> RenderTargetEncoder<C, Self::Buffer>;
}

impl<I> DepthStencilTargetDescription for DepthStencilTarget<I>
where
    I: IntoAttachment,
    I::Format: DepthStencilRenderable,
{
    type Buffer = DepthStencilBuffer<I::Format>;

    fn encode<C, Ds>(
        self,
        render_target_encoder: RenderTargetEncoder<C, Ds>,
    ) -> RenderTargetEncoder<C, Self::Buffer> {
        render_target_encoder.set_depth_stencil_buffer(self)
    }
}

impl<I> DepthStencilTargetDescription for DepthTarget<I>
where
    I: IntoAttachment,
    I::Format: DepthRenderable,
{
    type Buffer = DepthBuffer<I::Format>;

    fn encode<C, Ds>(
        self,
        render_target_encoder: RenderTargetEncoder<C, Ds>,
    ) -> RenderTargetEncoder<C, Self::Buffer> {
        render_target_encoder.set_depth_stencil_depth_buffer(self)
    }
}

impl<I> DepthStencilTargetDescription for StencilTarget<I>
where
    I: IntoAttachment,
    I::Format: StencilRenderable,
{
    type Buffer = StencilBuffer<I::Format>;

    fn encode<C, Ds>(
        self,
        render_target_encoder: RenderTargetEncoder<C, Ds>,
    ) -> RenderTargetEncoder<C, Self::Buffer> {
        render_target_encoder.set_depth_stencil_stencil_buffer(self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Hash)]
pub struct MaxColorAttachmentsExceeded;

pub struct EncodingContext {
    render_pass_id: usize,
}

pub struct RenderTargetEncoder<C, Ds> {
    color: C,
    depth_stencil: Ds,
    data: RenderTargetEncoderData,
}

struct RenderTargetEncoderData {
    render_pass_id: usize,
    load_ops: [LoadAction; 17],
    store_ops: [StoreOp; 17],
    color_count: usize,
    color_attachments: [Option<Attachment>; 16],
    depth_stencil_attachment: DepthStencilAttachmentDescriptor,
}

impl RenderTargetEncoder<(), ()> {
    pub fn new(context: &mut EncodingContext) -> Self {
        RenderTargetEncoder {
            color: (),
            depth_stencil: (),
            data: RenderTargetEncoderData {
                render_pass_id: context.render_pass_id,
                load_ops: [LoadAction::Load; 17],
                store_ops: [StoreOp::Store; 17],
                color_count: 0,
                color_attachments: [
                    None, None, None, None, None, None, None, None, None, None, None, None, None,
                    None, None, None,
                ],
                depth_stencil_attachment: DepthStencilAttachmentDescriptor::None,
            },
        }
    }
}

impl<C, Ds> RenderTargetEncoder<C, Ds> {
    pub fn add_color_float_buffer<I>(
        mut self,
        attachment: FloatTarget<I>,
    ) -> Result<RenderTargetEncoder<(FloatBuffer<I::Format>, C), Ds>, MaxColorAttachmentsExceeded>
    where
        I: IntoAttachment,
        I::Format: FloatRenderable,
    {
        let c = self.data.color_count;

        if c > 15 {
            Err(MaxColorAttachmentsExceeded)
        } else {
            let image_descriptor = attachment.image.into_attachment();
            let width = image_descriptor.width();
            let height = image_descriptor.height();

            self.data.color_attachments[c] = Some(image_descriptor);
            self.data.load_ops[c] = attachment.load_op.as_instance(c as i32);
            self.data.store_ops[c] = attachment.store_op;
            self.data.color_count += 1;

            Ok(RenderTargetEncoder {
                color: (
                    FloatBuffer::new(self.data.render_pass_id, c as i32, width, height),
                    self.color,
                ),
                depth_stencil: self.depth_stencil,
                data: self.data,
            })
        }
    }

    pub fn add_color_integer_buffer<I>(
        mut self,
        attachment: IntegerTarget<I>,
    ) -> Result<RenderTargetEncoder<(IntegerBuffer<I::Format>, C), Ds>, MaxColorAttachmentsExceeded>
    where
        I: IntoAttachment,
        I::Format: IntegerRenderable,
    {
        let c = self.data.color_count;

        if c > 15 {
            Err(MaxColorAttachmentsExceeded)
        } else {
            let image_descriptor = attachment.image.into_attachment();
            let width = image_descriptor.width();
            let height = image_descriptor.height();

            self.data.color_attachments[c] = Some(image_descriptor);
            self.data.load_ops[c] = attachment.load_op.as_instance(c as i32);
            self.data.store_ops[c] = attachment.store_op;
            self.data.color_count += 1;

            Ok(RenderTargetEncoder {
                color: (
                    IntegerBuffer::new(self.data.render_pass_id, c as i32, width, height),
                    self.color,
                ),
                depth_stencil: self.depth_stencil,
                data: self.data,
            })
        }
    }

    pub fn add_color_unsigned_integer_buffer<I>(
        mut self,
        attachment: UnsignedIntegerTarget<I>,
    ) -> Result<
        RenderTargetEncoder<(UnsignedIntegerBuffer<I::Format>, C), Ds>,
        MaxColorAttachmentsExceeded,
    >
    where
        I: IntoAttachment,
        I::Format: UnsignedIntegerRenderable,
    {
        let c = self.data.color_count;

        if c > 15 {
            Err(MaxColorAttachmentsExceeded)
        } else {
            let image_descriptor = attachment.image.into_attachment();
            let width = image_descriptor.width();
            let height = image_descriptor.height();

            self.data.color_attachments[c] = Some(image_descriptor);
            self.data.load_ops[c] = attachment.load_op.as_instance(c as i32);
            self.data.store_ops[c] = attachment.store_op;
            self.data.color_count += 1;

            Ok(RenderTargetEncoder {
                color: (
                    UnsignedIntegerBuffer::new(self.data.render_pass_id, c as i32, width, height),
                    self.color,
                ),
                depth_stencil: self.depth_stencil,
                data: self.data,
            })
        }
    }

    pub fn set_depth_stencil_buffer<I>(
        mut self,
        attachment: DepthStencilTarget<I>,
    ) -> RenderTargetEncoder<C, DepthStencilBuffer<I::Format>>
    where
        I: IntoAttachment,
        I::Format: DepthStencilRenderable,
    {
        let image_descriptor = attachment.image.into_attachment();
        let width = image_descriptor.width();
        let height = image_descriptor.height();

        self.data.load_ops[16] = attachment.load_op.as_instance();
        self.data.store_ops[16] = attachment.store_op;
        self.data.depth_stencil_attachment =
            DepthStencilAttachmentDescriptor::DepthStencil(image_descriptor);

        RenderTargetEncoder {
            color: self.color,
            depth_stencil: DepthStencilBuffer::new(self.data.render_pass_id, width, height),
            data: self.data,
        }
    }

    pub fn set_depth_stencil_depth_buffer<I>(
        mut self,
        attachment: DepthTarget<I>,
    ) -> RenderTargetEncoder<C, DepthBuffer<I::Format>>
    where
        I: IntoAttachment,
        I::Format: DepthRenderable,
    {
        let image_descriptor = attachment.image.into_attachment();
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
        attachment: StencilTarget<I>,
    ) -> RenderTargetEncoder<C, StencilBuffer<I::Format>>
    where
        I: IntoAttachment,
        I::Format: StencilRenderable,
    {
        let image_descriptor = attachment.image.into_attachment();
        let width = image_descriptor.width();
        let height = image_descriptor.height();

        self.data.load_ops[16] = attachment.load_op.as_action();
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
//
//impl<F> RenderTargetEncoding<Framebuffer<Vec<ColorFloatBuffer<F>>, ()>>
//where
//    F: ColorFloatRenderable,
//{
//    pub fn from_float_colors<'a, 'b: 'a, C, T>(context: &mut EncodingContext, colors: C) -> Self
//    where
//        C: IntoIterator<Item = &'a FloatAttachment<'b, T>>,
//        T: AttachableImageRef<Format = F> + 'b,
//    {
//        let mut color_attachments = [None; 16];
//        let mut load_ops = [LoadOpInstance::Load; 17];
//        let mut store_ops = [StoreOp::Store; 17];
//        let mut buffers = Vec::new();
//
//        for (index, attachment) in colors.into_iter().enumerate() {
//            let descriptor = attachment.image.descriptor();
//
//            buffers.push(ColorFloatBuffer::new(
//                context.render_pass_id,
//                index as i32,
//                descriptor.width(),
//                descriptor.height(),
//            ));
//
//            color_attachments[index] = Some(descriptor);
//            load_ops[index] = attachment.load_op.as_instance(index as i32);
//            store_ops[index] = attachment.store_op;
//        }
//
//        let color_count = buffers.len();
//
//        RenderTargetEncoding {
//            framebuffer: Framebuffer {
//                color: buffers,
//                depth_stencil: (),
//                render_pass_id: context.render_pass_id,
//            },
//            data: RenderTargetEncoderData {
//                render_pass_id: context.render_pass_id,
//                load_ops,
//                store_ops,
//                color_count,
//                color_attachments,
//                depth_stencil_attachment: DepthStencilAttachmentDescriptor::None,
//            },
//        }
//    }
//}
//
//impl<F0, F1> RenderTargetEncoding<Framebuffer<Vec<ColorFloatBuffer<F0>>, DepthStencilBuffer<F1>>>
//where
//    F0: ColorFloatRenderable,
//    F1: DepthStencilRenderable,
//{
//    pub fn from_float_colors_depth_stencil<'a, 'b: 'a, C, I0, I1>(
//        context: &mut EncodingContext,
//        colors: C,
//        depth_stencil: &DepthStencilAttachment<I1>,
//    ) -> Self
//    where
//        C: IntoIterator<Item = &'a FloatAttachment<'b, I0>>,
//        I0: AttachableImageRef<Format = F0> + 'b,
//        I1: AttachableImageRef<Format = F1>,
//    {
//        let mut color_attachments = [None; 16];
//        let mut load_ops = [LoadOpInstance::Load; 17];
//        let mut store_ops = [StoreOp::Store; 17];
//        let mut buffers = Vec::new();
//
//        for (index, attachment) in colors.into_iter().enumerate() {
//            let descriptor = attachment.image.descriptor();
//
//            buffers.push(ColorFloatBuffer::new(
//                context.render_pass_id,
//                index as i32,
//                descriptor.width(),
//                descriptor.height(),
//            ));
//
//            color_attachments[index] = Some(descriptor);
//            load_ops[index] = attachment.load_op.as_instance(index as i32);
//            store_ops[index] = attachment.store_op;
//        }
//
//        let color_count = buffers.len();
//
//        load_ops[16] = depth_stencil.load_op.as_instance();
//        store_ops[16] = depth_stencil.store_op;
//
//        let depth_stencil_image = depth_stencil.image.descriptor();
//
//        RenderTargetEncoding {
//            framebuffer: Framebuffer {
//                color: buffers,
//                depth_stencil: DepthStencilBuffer::new(
//                    context.render_pass_id,
//                    depth_stencil_image.width(),
//                    depth_stencil_image.height(),
//                ),
//                render_pass_id: context.render_pass_id,
//            },
//            data: RenderTargetEncoderData {
//                render_pass_id: context.render_pass_id,
//                load_ops,
//                store_ops,
//                color_count,
//                color_attachments,
//                depth_stencil_attachment: DepthStencilAttachmentDescriptor::DepthStencil(
//                    depth_stencil_image,
//                ),
//            },
//        }
//    }
//}
//
//impl<F0, F1> RenderTargetEncoding<Framebuffer<Vec<ColorFloatBuffer<F0>>, DepthBuffer<F1>>>
//where
//    F0: ColorFloatRenderable,
//    F1: DepthRenderable,
//{
//    pub fn from_float_colors_depth<'a, 'b: 'a, C, I0, I1>(
//        context: &mut EncodingContext,
//        colors: C,
//        depth: &DepthAttachment<I1>,
//    ) -> Self
//    where
//        C: IntoIterator<Item = &'a FloatAttachment<'b, I0>>,
//        I0: AttachableImageRef<Format = F0> + 'b,
//        I1: AttachableImageRef<Format = F1>,
//    {
//        let mut color_attachments = [None; 16];
//        let mut load_ops = [LoadOpInstance::Load; 17];
//        let mut store_ops = [StoreOp::Store; 17];
//        let mut buffers = Vec::new();
//
//        for (index, attachment) in colors.into_iter().enumerate() {
//            let descriptor = attachment.image.descriptor();
//
//            buffers.push(ColorFloatBuffer::new(
//                context.render_pass_id,
//                index as i32,
//                descriptor.width(),
//                descriptor.height(),
//            ));
//
//            color_attachments[index] = Some(descriptor);
//            load_ops[index] = attachment.load_op.as_instance(index as i32);
//            store_ops[index] = attachment.store_op;
//        }
//
//        let color_count = buffers.len();
//
//        load_ops[16] = depth.load_op.as_instance();
//        store_ops[16] = depth.store_op;
//
//        let depth_image = depth.image.descriptor();
//
//        RenderTargetEncoding {
//            framebuffer: Framebuffer {
//                color: buffers,
//                depth_stencil: DepthBuffer::new(
//                    context.render_pass_id,
//                    depth_image.width(),
//                    depth_image.height(),
//                ),
//                render_pass_id: context.render_pass_id,
//            },
//            data: RenderTargetEncoderData {
//                render_pass_id: context.render_pass_id,
//                load_ops,
//                store_ops,
//                color_count,
//                color_attachments,
//                depth_stencil_attachment: DepthStencilAttachmentDescriptor::Depth(depth_image),
//            },
//        }
//    }
//}
//
//impl<F0, F1> RenderTargetEncoding<Framebuffer<Vec<ColorFloatBuffer<F0>>, StencilBuffer<F1>>>
//where
//    F0: ColorFloatRenderable,
//    F1: StencilRenderable,
//{
//    pub fn from_float_colors_stencil<'a, 'b: 'a, C, I0, I1>(
//        context: &mut EncodingContext,
//        colors: C,
//        stencil: &StencilAttachment<I1>,
//    ) -> Self
//    where
//        C: IntoIterator<Item = &'a FloatAttachment<'b, I0>>,
//        I0: AttachableImageRef<Format = F0> + 'b,
//        I1: AttachableImageRef<Format = F1>,
//    {
//        let mut color_attachments = [None; 16];
//        let mut load_ops = [LoadOpInstance::Load; 17];
//        let mut store_ops = [StoreOp::Store; 17];
//        let mut buffers = Vec::new();
//
//        for (index, attachment) in colors.into_iter().enumerate() {
//            let descriptor = attachment.image.descriptor();
//
//            buffers.push(ColorFloatBuffer::new(
//                context.render_pass_id,
//                index as i32,
//                descriptor.width(),
//                descriptor.height(),
//            ));
//
//            color_attachments[index] = Some(descriptor);
//            load_ops[index] = attachment.load_op.as_instance(index as i32);
//            store_ops[index] = attachment.store_op;
//        }
//
//        let color_count = buffers.len();
//
//        load_ops[16] = stencil.load_op.as_instance();
//        store_ops[16] = stencil.store_op;
//
//        let stencil_image = stencil.image.descriptor();
//
//        RenderTargetEncoding {
//            framebuffer: Framebuffer {
//                color: buffers,
//                depth_stencil: StencilBuffer::new(
//                    context.render_pass_id,
//                    stencil_image.width(),
//                    stencil_image.height(),
//                ),
//                render_pass_id: context.render_pass_id,
//            },
//            data: RenderTargetEncoderData {
//                render_pass_id: context.render_pass_id,
//                load_ops,
//                store_ops,
//                color_count,
//                color_attachments,
//                depth_stencil_attachment: DepthStencilAttachmentDescriptor::Stencil(stencil_image),
//            },
//        }
//    }
//}
//
//impl<F> RenderTargetEncoding<Framebuffer<Vec<ColorIntegerBuffer<F>>, ()>>
//where
//    F: ColorIntegerRenderable,
//{
//    pub fn from_integer_colors<'a, 'b: 'a, C, T>(context: &mut EncodingContext, colors: C) -> Self
//    where
//        C: IntoIterator<Item = &'a IntegerAttachment<'b, T>>,
//        T: AttachableImageRef<Format = F> + 'b,
//    {
//        let mut color_attachments = [None; 16];
//        let mut load_ops = [LoadOpInstance::Load; 17];
//        let mut store_ops = [StoreOp::Store; 17];
//        let mut buffers = Vec::new();
//
//        for (index, attachment) in colors.into_iter().enumerate() {
//            let descriptor = attachment.image.descriptor();
//
//            buffers.push(ColorIntegerBuffer::new(
//                context.render_pass_id,
//                index as i32,
//                descriptor.width(),
//                descriptor.height(),
//            ));
//
//            color_attachments[index] = Some(descriptor);
//            load_ops[index] = attachment.load_op.as_instance(index as i32);
//            store_ops[index] = attachment.store_op;
//        }
//
//        let color_count = buffers.len();
//
//        RenderTargetEncoding {
//            framebuffer: Framebuffer {
//                color: buffers,
//                depth_stencil: (),
//                render_pass_id: context.render_pass_id,
//            },
//            data: RenderTargetEncoderData {
//                render_pass_id: context.render_pass_id,
//                load_ops,
//                store_ops,
//                color_count,
//                color_attachments,
//                depth_stencil_attachment: DepthStencilAttachmentDescriptor::None,
//            },
//        }
//    }
//}
//
//impl<F0, F1> RenderTargetEncoding<Framebuffer<Vec<ColorIntegerBuffer<F0>>, DepthStencilBuffer<F1>>>
//where
//    F0: ColorIntegerRenderable,
//    F1: DepthStencilRenderable,
//{
//    pub fn from_integer_colors_depth_stencil<'a, 'b: 'a, C, I0, I1>(
//        context: &mut EncodingContext,
//        colors: C,
//        depth_stencil: &DepthStencilAttachment<I1>,
//    ) -> Self
//    where
//        C: IntoIterator<Item = &'a IntegerAttachment<'b, I0>>,
//        I0: AttachableImageRef<Format = F0> + 'b,
//        I1: AttachableImageRef<Format = F1>,
//    {
//        let mut color_attachments = [None; 16];
//        let mut load_ops = [LoadOpInstance::Load; 17];
//        let mut store_ops = [StoreOp::Store; 17];
//        let mut buffers = Vec::new();
//
//        for (index, attachment) in colors.into_iter().enumerate() {
//            let descriptor = attachment.image.descriptor();
//
//            buffers.push(ColorIntegerBuffer::new(
//                context.render_pass_id,
//                index as i32,
//                descriptor.width(),
//                descriptor.height(),
//            ));
//
//            color_attachments[index] = Some(descriptor);
//            load_ops[index] = attachment.load_op.as_instance(index as i32);
//            store_ops[index] = attachment.store_op;
//        }
//
//        let color_count = buffers.len();
//
//        load_ops[16] = depth_stencil.load_op.as_instance();
//        store_ops[16] = depth_stencil.store_op;
//
//        let depth_stencil_image = depth_stencil.image.descriptor();
//
//        RenderTargetEncoding {
//            framebuffer: Framebuffer {
//                color: buffers,
//                depth_stencil: DepthStencilBuffer::new(
//                    context.render_pass_id,
//                    depth_stencil_image.width(),
//                    depth_stencil_image.height(),
//                ),
//                render_pass_id: context.render_pass_id,
//            },
//            data: RenderTargetEncoderData {
//                render_pass_id: context.render_pass_id,
//                load_ops,
//                store_ops,
//                color_count,
//                color_attachments,
//                depth_stencil_attachment: DepthStencilAttachmentDescriptor::DepthStencil(
//                    depth_stencil_image,
//                ),
//            },
//        }
//    }
//}
//
//impl<F0, F1> RenderTargetEncoding<Framebuffer<Vec<ColorIntegerBuffer<F0>>, DepthBuffer<F1>>>
//where
//    F0: ColorIntegerRenderable,
//    F1: DepthRenderable,
//{
//    pub fn from_integer_colors_depth<'a, 'b: 'a, C, I0, I1>(
//        context: &mut EncodingContext,
//        colors: C,
//        depth: &DepthAttachment<I1>,
//    ) -> Self
//    where
//        C: IntoIterator<Item = &'a IntegerAttachment<'b, I0>>,
//        I0: AttachableImageRef<Format = F0> + 'b,
//        I1: AttachableImageRef<Format = F1>,
//    {
//        let mut color_attachments = [None; 16];
//        let mut load_ops = [LoadOpInstance::Load; 17];
//        let mut store_ops = [StoreOp::Store; 17];
//        let mut buffers = Vec::new();
//
//        for (index, attachment) in colors.into_iter().enumerate() {
//            let descriptor = attachment.image.descriptor();
//
//            buffers.push(ColorIntegerBuffer::new(
//                context.render_pass_id,
//                index as i32,
//                descriptor.width(),
//                descriptor.height(),
//            ));
//
//            color_attachments[index] = Some(descriptor);
//            load_ops[index] = attachment.load_op.as_instance(index as i32);
//            store_ops[index] = attachment.store_op;
//        }
//
//        let color_count = buffers.len();
//
//        load_ops[16] = depth.load_op.as_instance();
//        store_ops[16] = depth.store_op;
//
//        let depth_image = depth.image.descriptor();
//
//        RenderTargetEncoding {
//            framebuffer: Framebuffer {
//                color: buffers,
//                depth_stencil: DepthBuffer::new(
//                    context.render_pass_id,
//                    depth_image.width(),
//                    depth_image.height(),
//                ),
//                render_pass_id: context.render_pass_id,
//            },
//            data: RenderTargetEncoderData {
//                render_pass_id: context.render_pass_id,
//                load_ops,
//                store_ops,
//                color_count,
//                color_attachments,
//                depth_stencil_attachment: DepthStencilAttachmentDescriptor::Depth(depth_image),
//            },
//        }
//    }
//}
//
//impl<F0, F1> RenderTargetEncoding<Framebuffer<Vec<ColorIntegerBuffer<F0>>, StencilBuffer<F1>>>
//where
//    F0: ColorIntegerRenderable,
//    F1: StencilRenderable,
//{
//    pub fn from_integer_colors_stencil<'a, 'b: 'a, C, I0, I1>(
//        context: &mut EncodingContext,
//        colors: C,
//        stencil: &StencilAttachment<I1>,
//    ) -> Self
//    where
//        C: IntoIterator<Item = &'a IntegerAttachment<'b, I0>>,
//        I0: AttachableImageRef<Format = F0> + 'b,
//        I1: AttachableImageRef<Format = F1>,
//    {
//        let mut color_attachments = [None; 16];
//        let mut load_ops = [LoadOpInstance::Load; 17];
//        let mut store_ops = [StoreOp::Store; 17];
//        let mut buffers = Vec::new();
//
//        for (index, attachment) in colors.into_iter().enumerate() {
//            let descriptor = attachment.image.descriptor();
//
//            buffers.push(ColorIntegerBuffer::new(
//                context.render_pass_id,
//                index as i32,
//                descriptor.width(),
//                descriptor.height(),
//            ));
//
//            color_attachments[index] = Some(descriptor);
//            load_ops[index] = attachment.load_op.as_instance(index as i32);
//            store_ops[index] = attachment.store_op;
//        }
//
//        let color_count = buffers.len();
//
//        load_ops[16] = stencil.load_op.as_instance();
//        store_ops[16] = stencil.store_op;
//
//        let stencil_image = stencil.image.descriptor();
//
//        RenderTargetEncoding {
//            framebuffer: Framebuffer {
//                color: buffers,
//                depth_stencil: StencilBuffer::new(
//                    context.render_pass_id,
//                    stencil_image.width(),
//                    stencil_image.height(),
//                ),
//                render_pass_id: context.render_pass_id,
//            },
//            data: RenderTargetEncoderData {
//                render_pass_id: context.render_pass_id,
//                load_ops,
//                store_ops,
//                color_count,
//                color_attachments,
//                depth_stencil_attachment: DepthStencilAttachmentDescriptor::Stencil(stencil_image),
//            },
//        }
//    }
//}
//
//impl<F> RenderTargetEncoding<Framebuffer<Vec<ColorUnsignedIntegerBuffer<F>>, ()>>
//where
//    F: ColorUnsignedIntegerRenderable,
//{
//    pub fn from_unsigned_integer_colors<'a, 'b: 'a, C, T>(
//        context: &mut EncodingContext,
//        colors: C,
//    ) -> Self
//    where
//        C: IntoIterator<Item = &'a UnsignedIntegerAttachment<'b, T>>,
//        T: AttachableImageRef<Format = F> + 'b,
//    {
//        let mut color_attachments = [None; 16];
//        let mut load_ops = [LoadOpInstance::Load; 17];
//        let mut store_ops = [StoreOp::Store; 17];
//        let mut buffers = Vec::new();
//
//        for (index, attachment) in colors.into_iter().enumerate() {
//            let descriptor = attachment.image.descriptor();
//
//            buffers.push(ColorUnsignedIntegerBuffer::new(
//                context.render_pass_id,
//                index as i32,
//                descriptor.width(),
//                descriptor.height(),
//            ));
//
//            color_attachments[index] = Some(descriptor);
//            load_ops[index] = attachment.load_op.as_instance(index as i32);
//            store_ops[index] = attachment.store_op;
//        }
//
//        let color_count = buffers.len();
//
//        RenderTargetEncoding {
//            framebuffer: Framebuffer {
//                color: buffers,
//                depth_stencil: (),
//                render_pass_id: context.render_pass_id,
//            },
//            data: RenderTargetEncoderData {
//                render_pass_id: context.render_pass_id,
//                load_ops,
//                store_ops,
//                color_count,
//                color_attachments,
//                depth_stencil_attachment: DepthStencilAttachmentDescriptor::None,
//            },
//        }
//    }
//}
//
//impl<F0, F1>
//    RenderTargetEncoding<Framebuffer<Vec<ColorUnsignedIntegerBuffer<F0>>, DepthStencilBuffer<F1>>>
//where
//    F0: ColorUnsignedIntegerRenderable,
//    F1: DepthStencilRenderable,
//{
//    pub fn from_unsigned_integer_colors_depth_stencil<'a, 'b: 'a, C, I0, I1>(
//        context: &mut EncodingContext,
//        colors: C,
//        depth_stencil: &DepthStencilAttachment<I1>,
//    ) -> Self
//    where
//        C: IntoIterator<Item = &'a UnsignedIntegerAttachment<'b, I0>>,
//        I0: AttachableImageRef<Format = F0> + 'b,
//        I1: AttachableImageRef<Format = F1>,
//    {
//        let mut color_attachments = [None; 16];
//        let mut load_ops = [LoadOpInstance::Load; 17];
//        let mut store_ops = [StoreOp::Store; 17];
//        let mut buffers = Vec::new();
//
//        for (index, attachment) in colors.into_iter().enumerate() {
//            let descriptor = attachment.image.descriptor();
//
//            buffers.push(ColorUnsignedIntegerBuffer::new(
//                context.render_pass_id,
//                index as i32,
//                descriptor.width(),
//                descriptor.height(),
//            ));
//
//            color_attachments[index] = Some(descriptor);
//            load_ops[index] = attachment.load_op.as_instance(index as i32);
//            store_ops[index] = attachment.store_op;
//        }
//
//        let color_count = buffers.len();
//
//        load_ops[16] = depth_stencil.load_op.as_instance();
//        store_ops[16] = depth_stencil.store_op;
//
//        let depth_stencil_image = depth_stencil.image.descriptor();
//
//        RenderTargetEncoding {
//            framebuffer: Framebuffer {
//                color: buffers,
//                depth_stencil: DepthStencilBuffer::new(
//                    context.render_pass_id,
//                    depth_stencil_image.width(),
//                    depth_stencil_image.height(),
//                ),
//                render_pass_id: context.render_pass_id,
//            },
//            data: RenderTargetEncoderData {
//                render_pass_id: context.render_pass_id,
//                load_ops,
//                store_ops,
//                color_count,
//                color_attachments,
//                depth_stencil_attachment: DepthStencilAttachmentDescriptor::DepthStencil(
//                    depth_stencil_image,
//                ),
//            },
//        }
//    }
//}
//
//impl<F0, F1> RenderTargetEncoding<Framebuffer<Vec<ColorUnsignedIntegerBuffer<F0>>, DepthBuffer<F1>>>
//where
//    F0: ColorUnsignedIntegerRenderable,
//    F1: DepthRenderable,
//{
//    pub fn from_unsigned_integer_colors_depth<'a, 'b: 'a, C, I0, I1>(
//        context: &mut EncodingContext,
//        colors: C,
//        depth: &DepthAttachment<I1>,
//    ) -> Self
//    where
//        C: IntoIterator<Item = &'a UnsignedIntegerAttachment<'b, I0>>,
//        I0: AttachableImageRef<Format = F0> + 'b,
//        I1: AttachableImageRef<Format = F1>,
//    {
//        let mut color_attachments = [None; 16];
//        let mut load_ops = [LoadOpInstance::Load; 17];
//        let mut store_ops = [StoreOp::Store; 17];
//        let mut buffers = Vec::new();
//
//        for (index, attachment) in colors.into_iter().enumerate() {
//            let descriptor = attachment.image.descriptor();
//
//            buffers.push(ColorUnsignedIntegerBuffer::new(
//                context.render_pass_id,
//                index as i32,
//                descriptor.width(),
//                descriptor.height(),
//            ));
//
//            color_attachments[index] = Some(descriptor);
//            load_ops[index] = attachment.load_op.as_instance(index as i32);
//            store_ops[index] = attachment.store_op;
//        }
//
//        let color_count = buffers.len();
//
//        load_ops[16] = depth.load_op.as_instance();
//        store_ops[16] = depth.store_op;
//
//        let depth_image = depth.image.descriptor();
//
//        RenderTargetEncoding {
//            framebuffer: Framebuffer {
//                color: buffers,
//                depth_stencil: DepthBuffer::new(
//                    context.render_pass_id,
//                    depth_image.width(),
//                    depth_image.height(),
//                ),
//                render_pass_id: context.render_pass_id,
//            },
//            data: RenderTargetEncoderData {
//                render_pass_id: context.render_pass_id,
//                load_ops,
//                store_ops,
//                color_count,
//                color_attachments,
//                depth_stencil_attachment: DepthStencilAttachmentDescriptor::Depth(depth_image),
//            },
//        }
//    }
//}
//
//impl<F0, F1>
//    RenderTargetEncoding<Framebuffer<Vec<ColorUnsignedIntegerBuffer<F0>>, StencilBuffer<F1>>>
//where
//    F0: ColorUnsignedIntegerRenderable,
//    F1: StencilRenderable,
//{
//    pub fn from_unsigned_integer_colors_stencil<'a, 'b: 'a, C, I0, I1>(
//        context: &mut EncodingContext,
//        colors: C,
//        stencil: &StencilAttachment<I1>,
//    ) -> Self
//    where
//        C: IntoIterator<Item = &'a UnsignedIntegerAttachment<'b, I0>>,
//        I0: AttachableImageRef<Format = F0> + 'b,
//        I1: AttachableImageRef<Format = F1>,
//    {
//        let mut color_attachments = [None; 16];
//        let mut load_ops = [LoadOpInstance::Load; 17];
//        let mut store_ops = [StoreOp::Store; 17];
//        let mut buffers = Vec::new();
//
//        for (index, attachment) in colors.into_iter().enumerate() {
//            let descriptor = attachment.image.descriptor();
//
//            buffers.push(ColorUnsignedIntegerBuffer::new(
//                context.render_pass_id,
//                index as i32,
//                descriptor.width(),
//                descriptor.height(),
//            ));
//
//            color_attachments[index] = Some(descriptor);
//            load_ops[index] = attachment.load_op.as_instance(index as i32);
//            store_ops[index] = attachment.store_op;
//        }
//
//        let color_count = buffers.len();
//
//        load_ops[16] = stencil.load_op.as_instance();
//        store_ops[16] = stencil.store_op;
//
//        let stencil_image = stencil.image.descriptor();
//
//        RenderTargetEncoding {
//            framebuffer: Framebuffer {
//                color: buffers,
//                depth_stencil: StencilBuffer::new(
//                    context.render_pass_id,
//                    stencil_image.width(),
//                    stencil_image.height(),
//                ),
//                render_pass_id: context.render_pass_id,
//            },
//            data: RenderTargetEncoderData {
//                render_pass_id: context.render_pass_id,
//                load_ops,
//                store_ops,
//                color_count,
//                color_attachments,
//                depth_stencil_attachment: DepthStencilAttachmentDescriptor::Stencil(stencil_image),
//            },
//        }
//    }
//}

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
    fn color_attachments(&self) -> &[Option<Attachment>] {
        &self.data.color_attachments[0..self.data.color_count]
    }

    fn depth_stencil_attachment(&self) -> &DepthStencilAttachmentDescriptor {
        &self.data.depth_stencil_attachment
    }
}
