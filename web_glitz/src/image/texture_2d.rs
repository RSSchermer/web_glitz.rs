use std::borrow::Borrow;
use std::hash::{Hash, Hasher};
use std::marker;
use std::mem;
use std::ops::{Deref, Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive};
use std::slice;
use std::sync::Arc;

use web_sys::WebGl2RenderingContext as Gl;

use crate::image::format::{ClientFormat, Filterable, TextureFormat};
use crate::image::image_source::Image2DSourceInternal;
use crate::image::renderbuffer::RenderbufferData;
use crate::image::texture_object_dropper::TextureObjectDropper;
use crate::image::util::{
    max_mipmap_levels, mipmap_size, region_2d_overlap_height, region_2d_overlap_width,
    region_2d_sub_image,
};
use crate::image::{Image2DSource, Region2D};
use crate::image::{MaxMipmapLevelsExceeded, MipmapLevels};
use crate::runtime::state::ContextUpdate;
use crate::runtime::{Connection, RenderingContext, TaskContextMismatch};
use crate::task::{GpuTask, Progress, ContextId};
use crate::util::{arc_get_mut_unchecked, identical, slice_make_mut, JsId};

pub struct Texture2DDescriptor<F>
where
    F: TextureFormat + 'static,
{
    pub format: F,
    pub width: u32,
    pub height: u32,
    pub levels: MipmapLevels,
}

pub struct Texture2D<F> {
    data: Arc<Texture2DData>,
    _marker: marker::PhantomData<[F]>,
}

impl<F> Texture2D<F> {
    pub(crate) fn data(&self) -> &Arc<Texture2DData> {
        &self.data
    }

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
    pub(crate) fn new<Rc>(
        context: &Rc,
        descriptor: &Texture2DDescriptor<F>,
    ) -> Result<Self, MaxMipmapLevelsExceeded>
    where
        Rc: RenderingContext + Clone + 'static,
    {
        let Texture2DDescriptor {
            width,
            height,
            levels,
            ..
        } = descriptor;
        let max_mipmap_levels = max_mipmap_levels(*width, *height);

        let levels = match levels {
            MipmapLevels::Auto => max_mipmap_levels,
            MipmapLevels::Manual(levels) => {
                if *levels > max_mipmap_levels {
                    return Err(MaxMipmapLevelsExceeded {
                        given: *levels,
                        max: max_mipmap_levels,
                    });
                }

                *levels
            }
        };

        let data = Arc::new(Texture2DData {
            id: None,
            context_id: context.id(),
            dropper: Box::new(context.clone()),
            width: *width,
            height: *height,
            levels,
            most_recent_unit: None,
        });

        context.submit(AllocateCommand::<F> {
            data: data.clone(),
            _marker: marker::PhantomData,
        });

        Ok(Texture2D {
            data,
            _marker: marker::PhantomData,
        })
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
            },
        }
    }

    pub fn levels(&self) -> Levels<F> {
        Levels {
            handle: self,
            offset: 0,
            len: self.data.levels,
        }
    }

    pub fn levels_mut(&mut self) -> LevelsMut<F> {
        LevelsMut {
            inner: Levels {
                handle: self,
                offset: 0,
                len: self.data.levels,
            },
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
    pub fn generate_mipmap_command(&self) -> GenerateMipmapCommand {
        GenerateMipmapCommand {
            texture_data: self.data.clone(),
        }
    }
}

pub(crate) struct Texture2DData {
    id: Option<JsId>,
    context_id: usize,
    dropper: Box<TextureObjectDropper>,
    width: u32,
    height: u32,
    levels: usize,
    most_recent_unit: Option<u32>,
}

impl Texture2DData {
    pub(crate) fn id(&self) -> Option<JsId> {
        self.id
    }

    pub(crate) fn context_id(&self) -> usize {
        self.context_id
    }
}

impl PartialEq for Texture2DData {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Hash for Texture2DData {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.id.hash(state);
    }
}

impl Drop for Texture2DData {
    fn drop(&mut self) {
        if let Some(id) = self.id {
            self.dropper.drop_texture_object(id);
        }
    }
}

pub struct Levels<'a, F> {
    handle: &'a Texture2D<F>,
    offset: usize,
    len: usize,
}

impl<'a, F> Levels<'a, F>
where
    F: TextureFormat,
{
    pub fn len(&self) -> usize {
        self.handle.data.levels
    }

    pub fn get<'b, I>(&'b self, index: I) -> Option<I::Output>
    where
        I: LevelsIndex<'b, F>,
    {
        index.get(self)
    }

    pub unsafe fn get_unchecked<'b, I>(&'b self, index: I) -> I::Output
    where
        I: LevelsIndex<'b, F>,
    {
        index.get_unchecked(self)
    }

    pub fn iter(&self) -> LevelsIter<F> {
        LevelsIter {
            handle: &self.handle,
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
            handle: &self.handle,
            current_level: self.offset,
            end_level: self.offset + self.len,
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

pub trait LevelsIndex<'a, F> {
    type Output;

    fn get(self, levels: &'a Levels<F>) -> Option<Self::Output>;

    unsafe fn get_unchecked(self, levels: &'a Levels<F>) -> Self::Output;
}

impl<'a, F> LevelsIndex<'a, F> for usize
where
    F: 'a,
{
    type Output = Level<'a, F>;

    fn get(self, levels: &'a Levels<F>) -> Option<Self::Output> {
        if self < levels.len {
            Some(Level {
                handle: levels.handle,
                level: levels.offset + self,
            })
        } else {
            None
        }
    }

    unsafe fn get_unchecked(self, levels: &'a Levels<F>) -> Self::Output {
        Level {
            handle: levels.handle,
            level: levels.offset + self,
        }
    }
}

impl<'a, F> LevelsIndex<'a, F> for RangeFull
where
    F: 'a,
{
    type Output = Levels<'a, F>;

    fn get(self, levels: &'a Levels<F>) -> Option<Self::Output> {
        Some(Levels {
            handle: levels.handle,
            offset: levels.offset,
            len: levels.len,
        })
    }

    unsafe fn get_unchecked(self, levels: &'a Levels<F>) -> Self::Output {
        Levels {
            handle: levels.handle,
            offset: levels.offset,
            len: levels.len,
        }
    }
}

impl<'a, F> LevelsIndex<'a, F> for Range<usize>
where
    F: 'a,
{
    type Output = Levels<'a, F>;

    fn get(self, levels: &'a Levels<F>) -> Option<Self::Output> {
        let Range { start, end } = self;

        if start > end || end > levels.len {
            None
        } else {
            Some(Levels {
                handle: levels.handle,
                offset: levels.offset + start,
                len: end - start,
            })
        }
    }

    unsafe fn get_unchecked(self, levels: &'a Levels<F>) -> Self::Output {
        let Range { start, end } = self;

        Levels {
            handle: levels.handle,
            offset: levels.offset + start,
            len: end - start,
        }
    }
}

impl<'a, F> LevelsIndex<'a, F> for RangeInclusive<usize>
where
    F: 'a,
{
    type Output = Levels<'a, F>;

    fn get(self, levels: &'a Levels<F>) -> Option<Self::Output> {
        if *self.end() == usize::max_value() {
            None
        } else {
            (*self.start()..self.end() + 1).get(levels)
        }
    }

    unsafe fn get_unchecked(self, levels: &'a Levels<F>) -> Self::Output {
        (*self.start()..self.end() + 1).get_unchecked(levels)
    }
}

impl<'a, F> LevelsIndex<'a, F> for RangeFrom<usize>
where
    F: 'a,
{
    type Output = Levels<'a, F>;

    fn get(self, levels: &'a Levels<F>) -> Option<Self::Output> {
        (self.start..levels.len).get(levels)
    }

    unsafe fn get_unchecked(self, levels: &'a Levels<F>) -> Self::Output {
        (self.start..levels.len).get_unchecked(levels)
    }
}

impl<'a, F> LevelsIndex<'a, F> for RangeTo<usize>
where
    F: 'a,
{
    type Output = Levels<'a, F>;

    fn get(self, levels: &'a Levels<F>) -> Option<Self::Output> {
        (0..self.end).get(levels)
    }

    unsafe fn get_unchecked(self, levels: &'a Levels<F>) -> Self::Output {
        (0..self.end).get_unchecked(levels)
    }
}

impl<'a, F> LevelsIndex<'a, F> for RangeToInclusive<usize>
where
    F: 'a,
{
    type Output = Levels<'a, F>;

    fn get(self, levels: &'a Levels<F>) -> Option<Self::Output> {
        (0..=self.end).get(levels)
    }

    unsafe fn get_unchecked(self, levels: &'a Levels<F>) -> Self::Output {
        (0..=self.end).get_unchecked(levels)
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
    pub(crate) fn texture_data(&self) -> &Arc<Texture2DData> {
        self.handle.data()
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

    pub fn sub_image(&self, region: Region2D) -> LevelSubImage<F> {
        LevelSubImage {
            handle: &self.handle,
            level: self.level,
            region,
        }
    }

    pub fn upload_command<D, T>(&self, data: Image2DSource<D, T>) -> UploadCommand<D, T, F>
    where
        T: ClientFormat<F>,
    {
        UploadCommand {
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
    pub(crate) fn level_ref(&self) -> Level<F> {
        Level {
            handle: self.handle,
            level: self.level,
        }
    }

    pub(crate) fn texture_data(&self) -> &Arc<Texture2DData> {
        &self.handle.data
    }

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

    pub fn upload_command<D, T>(&self, data: Image2DSource<D, T>) -> UploadCommand<D, T, F>
    where
        T: ClientFormat<F>,
    {
        UploadCommand {
            data,
            texture_data: self.handle.data.clone(),
            level: self.level,
            region: self.region,
            _marker: marker::PhantomData,
        }
    }
}

pub struct LevelsMut<'a, F> {
    inner: Levels<'a, F>,
}

impl<'a, F> LevelsMut<'a, F>
where
    F: TextureFormat,
{
    pub fn get_mut<'b, I>(&'b mut self, index: I) -> Option<I::Output>
    where
        I: LevelsMutIndex<'b, F>,
    {
        index.get_mut(self)
    }

    pub unsafe fn get_unchecked_mut<'b, I>(&'b mut self, index: I) -> I::Output
    where
        I: LevelsMutIndex<'b, F>,
    {
        index.get_unchecked_mut(self)
    }

    pub fn iter_mut(&mut self) -> LevelsMutIter<F> {
        LevelsMutIter {
            current_level: self.offset,
            end_level: self.offset + self.len,
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

    fn into_iter(mut self) -> Self::IntoIter {
        LevelsMutIter {
            current_level: self.offset,
            end_level: self.offset + self.len,
            handle: &self.inner.handle,
        }
    }
}

impl<'a, F> Deref for LevelsMut<'a, F>
where
    F: TextureFormat,
{
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
                    level,
                },
            })
        } else {
            None
        }
    }
}

pub trait LevelsMutIndex<'a, F> {
    type Output;

    fn get_mut(self, levels: &'a mut LevelsMut<F>) -> Option<Self::Output>;

    unsafe fn get_unchecked_mut(self, levels: &'a mut LevelsMut<F>) -> Self::Output;
}

impl<'a, F> LevelsMutIndex<'a, F> for usize
where
    F: 'a,
{
    type Output = LevelMut<'a, F>;

    fn get_mut(self, levels: &'a mut LevelsMut<F>) -> Option<Self::Output> {
        if self < levels.inner.len {
            Some(LevelMut {
                inner: Level {
                    handle: levels.inner.handle,
                    level: levels.inner.offset + self,
                },
            })
        } else {
            None
        }
    }

    unsafe fn get_unchecked_mut(self, levels: &'a mut LevelsMut<F>) -> Self::Output {
        LevelMut {
            inner: Level {
                handle: levels.inner.handle,
                level: levels.inner.offset + self,
            },
        }
    }
}

impl<'a, F> LevelsMutIndex<'a, F> for RangeFull
where
    F: 'a,
{
    type Output = LevelsMut<'a, F>;

    fn get_mut(self, levels: &'a mut LevelsMut<F>) -> Option<Self::Output> {
        Some(LevelsMut {
            inner: Levels {
                handle: levels.inner.handle,
                offset: levels.inner.offset,
                len: levels.inner.len,
            },
        })
    }

    unsafe fn get_unchecked_mut(self, levels: &'a mut LevelsMut<F>) -> Self::Output {
        LevelsMut {
            inner: Levels {
                handle: levels.inner.handle,
                offset: levels.inner.offset,
                len: levels.inner.len,
            },
        }
    }
}

impl<'a, F> LevelsMutIndex<'a, F> for Range<usize>
where
    F: 'a,
{
    type Output = LevelsMut<'a, F>;

    fn get_mut(self, levels: &'a mut LevelsMut<F>) -> Option<Self::Output> {
        let Range { start, end } = self;

        if start > end || end > levels.inner.len {
            None
        } else {
            Some(LevelsMut {
                inner: Levels {
                    handle: levels.inner.handle,
                    offset: levels.inner.offset + start,
                    len: end - start,
                },
            })
        }
    }

    unsafe fn get_unchecked_mut(self, levels: &'a mut LevelsMut<F>) -> Self::Output {
        let Range { start, end } = self;

        LevelsMut {
            inner: Levels {
                handle: levels.inner.handle,
                offset: levels.inner.offset + start,
                len: end - start,
            },
        }
    }
}

impl<'a, F> LevelsMutIndex<'a, F> for RangeInclusive<usize>
where
    F: 'a,
{
    type Output = LevelsMut<'a, F>;

    fn get_mut(self, levels: &'a mut LevelsMut<F>) -> Option<Self::Output> {
        if *self.end() == usize::max_value() {
            None
        } else {
            (*self.start()..self.end() + 1).get_mut(levels)
        }
    }

    unsafe fn get_unchecked_mut(self, levels: &'a mut LevelsMut<F>) -> Self::Output {
        (*self.start()..self.end() + 1).get_unchecked_mut(levels)
    }
}

impl<'a, F> LevelsMutIndex<'a, F> for RangeFrom<usize>
where
    F: 'a,
{
    type Output = LevelsMut<'a, F>;

    fn get_mut(self, levels: &'a mut LevelsMut<F>) -> Option<Self::Output> {
        (self.start..levels.inner.len).get_mut(levels)
    }

    unsafe fn get_unchecked_mut(self, levels: &'a mut LevelsMut<F>) -> Self::Output {
        (self.start..levels.inner.len).get_unchecked_mut(levels)
    }
}

impl<'a, F> LevelsMutIndex<'a, F> for RangeTo<usize>
where
    F: 'a,
{
    type Output = LevelsMut<'a, F>;

    fn get_mut(self, levels: &'a mut LevelsMut<F>) -> Option<Self::Output> {
        (0..self.end).get_mut(levels)
    }

    unsafe fn get_unchecked_mut(self, levels: &'a mut LevelsMut<F>) -> Self::Output {
        (0..self.end).get_unchecked_mut(levels)
    }
}

impl<'a, F> LevelsMutIndex<'a, F> for RangeToInclusive<usize>
where
    F: 'a,
{
    type Output = LevelsMut<'a, F>;

    fn get_mut(self, levels: &'a mut LevelsMut<F>) -> Option<Self::Output> {
        (0..=self.end).get_mut(levels)
    }

    unsafe fn get_unchecked_mut(self, levels: &'a mut LevelsMut<F>) -> Self::Output {
        (0..=self.end).get_unchecked_mut(levels)
    }
}

pub struct LevelMut<'a, F> {
    inner: Level<'a, F>,
}

impl<'a, F> Deref for LevelMut<'a, F> {
    type Target = Level<'a, F>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

struct AllocateCommand<F> {
    data: Arc<Texture2DData>,
    _marker: marker::PhantomData<[F]>,
}

unsafe impl<F> GpuTask<Connection> for AllocateCommand<F>
where
    F: TextureFormat,
{
    type Output = ();

    fn context_id(&self) -> ContextId {
        ContextId::Id(self.data.context_id)
    }

    fn progress(&mut self, connection: &mut Connection) -> Progress<Self::Output> {
        let (gl, state) = unsafe { connection.unpack_mut() };
        let data = unsafe { arc_get_mut_unchecked(&mut self.data) };

        let texture_object = gl.create_texture().unwrap();

        state.set_active_texture_lru().apply(gl).unwrap();
        state
            .set_bound_texture_2d(Some(&texture_object))
            .apply(gl)
            .unwrap();

        let levels = data.levels as i32;

        gl.tex_storage_2d(
            Gl::TEXTURE_2D,
            levels,
            F::id(),
            data.width as i32,
            data.height as i32,
        );

        gl.tex_parameteri(Gl::TEXTURE_2D, Gl::TEXTURE_MAX_LEVEL, levels);

        data.id = Some(JsId::from_value(texture_object.into()));

        Progress::Finished(())
    }
}

pub struct UploadCommand<D, T, F> {
    data: Image2DSource<D, T>,
    texture_data: Arc<Texture2DData>,
    level: usize,
    region: Region2D,
    _marker: marker::PhantomData<[F]>,
}

unsafe impl<D, T, F> GpuTask<Connection> for UploadCommand<D, T, F>
where
    D: Borrow<[T]>,
    T: ClientFormat<F>,
    F: TextureFormat,
{
    type Output = ();

    fn context_id(&self) -> ContextId {
        ContextId::Id(self.texture_data.context_id)
    }

    fn progress(&mut self, connection: &mut Connection) -> Progress<Self::Output> {
        let mut width = region_2d_overlap_width(self.texture_data.width, self.level, &self.region);
        let height = region_2d_overlap_height(self.texture_data.height, self.level, &self.region);

        if width == 0 || height == 0 {
            return Progress::Finished(());
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

        Progress::Finished(())
    }
}

pub struct GenerateMipmapCommand {
    texture_data: Arc<Texture2DData>,
}

unsafe impl GpuTask<Connection> for GenerateMipmapCommand {
    type Output = ();

    fn context_id(&self) -> ContextId {
        ContextId::Id(self.texture_data.context_id)
    }

    fn progress(&mut self, connection: &mut Connection) -> Progress<Self::Output> {
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

        Progress::Finished(())
    }
}
