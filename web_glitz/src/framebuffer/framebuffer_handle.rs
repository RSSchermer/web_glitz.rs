use std::sync::Arc;

use web_sys::WebGl2RenderingContext as Gl;

use crate::framebuffer::FramebufferDescriptor;
use crate::renderbuffer::{RenderbufferData, RenderbufferHandle};
use crate::runtime::{Connection, RenderingContext};
use crate::runtime::dropper::{DropObject, Dropper, RefCountedDropper};
use crate::runtime::dynamic_state::ContextUpdate;
use crate::task::{GpuTask, Progress};
use crate::texture::texture_2d::Texture2DData;
use crate::texture::texture_2d_array::Texture2DArrayData;
use crate::texture::texture_3d::Texture3DData;
use crate::texture::texture_cube::TextureCubeData;
use crate::texture::CubeFace;
use crate::util::{JsId, arc_get_mut_unchecked};

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

pub struct FramebufferHandle {
    data: Arc<FramebufferData>,
}

pub(crate) struct FramebufferData {
    dropper: RefCountedDropper,
    pub(crate) id: Option<JsId>,
    color_attachments: [FramebufferAttachment; 16],
    depth_attachment: FramebufferAttachment,
    stencil_attachment: FramebufferAttachment,
}

impl FramebufferHandle {
    pub(crate) fn new<D, Rc>(context: &Rc, dropper: RefCountedDropper, descriptor: &D) -> Self
    where
        D: FramebufferDescriptor,
        Rc: RenderingContext,
    {
        let data = Arc::new(FramebufferData {
            dropper,
            id: None,
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

impl Drop for FramebufferData {
    fn drop(&mut self) {
        if let Some(id) = self.id {
            self.dropper.drop_gl_object(DropObject::Framebuffer(id));
        }
    }
}

pub struct FramebufferAttachment {
    pub(crate) internal: FramebufferAttachmentInternal,
}

pub(crate) enum FramebufferAttachmentInternal {
    Texture2DLevel(Arc<Texture2DData>, usize),
    Texture2DArrayLevelLayer(Arc<Texture2DArrayData>, usize, usize),
    Texture3DLevelLayer(Arc<Texture3DData>, usize, usize),
    TextureCubeLevelFace(Arc<TextureCubeData>, usize, CubeFace),
    Renderbuffer(Arc<RenderbufferData>),
    Empty,
}

impl FramebufferAttachment {
    fn attach(&self, gl: &Gl, slot: u32) {
        let target = Gl::DRAW_FRAMEBUFFER;

        match &self.internal {
            FramebufferAttachmentInternal::Texture2DLevel(texture_data, level) => unsafe {
                texture_data
                    .id
                    .unwrap()
                    .with_value_unchecked(|texture_object| {
                        gl.framebuffer_texture_2d(
                            target,
                            slot,
                            Gl::TEXTURE_2D,
                            Some(&texture_object),
                            *level as i32,
                        );
                    });
            },
            FramebufferAttachmentInternal::Texture2DArrayLevelLayer(texture_data, level, layer) => unsafe {
                texture_data
                    .id
                    .unwrap()
                    .with_value_unchecked(|texture_object| {
                        gl.framebuffer_texture_layer(
                            target,
                            slot,
                            Some(&texture_object),
                            *level as i32,
                            *layer as i32,
                        );
                    });
            },
            FramebufferAttachmentInternal::Texture3DLevelLayer(texture_data, level, layer) => unsafe {
                texture_data
                    .id
                    .unwrap()
                    .with_value_unchecked(|texture_object| {
                        gl.framebuffer_texture_layer(
                            target,
                            slot,
                            Some(&texture_object),
                            *level as i32,
                            *layer as i32,
                        );
                    });
            },
            FramebufferAttachmentInternal::TextureCubeLevelFace(texture_data, level, face) => unsafe {
                let texture_target = match face {
                    CubeFace::PositiveX => Gl::TEXTURE_CUBE_MAP_POSITIVE_X,
                    CubeFace::NegativeX => Gl::TEXTURE_CUBE_MAP_NEGATIVE_X,
                    CubeFace::PositiveY => Gl::TEXTURE_CUBE_MAP_POSITIVE_Y,
                    CubeFace::NegativeY => Gl::TEXTURE_CUBE_MAP_NEGATIVE_Y,
                    CubeFace::PositiveZ => Gl::TEXTURE_CUBE_MAP_POSITIVE_Z,
                    CubeFace::NegativeZ => Gl::TEXTURE_CUBE_MAP_NEGATIVE_Z,
                };

                texture_data
                    .id
                    .unwrap()
                    .with_value_unchecked(|texture_object| {
                        gl.framebuffer_texture_2d(
                            target,
                            slot,
                            texture_target,
                            Some(&texture_object),
                            *level as i32,
                        );
                    });
            },
            FramebufferAttachmentInternal::Renderbuffer(renderbuffer_data) => unsafe {
                renderbuffer_data
                    .id
                    .unwrap()
                    .with_value_unchecked(|renderbuffer_object| {
                        gl.framebuffer_renderbuffer(
                            target,
                            slot,
                            Gl::RENDERBUFFER,
                            Some(&renderbuffer_object),
                        );
                    });
            },
            FramebufferAttachmentInternal::Empty => (),
        }
    }
}

pub trait AsFramebufferAttachment {
    fn as_framebuffer_attachment(&self) -> FramebufferAttachment;
}

impl AsFramebufferAttachment for () {
    fn as_framebuffer_attachment(&self) -> FramebufferAttachment {
        FramebufferAttachment {
            internal: FramebufferAttachmentInternal::Empty,
        }
    }
}

impl<T> AsFramebufferAttachment for Option<T>
where
    T: AsFramebufferAttachment,
{
    fn as_framebuffer_attachment(&self) -> FramebufferAttachment {
        match self {
            Some(a) => a.as_framebuffer_attachment(),
            None => FramebufferAttachment {
                internal: FramebufferAttachmentInternal::Empty,
            },
        }
    }
}

impl<F> AsFramebufferAttachment for RenderbufferHandle<F> {
    fn as_framebuffer_attachment(&self) -> FramebufferAttachment {
        FramebufferAttachment {
            internal: FramebufferAttachmentInternal::Renderbuffer(self.data.clone()),
        }
    }
}

struct FramebufferAllocateTask {
    data: Arc<FramebufferData>,
}

impl GpuTask<Connection> for FramebufferAllocateTask {
    type Output = ();

    fn progress(&mut self, connection: &mut Connection) -> Progress<Self::Output> {
        let Connection(gl, state) = connection;

        let framebuffer_object = gl.create_framebuffer().unwrap();
        let mut data = unsafe { arc_get_mut_unchecked(&mut self.data) };

        state
            .set_bound_draw_framebuffer(Some(&framebuffer_object))
            .apply(gl)
            .unwrap();

        for (i, color_attachment) in data.color_attachments.iter().enumerate() {
            color_attachment.attach(gl, COLOR_ATTACHMENT_IDS[i]);
        }

        data.depth_attachment.attach(gl, Gl::DEPTH_ATTACHMENT);
        data.stencil_attachment.attach(gl, Gl::STENCIL_ATTACHMENT);

        data.id = Some(JsId::from_value(framebuffer_object.into()));

        Progress::Finished(())
    }
}
