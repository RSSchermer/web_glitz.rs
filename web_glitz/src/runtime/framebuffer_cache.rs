use fnv::FnvHasher;
use runtime::Connection;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use texture::CubeFace;
use util::JsId;

use web_sys::{WebGl2RenderingContext as Gl, WebGlFramebuffer};

use crate::runtime::dynamic_state::ContextUpdate;
use fnv::FnvHashMap;
use wasm_bindgen::JsValue;

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum DrawBuffer {
    Color0,
    Color1,
    Color2,
    Color3,
    Color4,
    Color5,
    Color6,
    Color7,
    Color8,
    Color9,
    Color10,
    Color11,
    Color12,
    Color13,
    Color14,
    Color15,
    None,
}

impl DrawBuffer {
    fn id(&self) -> u32 {
        match self {
            DrawBuffer::Color0 => Gl::COLOR_ATTACHMENT0,
            DrawBuffer::Color1 => Gl::COLOR_ATTACHMENT1,
            DrawBuffer::Color2 => Gl::COLOR_ATTACHMENT2,
            DrawBuffer::Color3 => Gl::COLOR_ATTACHMENT3,
            DrawBuffer::Color4 => Gl::COLOR_ATTACHMENT4,
            DrawBuffer::Color5 => Gl::COLOR_ATTACHMENT5,
            DrawBuffer::Color6 => Gl::COLOR_ATTACHMENT6,
            DrawBuffer::Color7 => Gl::COLOR_ATTACHMENT7,
            DrawBuffer::Color8 => Gl::COLOR_ATTACHMENT8,
            DrawBuffer::Color9 => Gl::COLOR_ATTACHMENT9,
            DrawBuffer::Color10 => Gl::COLOR_ATTACHMENT10,
            DrawBuffer::Color11 => Gl::COLOR_ATTACHMENT11,
            DrawBuffer::Color12 => Gl::COLOR_ATTACHMENT12,
            DrawBuffer::Color13 => Gl::COLOR_ATTACHMENT13,
            DrawBuffer::Color14 => Gl::COLOR_ATTACHMENT14,
            DrawBuffer::Color15 => Gl::COLOR_ATTACHMENT15,
            DrawBuffer::None => Gl::NONE,
        }
    }
}

pub(crate) struct DrawFramebuffer {
    fbo: WebGlFramebuffer,
    draw_buffers: [DrawBuffer; 16],
}

impl DrawFramebuffer {
    pub(crate) fn bind(&mut self, connection: &mut Connection) -> BoundDrawFramebuffer {
        let Connection(gl, state) = connection;

        state
            .set_bound_draw_framebuffer(Some(&self.fbo))
            .apply(gl)
            .unwrap();

        BoundDrawFramebuffer { framebuffer: self }
    }
}

pub(crate) struct BoundDrawFramebuffer<'a> {
    framebuffer: &'a mut DrawFramebuffer,
}

impl<'a> BoundDrawFramebuffer<'a> {
    pub(crate) fn set_draw_buffers<I>(&mut self, draw_buffers: I, connection: &mut Connection)
    where
        I: IntoIterator<Item = DrawBuffer>,
    {
        let Connection(gl, state) = connection;
        let max_draw_buffers = state.max_draw_buffers();
        let framebuffer = &mut self.framebuffer;

        let mut needs_update = false;
        let mut buffer_count = 0;

        for buffer in draw_buffers {
            if buffer_count >= max_draw_buffers {
                panic!("Cannot bind more than {} draw buffers", max_draw_buffers);
            }

            if buffer != framebuffer.draw_buffers[buffer_count] {
                framebuffer.draw_buffers[buffer_count] = buffer;

                needs_update = true;
            }

            buffer_count += 1;
        }

        for i in buffer_count..max_draw_buffers {
            if DrawBuffer::None != framebuffer.draw_buffers[i] {
                framebuffer.draw_buffers[i] = DrawBuffer::None;

                needs_update = true;
            }
        }

        if needs_update {
            let mut buffer_ids = [0; 16];

            for (i, buffer) in framebuffer.draw_buffers[0..max_draw_buffers]
                .iter()
                .enumerate()
            {
                buffer_ids[i] = buffer.id();
            }

            // TODO: Actually update the draw buffers on the fbo once web-sys supports it, hopefully
            // with something along these lines:
            // gl.draw_buffers(Float32Array::view(&buffer_ids[0..max_draw_buffers]).into());
            unimplemented!()
        }
    }
}

pub(crate) struct FramebufferCache {
    framebuffers: FnvHashMap<u64, (DrawFramebuffer, [Option<JsId>; 17])>,
}

impl FramebufferCache {
    pub(crate) fn new() -> Self {
        FramebufferCache {
            framebuffers: FnvHashMap::default()
        }
    }
    
    pub(crate) fn get_or_create_mut<A>(
        &mut self,
        attachment_set: &A,
        connection: &mut Connection,
    ) -> &mut DrawFramebuffer
    where
        A: AttachmentSet,
    {
        let mut hasher = FnvHasher::default();

        attachment_set.hash(&mut hasher);

        let key = hasher.finish();

        let (framebuffer, _) = self.framebuffers.entry(key).or_insert_with(|| {
            let Connection(gl, state) = connection;
            let fbo = gl.create_framebuffer().unwrap();

            state
                .set_bound_draw_framebuffer(Some(&fbo))
                .apply(gl)
                .unwrap();

            let target = Gl::DRAW_FRAMEBUFFER;
            let mut attachment_ids = [None; 17];

            for (i, attachment) in attachment_set.color_attachments().iter().enumerate() {
                attachment.attach(gl, target, Gl::COLOR_ATTACHMENT0 + i as u32);

                attachment_ids[i] = Some(attachment.id());
            }

            if let Some((slot, image)) = match attachment_set.depth_stencil_attachment() {
                DepthStencilAttachmentDescriptor::Depth(image) => Some((Gl::DEPTH_ATTACHMENT, image)),
                DepthStencilAttachmentDescriptor::Stencil(image) => {
                    Some((Gl::STENCIL_ATTACHMENT, image))
                }
                DepthStencilAttachmentDescriptor::DepthStencil(image) => {
                    Some((Gl::DEPTH_STENCIL_ATTACHMENT, image))
                }
                DepthStencilAttachmentDescriptor::None => None,
            } {
                image.attach(gl, target, slot);

                attachment_ids[16] = Some(image.id());
            }

            let framebuffer = DrawFramebuffer {
                fbo,
                draw_buffers: [
                    DrawBuffer::Color0,
                    DrawBuffer::None,
                    DrawBuffer::None,
                    DrawBuffer::None,
                    DrawBuffer::None,
                    DrawBuffer::None,
                    DrawBuffer::None,
                    DrawBuffer::None,
                    DrawBuffer::None,
                    DrawBuffer::None,
                    DrawBuffer::None,
                    DrawBuffer::None,
                    DrawBuffer::None,
                    DrawBuffer::None,
                    DrawBuffer::None,
                    DrawBuffer::None,
                ],
            };

            (framebuffer, attachment_ids)
        });

        framebuffer
    }

    pub(crate) fn remove_attachment_dependents(
        &mut self,
        attachment_id: JsId,
        connection: &mut Connection,
    ) {
        let Connection(gl, _) = connection;

        self.framebuffers
            .retain(|_, (framebuffer, attachment_ids)| {
                let is_dependent = attachment_ids.iter().all(|id| id != &Some(attachment_id));

                if is_dependent {
                    gl.delete_framebuffer(Some(&framebuffer.fbo));
                }

                is_dependent
            })
    }
}

pub(crate) trait AttachmentSet: Hash {
    fn color_attachments(&self) -> &[AttachmentImage];

    fn depth_stencil_attachment(&self) -> &DepthStencilAttachmentDescriptor;
}

#[derive(Hash)]
pub(crate) enum AttachmentImage {
    Texture2DLevel { id: JsId, level: u8 },
    LayeredTextureLevelLayer { id: JsId, level: u8, layer: u16 },
    TextureCubeLevelFace { id: JsId, level: u8, face: CubeFace },
    Renderbuffer { id: JsId },
}

impl AttachmentImage {
    fn id(&self) -> JsId {
        match self {
            AttachmentImage::Texture2DLevel { id, .. } => *id,
            AttachmentImage::LayeredTextureLevelLayer { id, .. } => *id,
            AttachmentImage::TextureCubeLevelFace { id, .. } => *id,
            AttachmentImage::Renderbuffer { id } => *id,
        }
    }

    fn attach(&self, gl: &Gl, target: u32, slot: u32) {
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
                        gl.framebuffer_texture_layer(
                            target,
                            slot,
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

pub(crate) enum DepthStencilAttachmentDescriptor {
    Depth(AttachmentImage),
    Stencil(AttachmentImage),
    DepthStencil(AttachmentImage),
    None
}
