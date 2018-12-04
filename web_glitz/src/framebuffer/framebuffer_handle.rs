use std::mem;
use std::sync::Arc;

use wasm_bindgen::JsCast;
use web_sys::WebGl2RenderingContext as Gl;

use crate::framebuffer::FramebufferDescriptor;
use crate::image_format::{ColorRenderable, DepthRenderable, StencilRenderable};
use crate::renderbuffer::{RenderbufferData, RenderbufferHandle};
use crate::rendering_context::{Connection, ContextUpdate, RenderingContext};
use crate::task::{GpuTask, Progress};
use crate::texture::Texture2DLevel;
use crate::util::JsId;
use texture::texture_2d::Texture2DData;
use texture::texture_2d_array::Texture2DArrayData;
use texture::texture_3d::Texture3DData;
use texture::texture_cube::TextureCubeData;
use texture::CubeFace;

const COLOR_ATTACHMENT_IDS: [u32; 16] = [
    Gl::COLOR_ATTACHMENT0,
    Gl::COLOR_ATTACHMENT1,
    Gl::COLOR_ATTACHMENT2,
    Gl::COLOR_ATTACHMENT3,
    Gl::COLOR_ATTACHMENT4,
    Gl::COLOR_ATTACHMENT5,
    Gl::COLOR_ATTACHMENT6,
    Gl::COLOR_ATTACHMENT7,
    Gl::COLOR_ATTACHMENT8,
    Gl::COLOR_ATTACHMENT9,
    Gl::COLOR_ATTACHMENT10,
    Gl::COLOR_ATTACHMENT11,
    Gl::COLOR_ATTACHMENT12,
    Gl::COLOR_ATTACHMENT13,
    Gl::COLOR_ATTACHMENT14,
    Gl::COLOR_ATTACHMENT15,
];

pub struct FramebufferHandle<Rc>
where
    Rc: RenderingContext,
{
    data: Arc<FramebufferData<Rc>>,
}

pub(crate) struct FramebufferData<Rc>
where
    Rc: RenderingContext,
{
    context: Rc,
    pub(crate) gl_object_id: Option<JsId>,
    color_attachments: [FramebufferAttachment<Rc>; 16],
    depth_attachment: FramebufferAttachment<Rc>,
    stencil_attachment: FramebufferAttachment<Rc>,
}

impl<Rc> FramebufferHandle<Rc>
where
    Rc: RenderingContext + 'static,
{
    pub(crate) fn new<D>(context: &Rc, descriptor: &D) -> Self
    where
        D: FramebufferDescriptor<Rc>,
    {
        let data = Arc::new(FramebufferData {
            context: context.clone(),
            gl_object_id: None,
            color_attachments: [
                descriptor.color_attachment_0().as_framebuffer_attachment(),
                descriptor.color_attachment_1().as_framebuffer_attachment(),
                descriptor.color_attachment_2().as_framebuffer_attachment(),
                descriptor.color_attachment_3().as_framebuffer_attachment(),
                descriptor.color_attachment_4().as_framebuffer_attachment(),
                descriptor.color_attachment_5().as_framebuffer_attachment(),
                descriptor.color_attachment_6().as_framebuffer_attachment(),
                descriptor.color_attachment_7().as_framebuffer_attachment(),
                descriptor.color_attachment_8().as_framebuffer_attachment(),
                descriptor.color_attachment_9().as_framebuffer_attachment(),
                descriptor.color_attachment_10().as_framebuffer_attachment(),
                descriptor.color_attachment_11().as_framebuffer_attachment(),
                descriptor.color_attachment_12().as_framebuffer_attachment(),
                descriptor.color_attachment_13().as_framebuffer_attachment(),
                descriptor.color_attachment_14().as_framebuffer_attachment(),
                descriptor.color_attachment_15().as_framebuffer_attachment(),
            ],
            depth_attachment: descriptor.depth_attachment().as_framebuffer_attachment(),
            stencil_attachment: descriptor.stencil_attachment().as_framebuffer_attachment(),
        });

        context.submit(FramebufferAllocateTask { data: data.clone() });

        FramebufferHandle { data }
    }
}

impl<Rc> Drop for FramebufferData<Rc>
where
    Rc: RenderingContext,
{
    fn drop(&mut self) {
        if let Some(id) = self.gl_object_id {
            self.context.submit(FramebufferDropTask { id });
        }
    }
}

pub struct FramebufferAttachment<Rc>
where
    Rc: RenderingContext,
{
    pub(crate) internal: FramebufferAttachmentInternal<Rc>,
}

pub(crate) enum FramebufferAttachmentInternal<Rc>
where
    Rc: RenderingContext,
{
    Texture2DLevel(Arc<Texture2DData<Rc>>, usize),
    Texture2DArrayLevelLayer(Arc<Texture2DArrayData<Rc>>, usize, usize),
    Texture3DLevelLayer(Arc<Texture3DData<Rc>>, usize, usize),
    TextureCubeLevelFace(Arc<TextureCubeData<Rc>>, usize, CubeFace),
    Renderbuffer(Arc<RenderbufferData<Rc>>),
    Empty,
}

impl<Rc> FramebufferAttachment<Rc>
where
    Rc: RenderingContext,
{
    fn attach(&self, gl: &Gl, slot: u32) {
        let target = Gl::DRAW_FRAMEBUFFER;

        match &self.internal {
            FramebufferAttachmentInternal::Texture2DLevel(texture_data, level) => {
                texture_data.id.unwrap().with_value_unchecked(|texture_object| {
                    gl.framebuffer_texture_2d(
                        target,
                        slot,
                        Gl::TEXTURE_2D,
                        Some(&texture_object),
                        *level as i32,
                    );
                });
            },
            FramebufferAttachmentInternal::Texture2DArrayLevelLayer(texture_data, level, layer) => {
                texture_data.id.unwrap().with_value_unchecked(|texture_object| {
                    gl.framebuffer_texture_layer(
                        target,
                        slot,
                        Some(&texture_object),
                        *level as i32,
                        *layer as i32,
                    );
                });
            },
            FramebufferAttachmentInternal::Texture3DLevelLayer(texture_data, level, layer) => {
                texture_data.id.unwrap().with_value_unchecked(|texture_object| {
                    gl.framebuffer_texture_layer(
                        target,
                        slot,
                        Some(&texture_object),
                        *level as i32,
                        *layer as i32,
                    );
                });
            },
            FramebufferAttachmentInternal::TextureCubeLevelFace(texture_data, level, face) => {
                let texture_target = match face {
                    CubeFace::PositiveX => Gl::TEXTURE_CUBE_MAP_POSITIVE_X,
                    CubeFace::NegativeX => Gl::TEXTURE_CUBE_MAP_NEGATIVE_X,
                    CubeFace::PositiveY => Gl::TEXTURE_CUBE_MAP_POSITIVE_Y,
                    CubeFace::NegativeY => Gl::TEXTURE_CUBE_MAP_NEGATIVE_Y,
                    CubeFace::PositiveZ => Gl::TEXTURE_CUBE_MAP_POSITIVE_Z,
                    CubeFace::NegativeZ => Gl::TEXTURE_CUBE_MAP_NEGATIVE_Z,
                };

                texture_data.id.unwrap().with_value_unchecked(|texture_object| {
                    gl.framebuffer_texture_2d(
                        target,
                        slot,
                        texture_target,
                        Some(&texture_object),
                        *level as i32,
                    );
                });
            },
            FramebufferAttachmentInternal::Renderbuffer(renderbuffer_data) => {
                renderbuffer_data.id.unwrap().with_value_unchecked(|renderbuffer_object| {
                    gl.framebuffer_renderbuffer(target, slot, Gl::RENDERBUFFER, Some(&renderbuffer_object));
                });
            }
            FramebufferAttachmentInternal::Empty => (),
        }
    }
}

pub trait AsFramebufferAttachment<Rc>
where
    Rc: RenderingContext,
{
    fn as_framebuffer_attachment(&self) -> FramebufferAttachment<Rc>;
}

impl<Rc> AsFramebufferAttachment<Rc> for ()
where
    Rc: RenderingContext,
{
    fn as_framebuffer_attachment(&self) -> FramebufferAttachment<Rc> {
        FramebufferAttachment {
            internal: FramebufferAttachmentInternal::Empty,
        }
    }
}

impl<Rc, T> AsFramebufferAttachment<Rc> for Option<T>
where
    T: AsFramebufferAttachment<Rc>,
    Rc: RenderingContext,
{
    fn as_framebuffer_attachment(&self) -> FramebufferAttachment<Rc> {
        match self {
            Some(a) => a.as_framebuffer_attachment(),
            None => FramebufferAttachment {
                internal: FramebufferAttachmentInternal::Empty,
            },
        }
    }
}

impl<F, Rc> AsFramebufferAttachment<Rc> for RenderbufferHandle<F, Rc>
where
    Rc: RenderingContext,
{
    fn as_framebuffer_attachment(&self) -> FramebufferAttachment<Rc> {
        FramebufferAttachment {
            internal: FramebufferAttachmentInternal::Renderbuffer(self.data.clone()),
        }
    }
}

struct FramebufferAllocateTask<Rc>
where
    Rc: RenderingContext,
{
    data: Arc<FramebufferData<Rc>>,
}

impl<Rc> GpuTask<Connection> for FramebufferAllocateTask<Rc>
where
    Rc: RenderingContext,
{
    type Output = ();

    type Error = ();

    fn progress(&mut self, connection: &mut Connection) -> Progress<Self::Output, Self::Error> {
        let Connection(gl, state) = connection;

        let framebuffer_object = gl.create_framebuffer().unwrap();
        let data = &self.data;

        state
            .set_bound_draw_framebuffer(Some(&framebuffer_object))
            .apply(gl)
            .unwrap();

        for (i, color_attachment) in data.color_attachments.iter().enumerate() {
            color_attachment.attach(gl, COLOR_ATTACHMENT_IDS[i]);
        }

        data.depth_attachment.attach(gl, Gl::DEPTH_ATTACHMENT);
        data.stencil_attachment.attach(gl, Gl::STENCIL_ATTACHMENT);

        Progress::Finished(Ok(()))
    }
}

struct FramebufferDropTask {
    id: JsId,
}

impl GpuTask<Connection> for FramebufferDropTask {
    type Output = ();

    type Error = ();

    fn progress(&mut self, connection: &mut Connection) -> Progress<Self::Output, Self::Error> {
        let Connection(gl, _) = connection;

        unsafe {
            gl.delete_framebuffer(Some(&JsId::into_value(self.id).unchecked_into()));
        }

        Progress::Finished(Ok(()))
    }
}
