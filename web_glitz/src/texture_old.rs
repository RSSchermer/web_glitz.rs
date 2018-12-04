use std::marker;
use std::sync::Arc;

use crate::image_format::*;
use crate::util::JsId;
use rendering_context::Connection;
use rendering_context::RenderingContext;
use task::GpuTask;
use task::Progress;
use wasm_bindgen::JsCast;

struct LayeredImageRef<F, Rc> {

}

impl<F, Rc> LayeredImageRef<F, Rc> {
    pub fn layer(&self, index: usize) -> Option<Image2DRef<F, Rc>> {

    }

    pub fn upload_task<D, T>(&self, data: D, region: ImageRegion) -> LayeredImageUploadTask<T>
        where
            D: Into<LayeredImageSource<T>>,
            T: ClientFormat<F>,
    {
        LayeredImageUploadTask {
            data: data.into(),
            region,
        }
    }
}

impl<F, Rc> IntoIterator for LayeredImageRef<F, Rc> {
    type Item = Image2DRef<F, Rc>;

    type IntoIter = LayeredImageIntoIter<F, Rc>;

    fn into_iter(self) -> LayeredImageIntoIter<F, Rc> {

    }
}

struct LayeredImageIntoIter<F, Rc> {}

impl<F, Rc> Iterator for LayeredImageIntoIter<F, Rc> {
    type Item = Image2DRef<F, Rc>;

    fn next(&mut self) -> Self::Item {

    }
}

struct Image2DRef<F, Rc> {

}

pub trait Texture<F>
where
    F: TextureFormat,
{
    type Image: TextureImage<F>;

    fn level_count(&self) -> usize;

    fn layer_count(&self) -> usize;

    fn image(&self, level: usize, layer: usize) -> Option<Self::Image>;
}

pub trait TextureImage<F>
where
    F: TextureFormat,
{
    fn width(&self) -> u32;

    fn height(&self) -> u32;

    fn upload_task<D, T>(&self, data: D, region: ImageRegion) -> TextureImageUploadTask<T>
    where
        D: Into<ImageSource<T>>,
        T: ClientFormat<F>;
}

//pub enum TextureLevel {
//    Image2D(TextureImage2D),
//
//}

pub enum ImageRegion {
    Fill,
    Rectangle(u32, u32, u32, u32),
}

pub struct ImageSource<P> {
    _mark: marker::PhantomData<[P]>,
}



pub struct Texture2DHandle<F, C>
where
    C: RenderingContext,
{
    data: Arc<TextureData<C>>,
    _format: marker::PhantomData<Box<[F]>>,
}

#[derive(Debug)]
pub(crate) struct TextureData<C>
where
    C: RenderingContext,
{
    pub(crate) gl_object_id: Option<JsId>,
    context: C,
    width: u32,
    height: u32,
    level_count: usize,
}

impl<F, C> Texture<F> for Texture2DHandle<F, C>
where
    F: TextureFormat,
    C: RenderingContext,
{
    type Image = Texture2DImageRef<F, C>;

    fn level_count(&self) -> usize {
        self.data.level_count
    }

    fn layer_count(&self) -> usize {
        1
    }

    fn image(&self, level: usize, layer: usize) -> Option<Self::Image> {
        if layer == 0 && level < self.data.level_count {
            Some(Texture2DImageRef {
                data: TextureImageData {
                    texture_data: self.data.clone(),
                    level,
                    target: TextureImageTarget::Texture2D,
                },
                _format: marker::PhantomData,
            })
        } else {
            None
        }
    }
}

impl<C> Drop for TextureData<C>
where
    C: RenderingContext,
{
    fn drop(&mut self) {
        if let Some(id) = self.gl_object_id {
            self.context.submit(TextureDropTask { id });
        }
    }
}

struct TextureDropTask {
    id: JsId,
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

pub struct Texture2DImageRef<F, C>
where
    C: RenderingContext,
{
    pub(crate) data: TextureImageData<C>,
    _format: marker::PhantomData<Box<[F]>>,

    //TODO: track
}

#[derive(Clone, Debug)]
pub(crate) struct TextureImageData<C>
where
    C: RenderingContext,
{
    pub(crate) texture_data: Arc<TextureData<C>>,
    pub(crate) level: usize,
    pub(crate) target: TextureImageTarget,
}

#[derive(Clone, Debug)]
#[allow(unused)]
pub(crate) enum TextureImageTarget {
    Texture2D,
    TextureCubeMapPositiveX,
    TextureCubeMapNegativeX,
    TextureCubeMapPositiveY,
    TextureCubeMapNegativeY,
    TextureCubeMapPositiveZ,
    TextureCubeMapNegativeZ,
    Texture2DArray(usize),
    Texture3D(usize),
}

impl<F, C> TextureImage<F> for Texture2DImageRef<F, C>
where
    F: TextureFormat,
    C: RenderingContext,
{
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

    fn upload_task<D, T>(&self, data: D, region: ImageRegion) -> TextureImageUploadTask<T>
    where
        D: Into<ImageSource<T>>,
        T: ClientFormat<F>,
    {
        TextureImageUploadTask {
            data: data.into(),
            region,
        }
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
    region: ImageRegion,
}

struct Texture2DHandle<F, Rc> {}

struct Texture2DLevels<F, Rc> {}

struct TextureCubeHandle<F, Rc> {}

struct TextureCubeLevels<F, Rc> {}

struct Texture2DArrayHandle<F, Rc> {}

struct Texture2DArrayLevels<F, Rc> {}

struct Texture3DHandle<F, Rc> {}

struct Texture3DLevels<F, Rc> {}
