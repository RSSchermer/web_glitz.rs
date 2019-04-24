use std::sync::Arc;

use web_sys::WebGl2RenderingContext as Gl;

use crate::image::format::{
    DepthRenderable, DepthStencilRenderable, FloatRenderable, IntegerRenderable,
    RenderbufferFormat, StencilRenderable, TextureFormat, UnsignedIntegerRenderable,
};
use crate::image::renderbuffer::{Renderbuffer, RenderbufferData};
use crate::image::texture_2d::{Level as Texture2DLevel, Texture2DData};
use crate::image::texture_2d_array::{LevelLayer as Texture2DArrayLevelLayer, Texture2DArrayData};
use crate::image::texture_3d::{LevelLayer as Texture3DLevelLayer, Texture3DData};
use crate::image::texture_cube::{CubeFace, LevelFace as TextureCubeLevelFace, TextureCubeData};
use crate::render_pass::{
    DepthBuffer, DepthStencilBuffer, FloatBuffer, IntegerBuffer, RenderBuffer, StencilBuffer,
    UnsignedIntegerBuffer,
};
use crate::render_target::AsAttachableImageRef;

use crate::runtime::state::DepthStencilAttachmentDescriptor;
use crate::util::{slice_make_mut, JsId};
use std::marker;

pub trait ColorAttachmentDescription {
    type Buffer: RenderBuffer;

    fn encode<'a, 'b>(
        &'a mut self,
        context: &'b mut ColorAttachmentEncodingContext,
    ) -> ColorAttachmentEncoding<'b, 'a, Self::Buffer>;
}

pub struct ColorAttachmentEncodingContext {
    pub(crate) render_pass_id: usize,
    pub(crate) buffer_index: i32,
}

pub struct ColorAttachmentEncoding<'a, 'b, B> {
    pub(crate) buffer: B,
    pub(crate) load_action: LoadAction,
    pub(crate) store_op: StoreOp,
    pub(crate) image: AttachableImageData,
    _context: &'a mut ColorAttachmentEncodingContext,
    _image_ref: marker::PhantomData<AttachableImageRef<'b>>,
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
            load_action: load_op.as_action(context.buffer_index),
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
            load_action: load_op.as_action(context.buffer_index),
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
            load_action: load_op.as_action(context.buffer_index),
            store_op,
            image,
            _context: context,
            _image_ref: marker::PhantomData,
        }
    }
}

pub trait DepthStencilAttachmentDescription {
    type Buffer: RenderBuffer;

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
    _image_ref: marker::PhantomData<AttachableImageRef<'b>>,
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
            load_action: load_op.as_action(),
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
            load_action: load_op.as_action(),
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
            load_action: load_op.as_action(),
            store_op,
            depth_stencil_type: DepthStencilAttachmentType::Stencil,
            image,
            _context: context,
            _image_ref: marker::PhantomData,
        }
    }
}

pub struct AttachableImageRef<'a> {
    data: AttachableImageData,
    marker: marker::PhantomData<&'a ()>,
}

impl<'a> AttachableImageRef<'a> {
    pub(crate) fn from_texture_2d_level<F>(image: &Texture2DLevel<'a, F>) -> Self
    where
        F: TextureFormat,
    {
        AttachableImageRef {
            data: AttachableImageData {
                context_id: image.texture_data().context_id(),
                kind: AttachableImageRefKind::Texture2DLevel {
                    data: image.texture_data().clone(),
                    level: image.level() as u8,
                },
                width: image.width(),
                height: image.height(),
            },
            marker: marker::PhantomData,
        }
    }

    pub(crate) fn from_texture_2d_array_level_layer<F>(
        image: &Texture2DArrayLevelLayer<'a, F>,
    ) -> Self
    where
        F: TextureFormat,
    {
        AttachableImageRef {
            data: AttachableImageData {
                context_id: image.texture_data().context_id(),
                kind: AttachableImageRefKind::Texture2DArrayLevelLayer {
                    data: image.texture_data().clone(),
                    level: image.level() as u8,
                    layer: image.layer() as u16,
                },
                width: image.width(),
                height: image.height(),
            },
            marker: marker::PhantomData,
        }
    }

    pub(crate) fn from_texture_3d_level_layer<F>(image: &Texture3DLevelLayer<'a, F>) -> Self
    where
        F: TextureFormat,
    {
        AttachableImageRef {
            data: AttachableImageData {
                context_id: image.texture_data().context_id(),
                kind: AttachableImageRefKind::Texture3DLevelLayer {
                    data: image.texture_data().clone(),
                    level: image.level() as u8,
                    layer: image.layer() as u16,
                },
                width: image.width(),
                height: image.height(),
            },
            marker: marker::PhantomData,
        }
    }

    pub(crate) fn from_texture_cube_level_face<F>(image: &TextureCubeLevelFace<'a, F>) -> Self
    where
        F: TextureFormat,
    {
        AttachableImageRef {
            data: AttachableImageData {
                context_id: image.texture_data().context_id(),
                kind: AttachableImageRefKind::TextureCubeLevelFace {
                    data: image.texture_data().clone(),
                    level: image.level() as u8,
                    face: image.face(),
                },
                width: image.width(),
                height: image.height(),
            },
            marker: marker::PhantomData,
        }
    }

    pub(crate) fn from_renderbuffer<F>(render_buffer: &'a Renderbuffer<F>) -> Self
    where
        F: RenderbufferFormat + 'static,
    {
        AttachableImageRef {
            data: AttachableImageData {
                context_id: render_buffer.data().context_id(),
                kind: AttachableImageRefKind::Renderbuffer {
                    data: render_buffer.data().clone(),
                },
                width: render_buffer.width(),
                height: render_buffer.height(),
            },
            marker: marker::PhantomData,
        }
    }

    pub(crate) fn into_data(self) -> AttachableImageData {
        self.data
    }
}

#[derive(Hash, PartialEq)]
pub(crate) struct AttachableImageData {
    pub(crate) context_id: usize,
    pub(crate) kind: AttachableImageRefKind,
    pub(crate) width: u32,
    pub(crate) height: u32,
}

impl AttachableImageData {
    pub(crate) fn id(&self) -> JsId {
        match &self.kind {
            AttachableImageRefKind::Texture2DLevel { data, .. } => data.id().unwrap(),
            AttachableImageRefKind::Texture2DArrayLevelLayer { data, .. } => data.id().unwrap(),
            AttachableImageRefKind::Texture3DLevelLayer { data, .. } => data.id().unwrap(),
            AttachableImageRefKind::TextureCubeLevelFace { data, .. } => data.id().unwrap(),
            AttachableImageRefKind::Renderbuffer { data, .. } => data.id().unwrap(),
        }
    }

    pub(crate) fn attach(&self, gl: &Gl, target: u32, slot: u32) {
        unsafe {
            match &self.kind {
                AttachableImageRefKind::Texture2DLevel { data, level } => {
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
                AttachableImageRefKind::Texture2DArrayLevelLayer { data, level, layer } => {
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
                AttachableImageRefKind::Texture3DLevelLayer { data, level, layer } => {
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
                AttachableImageRefKind::TextureCubeLevelFace { data, level, face } => {
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
                AttachableImageRefKind::Renderbuffer { data } => {
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

#[derive(Hash, PartialEq)]
pub(crate) enum AttachableImageRefKind {
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

impl<'a> AttachableImageRef<'a> {
    pub fn width(&self) -> u32 {
        self.data.width
    }

    pub fn height(&self) -> u32 {
        self.data.height
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum LoadOp<T> {
    Load,
    Clear(T),
}

impl LoadOp<[f32; 4]> {
    pub(crate) fn as_action(&self, index: i32) -> LoadAction {
        match self {
            LoadOp::Load => LoadAction::Load,
            LoadOp::Clear(value) => LoadAction::ClearColorFloat(index, *value),
        }
    }
}

impl LoadOp<[i32; 4]> {
    pub(crate) fn as_action(&self, index: i32) -> LoadAction {
        match self {
            LoadOp::Load => LoadAction::Load,
            LoadOp::Clear(value) => LoadAction::ClearColorInteger(index, *value),
        }
    }
}

impl LoadOp<[u32; 4]> {
    pub(crate) fn as_action(&self, index: i32) -> LoadAction {
        match self {
            LoadOp::Load => LoadAction::Load,
            LoadOp::Clear(value) => LoadAction::ClearColorUnsignedInteger(index, *value),
        }
    }
}

impl LoadOp<(f32, i32)> {
    pub(crate) fn as_action(&self) -> LoadAction {
        match self {
            LoadOp::Load => LoadAction::Load,
            LoadOp::Clear((depth, stencil)) => LoadAction::ClearDepthStencil(*depth, *stencil),
        }
    }
}

impl LoadOp<f32> {
    pub(crate) fn as_action(&self) -> LoadAction {
        match self {
            LoadOp::Load => LoadAction::Load,
            LoadOp::Clear(depth) => LoadAction::ClearDepth(*depth),
        }
    }
}

impl LoadOp<i32> {
    pub(crate) fn as_action(&self) -> LoadAction {
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

pub struct FloatAttachment<I>
where
    I: AsAttachableImageRef,
    I::Format: FloatRenderable,
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
where
    I: AsAttachableImageRef,
    I::Format: IntegerRenderable,
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
where
    I: AsAttachableImageRef,
    I::Format: UnsignedIntegerRenderable,
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
where
    I: AsAttachableImageRef,
    I::Format: DepthStencilRenderable,
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
where
    I: AsAttachableImageRef,
    I::Format: DepthRenderable,
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
where
    I: AsAttachableImageRef,
    I::Format: StencilRenderable,
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
