use std::mem;
use std::sync::Arc;

use wasm_bindgen::JsCast;
use web_sys::WebGl2RenderingContext as Gl;

use crate::framebuffer::FramebufferDescriptor;
use crate::image_format::{ColorRenderable, DepthRenderable, StencilRenderable};
use crate::renderbuffer::{RenderbufferData, RenderbufferHandle};
use crate::rendering_context::{Connection, RenderingContext, ContextUpdate};
use crate::task::{GpuTask, Progress};
use crate::texture::{Texture2DImageRef, TextureImageData, TextureImageTarget};
use crate::util::JsId;

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

impl<Rc> FramebufferHandle<Rc> where Rc: RenderingContext + 'static {
    pub(crate) fn new<D>(context: &Rc, descriptor: &D) -> Self where D: FramebufferDescriptor<Rc> {
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

        context.submit(FramebufferAllocateTask {
            data: data.clone()
        });

        FramebufferHandle {
            data
        }
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
    internal: FramebufferAttachmentInternal<Rc>,
}

enum FramebufferAttachmentInternal<Rc>
    where
        Rc: RenderingContext,
{
    TextureImage(TextureImageData<Rc>),
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
            FramebufferAttachmentInternal::TextureImage(image) => {
                let object = unsafe {
                    JsId::into_value(image.texture_data.gl_object_id.unwrap()).unchecked_into()
                };
                let level = image.level as i32;

                match image.target {
                    TextureImageTarget::Texture2D => {
                        gl.framebuffer_texture_2d(
                            target,
                            slot,
                            Gl::TEXTURE_2D,
                            Some(&object),
                            level,
                        );
                    }
                    TextureImageTarget::TextureCubeMapPositiveX => {
                        gl.framebuffer_texture_2d(
                            target,
                            slot,
                            Gl::TEXTURE_CUBE_MAP_POSITIVE_X,
                            Some(&object),
                            level,
                        );
                    }
                    TextureImageTarget::TextureCubeMapNegativeX => {
                        gl.framebuffer_texture_2d(
                            target,
                            slot,
                            Gl::TEXTURE_CUBE_MAP_NEGATIVE_X,
                            Some(&object),
                            level,
                        );
                    }
                    TextureImageTarget::TextureCubeMapPositiveY => {
                        gl.framebuffer_texture_2d(
                            target,
                            slot,
                            Gl::TEXTURE_CUBE_MAP_POSITIVE_Y,
                            Some(&object),
                            level,
                        );
                    }
                    TextureImageTarget::TextureCubeMapNegativeY => {
                        gl.framebuffer_texture_2d(
                            target,
                            slot,
                            Gl::TEXTURE_CUBE_MAP_NEGATIVE_Y,
                            Some(&object),
                            level,
                        );
                    }
                    TextureImageTarget::TextureCubeMapPositiveZ => {
                        gl.framebuffer_texture_2d(
                            target,
                            slot,
                            Gl::TEXTURE_CUBE_MAP_POSITIVE_Z,
                            Some(&object),
                            level,
                        );
                    }
                    TextureImageTarget::TextureCubeMapNegativeZ => {
                        gl.framebuffer_texture_2d(
                            target,
                            slot,
                            Gl::TEXTURE_CUBE_MAP_NEGATIVE_Z,
                            Some(&object),
                            level,
                        );
                    }
                    TextureImageTarget::Texture2DArray(layer) => {
                        gl.framebuffer_texture_layer(
                            target,
                            slot,
                            Some(&object),
                            level,
                            layer as i32,
                        );
                    }
                    TextureImageTarget::Texture3D(layer) => {
                        gl.framebuffer_texture_layer(
                            target,
                            slot,
                            Some(&object),
                            level,
                            layer as i32,
                        );
                    }
                }

                mem::forget(object);
            }
            FramebufferAttachmentInternal::Renderbuffer(renderbuffer) => {
                let object = unsafe {
                    JsId::into_value(renderbuffer.gl_object_id.unwrap()).unchecked_into()
                };

                gl.framebuffer_renderbuffer(target, slot, Gl::RENDERBUFFER, Some(&object));

                mem::forget(object)
            }
            _ => (),
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

impl<F, Rc> AsFramebufferAttachment<Rc> for Texture2DImageRef<F, Rc>
    where
        Rc: RenderingContext,
{
    fn as_framebuffer_attachment(&self) -> FramebufferAttachment<Rc> {
        FramebufferAttachment {
            internal: FramebufferAttachmentInternal::TextureImage(self.data.clone()),
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
