use std::borrow::Borrow;
use std::marker;
use std::mem;
use std::ops::Deref;
use std::slice;
use std::sync::Arc;

use web_sys::WebGl2RenderingContext as Gl;

use crate::image::{Region2D, Image2DSource};
use crate::image::image_source::Image2DSourceInternal;
use crate::image::format::{ClientFormat, Filterable, TextureFormat};
use crate::image::texture_object_dropper::TextureObjectDropper;
use crate::image::util::{
    mipmap_size, region_2d_overlap_height, region_2d_overlap_width, region_2d_sub_image,
};
use crate::runtime::dynamic_state::ContextUpdate;
use crate::runtime::{Connection, RenderingContext, ContextMismatch};
use crate::task::{GpuTask, Progress};
use crate::util::{arc_get_mut_unchecked, identical, JsId, slice_make_mut};


pub struct Texture2D<F> {
    data: Arc<Texture2DData>,
    _marker: marker::PhantomData<[F]>,
}

impl<F> Texture2D<F> {
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
                    ) {
                    state.set_active_texture_lru().apply(gl).unwrap();
                    state
                        .set_bound_texture_2d(Some(&texture_object))
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

impl<F> Texture2D<F>
where
    F: TextureFormat + 'static,
{
    pub(crate) fn new<Rc>(context: &Rc, width: u32, height: u32) -> Self
    where
        Rc: RenderingContext + Clone + 'static,
    {
        let data = Arc::new(Texture2DData {
            id: None,
            context_id: context.id(),
            dropper: Box::new(context.clone()),
            width,
            height,
            levels: 1,
            most_recent_unit: None,
        });

        context.submit(Texture2DAllocateTask::<F> {
            data: data.clone(),
            _marker: marker::PhantomData,
        });

        Texture2D {
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

    pub fn base_level_mut(&mut self) -> LevelMut<F> {
        LevelMut {
            inner: Level {
                handle: self,
                level: 0,
            }
        }
    }

    pub fn width(&self) -> u32 {
        self.data.width
    }

    pub fn height(&self) -> u32 {
        self.data.height
    }
}

impl<F> Texture2D<F>
where
    F: TextureFormat + Filterable + 'static,
{
    pub(crate) fn new_mipmapped<Rc>(
        context: &Rc,
        width: u32,
        height: u32,
        levels: usize,
    ) -> Self
    where
        Rc: RenderingContext + Clone + 'static,
    {
        let data = Arc::new(Texture2DData {
            id: None,
            context_id: context.id(),
            dropper: Box::new(context.clone()),
            width,
            height,
            levels,
            most_recent_unit: None,
        });

        context.submit(Texture2DAllocateTask::<F> {
            data: data.clone(),
            _marker: marker::PhantomData,
        });

        Texture2D {
            data,
            _marker: marker::PhantomData,
        }
    }

    pub fn levels(&self) -> Levels<F> {
        Levels {
            handle: self
        }
    }

    pub fn levels_mut(&mut self) -> LevelsMut<F> {
        LevelsMut {
            inner: Levels {
                handle: self
            }
        }
    }

    pub fn generate_mipmap_command(&self) -> GenerateMipmapCommand {
        GenerateMipmapCommand {
            texture_data: self.data.clone(),
        }
    }
}

struct Texture2DData {
    id: Option<JsId>,
    context_id: usize,
    dropper: Box<TextureObjectDropper>,
    width: u32,
    height: u32,
    levels: usize,
    most_recent_unit: Option<u32>,
}

impl Drop for Texture2DData {
    fn drop(&mut self) {
        if let Some(id) = self.id {
            self.dropper.drop_texture_object(id);
        }
    }
}

pub struct Levels<'a, F> {
    handle: &'a Texture2D<F>
}

impl<'a, F> Levels<'a, F>
where
    F: TextureFormat,
{
    pub fn len(&self) -> usize {
        self.handle.data.levels
    }

    pub fn get(&self, level: usize) -> Option<Level<F>> {
        let texture_data = &self.handle.data;

        if level < texture_data.levels {
            Some(Level {
                handle: &self.handle,
                level,
            })
        } else {
            None
        }
    }

    pub unsafe fn get_unchecked(&self, level: usize) -> Level<F> {
        Level {
            handle: &self.handle,
            level,
        }
    }

    pub fn iter(&self) -> LevelsIter<F> {
        LevelsIter {
            handle: &self.handle,
            current_level: 0,
            end_level: self.handle.data.levels,
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
            handle: &self.handle,
            current_level: 0,
            end_level: self.handle.data.levels
        }
    }
}

pub struct LevelsIter<'a, F> {
    handle: &'a Texture2D<F>,
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
                handle: &self.handle,
                level,
            })
        } else {
            None
        }
    }
}

pub struct Level<'a, F> {
    handle: &'a Texture2D<F>,
    level: usize,
}

impl<'a, F> Level<'a, F>
where
    F: TextureFormat,
{
    pub fn level(&self) -> usize {
        self.level
    }

    pub fn width(&self) -> u32 {
        mipmap_size(self.handle.data.width, self.level)
    }

    pub fn height(&self) -> u32 {
        mipmap_size(self.handle.data.height, self.level)
    }

    pub fn sub_image(&self, region: Region2D) -> LevelSubImage<F> {
        LevelSubImage {
            handle: &self.handle,
            level: self.level,
            region,
        }
    }

    pub fn upload_task<D, T>(&self, data: Image2DSource<D, T>) -> Texture2DUploadTask<D, T, F>
    where
        T: ClientFormat<F>,
    {
        Texture2DUploadTask {
            data,
            texture_data: self.handle.data.clone(),
            level: self.level,
            region: Region2D::Fill,
            _marker: marker::PhantomData,
        }
    }
}

pub struct LevelSubImage<'a, F> {
    handle: &'a Texture2D<F>,
    level: usize,
    region: Region2D,
}

impl<'a, F> LevelSubImage<'a, F>
where
    F: TextureFormat,
{
    pub fn level(&self) -> usize {
        self.level
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

    pub fn sub_image(&self, region: Region2D) -> LevelSubImage<F> {
        LevelSubImage {
            handle: &self.handle,
            level: self.level,
            region: region_2d_sub_image(self.region, region),
        }
    }

    pub fn upload_task<D, T>(&self, data: Image2DSource<D, T>) -> Texture2DUploadTask<D, T, F>
    where
        T: ClientFormat<F>,
    {
        Texture2DUploadTask {
            data,
            texture_data: self.handle.data.clone(),
            level: self.level,
            region: self.region,
            _marker: marker::PhantomData,
        }
    }
}

pub struct LevelsMut<'a, F> {
    inner: Levels<'a, F>
}

impl<'a, F> LevelsMut<'a, F>
    where
        F: TextureFormat,
{
    pub fn get_mut(&mut self, level: usize) -> Option<LevelMut<F>> {
        if level < self.handle.data.levels {
            Some(LevelMut {
                inner: Level {
                    handle: &self.inner.handle,
                    level
                },
            })
        } else {
            None
        }
    }

    pub unsafe fn get_mut_unchecked(&mut self, level: usize) -> LevelMut<F> {
        LevelMut {
            inner: Level {
                handle: &self.inner.handle,
                level
            },
        }
    }

    pub fn iter_mut(&mut self) -> LevelsMutIter<F> {
        LevelsMutIter {
            current_level: 0,
            end_level: self.inner.handle.data.levels,
            handle: &self.inner.handle,
        }
    }
}

impl<'a, F> IntoIterator for LevelsMut<'a, F>
    where
        F: TextureFormat,
{
    type Item = LevelMut<'a, F>;

    type IntoIter = LevelsMutIter<'a, F>;

    fn into_iter(self) -> Self::IntoIter {
        LevelsMutIter {
            current_level: 0,
            end_level: self.inner.handle.data.levels,
            handle: &self.inner.handle,
        }
    }
}

impl<'a, F> Deref for LevelsMut<'a, F> where
    F: TextureFormat, {
    type Target = Levels<'a, F>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub struct LevelsMutIter<'a, F> {
    handle: &'a Texture2D<F>,
    current_level: usize,
    end_level: usize,
}

impl<'a, F> Iterator for LevelsMutIter<'a, F>
    where
        F: TextureFormat,
{
    type Item = LevelMut<'a, F>;

    fn next(&mut self) -> Option<Self::Item> {
        let level = self.current_level;

        if level < self.end_level {
            self.current_level += 1;

            Some(LevelMut {
                inner: Level {
                    handle: &self.handle,
                    level
                }
            })
        } else {
            None
        }
    }
}

pub struct LevelMut<'a, F> {
    inner: Level<'a, F>
}

impl<'a, F> Deref for LevelMut<'a, F> {
    type Target = Level<'a, F>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

struct Texture2DAllocateTask<F> {
    data: Arc<Texture2DData>,
    _marker: marker::PhantomData<[F]>,
}

impl<F> GpuTask<Connection> for Texture2DAllocateTask<F>
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
            .set_bound_texture_2d(Some(&texture_object))
            .apply(gl)
            .unwrap();

        gl.tex_storage_2d(
            Gl::TEXTURE_2D,
            data.levels as i32,
            F::id(),
            data.width as i32,
            data.height as i32,
        );

        data.id = Some(JsId::from_value(texture_object.into()));

        Progress::Finished(())
    }
}

pub struct Texture2DUploadTask<D, T, F> {
    data: Image2DSource<D, T>,
    texture_data: Arc<Texture2DData>,
    level: usize,
    region: Region2D,
    _marker: marker::PhantomData<[F]>,
}

impl<D, T, F> GpuTask<Connection> for Texture2DUploadTask<D, T, F>
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
                                .set_bound_texture_2d(Some(texture_object))
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
                    Region2D::Area((offset_x, offset_y), ..) => (offset_x, offset_y),
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

                    gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_opt_u8_array(
                        Gl::TEXTURE_2D,
                        self.level as i32,
                        offset_x as i32,
                        offset_y as i32,
                        width as i32,
                        height as i32,
                        T::format_id(),
                        T::type_id(),
                        Some(slice_make_mut(data)),
                    )
                    .unwrap();
                }
            }
        }

        Progress::Finished(Ok(()))
    }
}

pub struct GenerateMipmapCommand {
    texture_data: Arc<Texture2DData>,
}

impl GpuTask<Connection> for GenerateMipmapCommand {
    type Output = Result<(), ContextMismatch>;

    fn progress(&mut self, connection: &mut Connection) -> Progress<Self::Output> {
        if self.texture_data.context_id != connection.context_id() {
            return Progress::Finished(Err(ContextMismatch));
        }

        let (gl, state) = unsafe { connection.unpack_mut() };

        unsafe {
            self.texture_data
                .id
                .unwrap()
                .with_value_unchecked(|texture_object| {
                    state.set_bound_texture_2d(Some(texture_object));
                });
        }

        gl.generate_mipmap(Gl::TEXTURE_2D);

        Progress::Finished(Ok(()))
    }
}
