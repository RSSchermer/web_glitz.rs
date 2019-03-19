use std::hash::{Hash, Hasher};
use std::sync::Arc;

use js_sys::Uint32Array;
use web_sys::WebGl2RenderingContext as Gl;

use crate::image::format::{
    DepthRenderable, DepthStencilRenderable, FloatRenderable, IntegerRenderable, InternalFormat,
    RenderbufferFormat, StencilRenderable, TextureFormat, UnsignedIntegerRenderable,
};
use crate::image::renderbuffer::{Renderbuffer, RenderbufferData};
use crate::image::texture_2d::{
    Level as Texture2DLevel, LevelMut as Texture2DLevelMut, Texture2DData,
};
use crate::image::texture_2d_array::{
    LevelLayer as Texture2DArrayLevelLayer, LevelLayerMut as Texture2DArrayLevelLayerMut,
    Texture2DArrayData,
};
use crate::image::texture_3d::{
    LevelLayer as Texture3DLevelLayer, LevelLayerMut as Texture3DLevelLayerMut, Texture3DData,
};
use crate::image::texture_cube::{
    CubeFace, LevelFace as TextureCubeLevelFace, LevelFaceMut as TextureCubeLevelFaceMut,
    TextureCubeData,
};
use crate::render_pass::framebuffer::DefaultDepthBuffer;
use crate::render_pass::framebuffer::DefaultDepthStencilBuffer;
use crate::render_pass::framebuffer::DefaultRGBABuffer;
use crate::render_pass::framebuffer::DefaultRGBBuffer;
use crate::render_pass::framebuffer::DefaultStencilBuffer;
use crate::render_pass::framebuffer::{
    DepthBuffer, DepthStencilBuffer, FloatBuffer, Framebuffer, IntegerBuffer, RenderBuffer,
    StencilBuffer, UnsignedIntegerBuffer,
};
use crate::runtime::state::{
    AttachmentSet, ContextUpdate, DepthStencilAttachmentDescriptor, DrawBuffer, DynamicState,
};
use crate::runtime::Connection;
use crate::runtime::TaskContextMismatch;
use crate::task::{GpuTask, Progress, TryGpuTask, TryGpuTaskExt, ContextId};
use crate::util::{slice_make_mut, JsId};
use std::marker;

pub struct RenderPass<Fb, F> {
    id: usize,
    context_id: usize,
    framebuffer: Fb,
    render_target_encoding: RenderTargetEncodingData,
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

unsafe impl<Fb, F, T, O, E> GpuTask<Connection> for RenderPass<Fb, F>
where
    F: FnOnce(&Fb) -> T,
    for<'a> T: GpuTask<RenderPassContext<'a>, Output = O>,
    E: 'static,
{
    type Output = O;

    fn context_id(&self) -> ContextId {
        ContextId::Id(self.context_id)
    }

    fn progress(&mut self, connection: &mut Connection) -> Progress<Self::Output> {
        let (gl, state) = unsafe { connection.unpack_mut() };

        match &self.render_target_encoding {
            RenderTargetEncodingData::Default => {
                state.set_bound_draw_framebuffer(None).apply(gl).unwrap();

                let f = self.f.take().expect("Can only execute render pass once");

                f(&self.framebuffer).progress(&mut RenderPassContext {
                        connection,
                        render_pass_id: self.id,
                    })
            }
            RenderTargetEncodingData::FBO(data) => {
                state
                    .framebuffer_cache_mut()
                    .bind_or_create(data, gl)
                    .set_draw_buffers(data.draw_buffers());

                for i in 0..data.color_count {
                    data.load_ops[i].perform(gl);
                }

                if &data.depth_stencil_attachment != &DepthStencilAttachmentDescriptor::None {
                    data.load_ops[16].perform(gl);
                }

                let f = self.f.take().expect("Can only execute render pass once");

                let output = f(&self.framebuffer)
                    .progress(&mut RenderPassContext {
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

                if let Some(buffer_id) = match &data.depth_stencil_attachment {
                    DepthStencilAttachmentDescriptor::DepthStencil(_) => {
                        Some(Gl::DEPTH_STENCIL_ATTACHMENT)
                    }
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
                    let array =
                        unsafe { Uint32Array::view(&invalidate_buffers[0..invalidate_counter]) };

                    gl.invalidate_framebuffer(Gl::DRAW_FRAMEBUFFER, array.as_ref())
                        .unwrap();
                }

                output
            }
        }
    }
}

pub trait IntoFramebufferAttachment {
    type Format: InternalFormat;

    fn into_framebuffer_attachment(self) -> FramebufferAttachment;
}

impl<'a, F> IntoFramebufferAttachment for Texture2DLevelMut<'a, F>
where
    F: TextureFormat,
{
    type Format = F;

    fn into_framebuffer_attachment(self) -> FramebufferAttachment {
        FramebufferAttachment::from_texture_2d_level(&self)
    }
}

impl<'a, 'b, F> IntoFramebufferAttachment for &'a mut Texture2DLevelMut<'b, F>
where
    F: TextureFormat,
{
    type Format = F;

    fn into_framebuffer_attachment(self) -> FramebufferAttachment {
        FramebufferAttachment::from_texture_2d_level(self)
    }
}

impl<'a, F> IntoFramebufferAttachment for Texture2DArrayLevelLayerMut<'a, F>
where
    F: TextureFormat,
{
    type Format = F;

    fn into_framebuffer_attachment(self) -> FramebufferAttachment {
        FramebufferAttachment::from_texture_2d_array_level_layer(&self)
    }
}

impl<'a, 'b, F> IntoFramebufferAttachment for &'a mut Texture2DArrayLevelLayerMut<'b, F>
where
    F: TextureFormat,
{
    type Format = F;

    fn into_framebuffer_attachment(self) -> FramebufferAttachment {
        FramebufferAttachment::from_texture_2d_array_level_layer(self)
    }
}

impl<'a, F> IntoFramebufferAttachment for Texture3DLevelLayerMut<'a, F>
where
    F: TextureFormat,
{
    type Format = F;

    fn into_framebuffer_attachment(self) -> FramebufferAttachment {
        FramebufferAttachment::from_texture_3d_level_layer(&self)
    }
}

impl<'a, 'b, F> IntoFramebufferAttachment for &'a mut Texture3DLevelLayerMut<'b, F>
where
    F: TextureFormat,
{
    type Format = F;

    fn into_framebuffer_attachment(self) -> FramebufferAttachment {
        FramebufferAttachment::from_texture_3d_level_layer(self)
    }
}

impl<'a, F> IntoFramebufferAttachment for TextureCubeLevelFaceMut<'a, F>
where
    F: TextureFormat,
{
    type Format = F;

    fn into_framebuffer_attachment(self) -> FramebufferAttachment {
        FramebufferAttachment::from_texture_cube_level_face(&self)
    }
}

impl<'a, 'b, F> IntoFramebufferAttachment for &'a mut TextureCubeLevelFaceMut<'b, F>
where
    F: TextureFormat,
{
    type Format = F;

    fn into_framebuffer_attachment(self) -> FramebufferAttachment {
        FramebufferAttachment::from_texture_cube_level_face(self)
    }
}

impl<'a, F> IntoFramebufferAttachment for &'a mut Renderbuffer<F>
where
    F: RenderbufferFormat + 'static,
{
    type Format = F;

    fn into_framebuffer_attachment(self) -> FramebufferAttachment {
        FramebufferAttachment::from_renderbuffer(self)
    }
}

#[derive(Hash, PartialEq)]
pub struct FramebufferAttachment {
    internal: FramebufferAttachmentInternal,
    width: u32,
    height: u32,
}

impl FramebufferAttachment {
    pub(crate) fn from_texture_2d_level<F>(image: &Texture2DLevel<F>) -> Self
    where
        F: TextureFormat,
    {
        FramebufferAttachment {
            internal: FramebufferAttachmentInternal::Texture2DLevel {
                data: image.texture_data().clone(),
                level: image.level() as u8,
            },
            width: image.width(),
            height: image.height(),
        }
    }

    pub(crate) fn from_texture_2d_array_level_layer<F>(image: &Texture2DArrayLevelLayer<F>) -> Self
    where
        F: TextureFormat,
    {
        FramebufferAttachment {
            internal: FramebufferAttachmentInternal::Texture2DArrayLevelLayer {
                data: image.texture_data().clone(),
                level: image.level() as u8,
                layer: image.layer() as u16,
            },
            width: image.width(),
            height: image.height(),
        }
    }

    pub(crate) fn from_texture_3d_level_layer<F>(image: &Texture3DLevelLayer<F>) -> Self
    where
        F: TextureFormat,
    {
        FramebufferAttachment {
            internal: FramebufferAttachmentInternal::Texture3DLevelLayer {
                data: image.texture_data().clone(),
                level: image.level() as u8,
                layer: image.layer() as u16,
            },
            width: image.width(),
            height: image.height(),
        }
    }

    pub(crate) fn from_texture_cube_level_face<F>(image: &TextureCubeLevelFace<F>) -> Self
    where
        F: TextureFormat,
    {
        FramebufferAttachment {
            internal: FramebufferAttachmentInternal::TextureCubeLevelFace {
                data: image.texture_data().clone(),
                level: image.level() as u8,
                face: image.face(),
            },
            width: image.width(),
            height: image.height(),
        }
    }

    pub(crate) fn from_renderbuffer<F>(render_buffer: &Renderbuffer<F>) -> Self
    where
        F: RenderbufferFormat + 'static,
    {
        FramebufferAttachment {
            internal: FramebufferAttachmentInternal::Renderbuffer {
                data: render_buffer.data().clone(),
            },
            width: render_buffer.width(),
            height: render_buffer.height(),
        }
    }
}

#[derive(Hash, PartialEq)]
enum FramebufferAttachmentInternal {
    Texture2DLevel {
        data: Arc<Texture2DData>,
        level: u8,
    },
    Texture2DArrayLevelLayer {
        data: Arc<Texture2DArrayData>,
        level: u8,
        layer: u16,
    },
    Texture3DLevelLayer {
        data: Arc<Texture3DData>,
        level: u8,
        layer: u16,
    },
    TextureCubeLevelFace {
        data: Arc<TextureCubeData>,
        level: u8,
        face: CubeFace,
    },
    Renderbuffer {
        data: Arc<RenderbufferData>,
    },
}

impl FramebufferAttachment {
    pub(crate) fn id(&self) -> JsId {
        match &self.internal {
            FramebufferAttachmentInternal::Texture2DLevel { data, .. } => data.id().unwrap(),
            FramebufferAttachmentInternal::Texture2DArrayLevelLayer { data, .. } => {
                data.id().unwrap()
            }
            FramebufferAttachmentInternal::Texture3DLevelLayer { data, .. } => data.id().unwrap(),
            FramebufferAttachmentInternal::TextureCubeLevelFace { data, .. } => data.id().unwrap(),
            FramebufferAttachmentInternal::Renderbuffer { data, .. } => data.id().unwrap(),
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
            match &self.internal {
                FramebufferAttachmentInternal::Texture2DLevel { data, level } => {
                    data.id().unwrap().with_value_unchecked(|texture_object| {
                        gl.framebuffer_texture_2d(
                            target,
                            slot,
                            Gl::TEXTURE_2D,
                            Some(&texture_object),
                            *level as i32,
                        );
                    });
                }
                FramebufferAttachmentInternal::Texture2DArrayLevelLayer { data, level, layer } => {
                    data.id().unwrap().with_value_unchecked(|texture_object| {
                        gl.framebuffer_texture_layer(
                            target,
                            slot,
                            Some(&texture_object),
                            *level as i32,
                            *layer as i32,
                        );
                    });
                }
                FramebufferAttachmentInternal::Texture3DLevelLayer { data, level, layer } => {
                    data.id().unwrap().with_value_unchecked(|texture_object| {
                        gl.framebuffer_texture_layer(
                            target,
                            slot,
                            Some(&texture_object),
                            *level as i32,
                            *layer as i32,
                        );
                    });
                }
                FramebufferAttachmentInternal::TextureCubeLevelFace { data, level, face } => {
                    data.id().unwrap().with_value_unchecked(|texture_object| {
                        gl.framebuffer_texture_2d(
                            target,
                            slot,
                            face.id(),
                            Some(&texture_object),
                            *level as i32,
                        );
                    });
                }
                FramebufferAttachmentInternal::Renderbuffer { data } => {
                    data.id()
                        .unwrap()
                        .with_value_unchecked(|renderbuffer_object| {
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
    I: IntoFramebufferAttachment,
    I::Format: FloatRenderable,
{
    pub image: I,
    pub load_op: LoadOp<[f32; 4]>,
    pub store_op: StoreOp,
}

pub struct FloatTargets<I>
where
    I: IntoIterator,
    I::Item: IntoFramebufferAttachment,
    <I::Item as IntoFramebufferAttachment>::Format: FloatRenderable,
{
    pub images: I,
    pub load_op: LoadOp<[f32; 4]>,
    pub store_op: StoreOp,
}

pub struct IntegerTarget<I>
where
    I: IntoFramebufferAttachment,
    I::Format: IntegerRenderable,
{
    pub image: I,
    pub load_op: LoadOp<[i32; 4]>,
    pub store_op: StoreOp,
}

pub struct IntegerTargets<I>
where
    I: IntoIterator,
    I::Item: IntoFramebufferAttachment,
    <I::Item as IntoFramebufferAttachment>::Format: IntegerRenderable,
{
    pub images: I,
    pub load_op: LoadOp<[i32; 4]>,
    pub store_op: StoreOp,
}

pub struct UnsignedIntegerTarget<I>
where
    I: IntoFramebufferAttachment,
    I::Format: UnsignedIntegerRenderable,
{
    pub image: I,
    pub load_op: LoadOp<[u32; 4]>,
    pub store_op: StoreOp,
}

pub struct UnsignedIntegerTargets<I>
where
    I: IntoIterator,
    I::Item: IntoFramebufferAttachment,
    <I::Item as IntoFramebufferAttachment>::Format: UnsignedIntegerRenderable,
{
    pub images: I,
    pub load_op: LoadOp<[u32; 4]>,
    pub store_op: StoreOp,
}

pub struct DepthStencilTarget<I>
where
    I: IntoFramebufferAttachment,
    I::Format: DepthStencilRenderable,
{
    pub image: I,
    pub load_op: LoadOp<(f32, i32)>,
    pub store_op: StoreOp,
}

pub struct DepthTarget<I>
where
    I: IntoFramebufferAttachment,
    I::Format: DepthRenderable,
{
    pub image: I,
    pub load_op: LoadOp<f32>,
    pub store_op: StoreOp,
}

pub struct StencilTarget<I>
where
    I: IntoFramebufferAttachment,
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
    fn as_action(&self, index: i32) -> LoadAction {
        match self {
            LoadOp::Load => LoadAction::Load,
            LoadOp::Clear(value) => LoadAction::ClearColorFloat(index, *value),
        }
    }
}

impl LoadOp<[i32; 4]> {
    fn as_action(&self, index: i32) -> LoadAction {
        match self {
            LoadOp::Load => LoadAction::Load,
            LoadOp::Clear(value) => LoadAction::ClearColorInteger(index, *value),
        }
    }
}

impl LoadOp<[u32; 4]> {
    fn as_action(&self, index: i32) -> LoadAction {
        match self {
            LoadOp::Load => LoadAction::Load,
            LoadOp::Clear(value) => LoadAction::ClearColorUnsignedInteger(index, *value),
        }
    }
}

impl LoadOp<(f32, i32)> {
    fn as_action(&self) -> LoadAction {
        match self {
            LoadOp::Load => LoadAction::Load,
            LoadOp::Clear((depth, stencil)) => LoadAction::ClearDepthStencil(*depth, *stencil),
        }
    }
}

impl LoadOp<f32> {
    fn as_action(&self) -> LoadAction {
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

#[derive(Clone, Copy, PartialEq)]
pub struct DefaultRenderTarget<C, Ds> {
    context_id: usize,
    color_buffer: marker::PhantomData<C>,
    depth_stencil_buffer: marker::PhantomData<Ds>,
}

impl<C, Ds> DefaultRenderTarget<C, Ds> {
    pub(crate) fn new(context_id: usize) -> Self {
        DefaultRenderTarget {
            context_id,
            color_buffer: marker::PhantomData,
            depth_stencil_buffer: marker::PhantomData,
        }
    }
}

impl RenderTargetDescription for DefaultRenderTarget<DefaultRGBBuffer, ()> {
    type Framebuffer = Framebuffer<DefaultRGBBuffer, ()>;

    fn into_encoding(
        self,
        context: &mut EncodingContext,
    ) -> RenderTargetEncoding<Self::Framebuffer> {
        RenderTargetEncoding {
            framebuffer: Framebuffer {
                color: DefaultRGBBuffer::new(context.render_pass_id),
                depth_stencil: (),
                context_id: context.context_id,
                render_pass_id: context.render_pass_id,
            },
            context,
            data: RenderTargetEncodingData::Default,
        }
    }
}

impl RenderTargetDescription for DefaultRenderTarget<DefaultRGBBuffer, DefaultDepthStencilBuffer> {
    type Framebuffer = Framebuffer<DefaultRGBBuffer, DefaultDepthStencilBuffer>;

    fn into_encoding(
        self,
        context: &mut EncodingContext,
    ) -> RenderTargetEncoding<Self::Framebuffer> {
        RenderTargetEncoding {
            framebuffer: Framebuffer {
                color: DefaultRGBBuffer::new(context.render_pass_id),
                depth_stencil: DefaultDepthStencilBuffer::new(context.render_pass_id),
                context_id: context.context_id,
                render_pass_id: context.render_pass_id,
            },
            context,
            data: RenderTargetEncodingData::Default,
        }
    }
}

impl RenderTargetDescription for DefaultRenderTarget<DefaultRGBBuffer, DefaultDepthBuffer> {
    type Framebuffer = Framebuffer<DefaultRGBBuffer, DefaultDepthBuffer>;

    fn into_encoding(
        self,
        context: &mut EncodingContext,
    ) -> RenderTargetEncoding<Self::Framebuffer> {
        RenderTargetEncoding {
            framebuffer: Framebuffer {
                color: DefaultRGBBuffer::new(context.render_pass_id),
                depth_stencil: DefaultDepthBuffer::new(context.render_pass_id),
                context_id: context.context_id,
                render_pass_id: context.render_pass_id,
            },
            context,
            data: RenderTargetEncodingData::Default,
        }
    }
}

impl RenderTargetDescription for DefaultRenderTarget<DefaultRGBBuffer, DefaultStencilBuffer> {
    type Framebuffer = Framebuffer<DefaultRGBBuffer, DefaultStencilBuffer>;

    fn into_encoding(
        self,
        context: &mut EncodingContext,
    ) -> RenderTargetEncoding<Self::Framebuffer> {
        RenderTargetEncoding {
            framebuffer: Framebuffer {
                color: DefaultRGBBuffer::new(context.render_pass_id),
                depth_stencil: DefaultStencilBuffer::new(context.render_pass_id),
                context_id: context.context_id,
                render_pass_id: context.render_pass_id,
            },
            context,
            data: RenderTargetEncodingData::Default,
        }
    }
}

impl RenderTargetDescription for DefaultRenderTarget<DefaultRGBABuffer, ()> {
    type Framebuffer = Framebuffer<DefaultRGBABuffer, ()>;

    fn into_encoding(
        self,
        context: &mut EncodingContext,
    ) -> RenderTargetEncoding<Self::Framebuffer> {
        RenderTargetEncoding {
            framebuffer: Framebuffer {
                color: DefaultRGBABuffer::new(context.render_pass_id),
                depth_stencil: (),
                context_id: context.context_id,
                render_pass_id: context.render_pass_id,
            },
            context,
            data: RenderTargetEncodingData::Default,
        }
    }
}

impl RenderTargetDescription for DefaultRenderTarget<DefaultRGBABuffer, DefaultDepthStencilBuffer> {
    type Framebuffer = Framebuffer<DefaultRGBABuffer, DefaultDepthStencilBuffer>;

    fn into_encoding(
        self,
        context: &mut EncodingContext,
    ) -> RenderTargetEncoding<Self::Framebuffer> {
        RenderTargetEncoding {
            framebuffer: Framebuffer {
                color: DefaultRGBABuffer::new(context.render_pass_id),
                depth_stencil: DefaultDepthStencilBuffer::new(context.render_pass_id),
                context_id: context.context_id,
                render_pass_id: context.render_pass_id,
            },
            context,
            data: RenderTargetEncodingData::Default,
        }
    }
}

impl RenderTargetDescription for DefaultRenderTarget<DefaultRGBABuffer, DefaultDepthBuffer> {
    type Framebuffer = Framebuffer<DefaultRGBABuffer, DefaultDepthBuffer>;

    fn into_encoding(
        self,
        context: &mut EncodingContext,
    ) -> RenderTargetEncoding<Self::Framebuffer> {
        RenderTargetEncoding {
            framebuffer: Framebuffer {
                color: DefaultRGBABuffer::new(context.render_pass_id),
                depth_stencil: DefaultDepthBuffer::new(context.render_pass_id),
                context_id: context.context_id,
                render_pass_id: context.render_pass_id,
            },
            context,
            data: RenderTargetEncodingData::Default,
        }
    }
}

impl RenderTargetDescription for DefaultRenderTarget<DefaultRGBABuffer, DefaultStencilBuffer> {
    type Framebuffer = Framebuffer<DefaultRGBABuffer, DefaultStencilBuffer>;

    fn into_encoding(
        self,
        context: &mut EncodingContext,
    ) -> RenderTargetEncoding<Self::Framebuffer> {
        RenderTargetEncoding {
            framebuffer: Framebuffer {
                color: DefaultRGBABuffer::new(context.render_pass_id),
                depth_stencil: DefaultStencilBuffer::new(context.render_pass_id),
                context_id: context.context_id,
                render_pass_id: context.render_pass_id,
            },
            context,
            data: RenderTargetEncodingData::Default,
        }
    }
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

impl<I, F> RenderTargetDescription for RenderTarget<FloatTargets<I>, ()>
where
    I: IntoIterator,
    I::Item: IntoFramebufferAttachment<Format = F>,
    F: FloatRenderable,
{
    type Framebuffer = Framebuffer<Vec<FloatBuffer<F>>, ()>;

    fn into_encoding(
        self,
        context: &mut EncodingContext,
    ) -> RenderTargetEncoding<Self::Framebuffer> {
        let load_op = self.color.load_op;
        let store_op = self.color.store_op;

        let colors = self.color.images.into_iter().map(|image| FloatTarget {
            image,
            load_op,
            store_op,
        });

        RenderTargetEncoding::from_float_colors(context, colors)
    }
}

impl<I, Ds, F0, F1> RenderTargetDescription
    for RenderTarget<FloatTargets<I>, DepthStencilTarget<Ds>>
where
    I: IntoIterator,
    I::Item: IntoFramebufferAttachment<Format = F0>,
    F0: FloatRenderable,
    Ds: IntoFramebufferAttachment<Format = F1>,
    F1: DepthStencilRenderable,
{
    type Framebuffer = Framebuffer<Vec<FloatBuffer<F0>>, DepthStencilBuffer<F1>>;

    fn into_encoding(
        self,
        context: &mut EncodingContext,
    ) -> RenderTargetEncoding<Self::Framebuffer> {
        let load_op = self.color.load_op;
        let store_op = self.color.store_op;

        let colors = self.color.images.into_iter().map(|image| FloatTarget {
            image,
            load_op,
            store_op,
        });

        RenderTargetEncoding::from_float_colors_and_depth_stencil(
            context,
            colors,
            self.depth_stencil,
        )
    }
}

impl<I, Ds, F0, F1> RenderTargetDescription for RenderTarget<FloatTargets<I>, DepthTarget<Ds>>
where
    I: IntoIterator,
    I::Item: IntoFramebufferAttachment<Format = F0>,
    F0: FloatRenderable,
    Ds: IntoFramebufferAttachment<Format = F1>,
    F1: DepthRenderable,
{
    type Framebuffer = Framebuffer<Vec<FloatBuffer<F0>>, DepthBuffer<F1>>;

    fn into_encoding(
        self,
        context: &mut EncodingContext,
    ) -> RenderTargetEncoding<Self::Framebuffer> {
        let load_op = self.color.load_op;
        let store_op = self.color.store_op;

        let colors = self.color.images.into_iter().map(|image| FloatTarget {
            image,
            load_op,
            store_op,
        });

        RenderTargetEncoding::from_float_colors_and_depth(context, colors, self.depth_stencil)
    }
}

impl<I, Ds, F0, F1> RenderTargetDescription for RenderTarget<FloatTargets<I>, StencilTarget<Ds>>
where
    I: IntoIterator,
    I::Item: IntoFramebufferAttachment<Format = F0>,
    F0: FloatRenderable,
    Ds: IntoFramebufferAttachment<Format = F1>,
    F1: StencilRenderable,
{
    type Framebuffer = Framebuffer<Vec<FloatBuffer<F0>>, StencilBuffer<F1>>;

    fn into_encoding(
        self,
        context: &mut EncodingContext,
    ) -> RenderTargetEncoding<Self::Framebuffer> {
        let load_op = self.color.load_op;
        let store_op = self.color.store_op;

        let colors = self.color.images.into_iter().map(|image| FloatTarget {
            image,
            load_op,
            store_op,
        });

        RenderTargetEncoding::from_float_colors_and_stencil(context, colors, self.depth_stencil)
    }
}

impl<I, F> RenderTargetDescription for RenderTarget<IntegerTargets<I>, ()>
where
    I: IntoIterator,
    I::Item: IntoFramebufferAttachment<Format = F>,
    F: IntegerRenderable,
{
    type Framebuffer = Framebuffer<Vec<IntegerBuffer<F>>, ()>;

    fn into_encoding(
        self,
        context: &mut EncodingContext,
    ) -> RenderTargetEncoding<Self::Framebuffer> {
        let load_op = self.color.load_op;
        let store_op = self.color.store_op;

        let colors = self.color.images.into_iter().map(|image| IntegerTarget {
            image,
            load_op,
            store_op,
        });

        RenderTargetEncoding::from_integer_colors(context, colors)
    }
}

impl<I, Ds, F0, F1> RenderTargetDescription
    for RenderTarget<IntegerTargets<I>, DepthStencilTarget<Ds>>
where
    I: IntoIterator,
    I::Item: IntoFramebufferAttachment<Format = F0>,
    F0: IntegerRenderable,
    Ds: IntoFramebufferAttachment<Format = F1>,
    F1: DepthStencilRenderable,
{
    type Framebuffer = Framebuffer<Vec<IntegerBuffer<F0>>, DepthStencilBuffer<F1>>;

    fn into_encoding(
        self,
        context: &mut EncodingContext,
    ) -> RenderTargetEncoding<Self::Framebuffer> {
        let load_op = self.color.load_op;
        let store_op = self.color.store_op;

        let colors = self.color.images.into_iter().map(|image| IntegerTarget {
            image,
            load_op,
            store_op,
        });

        RenderTargetEncoding::from_integer_colors_and_depth_stencil(
            context,
            colors,
            self.depth_stencil,
        )
    }
}

impl<I, Ds, F0, F1> RenderTargetDescription for RenderTarget<IntegerTargets<I>, DepthTarget<Ds>>
where
    I: IntoIterator,
    I::Item: IntoFramebufferAttachment<Format = F0>,
    F0: IntegerRenderable,
    Ds: IntoFramebufferAttachment<Format = F1>,
    F1: DepthRenderable,
{
    type Framebuffer = Framebuffer<Vec<IntegerBuffer<F0>>, DepthBuffer<F1>>;

    fn into_encoding(
        self,
        context: &mut EncodingContext,
    ) -> RenderTargetEncoding<Self::Framebuffer> {
        let load_op = self.color.load_op;
        let store_op = self.color.store_op;

        let colors = self.color.images.into_iter().map(|image| IntegerTarget {
            image,
            load_op,
            store_op,
        });

        RenderTargetEncoding::from_integer_colors_and_depth(context, colors, self.depth_stencil)
    }
}

impl<I, Ds, F0, F1> RenderTargetDescription for RenderTarget<IntegerTargets<I>, StencilTarget<Ds>>
where
    I: IntoIterator,
    I::Item: IntoFramebufferAttachment<Format = F0>,
    F0: IntegerRenderable,
    Ds: IntoFramebufferAttachment<Format = F1>,
    F1: StencilRenderable,
{
    type Framebuffer = Framebuffer<Vec<IntegerBuffer<F0>>, StencilBuffer<F1>>;

    fn into_encoding(
        self,
        context: &mut EncodingContext,
    ) -> RenderTargetEncoding<Self::Framebuffer> {
        let load_op = self.color.load_op;
        let store_op = self.color.store_op;

        let colors = self.color.images.into_iter().map(|image| IntegerTarget {
            image,
            load_op,
            store_op,
        });

        RenderTargetEncoding::from_integer_colors_and_stencil(context, colors, self.depth_stencil)
    }
}

impl<I, F> RenderTargetDescription for RenderTarget<UnsignedIntegerTargets<I>, ()>
where
    I: IntoIterator,
    I::Item: IntoFramebufferAttachment<Format = F>,
    F: UnsignedIntegerRenderable,
{
    type Framebuffer = Framebuffer<Vec<UnsignedIntegerBuffer<F>>, ()>;

    fn into_encoding(
        self,
        context: &mut EncodingContext,
    ) -> RenderTargetEncoding<Self::Framebuffer> {
        let load_op = self.color.load_op;
        let store_op = self.color.store_op;

        let colors = self
            .color
            .images
            .into_iter()
            .map(|image| UnsignedIntegerTarget {
                image,
                load_op,
                store_op,
            });

        RenderTargetEncoding::from_unsigned_integer_colors(context, colors)
    }
}

impl<I, Ds, F0, F1> RenderTargetDescription
    for RenderTarget<UnsignedIntegerTargets<I>, DepthStencilTarget<Ds>>
where
    I: IntoIterator,
    I::Item: IntoFramebufferAttachment<Format = F0>,
    F0: UnsignedIntegerRenderable,
    Ds: IntoFramebufferAttachment<Format = F1>,
    F1: DepthStencilRenderable,
{
    type Framebuffer = Framebuffer<Vec<UnsignedIntegerBuffer<F0>>, DepthStencilBuffer<F1>>;

    fn into_encoding(
        self,
        context: &mut EncodingContext,
    ) -> RenderTargetEncoding<Self::Framebuffer> {
        let load_op = self.color.load_op;
        let store_op = self.color.store_op;

        let colors = self
            .color
            .images
            .into_iter()
            .map(|image| UnsignedIntegerTarget {
                image,
                load_op,
                store_op,
            });

        RenderTargetEncoding::from_unsigned_integer_colors_and_depth_stencil(
            context,
            colors,
            self.depth_stencil,
        )
    }
}

impl<I, Ds, F0, F1> RenderTargetDescription
    for RenderTarget<UnsignedIntegerTargets<I>, DepthTarget<Ds>>
where
    I: IntoIterator,
    I::Item: IntoFramebufferAttachment<Format = F0>,
    F0: UnsignedIntegerRenderable,
    Ds: IntoFramebufferAttachment<Format = F1>,
    F1: DepthRenderable,
{
    type Framebuffer = Framebuffer<Vec<UnsignedIntegerBuffer<F0>>, DepthBuffer<F1>>;

    fn into_encoding(
        self,
        context: &mut EncodingContext,
    ) -> RenderTargetEncoding<Self::Framebuffer> {
        let load_op = self.color.load_op;
        let store_op = self.color.store_op;

        let colors = self
            .color
            .images
            .into_iter()
            .map(|image| UnsignedIntegerTarget {
                image,
                load_op,
                store_op,
            });

        RenderTargetEncoding::from_unsigned_integer_colors_and_depth(
            context,
            colors,
            self.depth_stencil,
        )
    }
}

impl<I, Ds, F0, F1> RenderTargetDescription
    for RenderTarget<UnsignedIntegerTargets<I>, StencilTarget<Ds>>
where
    I: IntoIterator,
    I::Item: IntoFramebufferAttachment<Format = F0>,
    F0: UnsignedIntegerRenderable,
    Ds: IntoFramebufferAttachment<Format = F1>,
    F1: StencilRenderable,
{
    type Framebuffer = Framebuffer<Vec<UnsignedIntegerBuffer<F0>>, StencilBuffer<F1>>;

    fn into_encoding(
        self,
        context: &mut EncodingContext,
    ) -> RenderTargetEncoding<Self::Framebuffer> {
        let load_op = self.color.load_op;
        let store_op = self.color.store_op;

        let colors = self
            .color
            .images
            .into_iter()
            .map(|image| UnsignedIntegerTarget {
                image,
                load_op,
                store_op,
            });

        RenderTargetEncoding::from_unsigned_integer_colors_and_stencil(
            context,
            colors,
            self.depth_stencil,
        )
    }
}

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
    type Buffer: RenderBuffer;

    fn encode<C, Ds>(
        self,
        render_target_encoder: RenderTargetEncoder<C, Ds>,
    ) -> Result<RenderTargetEncoder<(Self::Buffer, C), Ds>, MaxColorAttachmentsExceeded>;
}

impl<I> ColorTargetDescription for FloatTarget<I>
where
    I: IntoFramebufferAttachment,
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
    I: IntoFramebufferAttachment,
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
    I: IntoFramebufferAttachment,
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
    type Buffer: RenderBuffer;

    fn encode<C, Ds>(
        self,
        render_target_encoder: RenderTargetEncoder<C, Ds>,
    ) -> RenderTargetEncoder<C, Self::Buffer>;
}

impl<I> DepthStencilTargetDescription for DepthStencilTarget<I>
where
    I: IntoFramebufferAttachment,
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
    I: IntoFramebufferAttachment,
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
    I: IntoFramebufferAttachment,
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
    context_id: usize,
    render_pass_id: usize,
}

pub struct RenderTargetEncoder<'a, C, Ds> {
    context: &'a mut EncodingContext,
    color: C,
    depth_stencil: Ds,
    data: FBOEncodingData,
}

enum RenderTargetEncodingData {
    Default,
    FBO(FBOEncodingData),
}

struct FBOEncodingData {
    load_ops: [LoadAction; 17],
    store_ops: [StoreOp; 17],
    color_count: usize,
    color_attachments: [Option<FramebufferAttachment>; 16],
    depth_stencil_attachment: DepthStencilAttachmentDescriptor,
}

impl<'a> RenderTargetEncoder<'a, (), ()> {
    pub fn new(context: &'a mut EncodingContext) -> Self {
        RenderTargetEncoder {
            context,
            color: (),
            depth_stencil: (),
            data: FBOEncodingData {
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

impl<'a, C, Ds> RenderTargetEncoder<'a, C, Ds> {
    pub fn add_color_float_buffer<I>(
        mut self,
        attachment: FloatTarget<I>,
    ) -> Result<RenderTargetEncoder<'a, (FloatBuffer<I::Format>, C), Ds>, MaxColorAttachmentsExceeded>
    where
        I: IntoFramebufferAttachment,
        I::Format: FloatRenderable,
    {
        let c = self.data.color_count;

        if c > 15 {
            Err(MaxColorAttachmentsExceeded)
        } else {
            let image_descriptor = attachment.image.into_framebuffer_attachment();
            let width = image_descriptor.width();
            let height = image_descriptor.height();

            self.data.color_attachments[c] = Some(image_descriptor);
            self.data.load_ops[c] = attachment.load_op.as_action(c as i32);
            self.data.store_ops[c] = attachment.store_op;
            self.data.color_count += 1;

            Ok(RenderTargetEncoder {
                color: (
                    FloatBuffer::new(self.context.render_pass_id, c as i32, width, height),
                    self.color,
                ),
                context: self.context,
                depth_stencil: self.depth_stencil,
                data: self.data,
            })
        }
    }

    pub fn add_color_integer_buffer<I>(
        mut self,
        attachment: IntegerTarget<I>,
    ) -> Result<
        RenderTargetEncoder<'a, (IntegerBuffer<I::Format>, C), Ds>,
        MaxColorAttachmentsExceeded,
    >
    where
        I: IntoFramebufferAttachment,
        I::Format: IntegerRenderable,
    {
        let c = self.data.color_count;

        if c > 15 {
            Err(MaxColorAttachmentsExceeded)
        } else {
            let image_descriptor = attachment.image.into_framebuffer_attachment();
            let width = image_descriptor.width();
            let height = image_descriptor.height();

            self.data.color_attachments[c] = Some(image_descriptor);
            self.data.load_ops[c] = attachment.load_op.as_action(c as i32);
            self.data.store_ops[c] = attachment.store_op;
            self.data.color_count += 1;

            Ok(RenderTargetEncoder {
                color: (
                    IntegerBuffer::new(self.context.render_pass_id, c as i32, width, height),
                    self.color,
                ),
                context: self.context,
                depth_stencil: self.depth_stencil,
                data: self.data,
            })
        }
    }

    pub fn add_color_unsigned_integer_buffer<I>(
        mut self,
        attachment: UnsignedIntegerTarget<I>,
    ) -> Result<
        RenderTargetEncoder<'a, (UnsignedIntegerBuffer<I::Format>, C), Ds>,
        MaxColorAttachmentsExceeded,
    >
    where
        I: IntoFramebufferAttachment,
        I::Format: UnsignedIntegerRenderable,
    {
        let c = self.data.color_count;

        if c > 15 {
            Err(MaxColorAttachmentsExceeded)
        } else {
            let image_descriptor = attachment.image.into_framebuffer_attachment();
            let width = image_descriptor.width();
            let height = image_descriptor.height();

            self.data.color_attachments[c] = Some(image_descriptor);
            self.data.load_ops[c] = attachment.load_op.as_action(c as i32);
            self.data.store_ops[c] = attachment.store_op;
            self.data.color_count += 1;

            Ok(RenderTargetEncoder {
                color: (
                    UnsignedIntegerBuffer::new(
                        self.context.render_pass_id,
                        c as i32,
                        width,
                        height,
                    ),
                    self.color,
                ),
                context: self.context,
                depth_stencil: self.depth_stencil,
                data: self.data,
            })
        }
    }

    pub fn set_depth_stencil_buffer<I>(
        mut self,
        attachment: DepthStencilTarget<I>,
    ) -> RenderTargetEncoder<'a, C, DepthStencilBuffer<I::Format>>
    where
        I: IntoFramebufferAttachment,
        I::Format: DepthStencilRenderable,
    {
        let image_descriptor = attachment.image.into_framebuffer_attachment();
        let width = image_descriptor.width();
        let height = image_descriptor.height();

        self.data.load_ops[16] = attachment.load_op.as_action();
        self.data.store_ops[16] = attachment.store_op;
        self.data.depth_stencil_attachment =
            DepthStencilAttachmentDescriptor::DepthStencil(image_descriptor);

        RenderTargetEncoder {
            color: self.color,
            depth_stencil: DepthStencilBuffer::new(self.context.render_pass_id, width, height),
            data: self.data,
            context: self.context,
        }
    }

    pub fn set_depth_stencil_depth_buffer<I>(
        mut self,
        attachment: DepthTarget<I>,
    ) -> RenderTargetEncoder<'a, C, DepthBuffer<I::Format>>
    where
        I: IntoFramebufferAttachment,
        I::Format: DepthRenderable,
    {
        let image_descriptor = attachment.image.into_framebuffer_attachment();
        let width = image_descriptor.width();
        let height = image_descriptor.height();

        self.data.load_ops[16] = attachment.load_op.as_action();
        self.data.store_ops[16] = attachment.store_op;
        self.data.depth_stencil_attachment =
            DepthStencilAttachmentDescriptor::Depth(image_descriptor);

        RenderTargetEncoder {
            color: self.color,
            depth_stencil: DepthBuffer::new(self.context.render_pass_id, width, height),
            data: self.data,
            context: self.context,
        }
    }

    pub fn set_depth_stencil_stencil_buffer<I>(
        mut self,
        attachment: StencilTarget<I>,
    ) -> RenderTargetEncoder<'a, C, StencilBuffer<I::Format>>
    where
        I: IntoFramebufferAttachment,
        I::Format: StencilRenderable,
    {
        let image_descriptor = attachment.image.into_framebuffer_attachment();
        let width = image_descriptor.width();
        let height = image_descriptor.height();

        self.data.load_ops[16] = attachment.load_op.as_action();
        self.data.store_ops[16] = attachment.store_op;
        self.data.depth_stencil_attachment =
            DepthStencilAttachmentDescriptor::Stencil(image_descriptor);

        RenderTargetEncoder {
            color: self.color,
            depth_stencil: StencilBuffer::new(self.context.render_pass_id, width, height),
            data: self.data,
            context: self.context,
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
        impl<'a, $($C),*> RenderTargetEncoder<'a, nest_pairs_reverse!([(), $($C),*]), ()>
            where $($C: Buffer),*
        {
            pub fn finish(self) -> RenderTargetEncoding<'a, Framebuffer<($($C),*), ()>> {
                #[allow(non_snake_case)]
                let nest_pairs_reverse!([_, $($C),*]) = self.color;

                RenderTargetEncoding {
                    framebuffer: Framebuffer {
                        color: ($($C),*),
                        depth_stencil: (),
                        context_id: self.context.context_id,
                        render_pass_id: self.context.render_pass_id,
                    },
                    context: self.context,
                    data: RenderTargetEncodingData::FBO(self.data),
                }
            }
        }

        impl<'a, $($C),*, Ds> RenderTargetEncoder<'a, nest_pairs_reverse!([(), $($C),*]), Ds>
        where
            $($C: Buffer),*,
            Ds: Buffer
        {
            pub fn finish(self) -> RenderTargetEncoding<'a, Framebuffer<($($C),*), Ds>> {
                #[allow(non_snake_case)]
                let nest_pairs_reverse!([_, $($C),*]) = self.color;

                RenderTargetEncoding {
                    framebuffer: Framebuffer {
                        color: ($($C),*),
                        depth_stencil: self.depth_stencil,
                        context_id: self.context.context_id,
                        render_pass_id: self.context.render_pass_id,
                    },
                    context: self.context,
                    data: RenderTargetEncodingData::FBO(self.data),
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

pub struct RenderTargetEncoding<'a, F> {
    context: &'a mut EncodingContext,
    framebuffer: F,
    data: RenderTargetEncodingData,
}

impl<'a, F> RenderTargetEncoding<'a, Framebuffer<Vec<FloatBuffer<F>>, ()>>
where
    F: FloatRenderable,
{
    pub fn from_float_colors<C, I>(context: &'a mut EncodingContext, colors: C) -> Self
    where
        C: IntoIterator<Item = FloatTarget<I>>,
        I: IntoFramebufferAttachment<Format = F>,
    {
        let mut color_attachments = [
            None, None, None, None, None, None, None, None, None, None, None, None, None, None,
            None, None,
        ];
        let mut load_ops = [LoadAction::Load; 17];
        let mut store_ops = [StoreOp::Store; 17];
        let mut buffers = Vec::new();

        for (index, target) in colors.into_iter().enumerate() {
            let attachment = target.image.into_framebuffer_attachment();

            buffers.push(FloatBuffer::new(
                context.render_pass_id,
                index as i32,
                attachment.width(),
                attachment.height(),
            ));

            color_attachments[index] = Some(attachment);
            load_ops[index] = target.load_op.as_action(index as i32);
            store_ops[index] = target.store_op;
        }

        let color_count = buffers.len();

        RenderTargetEncoding {
            framebuffer: Framebuffer {
                color: buffers,
                depth_stencil: (),
                context_id: context.context_id,
                render_pass_id: context.render_pass_id,
            },
            data: RenderTargetEncodingData::FBO(FBOEncodingData {
                load_ops,
                store_ops,
                color_count,
                color_attachments,
                depth_stencil_attachment: DepthStencilAttachmentDescriptor::None,
            }),
            context,
        }
    }
}

impl<'a, F0, F1> RenderTargetEncoding<'a, Framebuffer<Vec<FloatBuffer<F0>>, DepthStencilBuffer<F1>>>
where
    F0: FloatRenderable,
    F1: DepthStencilRenderable,
{
    pub fn from_float_colors_and_depth_stencil<C, I, Ds>(
        context: &'a mut EncodingContext,
        colors: C,
        depth_stencil: DepthStencilTarget<Ds>,
    ) -> Self
    where
        C: IntoIterator<Item = FloatTarget<I>>,
        I: IntoFramebufferAttachment<Format = F0>,
        Ds: IntoFramebufferAttachment<Format = F1>,
    {
        let mut color_attachments = [
            None, None, None, None, None, None, None, None, None, None, None, None, None, None,
            None, None,
        ];
        let mut load_ops = [LoadAction::Load; 17];
        let mut store_ops = [StoreOp::Store; 17];
        let mut buffers = Vec::new();

        for (index, target) in colors.into_iter().enumerate() {
            let attachment = target.image.into_framebuffer_attachment();

            buffers.push(FloatBuffer::new(
                context.render_pass_id,
                index as i32,
                attachment.width(),
                attachment.height(),
            ));

            color_attachments[index] = Some(attachment);
            load_ops[index] = target.load_op.as_action(index as i32);
            store_ops[index] = target.store_op;
        }

        let color_count = buffers.len();

        load_ops[16] = depth_stencil.load_op.as_action();
        store_ops[16] = depth_stencil.store_op;

        let depth_stencil_attachment = depth_stencil.image.into_framebuffer_attachment();

        RenderTargetEncoding {
            framebuffer: Framebuffer {
                color: buffers,
                depth_stencil: DepthStencilBuffer::new(
                    context.render_pass_id,
                    depth_stencil_attachment.width(),
                    depth_stencil_attachment.height(),
                ),
                context_id: context.context_id,
                render_pass_id: context.render_pass_id,
            },
            data: RenderTargetEncodingData::FBO(FBOEncodingData {
                load_ops,
                store_ops,
                color_count,
                color_attachments,
                depth_stencil_attachment: DepthStencilAttachmentDescriptor::DepthStencil(
                    depth_stencil_attachment,
                ),
            }),
            context,
        }
    }
}

impl<'a, F0, F1> RenderTargetEncoding<'a, Framebuffer<Vec<FloatBuffer<F0>>, DepthBuffer<F1>>>
where
    F0: FloatRenderable,
    F1: DepthRenderable,
{
    pub fn from_float_colors_and_depth<C, I, Ds>(
        context: &'a mut EncodingContext,
        colors: C,
        depth: DepthTarget<Ds>,
    ) -> Self
    where
        C: IntoIterator<Item = FloatTarget<I>>,
        I: IntoFramebufferAttachment<Format = F0>,
        Ds: IntoFramebufferAttachment<Format = F1>,
    {
        let mut color_attachments = [
            None, None, None, None, None, None, None, None, None, None, None, None, None, None,
            None, None,
        ];
        let mut load_ops = [LoadAction::Load; 17];
        let mut store_ops = [StoreOp::Store; 17];
        let mut buffers = Vec::new();

        for (index, target) in colors.into_iter().enumerate() {
            let attachment = target.image.into_framebuffer_attachment();

            buffers.push(FloatBuffer::new(
                context.render_pass_id,
                index as i32,
                attachment.width(),
                attachment.height(),
            ));

            color_attachments[index] = Some(attachment);
            load_ops[index] = target.load_op.as_action(index as i32);
            store_ops[index] = target.store_op;
        }

        let color_count = buffers.len();

        load_ops[16] = depth.load_op.as_action();
        store_ops[16] = depth.store_op;

        let depth_attachment = depth.image.into_framebuffer_attachment();

        RenderTargetEncoding {
            framebuffer: Framebuffer {
                color: buffers,
                depth_stencil: DepthBuffer::new(
                    context.render_pass_id,
                    depth_attachment.width(),
                    depth_attachment.height(),
                ),
                context_id: context.context_id,
                render_pass_id: context.render_pass_id,
            },
            data: RenderTargetEncodingData::FBO(FBOEncodingData {
                load_ops,
                store_ops,
                color_count,
                color_attachments,
                depth_stencil_attachment: DepthStencilAttachmentDescriptor::Depth(depth_attachment),
            }),
            context,
        }
    }
}

impl<'a, F0, F1> RenderTargetEncoding<'a, Framebuffer<Vec<FloatBuffer<F0>>, StencilBuffer<F1>>>
where
    F0: FloatRenderable,
    F1: StencilRenderable,
{
    pub fn from_float_colors_and_stencil<C, I, Ds>(
        context: &'a mut EncodingContext,
        colors: C,
        stencil: StencilTarget<Ds>,
    ) -> Self
    where
        C: IntoIterator<Item = FloatTarget<I>>,
        I: IntoFramebufferAttachment<Format = F0>,
        Ds: IntoFramebufferAttachment<Format = F1>,
    {
        let mut color_attachments = [
            None, None, None, None, None, None, None, None, None, None, None, None, None, None,
            None, None,
        ];
        let mut load_ops = [LoadAction::Load; 17];
        let mut store_ops = [StoreOp::Store; 17];
        let mut buffers = Vec::new();

        for (index, target) in colors.into_iter().enumerate() {
            let attachment = target.image.into_framebuffer_attachment();

            buffers.push(FloatBuffer::new(
                context.render_pass_id,
                index as i32,
                attachment.width(),
                attachment.height(),
            ));

            color_attachments[index] = Some(attachment);
            load_ops[index] = target.load_op.as_action(index as i32);
            store_ops[index] = target.store_op;
        }

        let color_count = buffers.len();

        load_ops[16] = stencil.load_op.as_action();
        store_ops[16] = stencil.store_op;

        let stencil_attachment = stencil.image.into_framebuffer_attachment();

        RenderTargetEncoding {
            framebuffer: Framebuffer {
                color: buffers,
                depth_stencil: StencilBuffer::new(
                    context.render_pass_id,
                    stencil_attachment.width(),
                    stencil_attachment.height(),
                ),
                context_id: context.context_id,
                render_pass_id: context.render_pass_id,
            },
            data: RenderTargetEncodingData::FBO(FBOEncodingData {
                load_ops,
                store_ops,
                color_count,
                color_attachments,
                depth_stencil_attachment: DepthStencilAttachmentDescriptor::Stencil(
                    stencil_attachment,
                ),
            }),
            context,
        }
    }
}

impl<'a, F> RenderTargetEncoding<'a, Framebuffer<Vec<IntegerBuffer<F>>, ()>>
where
    F: IntegerRenderable,
{
    pub fn from_integer_colors<C, I>(context: &'a mut EncodingContext, colors: C) -> Self
    where
        C: IntoIterator<Item = IntegerTarget<I>>,
        I: IntoFramebufferAttachment<Format = F>,
    {
        let mut color_attachments = [
            None, None, None, None, None, None, None, None, None, None, None, None, None, None,
            None, None,
        ];
        let mut load_ops = [LoadAction::Load; 17];
        let mut store_ops = [StoreOp::Store; 17];
        let mut buffers = Vec::new();

        for (index, target) in colors.into_iter().enumerate() {
            let attachment = target.image.into_framebuffer_attachment();

            buffers.push(IntegerBuffer::new(
                context.render_pass_id,
                index as i32,
                attachment.width(),
                attachment.height(),
            ));

            color_attachments[index] = Some(attachment);
            load_ops[index] = target.load_op.as_action(index as i32);
            store_ops[index] = target.store_op;
        }

        let color_count = buffers.len();

        RenderTargetEncoding {
            framebuffer: Framebuffer {
                color: buffers,
                depth_stencil: (),
                context_id: context.context_id,
                render_pass_id: context.render_pass_id,
            },
            data: RenderTargetEncodingData::FBO(FBOEncodingData {
                load_ops,
                store_ops,
                color_count,
                color_attachments,
                depth_stencil_attachment: DepthStencilAttachmentDescriptor::None,
            }),
            context,
        }
    }
}

impl<'a, F0, F1>
    RenderTargetEncoding<'a, Framebuffer<Vec<IntegerBuffer<F0>>, DepthStencilBuffer<F1>>>
where
    F0: IntegerRenderable,
    F1: DepthStencilRenderable,
{
    pub fn from_integer_colors_and_depth_stencil<C, I, Ds>(
        context: &'a mut EncodingContext,
        colors: C,
        depth_stencil: DepthStencilTarget<Ds>,
    ) -> Self
    where
        C: IntoIterator<Item = IntegerTarget<I>>,
        I: IntoFramebufferAttachment<Format = F0>,
        Ds: IntoFramebufferAttachment<Format = F1>,
    {
        let mut color_attachments = [
            None, None, None, None, None, None, None, None, None, None, None, None, None, None,
            None, None,
        ];
        let mut load_ops = [LoadAction::Load; 17];
        let mut store_ops = [StoreOp::Store; 17];
        let mut buffers = Vec::new();

        for (index, target) in colors.into_iter().enumerate() {
            let attachment = target.image.into_framebuffer_attachment();

            buffers.push(IntegerBuffer::new(
                context.render_pass_id,
                index as i32,
                attachment.width(),
                attachment.height(),
            ));

            color_attachments[index] = Some(attachment);
            load_ops[index] = target.load_op.as_action(index as i32);
            store_ops[index] = target.store_op;
        }

        let color_count = buffers.len();

        load_ops[16] = depth_stencil.load_op.as_action();
        store_ops[16] = depth_stencil.store_op;

        let depth_stencil_attachment = depth_stencil.image.into_framebuffer_attachment();

        RenderTargetEncoding {
            framebuffer: Framebuffer {
                color: buffers,
                depth_stencil: DepthStencilBuffer::new(
                    context.render_pass_id,
                    depth_stencil_attachment.width(),
                    depth_stencil_attachment.height(),
                ),
                context_id: context.context_id,
                render_pass_id: context.render_pass_id,
            },
            data: RenderTargetEncodingData::FBO(FBOEncodingData {
                load_ops,
                store_ops,
                color_count,
                color_attachments,
                depth_stencil_attachment: DepthStencilAttachmentDescriptor::DepthStencil(
                    depth_stencil_attachment,
                ),
            }),
            context,
        }
    }
}

impl<'a, F0, F1> RenderTargetEncoding<'a, Framebuffer<Vec<IntegerBuffer<F0>>, DepthBuffer<F1>>>
where
    F0: IntegerRenderable,
    F1: DepthRenderable,
{
    pub fn from_integer_colors_and_depth<C, I, Ds>(
        context: &'a mut EncodingContext,
        colors: C,
        depth: DepthTarget<Ds>,
    ) -> Self
    where
        C: IntoIterator<Item = IntegerTarget<I>>,
        I: IntoFramebufferAttachment<Format = F0>,
        Ds: IntoFramebufferAttachment<Format = F1>,
    {
        let mut color_attachments = [
            None, None, None, None, None, None, None, None, None, None, None, None, None, None,
            None, None,
        ];
        let mut load_ops = [LoadAction::Load; 17];
        let mut store_ops = [StoreOp::Store; 17];
        let mut buffers = Vec::new();

        for (index, target) in colors.into_iter().enumerate() {
            let attachment = target.image.into_framebuffer_attachment();

            buffers.push(IntegerBuffer::new(
                context.render_pass_id,
                index as i32,
                attachment.width(),
                attachment.height(),
            ));

            color_attachments[index] = Some(attachment);
            load_ops[index] = target.load_op.as_action(index as i32);
            store_ops[index] = target.store_op;
        }

        let color_count = buffers.len();

        load_ops[16] = depth.load_op.as_action();
        store_ops[16] = depth.store_op;

        let depth_attachment = depth.image.into_framebuffer_attachment();

        RenderTargetEncoding {
            framebuffer: Framebuffer {
                color: buffers,
                depth_stencil: DepthBuffer::new(
                    context.render_pass_id,
                    depth_attachment.width(),
                    depth_attachment.height(),
                ),
                context_id: context.context_id,
                render_pass_id: context.render_pass_id,
            },
            data: RenderTargetEncodingData::FBO(FBOEncodingData {
                load_ops,
                store_ops,
                color_count,
                color_attachments,
                depth_stencil_attachment: DepthStencilAttachmentDescriptor::Depth(depth_attachment),
            }),
            context,
        }
    }
}

impl<'a, F0, F1> RenderTargetEncoding<'a, Framebuffer<Vec<IntegerBuffer<F0>>, StencilBuffer<F1>>>
where
    F0: IntegerRenderable,
    F1: StencilRenderable,
{
    pub fn from_integer_colors_and_stencil<C, I, Ds>(
        context: &'a mut EncodingContext,
        colors: C,
        stencil: StencilTarget<Ds>,
    ) -> Self
    where
        C: IntoIterator<Item = IntegerTarget<I>>,
        I: IntoFramebufferAttachment<Format = F0>,
        Ds: IntoFramebufferAttachment<Format = F1>,
    {
        let mut color_attachments = [
            None, None, None, None, None, None, None, None, None, None, None, None, None, None,
            None, None,
        ];
        let mut load_ops = [LoadAction::Load; 17];
        let mut store_ops = [StoreOp::Store; 17];
        let mut buffers = Vec::new();

        for (index, target) in colors.into_iter().enumerate() {
            let attachment = target.image.into_framebuffer_attachment();

            buffers.push(IntegerBuffer::new(
                context.render_pass_id,
                index as i32,
                attachment.width(),
                attachment.height(),
            ));

            color_attachments[index] = Some(attachment);
            load_ops[index] = target.load_op.as_action(index as i32);
            store_ops[index] = target.store_op;
        }

        let color_count = buffers.len();

        load_ops[16] = stencil.load_op.as_action();
        store_ops[16] = stencil.store_op;

        let stencil_attachment = stencil.image.into_framebuffer_attachment();

        RenderTargetEncoding {
            framebuffer: Framebuffer {
                color: buffers,
                depth_stencil: StencilBuffer::new(
                    context.render_pass_id,
                    stencil_attachment.width(),
                    stencil_attachment.height(),
                ),
                context_id: context.context_id,
                render_pass_id: context.render_pass_id,
            },
            data: RenderTargetEncodingData::FBO(FBOEncodingData {
                load_ops,
                store_ops,
                color_count,
                color_attachments,
                depth_stencil_attachment: DepthStencilAttachmentDescriptor::Stencil(
                    stencil_attachment,
                ),
            }),
            context,
        }
    }
}

impl<'a, F> RenderTargetEncoding<'a, Framebuffer<Vec<UnsignedIntegerBuffer<F>>, ()>>
where
    F: UnsignedIntegerRenderable,
{
    pub fn from_unsigned_integer_colors<C, I>(context: &'a mut EncodingContext, colors: C) -> Self
    where
        C: IntoIterator<Item = UnsignedIntegerTarget<I>>,
        I: IntoFramebufferAttachment<Format = F>,
    {
        let mut color_attachments = [
            None, None, None, None, None, None, None, None, None, None, None, None, None, None,
            None, None,
        ];
        let mut load_ops = [LoadAction::Load; 17];
        let mut store_ops = [StoreOp::Store; 17];
        let mut buffers = Vec::new();

        for (index, target) in colors.into_iter().enumerate() {
            let attachment = target.image.into_framebuffer_attachment();

            buffers.push(UnsignedIntegerBuffer::new(
                context.render_pass_id,
                index as i32,
                attachment.width(),
                attachment.height(),
            ));

            color_attachments[index] = Some(attachment);
            load_ops[index] = target.load_op.as_action(index as i32);
            store_ops[index] = target.store_op;
        }

        let color_count = buffers.len();

        RenderTargetEncoding {
            framebuffer: Framebuffer {
                color: buffers,
                depth_stencil: (),
                context_id: context.context_id,
                render_pass_id: context.render_pass_id,
            },
            data: RenderTargetEncodingData::FBO(FBOEncodingData {
                load_ops,
                store_ops,
                color_count,
                color_attachments,
                depth_stencil_attachment: DepthStencilAttachmentDescriptor::None,
            }),
            context,
        }
    }
}

impl<'a, F0, F1>
    RenderTargetEncoding<'a, Framebuffer<Vec<UnsignedIntegerBuffer<F0>>, DepthStencilBuffer<F1>>>
where
    F0: UnsignedIntegerRenderable,
    F1: DepthStencilRenderable,
{
    pub fn from_unsigned_integer_colors_and_depth_stencil<C, I, Ds>(
        context: &'a mut EncodingContext,
        colors: C,
        depth_stencil: DepthStencilTarget<Ds>,
    ) -> Self
    where
        C: IntoIterator<Item = UnsignedIntegerTarget<I>>,
        I: IntoFramebufferAttachment<Format = F0>,
        Ds: IntoFramebufferAttachment<Format = F1>,
    {
        let mut color_attachments = [
            None, None, None, None, None, None, None, None, None, None, None, None, None, None,
            None, None,
        ];
        let mut load_ops = [LoadAction::Load; 17];
        let mut store_ops = [StoreOp::Store; 17];
        let mut buffers = Vec::new();

        for (index, target) in colors.into_iter().enumerate() {
            let attachment = target.image.into_framebuffer_attachment();

            buffers.push(UnsignedIntegerBuffer::new(
                context.render_pass_id,
                index as i32,
                attachment.width(),
                attachment.height(),
            ));

            color_attachments[index] = Some(attachment);
            load_ops[index] = target.load_op.as_action(index as i32);
            store_ops[index] = target.store_op;
        }

        let color_count = buffers.len();

        load_ops[16] = depth_stencil.load_op.as_action();
        store_ops[16] = depth_stencil.store_op;

        let depth_stencil_attachment = depth_stencil.image.into_framebuffer_attachment();

        RenderTargetEncoding {
            framebuffer: Framebuffer {
                color: buffers,
                depth_stencil: DepthStencilBuffer::new(
                    context.render_pass_id,
                    depth_stencil_attachment.width(),
                    depth_stencil_attachment.height(),
                ),
                context_id: context.context_id,
                render_pass_id: context.render_pass_id,
            },
            data: RenderTargetEncodingData::FBO(FBOEncodingData {
                load_ops,
                store_ops,
                color_count,
                color_attachments,
                depth_stencil_attachment: DepthStencilAttachmentDescriptor::DepthStencil(
                    depth_stencil_attachment,
                ),
            }),
            context,
        }
    }
}

impl<'a, F0, F1>
    RenderTargetEncoding<'a, Framebuffer<Vec<UnsignedIntegerBuffer<F0>>, DepthBuffer<F1>>>
where
    F0: UnsignedIntegerRenderable,
    F1: DepthRenderable,
{
    pub fn from_unsigned_integer_colors_and_depth<C, I, Ds>(
        context: &'a mut EncodingContext,
        colors: C,
        depth: DepthTarget<Ds>,
    ) -> Self
    where
        C: IntoIterator<Item = UnsignedIntegerTarget<I>>,
        I: IntoFramebufferAttachment<Format = F0>,
        Ds: IntoFramebufferAttachment<Format = F1>,
    {
        let mut color_attachments = [
            None, None, None, None, None, None, None, None, None, None, None, None, None, None,
            None, None,
        ];
        let mut load_ops = [LoadAction::Load; 17];
        let mut store_ops = [StoreOp::Store; 17];
        let mut buffers = Vec::new();

        for (index, target) in colors.into_iter().enumerate() {
            let attachment = target.image.into_framebuffer_attachment();

            buffers.push(UnsignedIntegerBuffer::new(
                context.render_pass_id,
                index as i32,
                attachment.width(),
                attachment.height(),
            ));

            color_attachments[index] = Some(attachment);
            load_ops[index] = target.load_op.as_action(index as i32);
            store_ops[index] = target.store_op;
        }

        let color_count = buffers.len();

        load_ops[16] = depth.load_op.as_action();
        store_ops[16] = depth.store_op;

        let depth_attachment = depth.image.into_framebuffer_attachment();

        RenderTargetEncoding {
            framebuffer: Framebuffer {
                color: buffers,
                depth_stencil: DepthBuffer::new(
                    context.render_pass_id,
                    depth_attachment.width(),
                    depth_attachment.height(),
                ),
                context_id: context.context_id,
                render_pass_id: context.render_pass_id,
            },
            data: RenderTargetEncodingData::FBO(FBOEncodingData {
                load_ops,
                store_ops,
                color_count,
                color_attachments,
                depth_stencil_attachment: DepthStencilAttachmentDescriptor::Depth(depth_attachment),
            }),
            context,
        }
    }
}

impl<'a, F0, F1>
    RenderTargetEncoding<'a, Framebuffer<Vec<UnsignedIntegerBuffer<F0>>, StencilBuffer<F1>>>
where
    F0: UnsignedIntegerRenderable,
    F1: StencilRenderable,
{
    pub fn from_unsigned_integer_colors_and_stencil<C, I, Ds>(
        context: &'a mut EncodingContext,
        colors: C,
        stencil: StencilTarget<Ds>,
    ) -> Self
    where
        C: IntoIterator<Item = UnsignedIntegerTarget<I>>,
        I: IntoFramebufferAttachment<Format = F0>,
        Ds: IntoFramebufferAttachment<Format = F1>,
    {
        let mut color_attachments = [
            None, None, None, None, None, None, None, None, None, None, None, None, None, None,
            None, None,
        ];
        let mut load_ops = [LoadAction::Load; 17];
        let mut store_ops = [StoreOp::Store; 17];
        let mut buffers = Vec::new();

        for (index, target) in colors.into_iter().enumerate() {
            let attachment = target.image.into_framebuffer_attachment();

            buffers.push(UnsignedIntegerBuffer::new(
                context.render_pass_id,
                index as i32,
                attachment.width(),
                attachment.height(),
            ));

            color_attachments[index] = Some(attachment);
            load_ops[index] = target.load_op.as_action(index as i32);
            store_ops[index] = target.store_op;
        }

        let color_count = buffers.len();

        load_ops[16] = stencil.load_op.as_action();
        store_ops[16] = stencil.store_op;

        let stencil_attachment = stencil.image.into_framebuffer_attachment();

        RenderTargetEncoding {
            framebuffer: Framebuffer {
                color: buffers,
                depth_stencil: StencilBuffer::new(
                    context.render_pass_id,
                    stencil_attachment.width(),
                    stencil_attachment.height(),
                ),
                context_id: context.context_id,
                render_pass_id: context.render_pass_id,
            },
            data: RenderTargetEncodingData::FBO(FBOEncodingData {
                load_ops,
                store_ops,
                color_count,
                color_attachments,
                depth_stencil_attachment: DepthStencilAttachmentDescriptor::Stencil(
                    stencil_attachment,
                ),
            }),
            context,
        }
    }
}

impl FBOEncodingData {
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

        &DRAW_BUFFERS_SEQUENTIAL[0..self.color_count]
    }
}

impl Hash for FBOEncodingData {
    fn hash<H>(&self, hasher: &mut H)
    where
        H: Hasher,
    {
        self.color_attachments().hash(hasher);
        self.depth_stencil_attachment().hash(hasher);
    }
}

impl AttachmentSet for FBOEncodingData {
    fn color_attachments(&self) -> &[Option<FramebufferAttachment>] {
        &self.color_attachments[0..self.color_count]
    }

    fn depth_stencil_attachment(&self) -> &DepthStencilAttachmentDescriptor {
        &self.depth_stencil_attachment
    }
}
