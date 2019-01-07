use rendering_context::Connection;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::Hash;
use task::GpuTask;
use task::Progress;
use texture::CubeFace;
use util::JsId;

struct SinglePassAttachments<C, D> {
    color_attachments: C,
    depth_attachments: D,
}

trait ColorAttachments {
    fn attach_all(self, builder: &mut FramebufferStateBuilder);
}

pub struct FramebufferState<C, D> {
    pub color: C,
    pub depth_stencil: D,
}

pub struct SinglePassAttachmentDescriptor<'a, I, T> {
    pub image: &'a mut I,
    pub load_op: LoadOp<T>,
    pub store_op: StoreOp,
}

pub enum LoadOp<T> {
    Load,
    Clear(T),
    DontCare,
}

pub enum StoreOp {
    Store,
    DontCare,
}

pub struct SinglePassRenderPass<A, T> {
    attachments: A,
    task: T,
}

pub struct SinglePassRenderPassContext {
    connection: *mut Connection,
}

impl<A, T> GpuTask<Connection> for SinglePassRenderPass<A, T>
where
    A: SinglePassAttachmentSet,
    T: GpuTask<SinglePassRenderPassContext>,
{
    type Output = ();

    fn progress(&mut self, context: &mut SinglePassRenderPassContext) -> Progress<Self::Output> {
        let Connection(gl, state) = unsafe { &mut *context.connection };
    }
}

struct FramebufferCache {
    framebuffers: HashMap<u64, (WebGlFramebuffer, [JsId; 16])>,
}

impl FramebufferCache {
    fn get_or_create<A>(
        &mut self,
        attachment_set: &A,
        connection: &mut Connection,
    ) -> &WebGlFramebuffer
    where
        A: AttachmentSet,
    {
        let mut hasher = Hasher::default();
        let key = attachment_set.hash(&mut hasher).finish();

        self.framebuffers
            .entry(key)
            .or_insert_with(|| {
                let Connection(gl, state) = connection;
                let framebuffer = gl.create_framebuffer().unwrap();

                state
                    .set_bound_draw_framebuffer(Some(&framebuffer))
                    .apply(gl)
                    .unwrap();

                let target = Gl::DRAW_FRAMEBUFFER;

                for (i, attachment) in attachment_set.color_attachments().iter().enumerate() {
                    attachment.attach(gl, target, Gl::COLOR_ATTACHMENT0 + i);
                }

                if let Some((slot, image)) = match attachment_set.depth_stencil_attachment() {
                    Some(DepthStencilAttachment::Depth(image)) => {
                        Some((Gl::DEPTH_ATTACHMENT, image))
                    }
                    Some(DepthStencilAttachment::Stencil(image)) => {
                        Some((Gl::STENCIL_ATTACHMENT, image))
                    }
                    Some(DepthStencilAttachment::DepthStencil(image)) => {
                        Some((Gl::DEPTH_STENCIL_ATTACHMENT, image))
                    }
                } {
                    image.attach(gl, target, slot)
                }
            })
            .0
    }

    fn remove_attachment_dependents(&mut self, attachment_id: JsId, connection: &mut Connection) {
        let Connection(gl, _) = connection;

        self.framebuffers.retain(|(fbo, attachment_ids)| {
            let is_dependent = attachment_ids.all(|id| id != attachment_id);

            if is_dependent {
                gl.delete_framebuffer(fbo);
            }

            is_dependent
        })
    }
}

trait AttachmentSet: Hash {
    fn color_attachments(&self) -> &[AttachmentImage];

    fn depth_stencil_attachment(&self) -> Option<DepthStencilAttachment>;
}

#[derive(Hash)]
enum AttachmentImage {
    Texture2DLevel { id: JsId, level: u8 },
    LayeredTextureLevelLayer { id: JsId, level: u8, layer: u16 },
    TextureCubeLevelFace { id: JsId, level: u8, face: CubeFace },
    Renderbuffer { id: JsId },
}

impl AttachmentImage {
    fn attach(&self, gl: &WebGl2RenderingContext, target: u32, slot: u32) {
        unsafe {
            match self {
                AttachmentImage::Texture2DLevel { id, level } => {
                    id.with_value_unchecked(|texture_object| {
                        gl.framebuffer_texture_2d(
                            target,
                            slot,
                            Gl::TEXTURE_2D,
                            Some(&texture_object),
                            *level as i32,
                        );
                    });
                }
                AttachmentImage::LayeredTextureLevelLayer { id, level, layer } => {
                    id.with_value_unchecked(|texture_object| {
                        gl.framebuffer_texture_3d(
                            target,
                            slot,
                            Gl::TEXTURE_2D,
                            Some(&texture_object),
                            *level as i32,
                            *layer as i32,
                        );
                    });
                }
                AttachmentImage::TextureCubeLevelFace { id, level, face } => {
                    id.with_value_unchecked(|texture_object| {
                        gl.framebuffer_texture_2d(
                            target,
                            slot,
                            face.id(),
                            Some(&texture_object),
                            *level as i32,
                        );
                    });
                }
                AttachmentImage::Renderbuffer { id } => {
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

enum DepthStencilAttachment {
    Depth(AttachmentImage),
    Stencil(AttachmentImage),
    DepthStencil(AttachmentImage),
}
