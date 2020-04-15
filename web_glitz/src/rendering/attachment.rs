use std::marker;
use std::sync::Arc;

use web_sys::WebGl2RenderingContext as Gl;

use crate::image::format::{
    InternalFormat, Multisamplable, Multisample, RenderbufferFormat, TextureFormat,
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
use crate::util::JsId;

/// Trait implemented for image references that can be attached to a render target.
///
/// See also [RenderTargetDescriptor].
pub trait AsAttachment {
    /// The type of image storage format the image is stored in.
    type Format: InternalFormat;

    /// Converts the image reference into a render target attachment.
    fn as_attachment(&mut self) -> Attachment<Self::Format>;
}

impl<'a, T> AsAttachment for &'a mut T
where
    T: AsAttachment,
{
    type Format = T::Format;

    fn as_attachment(&mut self) -> Attachment<Self::Format> {
        (*self).as_attachment()
    }
}

impl<'a, F> AsAttachment for Texture2DLevelMut<'a, F>
where
    F: TextureFormat,
{
    type Format = F;

    fn as_attachment(&mut self) -> Attachment<Self::Format> {
        Attachment::from_texture_2d_level(&self)
    }
}

impl<'a, F> AsAttachment for Texture2DArrayLevelLayerMut<'a, F>
where
    F: TextureFormat,
{
    type Format = F;

    fn as_attachment(&mut self) -> Attachment<Self::Format> {
        Attachment::from_texture_2d_array_level_layer(&self)
    }
}

impl<'a, F> AsAttachment for Texture3DLevelLayerMut<'a, F>
where
    F: TextureFormat,
{
    type Format = F;

    fn as_attachment(&mut self) -> Attachment<Self::Format> {
        Attachment::from_texture_3d_level_layer(&self)
    }
}

impl<'a, F> AsAttachment for TextureCubeLevelFaceMut<'a, F>
where
    F: TextureFormat,
{
    type Format = F;

    fn as_attachment(&mut self) -> Attachment<Self::Format> {
        Attachment::from_texture_cube_level_face(&self)
    }
}

impl<F> AsAttachment for Renderbuffer<F>
where
    F: RenderbufferFormat + 'static,
{
    type Format = F;

    fn as_attachment(&mut self) -> Attachment<Self::Format> {
        Attachment::from_renderbuffer(self)
    }
}

/// Exclusive reference to an image that may be attached to a [RenderTarget].
pub struct Attachment<'a, F> {
    data: AttachmentData,
    marker: marker::PhantomData<&'a F>,
}

impl<'a, F> Attachment<'a, F> {
    pub(crate) fn from_texture_2d_level(image: &Texture2DLevel<'a, F>) -> Self
    where
        F: TextureFormat,
    {
        Attachment {
            data: AttachmentData {
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

    pub(crate) fn from_texture_2d_array_level_layer(image: &Texture2DArrayLevelLayer<'a, F>) -> Self
    where
        F: TextureFormat,
    {
        Attachment {
            data: AttachmentData {
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

    pub(crate) fn from_texture_3d_level_layer(image: &Texture3DLevelLayer<'a, F>) -> Self
    where
        F: TextureFormat,
    {
        Attachment {
            data: AttachmentData {
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

    pub(crate) fn from_texture_cube_level_face(image: &TextureCubeLevelFace<'a, F>) -> Self
    where
        F: TextureFormat,
    {
        Attachment {
            data: AttachmentData {
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

    pub(crate) fn from_renderbuffer(render_buffer: &'a Renderbuffer<F>) -> Self {
        Attachment {
            data: AttachmentData {
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

    pub(crate) fn into_data(self) -> AttachmentData {
        self.data
    }
}

/// Trait implemented for image references that can be attached to a multisample render target.
///
/// See also [MultisampleRenderTargetDescriptor].
pub trait AsMultisampleAttachment {
    /// The type of image storage format the image is stored in.
    type SampleFormat: InternalFormat + Multisamplable;

    /// Converts the image reference into a render target attachment.
    fn as_multisample_attachment(&mut self) -> MultisampleAttachment<Self::SampleFormat>;
}

impl<'a, T> AsMultisampleAttachment for &'a mut T
where
    T: AsMultisampleAttachment,
{
    type SampleFormat = T::SampleFormat;

    fn as_multisample_attachment(&mut self) -> MultisampleAttachment<Self::SampleFormat> {
        (*self).as_multisample_attachment()
    }
}

impl<F> AsMultisampleAttachment for Renderbuffer<Multisample<F>>
where
    F: RenderbufferFormat + Multisamplable + 'static,
{
    type SampleFormat = F;

    fn as_multisample_attachment(&mut self) -> MultisampleAttachment<Self::SampleFormat> {
        MultisampleAttachment::from_renderbuffer(self)
    }
}

/// Exclusive reference to a multisample image that may be attached to a [MultisampleRenderTarget].
pub struct MultisampleAttachment<'a, F>
where
    F: Multisamplable,
{
    internal: MultisampleAttachmentInternal<'a, F>,
}

impl<'a, F> MultisampleAttachment<'a, F>
where
    F: Multisamplable,
{
    pub(crate) fn from_renderbuffer(renderbuffer: &'a Renderbuffer<Multisample<F>>) -> Self {
        MultisampleAttachment {
            internal: renderbuffer.into(),
        }
    }

    pub fn samples(&self) -> u8 {
        self.internal.samples()
    }

    pub(crate) fn into_data(self) -> AttachmentData {
        self.internal.into_data()
    }
}

enum MultisampleAttachmentInternal<'a, F>
where
    F: Multisamplable,
{
    Renderbuffer(&'a Renderbuffer<Multisample<F>>),
}

impl<'a, F> MultisampleAttachmentInternal<'a, F>
where
    F: Multisamplable,
{
    fn samples(&self) -> u8 {
        match self {
            MultisampleAttachmentInternal::Renderbuffer(renderbufer) => renderbufer.samples(),
        }
    }

    fn into_data(self) -> AttachmentData {
        match self {
            MultisampleAttachmentInternal::Renderbuffer(renderbuffer) => AttachmentData {
                context_id: renderbuffer.data().context_id(),
                kind: AttachableImageRefKind::Renderbuffer {
                    data: renderbuffer.data().clone(),
                },
                width: renderbuffer.width(),
                height: renderbuffer.height(),
            },
        }
    }
}

impl<'a, F> From<&'a Renderbuffer<Multisample<F>>> for MultisampleAttachmentInternal<'a, F>
where
    F: Multisamplable,
{
    fn from(renderbuffer: &'a Renderbuffer<Multisample<F>>) -> Self {
        MultisampleAttachmentInternal::Renderbuffer(renderbuffer)
    }
}

#[derive(Clone, Hash, PartialEq)]
pub(crate) struct AttachmentData {
    pub(crate) context_id: u64,
    pub(crate) kind: AttachableImageRefKind,
    pub(crate) width: u32,
    pub(crate) height: u32,
}

impl AttachmentData {
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

#[derive(Clone, Hash, PartialEq)]
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
