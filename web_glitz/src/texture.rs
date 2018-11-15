use std::marker;
use std::sync::Arc;

use crate::image_format::*;
use crate::util::JsId;

pub trait Texture<F> where F: TextureFormat {
    type Image: Image;

    fn level_count(&self) -> usize;

    fn layer_count(&self) -> usize;

    fn image(&self, level: usize, layer: usize) -> Option<Self::Image>;
}

pub unsafe trait Filterable {}

unsafe impl<T> Filterable for T where T: Texture<R8> {}
unsafe impl<T> Filterable for T where T: Texture<R16F> {}
unsafe impl<T> Filterable for T where T: Texture<RG8> {}
unsafe impl<T> Filterable for T where T: Texture<RG16F> {}
unsafe impl<T> Filterable for T where T: Texture<RGB8> {}
unsafe impl<T> Filterable for T where T: Texture<SRGB8> {}
unsafe impl<T> Filterable for T where T: Texture<RGB565> {}
unsafe impl<T> Filterable for T where T: Texture<R11F_G11F_B10F> {}
unsafe impl<T> Filterable for T where T: Texture<RGB9_E5> {}
unsafe impl<T> Filterable for T where T: Texture<RGB16F> {}
unsafe impl<T> Filterable for T where T: Texture<RGBA8> {}
unsafe impl<T> Filterable for T where T: Texture<SRGB8_APLHA8> {}
unsafe impl<T> Filterable for T where T: Texture<RGB5_A1> {}
unsafe impl<T> Filterable for T where T: Texture<RGBA4> {}
unsafe impl<T> Filterable for T where T: Texture<RGB10_A2> {}
unsafe impl<T> Filterable for T where T: Texture<RGBA16F> {}

pub trait Image<F> where F: InternalImageFormat {
    fn width(&self) -> usize;

    fn height(&self) -> usize;

    fn upload_task<D, T>(&self, data: D) -> ImageUploadTask<D> where D: ImageData<T>, T: ClientFormat<F::ClientFormat>;
}

pub unsafe trait ColorAttachable {}

unsafe impl<T> ColorAttachable for T where T: Image<R8> {}
unsafe impl<T> ColorAttachable for T where T: Image<R8UI> {}
unsafe impl<T> ColorAttachable for T where T: Image<RG8> {}
unsafe impl<T> ColorAttachable for T where T: Image<RG8UI> {}
unsafe impl<T> ColorAttachable for T where T: Image<RGB8> {}
unsafe impl<T> ColorAttachable for T where T: Image<RGB565> {}
unsafe impl<T> ColorAttachable for T where T: Image<RGBA8> {}
unsafe impl<T> ColorAttachable for T where T: Image<SRGB8_APLHA8> {}
unsafe impl<T> ColorAttachable for T where T: Image<RGB5_A1> {}
unsafe impl<T> ColorAttachable for T where T: Image<RGBA4> {}
unsafe impl<T> ColorAttachable for T where T: Image<RGB10_A2> {}
unsafe impl<T> ColorAttachable for T where T: Image<RGBA8UI> {}

pub struct Texture2D<F, C> {
    data: Arc<Texture2DData<F, C>>
}

struct Texture2DData<F, C> {
    gl_object_id: Option<JsId>,
    context: C,
    width: usize,
    height: usize,
    level_count: usize,
    format: marker::PhantomData<Box<[F]>>,
}

impl<F, C> Texture<F> for Texture2D<F, C> where F: InternalImageFormat {
    type Image = Texture2DImage<F, C>;

    fn level_count(&self) -> usize {
        self.data.level_count
    }

    fn layer_count(&self) -> usize {
        1
    }

    fn image(&self, level: usize, layer: usize) -> Option<Self::Image> {
        if layer == 0 && level < self.data.level_count {
            Some(Texture2DImage {
                texture_data: self.data.clone(),
                level
            })
        } else {
            None
        }
    }
}

pub struct Texture2DImage<F, C> {
    texture_data: Arc<Texture2DData<F, C>>,
    level: usize
}

struct Texture2DLevels<F> {
    format: F
}

impl<F> Texture2DLevels<F> where F: InternalImageFormat {

}




struct Texture2DArray {

}

impl Texture2DArray {
    pub fn width(&self) -> usize {

    }

    pub fn height(&self) -> usize {

    }

    pub fn layers(&self) -> TextureLayers {

    }
}

struct TextureLayers {

}

struct TextureLayer {

}

pub unsafe trait TextureFormat: InternalImageFormat {}

unsafe impl TextureFormat for R8 {}
unsafe impl TextureFormat for R16F {}
unsafe impl TextureFormat for R32F {}
unsafe impl TextureFormat for R8UI {}
unsafe impl TextureFormat for RG8 {}
unsafe impl TextureFormat for RG16F {}
unsafe impl TextureFormat for RG32F {}
unsafe impl TextureFormat for RG8UI {}
unsafe impl TextureFormat for RGB8 {}
unsafe impl TextureFormat for SRGB8 {}
unsafe impl TextureFormat for RGB565 {}
unsafe impl TextureFormat for R11F_G11F_B10F {}
unsafe impl TextureFormat for RGB9_E5 {}
unsafe impl TextureFormat for RGB16F {}
unsafe impl TextureFormat for RGB32F {}
unsafe impl TextureFormat for RGB8UI {}
unsafe impl TextureFormat for RGBA8 {}
unsafe impl TextureFormat for SRGB8_APLHA8 {}
unsafe impl TextureFormat for RGB5_A1 {}
unsafe impl TextureFormat for RGBA4 {}
unsafe impl TextureFormat for RGB10_A2 {}
unsafe impl TextureFormat for RGBA16F {}
unsafe impl TextureFormat for RGBA32F {}
unsafe impl TextureFormat for RGBA8UI {}
unsafe impl TextureFormat for DepthComponent16 {}
unsafe impl TextureFormat for DepthComponent24 {}
unsafe impl TextureFormat for DepthComponent32F {}
unsafe impl TextureFormat for Depth24Stencil8 {}
unsafe impl TextureFormat for Depth32FStencil8 {}
unsafe impl TextureFormat for Luminance {}
unsafe impl TextureFormat for LuminanceAlph {}
