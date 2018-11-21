use std::marker;
use std::sync::Arc;

use crate::image_format::*;
use crate::util::JsId;
use task::GpuTask;
use rendering_context::Connection;
use task::Progress;
use rendering_context::RenderingContext;
use wasm_bindgen::JsCast;

pub trait Texture<F> where F: TextureFormat {
    type Image: TextureImage<F>;

    fn level_count(&self) -> usize;

    fn layer_count(&self) -> usize;

    fn image(&self, level: usize, layer: usize) -> Option<Self::Image>;
}

pub trait TextureImage<F> where F: TextureFormat {
    fn width(&self) -> u32;

    fn height(&self) -> u32;

    fn upload_task<D, T>(&self, data: D, region: ImageRegion) -> TextureImageUploadTask<T> where D: Into<ImageSource<T>>, T: ClientFormat<F>;
}

pub enum ImageRegion {
    Fill,
    Rectangle(u32, u32, u32, u32)
}

pub struct ImageSource<P> {
    _mark: marker::PhantomData<[P]>
}

pub unsafe trait TextureFormat: InternalFormat {}

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
unsafe impl TextureFormat for SRGB8_ALPHA8 {}
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
unsafe impl TextureFormat for LuminanceAlpha {}

pub struct Texture2D<F, C> where C: RenderingContext {
    data: Arc<TextureData<C>>,
    _format: marker::PhantomData<Box<[F]>>
}

#[derive(Debug)]
struct TextureData<C> where C: RenderingContext {
    gl_object_id: Option<JsId>,
    context: C,
    width: u32,
    height: u32,
    level_count: usize
}

impl<F, C> Texture<F> for Texture2D<F, C> where F: TextureFormat, C: RenderingContext {
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
                data: TextureImageData {
                    texture_data: self.data.clone(),
                    level,
                    target: TextureImageTarget::Texture2D
                },
                _format: marker::PhantomData
            })
        } else {
            None
        }
    }
}

impl<C> Drop for TextureData<C> where C: RenderingContext {
    fn drop(&mut self) {
        if let Some(id) = self.gl_object_id {
            self.context.submit(TextureDropTask {
                id
            });
        }
    }
}

struct TextureDropTask {
    id: JsId
}

impl GpuTask<Connection> for TextureDropTask {
    type Output = ();

    type Error = ();

    fn progress(&mut self, connection: &mut Connection) -> Progress<Self::Output, Self::Error> {
        let Connection(gl, _) = connection;

        unsafe {
            gl.delete_texture(Some(&JsId::into_value(self.id).unchecked_into()));
        }

        Progress::Finished(Ok(()))
    }
}

pub struct Texture2DImage<F, C> where C: RenderingContext {
    data: TextureImageData<C>,
    _format: marker::PhantomData<Box<[F]>>
}

#[derive(Clone, Debug)]
pub(crate) struct TextureImageData<C> where C: RenderingContext {
    texture_data: Arc<TextureData<C>>,
    level: usize,
    target: TextureImageTarget
}

pub(crate) trait TextureImageReference<C> where C: RenderingContext {
    fn reference(&self) -> TextureImageData<C>;
}

#[derive(Clone, Debug)]
enum TextureImageTarget {
    Texture2D,
    TextureCubeMapPositiveX,
    TextureCubeMapNegativeX,
    TextureCubeMapPositiveY,
    TextureCubeMapNegativeY,
    TextureCubeMapPositiveZ,
    TextureCubeMapNegativeZ,
    Texture2DArray(usize),
    Texture3D(usize)
}

impl<F, C> TextureImage<F> for Texture2DImage<F, C> where F: TextureFormat, C: RenderingContext {
    fn width(&self) -> u32 {
        let base_width = self.data.texture_data.width;
        let level_width = base_width / 2 ^ (self.data.level as u32);

        if level_width < 1 {
            1
        } else {
            level_width
        }
    }

    fn height(&self) -> u32 {
        let base_height = self.data.texture_data.height;
        let level_height = base_height / 2 ^ (self.data.level as u32);

        if level_height < 1 {
            1
        } else {
            level_height
        }
    }

    fn upload_task<D, T>(&self, data: D, region: ImageRegion) -> TextureImageUploadTask<T> where D: Into<ImageSource<T>>, T: ClientFormat<F> {
        TextureImageUploadTask {
            data: data.into(),
            region
        }
    }
}

impl<F, C> TextureImageReference<C> for Texture2DImage<F, C> where C: RenderingContext {
    fn reference(&self) -> TextureImageData<C> {
        self.data.clone()
    }
}

//struct Texture2DArray {
//
//}
//
//impl Texture2DArray {
//    pub fn width(&self) -> usize {
//
//    }
//
//    pub fn height(&self) -> usize {
//
//    }
//}

pub struct TextureImageUploadTask<T> {
    data: ImageSource<T>,
    region: ImageRegion
}

