use crate::image::format::{InternalFormat, RenderbufferFormat, TextureFormat};
use crate::image::renderbuffer::Renderbuffer;
use crate::image::texture_2d::LevelMut as Texture2DLevelMut;
use crate::image::texture_2d_array::LevelLayerMut as Texture2DArrayLevelLayerMut;
use crate::image::texture_3d::LevelLayerMut as Texture3DLevelLayerMut;
use crate::image::texture_cube::LevelFaceMut as TextureCubeLevelFaceMut;
use crate::render_target::AttachableImageRef;

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
