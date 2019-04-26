use std::marker;
use std::sync::Arc;

use web_sys::WebGl2RenderingContext as Gl;

use crate::image::format::{InternalFormat, RenderbufferFormat, TextureFormat};
use crate::image::renderbuffer::{Renderbuffer, RenderbufferData};
use crate::image::texture_2d::{LevelMut as Texture2DLevelMut, Level as Texture2DLevel, Texture2DData};
use crate::image::texture_2d_array::{LevelLayerMut as Texture2DArrayLevelLayerMut, LevelLayer as Texture2DArrayLevelLayer, Texture2DArrayData};
use crate::image::texture_3d::{LevelLayerMut as Texture3DLevelLayerMut, LevelLayer as Texture3DLevelLayer, Texture3DData};
use crate::image::texture_cube::{LevelFaceMut as TextureCubeLevelFaceMut, LevelFace as TextureCubeLevelFace, TextureCubeData, CubeFace};
use crate::util::JsId;

/// Trait implemented for image references that can be attached to a render target.
///
/// See also [RenderTarget] and [RenderTargetDescription].
pub trait AsAttachableImageRef {
    /// The type of image storage format the image is stored in.
    type Format: InternalFormat;

    /// Converts the image reference into a render target attachment.
    fn as_attachable_image_ref(&mut self) -> AttachableImageRef<Self::Format>;
}

impl<'a, T> AsAttachableImageRef for &'a mut T where T: AsAttachableImageRef {
    type Format = T::Format;

    fn as_attachable_image_ref(&mut self) -> AttachableImageRef<Self::Format> {
        (*self).as_attachable_image_ref()
    }
}

impl<'a, F> AsAttachableImageRef for Texture2DLevelMut<'a, F>
where
    F: TextureFormat,
{
    type Format = F;

    fn as_attachable_image_ref(&mut self) -> AttachableImageRef<Self::Format> {
        AttachableImageRef::from_texture_2d_level(&self)
    }
}

impl<'a, F> AsAttachableImageRef for Texture2DArrayLevelLayerMut<'a, F>
where
    F: TextureFormat,
{
    type Format = F;

    fn as_attachable_image_ref(&mut self) -> AttachableImageRef<Self::Format> {
        AttachableImageRef::from_texture_2d_array_level_layer(&self)
    }
}

impl<'a, F> AsAttachableImageRef for Texture3DLevelLayerMut<'a, F>
where
    F: TextureFormat,
{
    type Format = F;

    fn as_attachable_image_ref(&mut self) -> AttachableImageRef<Self::Format> {
        AttachableImageRef::from_texture_3d_level_layer(&self)
    }
}

impl<'a, F> AsAttachableImageRef for TextureCubeLevelFaceMut<'a, F>
where
    F: TextureFormat,
{
    type Format = F;

    fn as_attachable_image_ref(&mut self) -> AttachableImageRef<Self::Format> {
        AttachableImageRef::from_texture_cube_level_face(&self)
    }
}

impl<F> AsAttachableImageRef for Renderbuffer<F>
where
    F: RenderbufferFormat + 'static,
{
    type Format = F;

    fn as_attachable_image_ref(&mut self) -> AttachableImageRef<Self::Format> {
        AttachableImageRef::from_renderbuffer(self)
    }
}

pub struct AttachableImageRef<'a, F> {
    data: AttachableImageData,
    marker: marker::PhantomData<&'a F>,
}

impl<'a, F> AttachableImageRef<'a, F> {
    pub(crate) fn from_texture_2d_level(image: &Texture2DLevel<'a, F>) -> Self
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

    pub(crate) fn from_texture_2d_array_level_layer(
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

    pub(crate) fn from_texture_3d_level_layer(image: &Texture3DLevelLayer<'a, F>) -> Self
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

    pub(crate) fn from_texture_cube_level_face(image: &TextureCubeLevelFace<'a, F>) -> Self
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

    pub(crate) fn from_renderbuffer(render_buffer: &'a Renderbuffer<F>) -> Self
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

    pub fn width(&self) -> u32 {
        self.data.width
    }

    pub fn height(&self) -> u32 {
        self.data.height
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
