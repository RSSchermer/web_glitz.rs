use std::borrow::Borrow;
use std::cmp;
use std::marker;
use std::mem;
use std::slice;
use std::sync::Arc;

use wasm_bindgen::JsCast;
use web_sys::WebGl2RenderingContext as Gl;

use crate::image::format::{ClientFormat, TextureFormat};
use crate::image::image_source::{Image2DSourceInternal, Image3DSourceInternal};
use crate::image::texture_object_dropper::TextureObjectDropper;
use crate::image::util::{
    mipmap_size, region_2d_overlap_height, region_2d_overlap_width, region_2d_sub_image,
    region_3d_overlap_depth, region_3d_overlap_height, region_3d_overlap_width,
    region_3d_sub_image,
};
use crate::image::{Image2DSource, Image3DSource, Region2D, Region3D};
use crate::runtime::dynamic_state::ContextUpdate;
use crate::runtime::{Connection, ContextMismatch, RenderingContext};
use crate::task::{GpuTask, Progress};
use crate::util::{arc_get_mut_unchecked, identical, JsId};
use std::hash::Hash;
use std::hash::Hasher;

pub struct Texture2DArray<F> {
    data: Arc<Texture2DArrayData>,
    _marker: marker::PhantomData<[F]>,
}

impl<F> Texture2DArray<F> {
    pub(crate) fn bind(&self, connection: &mut Connection) -> u32 {
        let (gl, state) = unsafe { connection.unpack_mut() };

        unsafe {
            let data = arc_get_mut_unchecked(&self.data);
            let most_recent_unit = &mut data.most_recent_unit;

            data.id.unwrap().with_value_unchecked(|texture_object| {
                if most_recent_unit.is_none()
                    || !identical(
                        state.texture_units_textures()[most_recent_unit.unwrap() as usize].as_ref(),
                        Some(&texture_object),
                    )
                {
                    state.set_active_texture_lru().apply(gl).unwrap();
                    state
                        .set_bound_texture_2d_array(Some(&texture_object))
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

impl<F> Texture2DArray<F>
where
    F: TextureFormat + 'static,
{
    pub(crate) fn new<Rc>(context: &Rc, width: u32, height: u32, depth: u32, levels: usize) -> Self
    where
        Rc: RenderingContext + Clone + 'static,
    {
        let data = Arc::new(Texture2DArrayData {
            id: None,
            context_id: context.id(),
            dropper: Box::new(context.clone()),
            width,
            height,
            depth,
            levels,
            most_recent_unit: None,
        });

        context.submit(AllocateCommand::<F> {
            data: data.clone(),
            _marker: marker::PhantomData,
        });

        Texture2DArray {
            data,
            _marker: marker::PhantomData,
        }
    }

    pub fn base_level(&self) -> Level<F> {
        Level {
            handle: self,
            level: 0,
        }
    }

    pub fn levels(&self) -> Levels<F> {
        Levels {
            handle: self,
            offset: 0,
            len: self.data.levels
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

pub(crate) struct Texture2DArrayData {
    id: Option<JsId>,
    context_id: usize,
    dropper: Box<TextureObjectDropper>,
    width: u32,
    height: u32,
    depth: u32,
    levels: usize,
    most_recent_unit: Option<u32>,
}

impl Texture2DArrayData {
    pub(crate) fn id(&self) -> Option<JsId> {
        self.id
    }
}

impl PartialEq for Texture2DArrayData {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Hash for Texture2DArrayData {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.id.hash(state);
    }
}

impl Drop for Texture2DArrayData {
    fn drop(&mut self) {
        if let Some(id) = self.id {
            self.dropper.drop_texture_object(id);
        }
    }
}

#[derive(Clone)]
pub struct Levels<'a, F> {
    handle: &'a Texture2DArray<F>,
    offset: usize,
    len: usize
}

impl<'a, F> Levels<'a, F>
where
    F: TextureFormat,
{
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn iter(&self) -> LevelsIter<F> {
        LevelsIter {
            handle: self.handle,
            current_level: self.offset,
            end_level: self.offset + self.len,
        }
    }
}

impl<'a, F> IntoIterator for Levels<'a, F>
where
    F: TextureFormat,
{
    type Item = Level<'a, F>;

    type IntoIter = LevelsIter<'a, F>;

    fn into_iter(self) -> Self::IntoIter {
        LevelsIter {
            handle: self.handle,
            current_level: self.offset,
            end_level: self.offset + self.len,
        }
    }
}

pub struct LevelsIter<'a, F> {
    handle: &'a Texture2DArray<F>,
    current_level: usize,
    end_level: usize,
}

impl<'a, F> Iterator for LevelsIter<'a, F>
where
    F: TextureFormat,
{
    type Item = Level<'a, F>;

    fn next(&mut self) -> Option<Self::Item> {
        let level = self.current_level;

        if level < self.end_level {
            self.current_level += 1;

            Some(Level {
                handle: self.handle,
                level,
            })
        } else {
            None
        }
    }
}

#[derive(Clone)]
pub struct Level<'a, F> {
    handle: &'a Texture2DArray<F>,
    level: usize,
}

impl<'a, F> Level<'a, F>
where
    F: TextureFormat,
{
    pub(crate) fn texture_data(&self) -> &Arc<Texture2DArrayData> {
        &self.handle.data
    }

    pub fn level(&self) -> usize {
        self.level
    }

    pub fn width(&self) -> u32 {
        mipmap_size(self.handle.data.width, self.level)
    }

    pub fn height(&self) -> u32 {
        mipmap_size(self.handle.data.height, self.level)
    }

    pub fn depth(&self) -> u32 {
        self.handle.data.depth
    }

    pub fn layers(&self) -> LevelLayers<F> {
        LevelLayers {
            handle: self.handle,
            level: self.level,
            offset: 0,
            len: self.handle.data.depth as usize
        }
    }

    pub fn sub_image(&self, region: Region3D) -> LevelSubImage<F> {
        LevelSubImage {
            handle: self.handle,
            level: self.level,
            region,
        }
    }

    pub fn upload_command<D, T>(
        &self,
        data: Image3DSource<D, T>,
    ) -> LevelUploadCommand<D, T, F>
    where
        T: ClientFormat<F>,
    {
        LevelUploadCommand {
            data,
            texture_data: self.handle.data.clone(),
            level: self.level,
            region: Region3D::Fill,
            _marker: marker::PhantomData,
        }
    }
}

#[derive(Clone)]
pub struct LevelLayers<'a, F> {
    handle: &'a Texture2DArray<F>,
    level: usize,
    offset: usize,
    len: usize,
}

impl<'a, F> LevelLayers<'a, F>
where
    F: TextureFormat,
{
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn iter(&self) -> LevelLayersIter<F> {
        LevelLayersIter {
            handle: self.handle,
            level: self.level,
            current_layer: self.offset,
            end_layer: self.offset + self.len,
        }
    }
}

impl<'a, F> IntoIterator for LevelLayers<'a, F>
where
    F: TextureFormat,
{
    type Item = LevelLayer<'a, F>;

    type IntoIter = LevelLayersIter<'a, F>;

    fn into_iter(self) -> Self::IntoIter {
        LevelLayersIter {
            handle: self.handle,
            level: self.level,
            current_layer: self.offset,
            end_layer: self.offset + self.len,
        }
    }
}

pub struct LevelLayersIter<'a, F> {
    handle: &'a Texture2DArray<F>,
    level: usize,
    current_layer: usize,
    end_layer: usize,
}

impl<'a, F> Iterator for LevelLayersIter<'a, F>
where
    F: TextureFormat,
{
    type Item = LevelLayer<'a, F>;

    fn next(&mut self) -> Option<Self::Item> {
        let layer = self.current_layer;

        if layer < self.end_layer {
            self.current_layer += 1;

            Some(LevelLayer {
                handle: self.handle,
                level: self.level,
                layer,
            })
        } else {
            None
        }
    }
}

#[derive(Clone)]
pub struct LevelLayer<'a, F> {
    handle: &'a Texture2DArray<F>,
    level: usize,
    layer: usize,
}

impl<'a, F> LevelLayer<'a, F>
where
    F: TextureFormat,
{
    pub(crate) fn texture_data(&self) -> &Arc<Texture2DArrayData> {
        &self.handle.data
    }

    pub fn level(&self) -> usize {
        self.level
    }

    pub fn layer(&self) -> usize {
        self.layer
    }

    pub fn width(&self) -> u32 {
        mipmap_size(self.handle.data.width, self.level)
    }

    pub fn height(&self) -> u32 {
        mipmap_size(self.handle.data.height, self.level)
    }

    pub fn sub_image(&self, region: Region2D) -> LevelLayerSubImage<F> {
        LevelLayerSubImage {
            handle: self.handle,
            level: self.level,
            layer: self.layer,
            region,
        }
    }

    pub fn upload_command<D, T>(
        &self,
        data: Image2DSource<D, T>,
    ) -> LevelLayerUploadCommand<D, T, F>
    where
        T: ClientFormat<F>,
    {
        LevelLayerUploadCommand {
            data,
            texture_data: self.handle.data.clone(),
            level: self.level,
            layer: self.layer,
            region: Region2D::Fill,
            _marker: marker::PhantomData,
        }
    }
}

#[derive(Clone)]
pub struct LevelSubImage<'a, F> {
    handle: &'a Texture2DArray<F>,
    level: usize,
    region: Region3D,
}

impl<'a, F> LevelSubImage<'a, F>
where
    F: TextureFormat,
{
    pub fn level(&self) -> usize {
        self.level
    }

    pub fn region(&self) -> Region3D {
        self.region
    }

    pub fn width(&self) -> u32 {
        region_3d_overlap_width(self.handle.data.width, self.level, &self.region)
    }

    pub fn height(&self) -> u32 {
        region_3d_overlap_height(self.handle.data.height, self.level, &self.region)
    }

    pub fn depth(&self) -> u32 {
        region_3d_overlap_depth(self.handle.data.depth, &self.region)
    }

    pub fn layers(&self) -> LevelSubImageLayers<F> {
        let (start_layer, end_layer, region) = match self.region {
            Region3D::Area((offset_x, offset_y, offset_z), width, height, depth) => {
                let max_layer = cmp::min(self.handle.data.depth, offset_z + depth);

                (
                    offset_z,
                    max_layer,
                    Region2D::Area((offset_x, offset_y), width, height),
                )
            }
            Region3D::Fill => (0, self.handle.data.depth, Region2D::Fill),
        };

        LevelSubImageLayers {
            handle: self.handle,
            level: self.level,
            offset: start_layer as usize,
            len: (end_layer - start_layer) as usize,
            region,
        }
    }

    pub fn sub_image(&self, region: Region3D) -> LevelSubImage<F> {
        LevelSubImage {
            handle: self.handle,
            level: self.level,
            region: region_3d_sub_image(self.region, region),
        }
    }

    pub fn upload_command<D, T>(
        &self,
        data: Image3DSource<D, T>,
    ) -> LevelUploadCommand<D, T, F>
    where
        T: ClientFormat<F>,
    {
        LevelUploadCommand {
            data,
            texture_data: self.handle.data.clone(),
            level: self.level,
            region: self.region,
            _marker: marker::PhantomData,
        }
    }
}

#[derive(Clone)]
pub struct LevelSubImageLayers<'a, F> {
    handle: &'a Texture2DArray<F>,
    level: usize,
    offset: usize,
    len: usize,
    region: Region2D,
}

impl<'a, F> LevelSubImageLayers<'a, F>
where
    F: TextureFormat,
{
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn iter(&self) -> LevelSubImageLayersIter<F> {
        LevelSubImageLayersIter {
            handle: self.handle,
            level: self.level,
            region: self.region,
            current_layer: self.offset as usize,
            end_layer: self.offset + self.len as usize,
        }
    }
}

impl<'a, F> IntoIterator for LevelSubImageLayers<'a, F>
where
    F: TextureFormat,
{
    type Item = LevelLayerSubImage<'a, F>;

    type IntoIter = LevelSubImageLayersIter<'a, F>;

    fn into_iter(self) -> Self::IntoIter {
        LevelSubImageLayersIter {
            handle: self.handle,
            level: self.level,
            region: self.region,
            current_layer: self.offset as usize,
            end_layer: self.offset + self.len as usize,
        }
    }
}

pub struct LevelSubImageLayersIter<'a, F> {
    handle: &'a Texture2DArray<F>,
    level: usize,
    region: Region2D,
    current_layer: usize,
    end_layer: usize,
}

impl<'a, F> Iterator for LevelSubImageLayersIter<'a, F>
where
    F: TextureFormat,
{
    type Item = LevelLayerSubImage<'a, F>;

    fn next(&mut self) -> Option<Self::Item> {
        let layer = self.current_layer;

        if layer < self.end_layer {
            self.current_layer += 1;

            Some(LevelLayerSubImage {
                handle: self.handle,
                level: self.level,
                layer,
                region: self.region,
            })
        } else {
            None
        }
    }
}

#[derive(Clone)]
pub struct LevelLayerSubImage<'a, F> {
    handle: &'a Texture2DArray<F>,
    level: usize,
    layer: usize,
    region: Region2D,
}

impl<'a, F> LevelLayerSubImage<'a, F>
where
    F: TextureFormat,
{
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
        region_2d_overlap_width(self.handle.data.width, self.level, &self.region)
    }

    pub fn height(&self) -> u32 {
        region_2d_overlap_height(self.handle.data.height, self.level, &self.region)
    }

    pub fn sub_image(&self, region: Region2D) -> LevelLayerSubImage<F> {
        LevelLayerSubImage {
            handle: self.handle,
            level: self.level,
            layer: self.layer,
            region: region_2d_sub_image(self.region, region),
        }
    }

    pub fn upload_command<D, T>(
        &self,
        data: Image2DSource<D, T>,
    ) -> LevelLayerUploadCommand<D, T, F>
    where
        T: ClientFormat<F>,
    {
        LevelLayerUploadCommand {
            data,
            texture_data: self.handle.data.clone(),
            level: self.level,
            layer: self.layer,
            region: self.region,
            _marker: marker::PhantomData,
        }
    }
}

struct AllocateCommand<F> {
    data: Arc<Texture2DArrayData>,
    _marker: marker::PhantomData<[F]>,
}

impl<F> GpuTask<Connection> for AllocateCommand<F>
where
    F: TextureFormat,
{
    type Output = ();

    fn progress(&mut self, connection: &mut Connection) -> Progress<Self::Output> {
        let (gl, state) = unsafe { connection.unpack_mut() };
        let data = unsafe { arc_get_mut_unchecked(&mut self.data) };

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

        Progress::Finished(())
    }
}

pub struct LevelUploadCommand<D, T, F> {
    data: Image3DSource<D, T>,
    texture_data: Arc<Texture2DArrayData>,
    level: usize,
    region: Region3D,
    _marker: marker::PhantomData<[F]>,
}

impl<D, T, F> GpuTask<Connection> for LevelUploadCommand<D, T, F>
where
    D: Borrow<[T]>,
    T: ClientFormat<F>,
    F: TextureFormat,
{
    type Output = Result<(), ContextMismatch>;

    fn progress(&mut self, connection: &mut Connection) -> Progress<Self::Output> {
        if self.texture_data.context_id != connection.context_id() {
            return Progress::Finished(Err(ContextMismatch));
        }

        let mut width = region_3d_overlap_width(self.texture_data.width, self.level, &self.region);
        let mut height =
            region_3d_overlap_height(self.texture_data.height, self.level, &self.region);
        let depth = region_3d_overlap_depth(self.texture_data.height, &self.region);

        if width == 0 || height == 0 || depth == 0 {
            return Progress::Finished(Ok(()));
        }

        let (gl, state) = unsafe { connection.unpack_mut() };

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
                                .set_bound_texture_2d_array(Some(texture_object))
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
                    )
                    .unwrap();
                }
            }
        }

        Progress::Finished(Ok(()))
    }
}

pub struct LevelLayerUploadCommand<D, T, F> {
    data: Image2DSource<D, T>,
    texture_data: Arc<Texture2DArrayData>,
    level: usize,
    layer: usize,
    region: Region2D,
    _marker: marker::PhantomData<[F]>,
}

impl<D, T, F> GpuTask<Connection> for LevelLayerUploadCommand<D, T, F>
where
    D: Borrow<[T]>,
    T: ClientFormat<F>,
    F: TextureFormat,
{
    type Output = Result<(), ContextMismatch>;

    fn progress(&mut self, connection: &mut Connection) -> Progress<Self::Output> {
        if self.texture_data.context_id != connection.context_id() {
            return Progress::Finished(Err(ContextMismatch));
        }

        let mut width = region_2d_overlap_width(self.texture_data.width, self.level, &self.region);
        let height = region_2d_overlap_height(self.texture_data.height, self.level, &self.region);

        if width == 0 || height == 0 {
            return Progress::Finished(Ok(()));
        }

        let (gl, state) = unsafe { connection.unpack_mut() };

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
                                .set_bound_texture_2d_array(Some(texture_object))
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
                    )
                    .unwrap();
                }
            }
        }

        Progress::Finished(Ok(()))
    }
}
