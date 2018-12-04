use std::borrow::Borrow;
use std::cmp;
use std::marker;
use std::mem;
use std::slice;
use std::sync::Arc;

use wasm_bindgen::JsCast;
use web_sys::WebGl2RenderingContext as Gl;

use crate::framebuffer::framebuffer_handle::FramebufferAttachmentInternal;
use crate::framebuffer::{AsFramebufferAttachment, FramebufferAttachment};
use crate::image_format::ClientFormat;
use crate::image_region::{Region2D, Region3D};
use crate::rendering_context::{Connection, ContextUpdate, RenderingContext};
use crate::task::{GpuTask, Progress};
use crate::texture::image_source::{Image2DSourceInternal, Image3DSourceInternal};
use crate::texture::{Image2DSource, Image3DSource, TextureFormat};
use crate::util::JsId;
use texture::util::mipmap_size;
use texture::util::region_3d_overlap_width;
use texture::util::region_3d_overlap_height;
use texture::util::region_3d_overlap_depth;
use texture::util::region_2d_overlap_width;
use texture::util::region_2d_overlap_height;
use texture::util::region_2d_sub_image;
use texture::util::region_3d_sub_image;

#[derive(Clone)]
pub struct Texture2DArrayHandle<F, Rc> where Rc: RenderingContext {
    data: Arc<Texture2DArrayData<Rc>>,
    _marker: marker::PhantomData<[F]>,
}

impl<F, Rc> Texture2DArrayHandle<F, Rc>
    where
        F: TextureFormat + 'static,
        Rc: RenderingContext + 'static,
{
    pub(crate) fn new(context: &Rc, width: u32, height: u32, depth: u32, levels: usize) -> Self {
        let data = Arc::new(Texture2DArrayData {
            id: None,
            context: context.clone(),
            width,
            height,
            depth,
            levels,
        });

        context.submit(Texture2DArrayAllocateTask::<F, Rc> {
            data: data.clone(),
            _marker: marker::PhantomData,
        });

        Texture2DArrayHandle {
            data,
            _marker: marker::PhantomData,
        }
    }

    pub fn base_level(&self) -> Texture2DArrayLevel<F, Rc> {
        Texture2DArrayLevel {
            texture_data: self.data.clone(),
            level: 0,
            _marker: marker::PhantomData,
        }
    }

    pub fn levels(&self) -> Texture2DArrayLevels<F, Rc> {
        Texture2DArrayLevels {
            texture_data: self.data.clone(),
            _marker: marker::PhantomData,
        }
    }

    pub fn width(&self) -> u32 {
        self.data.width
    }

    pub fn height(&self) -> u32 {
        self.data.height
    }

    pub fn depth(&self) -> u32 {
        self.data.depth
    }
}

pub(crate) struct Texture2DArrayData<Rc> where Rc: RenderingContext {
    pub(crate) id: Option<JsId>,
    context: Rc,
    width: u32,
    height: u32,
    depth: u32,
    levels: usize,
}

impl<Rc> Drop for Texture2DArrayData<Rc>
    where
        Rc: RenderingContext,
{
    fn drop(&mut self) {
        if let Some(id) = self.id {
            self.context.submit(Texture2DArrayDropTask { id });
        }
    }
}

#[derive(Clone)]
pub struct Texture2DArrayLevels<F, Rc> where Rc: RenderingContext {
    texture_data: Arc<Texture2DArrayData<Rc>>,
    _marker: marker::PhantomData<[F]>,
}

impl<F, Rc> Texture2DArrayLevels<F, Rc>
    where
        F: TextureFormat,
        Rc: RenderingContext,
{
    pub fn len(&self) -> usize {
        self.texture_data.levels
    }

    pub fn get(&self, level: usize) -> Option<Texture2DArrayLevel<F, Rc>> {
        let texture_data = &self.texture_data;

        if level < texture_data.levels {
            Some(Texture2DArrayLevel {
                texture_data: texture_data.clone(),
                level,
                _marker: marker::PhantomData,
            })
        } else {
            None
        }
    }

    pub unsafe fn get_unchecked(&self, level: usize) -> Texture2DArrayLevel<F, Rc> {
        Texture2DArrayLevel {
            texture_data: self.texture_data.clone(),
            level,
            _marker: marker::PhantomData,
        }
    }

    pub fn iter(&self) -> Texture2DArrayLevelsIter<F, Rc> {
        Texture2DArrayLevelsIter {
            texture_data: self.texture_data.clone(),
            current_level: 0,
            end_level: self.texture_data.levels,
            _marker: marker::PhantomData,
        }
    }
}

impl<F, Rc> IntoIterator for Texture2DArrayLevels<F, Rc>
    where
        F: TextureFormat,
        Rc: RenderingContext,
{
    type Item = Texture2DArrayLevel<F, Rc>;

    type IntoIter = Texture2DArrayLevelsIter<F, Rc>;

    fn into_iter(self) -> Self::IntoIter {
        Texture2DArrayLevelsIter {
            current_level: 0,
            end_level: self.texture_data.levels,
            texture_data: self.texture_data,
            _marker: marker::PhantomData,
        }
    }
}

pub struct Texture2DArrayLevelsIter<F, Rc> where Rc: RenderingContext {
    texture_data: Arc<Texture2DArrayData<Rc>>,
    current_level: usize,
    end_level: usize,
    _marker: marker::PhantomData<[F]>,
}

impl<F, Rc> Iterator for Texture2DArrayLevelsIter<F, Rc>
    where
        F: TextureFormat,
        Rc: RenderingContext,
{
    type Item = Texture2DArrayLevel<F, Rc>;

    fn next(&mut self) -> Option<Self::Item> {
        let level = self.current_level;

        if level < self.end_level {
            self.current_level += 1;

            Some(Texture2DArrayLevel {
                texture_data: self.texture_data.clone(),
                level,
                _marker: marker::PhantomData,
            })
        } else {
            None
        }
    }
}

#[derive(Clone)]
pub struct Texture2DArrayLevel<F, Rc> where Rc: RenderingContext {
    texture_data: Arc<Texture2DArrayData<Rc>>,
    level: usize,
    _marker: marker::PhantomData<[F]>,
}

impl<F, Rc> Texture2DArrayLevel<F, Rc>
    where
        F: TextureFormat,
        Rc: RenderingContext,
{
    pub fn texture(&self) -> Texture2DArrayHandle<F, Rc> {
        Texture2DArrayHandle {
            data: self.texture_data.clone(),
            _marker: marker::PhantomData,
        }
    }

    pub fn level(&self) -> usize {
        self.level
    }

    pub fn width(&self) -> u32 {
        mipmap_size(self.texture_data.width, self.level)
    }

    pub fn height(&self) -> u32 {
        mipmap_size(self.texture_data.height, self.level)
    }

    pub fn depth(&self) -> u32 {
        self.texture_data.depth
    }

    pub fn layers(&self) -> Texture2DArrayLevelLayers<F, Rc> {
        Texture2DArrayLevelLayers {
            texture_data: self.texture_data.clone(),
            level: self.level,
            _marker: marker::PhantomData,
        }
    }

    pub fn sub_image(&self, region: Region3D) -> Texture2DArrayLevelSubImage<F, Rc> {
        Texture2DArrayLevelSubImage {
            texture_data: self.texture_data.clone(),
            level: self.level,
            region,
            _marker: marker::PhantomData,
        }
    }

    pub fn upload_task<D, T>(
        &self,
        data: Image3DSource<D, T>,
    ) -> Texture2DArrayLevelUploadTask<D, T, F, Rc>
        where
            T: ClientFormat<F>,
    {
        Texture2DArrayLevelUploadTask {
            data,
            texture_data: self.texture_data.clone(),
            level: self.level,
            region: Region3D::Fill,
            _marker: marker::PhantomData,
        }
    }
}

#[derive(Clone)]
pub struct Texture2DArrayLevelLayers<F, Rc> where Rc: RenderingContext {
    texture_data: Arc<Texture2DArrayData<Rc>>,
    level: usize,
    _marker: marker::PhantomData<[F]>,
}

impl<F, Rc> Texture2DArrayLevelLayers<F, Rc>
    where
        F: TextureFormat,
        Rc: RenderingContext,
{
    pub fn len(&self) -> usize {
        self.texture_data.depth as usize
    }

    pub fn get(&self, index: usize) -> Option<Texture2DArrayLevelLayer<F, Rc>> {
        if index < self.texture_data.depth as usize {
            Some(Texture2DArrayLevelLayer {
                texture_data: self.texture_data.clone(),
                level: self.level,
                layer: index,
                _marker: marker::PhantomData,
            })
        } else {
            None
        }
    }

    pub fn get_unchecked(&self, index: usize) -> Texture2DArrayLevelLayer<F, Rc> {
        Texture2DArrayLevelLayer {
            texture_data: self.texture_data.clone(),
            level: self.level,
            layer: index,
            _marker: marker::PhantomData,
        }
    }

    pub fn iter(&self) -> Texture2DArrayLevelLayersIter<F, Rc> {
        Texture2DArrayLevelLayersIter {
            texture_data: self.texture_data.clone(),
            level: self.level,
            current_layer: 0,
            end_layer: self.texture_data.depth as usize,
            _marker: marker::PhantomData,
        }
    }
}

impl<F, Rc> IntoIterator for Texture2DArrayLevelLayers<F, Rc>
    where
        F: TextureFormat,
        Rc: RenderingContext,
{
    type Item = Texture2DArrayLevelLayer<F, Rc>;

    type IntoIter = Texture2DArrayLevelLayersIter<F, Rc>;

    fn into_iter(self) -> Self::IntoIter {
        Texture2DArrayLevelLayersIter {
            level: self.level,
            current_layer: 0,
            end_layer: self.texture_data.depth as usize,
            texture_data: self.texture_data.clone(),
            _marker: marker::PhantomData,
        }
    }
}

pub struct Texture2DArrayLevelLayersIter<F, Rc> where Rc: RenderingContext {
    texture_data: Arc<Texture2DArrayData<Rc>>,
    level: usize,
    current_layer: usize,
    end_layer: usize,
    _marker: marker::PhantomData<[F]>,
}

impl<F, Rc> Iterator for Texture2DArrayLevelLayersIter<F, Rc>
    where
        F: TextureFormat,
        Rc: RenderingContext,
{
    type Item = Texture2DArrayLevelLayer<F, Rc>;

    fn next(&mut self) -> Option<Self::Item> {
        let layer = self.current_layer;

        if layer < self.end_layer {
            self.current_layer += 1;

            Some(Texture2DArrayLevelLayer {
                texture_data: self.texture_data.clone(),
                level: self.level,
                layer,
                _marker: marker::PhantomData,
            })
        } else {
            None
        }
    }
}

#[derive(Clone)]
pub struct Texture2DArrayLevelLayer<F, Rc> where Rc: RenderingContext {
    texture_data: Arc<Texture2DArrayData<Rc>>,
    level: usize,
    layer: usize,
    _marker: marker::PhantomData<[F]>,
}

impl<F, Rc> Texture2DArrayLevelLayer<F, Rc>
    where
        F: TextureFormat,
        Rc: RenderingContext,
{
    pub fn texture(&self) -> Texture2DArrayHandle<F, Rc> {
        Texture2DArrayHandle {
            data: self.texture_data.clone(),
            _marker: marker::PhantomData,
        }
    }

    pub fn level(&self) -> usize {
        self.level
    }

    pub fn layer(&self) -> usize {
        self.layer
    }

    pub fn width(&self) -> u32 {
        mipmap_size(self.texture_data.width, self.level)
    }

    pub fn height(&self) -> u32 {
        mipmap_size(self.texture_data.height, self.level)
    }

    pub fn sub_image(&self, region: Region2D) -> Texture2DArrayLevelLayerSubImage<F, Rc> {
        Texture2DArrayLevelLayerSubImage {
            texture_data: self.texture_data.clone(),
            level: self.level,
            layer: self.layer,
            region,
            _marker: marker::PhantomData,
        }
    }

    pub fn upload_task<D, T>(
        &self,
        data: Image2DSource<D, T>,
    ) -> Texture2DArrayLevelLayerUploadTask<D, T, F, Rc>
        where
            T: ClientFormat<F>,
    {
        Texture2DArrayLevelLayerUploadTask {
            data,
            texture_data: self.texture_data.clone(),
            level: self.level,
            layer: self.layer,
            region: Region2D::Fill,
            _marker: marker::PhantomData,
        }
    }
}

impl<F, Rc> AsFramebufferAttachment<Rc> for Texture2DArrayLevelLayer<F, Rc>
    where
        Rc: RenderingContext,
{
    fn as_framebuffer_attachment(&self) -> FramebufferAttachment<Rc> {
        FramebufferAttachment {
            internal: FramebufferAttachmentInternal::Texture2DArrayLevelLayer(self.texture_data.clone(), self.level, self.layer),
        }
    }
}

#[derive(Clone)]
pub struct Texture2DArrayLevelSubImage<F, Rc> where Rc: RenderingContext {
    texture_data: Arc<Texture2DArrayData<Rc>>,
    level: usize,
    region: Region3D,
    _marker: marker::PhantomData<[F]>,
}

impl<F, Rc> Texture2DArrayLevelSubImage<F, Rc>
    where
        F: TextureFormat,
        Rc: RenderingContext,
{
    pub fn texture(&self) -> Texture2DArrayHandle<F, Rc> {
        Texture2DArrayHandle {
            data: self.texture_data.clone(),
            _marker: marker::PhantomData,
        }
    }

    pub fn level(&self) -> usize {
        self.level
    }

    pub fn region(&self) -> Region3D {
        self.region
    }

    pub fn width(&self) -> u32 {
        region_3d_overlap_width(self.texture_data.width, self.level, &self.region)
    }

    pub fn height(&self) -> u32 {
        region_3d_overlap_height(self.texture_data.height, self.level, &self.region)
    }

    pub fn depth(&self) -> u32 {
        region_3d_overlap_depth(self.texture_data.depth, &self.region)
    }

    pub fn layers(&self) -> Texture2DArrayLevelSubImageLayers<F, Rc> {
        let (start_layer, end_layer, region) = match self.region {
            Region3D::Area((offset_x, offset_y, offset_z), width, height, depth) => {
                let max_layer = cmp::min(self.texture_data.depth, offset_z + depth);

                (
                    offset_z,
                    max_layer,
                    Region2D::Area((offset_x, offset_y), width, height),
                )
            }
            Region3D::Fill => (0, self.texture_data.depth, Region2D::Fill),
        };

        Texture2DArrayLevelSubImageLayers {
            texture_data: self.texture_data.clone(),
            level: self.level,
            start_layer: start_layer as usize,
            end_layer: end_layer as usize,
            region,
            _marker: marker::PhantomData,
        }
    }

    pub fn sub_image(&self, region: Region3D) -> Texture2DArrayLevelSubImage<F, Rc> {
        Texture2DArrayLevelSubImage {
            texture_data: self.texture_data.clone(),
            level: self.level,
            region: region_3d_sub_image(self.region, region),
            _marker: marker::PhantomData,
        }
    }

    pub fn upload_task<D, T>(
        &self,
        data: Image3DSource<D, T>,
    ) -> Texture2DArrayLevelUploadTask<D, T, F, Rc>
        where
            T: ClientFormat<F>,
    {
        Texture2DArrayLevelUploadTask {
            data,
            texture_data: self.texture_data.clone(),
            level: self.level,
            region: self.region,
            _marker: marker::PhantomData,
        }
    }
}

#[derive(Clone)]
pub struct Texture2DArrayLevelSubImageLayers<F, Rc> where Rc: RenderingContext {
    texture_data: Arc<Texture2DArrayData<Rc>>,
    level: usize,
    start_layer: usize,
    end_layer: usize,
    region: Region2D,
    _marker: marker::PhantomData<[F]>,
}

impl<F, Rc> Texture2DArrayLevelSubImageLayers<F, Rc>
    where
        F: TextureFormat,
        Rc: RenderingContext,
{
    pub fn len(&self) -> usize {
        self.end_layer - self.start_layer
    }

    pub fn get(&self, index: usize) -> Option<Texture2DArrayLevelLayerSubImage<F, Rc>> {
        let layer = self.start_layer + index;

        if layer < self.end_layer {
            Some(Texture2DArrayLevelLayerSubImage {
                texture_data: self.texture_data.clone(),
                level: self.level,
                layer,
                region: self.region,
                _marker: marker::PhantomData,
            })
        } else {
            None
        }
    }

    pub fn get_unchecked(&self, index: usize) -> Texture2DArrayLevelLayerSubImage<F, Rc> {
        Texture2DArrayLevelLayerSubImage {
            texture_data: self.texture_data.clone(),
            level: self.level,
            layer: self.start_layer + index,
            region: self.region,
            _marker: marker::PhantomData,
        }
    }

    pub fn iter(&self) -> Texture2DArrayLevelSubImageLayersIter<F, Rc> {
        Texture2DArrayLevelSubImageLayersIter {
            texture_data: self.texture_data.clone(),
            level: self.level,
            region: self.region,
            current_layer: self.start_layer as usize,
            end_layer: self.end_layer as usize,
            _marker: marker::PhantomData,
        }
    }
}

impl<F, Rc> IntoIterator for Texture2DArrayLevelSubImageLayers<F, Rc>
    where
        F: TextureFormat,
        Rc: RenderingContext,
{
    type Item = Texture2DArrayLevelLayerSubImage<F, Rc>;

    type IntoIter = Texture2DArrayLevelSubImageLayersIter<F, Rc>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub struct Texture2DArrayLevelSubImageLayersIter<F, Rc> where Rc: RenderingContext {
    texture_data: Arc<Texture2DArrayData<Rc>>,
    level: usize,
    region: Region2D,
    current_layer: usize,
    end_layer: usize,
    _marker: marker::PhantomData<[F]>,
}

impl<F, Rc> Iterator for Texture2DArrayLevelSubImageLayersIter<F, Rc>
    where
        F: TextureFormat,
        Rc: RenderingContext,
{
    type Item = Texture2DArrayLevelLayerSubImage<F, Rc>;

    fn next(&mut self) -> Option<Self::Item> {
        let layer = self.current_layer;

        if layer < self.end_layer {
            self.current_layer += 1;

            Some(Texture2DArrayLevelLayerSubImage {
                texture_data: self.texture_data.clone(),
                level: self.level,
                layer,
                region: self.region,
                _marker: marker::PhantomData,
            })
        } else {
            None
        }
    }
}

#[derive(Clone)]
pub struct Texture2DArrayLevelLayerSubImage<F, Rc> where Rc: RenderingContext {
    texture_data: Arc<Texture2DArrayData<Rc>>,
    level: usize,
    layer: usize,
    region: Region2D,
    _marker: marker::PhantomData<[F]>,
}

impl<F, Rc> Texture2DArrayLevelLayerSubImage<F, Rc>
    where
        F: TextureFormat,
        Rc: RenderingContext,
{
    pub fn texture(&self) -> Texture2DArrayHandle<F, Rc> {
        Texture2DArrayHandle {
            data: self.texture_data.clone(),
            _marker: marker::PhantomData,
        }
    }

    pub fn level(&self) -> usize {
        self.level
    }

    pub fn layer(&self) -> usize {
        self.layer
    }

    pub fn region(&self) -> Region2D {
        self.region
    }

    pub fn width(&self) -> u32 {
        region_2d_overlap_width(self.texture_data.width, self.level, &self.region)
    }

    pub fn height(&self) -> u32 {
        region_2d_overlap_height(self.texture_data.height, self.level, &self.region)
    }

    pub fn sub_image(&self, region: Region2D) -> Texture2DArrayLevelLayerSubImage<F, Rc> {
        Texture2DArrayLevelLayerSubImage {
            texture_data: self.texture_data.clone(),
            level: self.level,
            layer: self.layer,
            region: region_2d_sub_image(self.region, region),
            _marker: marker::PhantomData,
        }
    }

    pub fn upload_task<D, T>(
        &self,
        data: Image2DSource<D, T>,
    ) -> Texture2DArrayLevelLayerUploadTask<D, T, F, Rc>
        where
            T: ClientFormat<F>,
    {
        Texture2DArrayLevelLayerUploadTask {
            data,
            texture_data: self.texture_data.clone(),
            level: self.level,
            layer: self.layer,
            region: self.region,
            _marker: marker::PhantomData,
        }
    }
}

struct Texture2DArrayAllocateTask<F, Rc> where Rc: RenderingContext {
    data: Arc<Texture2DArrayData<Rc>>,
    _marker: marker::PhantomData<[F]>,
}

impl<F, Rc> GpuTask<Connection> for Texture2DArrayAllocateTask<F, Rc>
    where
        F: TextureFormat,
        Rc: RenderingContext,
{
    type Output = ();

    type Error = ();

    fn progress(&mut self, connection: &mut Connection) -> Progress<Self::Output, Self::Error> {
        let Connection(gl, state) = connection;
        let mut data = Arc::get_mut(&mut self.data).unwrap();

        let texture_object = gl.create_texture().unwrap();

        state.set_active_texture_lru().apply(gl).unwrap();
        state
            .set_bound_texture_2d_array(Some(&texture_object))
            .apply(gl)
            .unwrap();

        gl.tex_storage_3d(
            Gl::TEXTURE_2D_ARRAY,
            data.levels as i32,
            F::id(),
            data.width as i32,
            data.height as i32,
            data.depth as i32,
        );

        data.id = Some(JsId::from_value(texture_object.into()));

        Progress::Finished(Ok(()))
    }
}

struct Texture2DArrayDropTask {
    id: JsId,
}

impl GpuTask<Connection> for Texture2DArrayDropTask {
    type Output = ();

    type Error = ();

    fn progress(&mut self, connection: &mut Connection) -> Progress<Self::Output, Self::Error> {
        let Connection(gl, _) = connection;
        let texture_object = unsafe { JsId::into_value(self.id).unchecked_into() };

        gl.delete_texture(Some(&texture_object));

        Progress::Finished(Ok(()))
    }
}

pub struct Texture2DArrayLevelUploadTask<D, T, F, Rc> where Rc: RenderingContext {
    data: Image3DSource<D, T>,
    texture_data: Arc<Texture2DArrayData<Rc>>,
    level: usize,
    region: Region3D,
    _marker: marker::PhantomData<[F]>,
}

impl<D, T, F, Rc> GpuTask<Connection> for Texture2DArrayLevelUploadTask<D, T, F, Rc>
    where
        D: Borrow<[T]>,
        T: ClientFormat<F>,
        F: TextureFormat,
        Rc: RenderingContext,
{
    type Output = ();

    type Error = ();

    fn progress(&mut self, connection: &mut Connection) -> Progress<Self::Output, Self::Error> {
        let mut width = region_3d_overlap_width(self.texture_data.width, self.level, &self.region);
        let mut height =
            region_3d_overlap_height(self.texture_data.height, self.level, &self.region);
        let depth = region_3d_overlap_depth(self.texture_data.height, &self.region);

        if width == 0 || height == 0 || depth == 0 {
            return Progress::Finished(Ok(()));
        }

        let Connection(gl, state) = connection;

        match &self.data.internal {
            Image3DSourceInternal::PixelData {
                data,
                row_length,
                image_height,
                image_count,
                alignment,
            } => {
                state.set_active_texture_lru().apply(gl).unwrap();

                self.texture_data
                    .id
                    .unwrap()
                    .with_value_unchecked(|texture_object| {
                        state
                            .set_bound_texture_2d_array(Some(texture_object))
                            .apply(gl)
                            .unwrap();
                    });

                state
                    .set_pixel_unpack_alignment((*alignment).into())
                    .apply(gl)
                    .unwrap();

                if width < *row_length {
                    state
                        .set_pixel_unpack_row_length(*row_length as i32)
                        .apply(gl)
                        .unwrap();
                } else {
                    width = *row_length;

                    state.set_pixel_unpack_row_length(0).apply(gl).unwrap();
                }

                if height < *image_height {
                    state
                        .set_pixel_unpack_image_height(*image_height as i32)
                        .apply(gl)
                        .unwrap();
                } else {
                    height = *image_height;

                    state.set_pixel_unpack_image_height(0).apply(gl).unwrap();
                }

                let (offset_x, offset_y, offset_z) = match self.region {
                    Region3D::Fill => (0, 0, 0),
                    Region3D::Area(offset, ..) => offset
                };

                let element_size = mem::size_of::<T>() as u32;

                unsafe {
                    let len = row_length * image_height * image_count * element_size;
                    let mut data = slice::from_raw_parts(
                        self.data.borrow() as *const _ as *const u8,
                        (element_size * len) as usize,
                    );
                    let max_len = element_size * row_length * image_height * depth;

                    if max_len > len {
                        data = &data[0..max_len as usize];
                    }

                    gl.tex_sub_image_3d_with_opt_u8_array(
                        Gl::TEXTURE_2D_ARRAY,
                        self.level as i32,
                        offset_x as i32,
                        offset_y as i32,
                        offset_z as i32,
                        width as i32,
                        height as i32,
                        depth as i32,
                        T::format_id(),
                        T::type_id(),
                        Some(&mut *(data as *const _ as *mut _)),
                    );
                }
            }
        }

        Progress::Finished(Ok(()))
    }
}

pub struct Texture2DArrayLevelLayerUploadTask<D, T, F, Rc> where Rc: RenderingContext {
    data: Image2DSource<D, T>,
    texture_data: Arc<Texture2DArrayData<Rc>>,
    level: usize,
    layer: usize,
    region: Region2D,
    _marker: marker::PhantomData<[F]>,
}

impl<D, T, F, Rc> GpuTask<Connection> for Texture2DArrayLevelLayerUploadTask<D, T, F, Rc>
    where
        D: Borrow<[T]>,
        T: ClientFormat<F>,
        F: TextureFormat,
        Rc: RenderingContext,
{
    type Output = ();

    type Error = ();

    fn progress(&mut self, connection: &mut Connection) -> Progress<Self::Output, Self::Error> {
        let mut width = region_2d_overlap_width(self.texture_data.width, self.level, &self.region);
        let height = region_2d_overlap_height(self.texture_data.height, self.level, &self.region);

        if width == 0 || height == 0 {
            return Progress::Finished(Ok(()));
        }

        let Connection(gl, state) = connection;

        match &self.data.internal {
            Image2DSourceInternal::PixelData {
                data,
                row_length,
                image_height,
                alignment,
            } => {
                state.set_active_texture_lru().apply(gl).unwrap();

                self.texture_data
                    .id
                    .unwrap()
                    .with_value_unchecked(|texture_object| {
                        state
                            .set_bound_texture_2d_array(Some(texture_object))
                            .apply(gl)
                            .unwrap();
                    });

                state
                    .set_pixel_unpack_alignment((*alignment).into())
                    .apply(gl)
                    .unwrap();

                if width < *row_length {
                    state
                        .set_pixel_unpack_row_length(*row_length as i32)
                        .apply(gl)
                        .unwrap();
                } else {
                    width = *row_length;

                    state.set_pixel_unpack_row_length(0).apply(gl).unwrap();
                }

                let (offset_x, offset_y) = match self.region {
                    Region2D::Fill => (0, 0),
                    Region2D::Area(offset, ..) => offset
                };
                let element_size = mem::size_of::<T>() as u32;

                unsafe {
                    let len= row_length * image_height * element_size;
                    let mut data = slice::from_raw_parts(
                        self.data.borrow() as *const _ as *const u8,
                        (element_size * len) as usize,
                    );
                    let max_len = element_size * row_length * height;

                    if max_len > len {
                        data = &data[0..max_len as usize];
                    }

                    gl.tex_sub_image_3d_with_opt_u8_array(
                        Gl::TEXTURE_2D_ARRAY,
                        self.level as i32,
                        offset_x as i32,
                        offset_y as i32,
                        self.layer as i32,
                        width as i32,
                        height as i32,
                        1,
                        T::format_id(),
                        T::type_id(),
                        Some(&mut *(data as *const _ as *mut _)),
                    );
                }
            }
        }

        Progress::Finished(Ok(()))
    }
}
