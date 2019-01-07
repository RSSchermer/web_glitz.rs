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
use crate::runtime::{Connection, RenderingContext};
use crate::runtime::dropper::{DropObject, Dropper, RefCountedDropper};
use crate::runtime::dynamic_state::ContextUpdate;
use crate::task::{GpuTask, Progress};
use crate::texture::image_source::{Image2DSourceInternal, Image3DSourceInternal};
use crate::texture::{Image2DSource, Image3DSource, TextureFormat};
use crate::texture::util::{
    mipmap_size,
    region_2d_overlap_height,
    region_2d_overlap_width,
    region_2d_sub_image,
    region_3d_overlap_depth,
    region_3d_overlap_height,
    region_3d_overlap_width,
    region_3d_sub_image,
};
use crate::util::{JsId, arc_get_mut_unchecked, identical};

#[derive(Clone)]
pub struct Texture3DHandle<F> {
    data: Arc<Texture3DData>,
    _marker: marker::PhantomData<[F]>,
}

impl<F> Texture3DHandle<F> {
    pub(crate) fn bind(&self, connection: &mut Connection) -> u32 {
        let Connection(gl, state) = connection;

        unsafe {
            let data = arc_get_mut_unchecked(&self.data);
            let most_recent_unit = &mut data.most_recent_unit;

            data.id.unwrap().with_value_unchecked(|texture_object| {
                if most_recent_unit.is_none()
                    || !identical(
                        state.texture_units_textures()[most_recent_unit.unwrap() as usize].as_ref(),
                        Some(&texture_object),
                    ) {
                    state.set_active_texture_lru().apply(gl).unwrap();
                    state
                        .set_bound_texture_3d(Some(&texture_object))
                        .apply(gl)
                        .unwrap();

                    let unit = state.active_texture();

                    *most_recent_unit = Some(unit);

                    unit
                } else {
                    most_recent_unit.unwrap()
                }
            })
        }
    }
}

impl<F> Texture3DHandle<F>
where
    F: TextureFormat + 'static,
{
    pub(crate) fn new<Rc>(
        context: &Rc,
        dropper: RefCountedDropper,
        width: u32,
        height: u32,
        depth: u32,
        levels: usize,
    ) -> Self
    where
        Rc: RenderingContext,
    {
        let data = Arc::new(Texture3DData {
            id: None,
            dropper,
            width,
            height,
            depth,
            levels,
            most_recent_unit: None,
        });

        context.submit(Texture3DAllocateTask::<F> {
            data: data.clone(),
            _marker: marker::PhantomData,
        });

        Texture3DHandle {
            data,
            _marker: marker::PhantomData,
        }
    }

    pub fn base_level(&self) -> Texture3DLevel<F> {
        Texture3DLevel {
            texture_data: self.data.clone(),
            level: 0,
            _marker: marker::PhantomData,
        }
    }

    pub fn levels(&self) -> Texture3DLevels<F> {
        Texture3DLevels {
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

pub(crate) struct Texture3DData {
    pub(crate) id: Option<JsId>,
    dropper: RefCountedDropper,
    width: u32,
    height: u32,
    depth: u32,
    levels: usize,
    pub(crate) most_recent_unit: Option<u32>,
}

impl Drop for Texture3DData {
    fn drop(&mut self) {
        if let Some(id) = self.id {
            self.dropper.drop_gl_object(DropObject::Texture(id));
        }
    }
}

#[derive(Clone)]
pub struct Texture3DLevels<F> {
    texture_data: Arc<Texture3DData>,
    _marker: marker::PhantomData<[F]>,
}

impl<F> Texture3DLevels<F>
where
    F: TextureFormat,
{
    pub fn len(&self) -> usize {
        self.texture_data.levels
    }

    pub fn get(&self, level: usize) -> Option<Texture3DLevel<F>> {
        let texture_data = &self.texture_data;

        if level < texture_data.levels {
            Some(Texture3DLevel {
                texture_data: texture_data.clone(),
                level,
                _marker: marker::PhantomData,
            })
        } else {
            None
        }
    }

    pub unsafe fn get_unchecked(&self, level: usize) -> Texture3DLevel<F> {
        Texture3DLevel {
            texture_data: self.texture_data.clone(),
            level,
            _marker: marker::PhantomData,
        }
    }

    pub fn iter(&self) -> Texture3DLevelsIter<F> {
        Texture3DLevelsIter {
            texture_data: self.texture_data.clone(),
            current_level: 0,
            end_level: self.texture_data.levels,
            _marker: marker::PhantomData,
        }
    }
}

impl<F> IntoIterator for Texture3DLevels<F>
where
    F: TextureFormat,
{
    type Item = Texture3DLevel<F>;

    type IntoIter = Texture3DLevelsIter<F>;

    fn into_iter(self) -> Self::IntoIter {
        Texture3DLevelsIter {
            current_level: 0,
            end_level: self.texture_data.levels,
            texture_data: self.texture_data.clone(),
            _marker: marker::PhantomData,
        }
    }
}

pub struct Texture3DLevelsIter<F> {
    texture_data: Arc<Texture3DData>,
    current_level: usize,
    end_level: usize,
    _marker: marker::PhantomData<[F]>,
}

impl<F> Iterator for Texture3DLevelsIter<F>
where
    F: TextureFormat,
{
    type Item = Texture3DLevel<F>;

    fn next(&mut self) -> Option<Self::Item> {
        let level = self.current_level;

        if level < self.end_level {
            self.current_level += 1;

            Some(Texture3DLevel {
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
pub struct Texture3DLevel<F> {
    texture_data: Arc<Texture3DData>,
    level: usize,
    _marker: marker::PhantomData<[F]>,
}

impl<F> Texture3DLevel<F>
where
    F: TextureFormat,
{
    pub fn texture(&self) -> Texture3DHandle<F> {
        Texture3DHandle {
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

    pub fn layers(&self) -> Texture3DLevelLayers<F> {
        Texture3DLevelLayers {
            texture_data: self.texture_data.clone(),
            level: self.level,
            _marker: marker::PhantomData,
        }
    }

    pub fn sub_image(&self, region: Region3D) -> Texture3DLevelSubImage<F> {
        Texture3DLevelSubImage {
            texture_data: self.texture_data.clone(),
            level: self.level,
            region,
            _marker: marker::PhantomData,
        }
    }

    pub fn upload_task<D, T>(&self, data: Image3DSource<D, T>) -> Texture3DLevelUploadTask<D, T, F>
    where
        T: ClientFormat<F>,
    {
        Texture3DLevelUploadTask {
            data,
            texture_data: self.texture_data.clone(),
            level: self.level,
            region: Region3D::Fill,
            _marker: marker::PhantomData,
        }
    }
}

#[derive(Clone)]
pub struct Texture3DLevelLayers<F> {
    texture_data: Arc<Texture3DData>,
    level: usize,
    _marker: marker::PhantomData<[F]>,
}

impl<F> Texture3DLevelLayers<F>
where
    F: TextureFormat,
{
    pub fn len(&self) -> usize {
        self.texture_data.depth as usize
    }

    pub fn get(&self, index: usize) -> Option<Texture3DLevelLayer<F>> {
        if index < self.texture_data.depth as usize {
            Some(Texture3DLevelLayer {
                texture_data: self.texture_data.clone(),
                level: self.level,
                layer: index,
                _marker: marker::PhantomData,
            })
        } else {
            None
        }
    }

    pub fn get_unchecked(&self, index: usize) -> Texture3DLevelLayer<F> {
        Texture3DLevelLayer {
            texture_data: self.texture_data.clone(),
            level: self.level,
            layer: index,
            _marker: marker::PhantomData,
        }
    }

    pub fn iter(&self) -> Texture3DLevelLayersIter<F> {
        Texture3DLevelLayersIter {
            level: self.level,
            current_layer: 0,
            end_layer: self.texture_data.depth as usize,
            texture_data: self.texture_data.clone(),
            _marker: marker::PhantomData,
        }
    }
}

impl<F> IntoIterator for Texture3DLevelLayers<F>
where
    F: TextureFormat,
{
    type Item = Texture3DLevelLayer<F>;

    type IntoIter = Texture3DLevelLayersIter<F>;

    fn into_iter(self) -> Self::IntoIter {
        Texture3DLevelLayersIter {
            level: self.level,
            current_layer: 0,
            end_layer: self.texture_data.depth as usize,
            texture_data: self.texture_data,
            _marker: marker::PhantomData,
        }
    }
}

pub struct Texture3DLevelLayersIter<F> {
    texture_data: Arc<Texture3DData>,
    level: usize,
    current_layer: usize,
    end_layer: usize,
    _marker: marker::PhantomData<[F]>,
}

impl<F> Iterator for Texture3DLevelLayersIter<F>
where
    F: TextureFormat,
{
    type Item = Texture3DLevelLayer<F>;

    fn next(&mut self) -> Option<Self::Item> {
        let layer = self.current_layer;

        if layer < self.end_layer {
            self.current_layer += 1;

            Some(Texture3DLevelLayer {
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
pub struct Texture3DLevelLayer<F> {
    texture_data: Arc<Texture3DData>,
    level: usize,
    layer: usize,
    _marker: marker::PhantomData<[F]>,
}

impl<F> Texture3DLevelLayer<F>
where
    F: TextureFormat,
{
    pub fn texture(&self) -> Texture3DHandle<F> {
        Texture3DHandle {
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

    pub fn sub_image(&self, region: Region2D) -> Texture3DLevelLayerSubImage<F> {
        Texture3DLevelLayerSubImage {
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
    ) -> Texture3DLevelLayerUploadTask<D, T, F>
    where
        T: ClientFormat<F>,
    {
        Texture3DLevelLayerUploadTask {
            data,
            texture_data: self.texture_data.clone(),
            level: self.level,
            layer: self.layer,
            region: Region2D::Fill,
            _marker: marker::PhantomData,
        }
    }
}

impl<F> AsFramebufferAttachment for Texture3DLevelLayer<F> {
    fn as_framebuffer_attachment(&self) -> FramebufferAttachment {
        FramebufferAttachment {
            internal: FramebufferAttachmentInternal::Texture3DLevelLayer(
                self.texture_data.clone(),
                self.level,
                self.layer,
            ),
        }
    }
}

#[derive(Clone)]
pub struct Texture3DLevelSubImage<F> {
    texture_data: Arc<Texture3DData>,
    level: usize,
    region: Region3D,
    _marker: marker::PhantomData<[F]>,
}

impl<F> Texture3DLevelSubImage<F>
where
    F: TextureFormat,
{
    pub fn texture(&self) -> Texture3DHandle<F> {
        Texture3DHandle {
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

    pub fn layers(&self) -> Texture3DLevelSubImageLayers<F> {
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

        Texture3DLevelSubImageLayers {
            texture_data: self.texture_data.clone(),
            level: self.level,
            start_layer: start_layer as usize,
            end_layer: end_layer as usize,
            region,
            _marker: marker::PhantomData,
        }
    }

    pub fn sub_image(&self, region: Region3D) -> Texture3DLevelSubImage<F> {
        Texture3DLevelSubImage {
            texture_data: self.texture_data.clone(),
            level: self.level,
            region: region_3d_sub_image(self.region, region),
            _marker: marker::PhantomData,
        }
    }

    pub fn upload_task<D, T>(&self, data: Image3DSource<D, T>) -> Texture3DLevelUploadTask<D, T, F>
    where
        T: ClientFormat<F>,
    {
        Texture3DLevelUploadTask {
            data,
            texture_data: self.texture_data.clone(),
            level: self.level,
            region: self.region,
            _marker: marker::PhantomData,
        }
    }
}

#[derive(Clone)]
pub struct Texture3DLevelSubImageLayers<F> {
    texture_data: Arc<Texture3DData>,
    level: usize,
    start_layer: usize,
    end_layer: usize,
    region: Region2D,
    _marker: marker::PhantomData<[F]>,
}

impl<F> Texture3DLevelSubImageLayers<F>
where
    F: TextureFormat,
{
    pub fn len(&self) -> usize {
        self.end_layer - self.start_layer
    }

    pub fn get(&self, index: usize) -> Option<Texture3DLevelLayerSubImage<F>> {
        let layer = self.start_layer + index;

        if layer < self.end_layer {
            Some(Texture3DLevelLayerSubImage {
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

    pub fn get_unchecked(&self, index: usize) -> Texture3DLevelLayerSubImage<F> {
        Texture3DLevelLayerSubImage {
            texture_data: self.texture_data.clone(),
            level: self.level,
            layer: self.start_layer + index,
            region: self.region,
            _marker: marker::PhantomData,
        }
    }

    pub fn iter(&self) -> Texture3DLevelSubImageLayersIter<F> {
        Texture3DLevelSubImageLayersIter {
            texture_data: self.texture_data.clone(),
            level: self.level,
            region: self.region,
            current_layer: self.start_layer as usize,
            end_layer: self.end_layer as usize,
            _marker: marker::PhantomData,
        }
    }
}

impl<F> IntoIterator for Texture3DLevelSubImageLayers<F>
where
    F: TextureFormat,
{
    type Item = Texture3DLevelLayerSubImage<F>;

    type IntoIter = Texture3DLevelSubImageLayersIter<F>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub struct Texture3DLevelSubImageLayersIter<F> {
    texture_data: Arc<Texture3DData>,
    level: usize,
    region: Region2D,
    current_layer: usize,
    end_layer: usize,
    _marker: marker::PhantomData<[F]>,
}

impl<F> Iterator for Texture3DLevelSubImageLayersIter<F>
where
    F: TextureFormat,
{
    type Item = Texture3DLevelLayerSubImage<F>;

    fn next(&mut self) -> Option<Self::Item> {
        let layer = self.current_layer;

        if layer < self.end_layer {
            self.current_layer += 1;

            Some(Texture3DLevelLayerSubImage {
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
pub struct Texture3DLevelLayerSubImage<F> {
    texture_data: Arc<Texture3DData>,
    level: usize,
    layer: usize,
    region: Region2D,
    _marker: marker::PhantomData<[F]>,
}

impl<F> Texture3DLevelLayerSubImage<F>
where
    F: TextureFormat,
{
    pub fn texture(&self) -> Texture3DHandle<F> {
        Texture3DHandle {
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

    pub fn sub_image(&self, region: Region2D) -> Texture3DLevelLayerSubImage<F> {
        Texture3DLevelLayerSubImage {
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
    ) -> Texture3DLevelLayerUploadTask<D, T, F>
    where
        T: ClientFormat<F>,
    {
        Texture3DLevelLayerUploadTask {
            data,
            texture_data: self.texture_data.clone(),
            level: self.level,
            layer: self.layer,
            region: self.region,
            _marker: marker::PhantomData,
        }
    }
}

struct Texture3DAllocateTask<F> {
    data: Arc<Texture3DData>,
    _marker: marker::PhantomData<[F]>,
}

impl<F> GpuTask<Connection> for Texture3DAllocateTask<F>
where
    F: TextureFormat,
{
    type Output = ();

    fn progress(&mut self, connection: &mut Connection) -> Progress<Self::Output> {
        let Connection(gl, state) = connection;
        let mut data = unsafe { arc_get_mut_unchecked(&mut self.data) };

        let texture_object = gl.create_texture().unwrap();

        state.set_active_texture_lru().apply(gl).unwrap();
        state
            .set_bound_texture_3d(Some(&texture_object))
            .apply(gl)
            .unwrap();

        gl.tex_storage_3d(
            Gl::TEXTURE_3D,
            data.levels as i32,
            F::id(),
            data.width as i32,
            data.height as i32,
            data.depth as i32,
        );

        data.id = Some(JsId::from_value(texture_object.into()));

        Progress::Finished(())
    }
}

pub struct Texture3DLevelUploadTask<D, T, F> {
    data: Image3DSource<D, T>,
    texture_data: Arc<Texture3DData>,
    level: usize,
    region: Region3D,
    _marker: marker::PhantomData<[F]>,
}

impl<D, T, F> GpuTask<Connection> for Texture3DLevelUploadTask<D, T, F>
where
    D: Borrow<[T]>,
    T: ClientFormat<F>,
    F: TextureFormat,
{
    type Output = ();

    fn progress(&mut self, connection: &mut Connection) -> Progress<Self::Output> {
        let mut width = region_3d_overlap_width(self.texture_data.width, self.level, &self.region);
        let mut height =
            region_3d_overlap_height(self.texture_data.height, self.level, &self.region);
        let depth = region_3d_overlap_depth(self.texture_data.height, &self.region);

        if width == 0 || height == 0 || depth == 0 {
            return Progress::Finished(());
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

                unsafe {
                    self.texture_data
                        .id
                        .unwrap()
                        .with_value_unchecked(|texture_object| {
                            state
                                .set_bound_texture_3d(Some(texture_object))
                                .apply(gl)
                                .unwrap();
                        });
                }

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
                    Region3D::Area(offset, ..) => offset,
                };
                let element_size = mem::size_of::<T>() as u32;

                unsafe {
                    let len = row_length * image_height * image_count * element_size;
                    let mut data = slice::from_raw_parts(
                        data.borrow() as *const _ as *const u8,
                        (element_size * len) as usize,
                    );
                    let max_len = element_size * row_length * image_height * depth;

                    if max_len > len {
                        data = &data[0..max_len as usize];
                    }

                    gl.tex_sub_image_3d_with_opt_u8_array(
                        Gl::TEXTURE_3D,
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
                    )
                    .unwrap();
                }
            }
        }

        Progress::Finished(())
    }
}

pub struct Texture3DLevelLayerUploadTask<D, T, F> {
    data: Image2DSource<D, T>,
    texture_data: Arc<Texture3DData>,
    level: usize,
    layer: usize,
    region: Region2D,
    _marker: marker::PhantomData<[F]>,
}

impl<D, T, F> GpuTask<Connection> for Texture3DLevelLayerUploadTask<D, T, F>
where
    D: Borrow<[T]>,
    T: ClientFormat<F>,
    F: TextureFormat,
{
    type Output = ();

    fn progress(&mut self, connection: &mut Connection) -> Progress<Self::Output> {
        let mut width = region_2d_overlap_width(self.texture_data.width, self.level, &self.region);
        let height = region_2d_overlap_height(self.texture_data.height, self.level, &self.region);

        if width == 0 || height == 0 {
            return Progress::Finished(());
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

                unsafe {
                    self.texture_data
                        .id
                        .unwrap()
                        .with_value_unchecked(|texture_object| {
                            state
                                .set_bound_texture_3d(Some(texture_object))
                                .apply(gl)
                                .unwrap();
                        });
                }

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
                    Region2D::Area(offset, ..) => offset,
                };
                let element_size = mem::size_of::<T>() as u32;

                unsafe {
                    let len = row_length * image_height * element_size;
                    let mut data = slice::from_raw_parts(
                        data.borrow() as *const _ as *const u8,
                        (element_size * len) as usize,
                    );
                    let max_len = element_size * row_length * height;

                    if max_len > len {
                        data = &data[0..max_len as usize];
                    }

                    gl.tex_sub_image_3d_with_opt_u8_array(
                        Gl::TEXTURE_3D,
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
                    )
                    .unwrap();
                }
            }
        }

        Progress::Finished(())
    }
}
