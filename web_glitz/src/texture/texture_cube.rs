use std::borrow::Borrow;
use std::marker;
use std::mem;
use std::slice;
use std::sync::Arc;

use wasm_bindgen::JsCast;
use web_sys::WebGl2RenderingContext as Gl;

use crate::framebuffer::framebuffer_handle::FramebufferAttachmentInternal;
use crate::framebuffer::{AsFramebufferAttachment, FramebufferAttachment};
use crate::image_format::ClientFormat;
use crate::image_region::Region2D;
use crate::rendering_context::{Connection, ContextUpdate, RenderingContext};
use crate::task::{GpuTask, Progress};
use crate::texture::image_source::Image2DSourceInternal;
use crate::texture::{Image2DSource, TextureFormat};
use crate::util::JsId;
use rendering_context::DropObject;
use rendering_context::Dropper;
use rendering_context::RefCountedDropper;
use texture::util::mipmap_size;
use texture::util::region_2d_overlap_height;
use texture::util::region_2d_overlap_width;
use texture::util::region_2d_sub_image;
use util::arc_get_mut_unchecked;
use util::identical;

#[derive(Clone)]
pub struct TextureCubeHandle<F> {
    pub(crate) data: Arc<TextureCubeData>,
    _marker: marker::PhantomData<[F]>,
}

impl<F> TextureCubeHandle<F> {
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
                        .set_bound_texture_cube_map(Some(&texture_object))
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

impl<F> TextureCubeHandle<F>
where
    F: TextureFormat + 'static,
{
    pub(crate) fn new<Rc>(
        context: &Rc,
        dropper: RefCountedDropper,
        width: u32,
        height: u32,
        levels: usize,
    ) -> Self
    where
        Rc: RenderingContext,
    {
        let data = Arc::new(TextureCubeData {
            id: None,
            dropper,
            width,
            height,
            levels,
            most_recent_unit: None,
        });

        context.submit(TextureCubeAllocateTask::<F> {
            data: data.clone(),
            _marker: marker::PhantomData,
        });

        TextureCubeHandle {
            data,
            _marker: marker::PhantomData,
        }
    }

    pub fn base_level(&self) -> TextureCubeLevel<F> {
        TextureCubeLevel {
            texture_data: self.data.clone(),
            level: 0,
            _marker: marker::PhantomData,
        }
    }

    pub fn levels(&self) -> TextureCubeLevels<F> {
        TextureCubeLevels {
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
}

pub(crate) struct TextureCubeData {
    pub(crate) id: Option<JsId>,
    dropper: RefCountedDropper,
    width: u32,
    height: u32,
    levels: usize,
    pub(crate) most_recent_unit: Option<u32>,
}

impl Drop for TextureCubeData {
    fn drop(&mut self) {
        if let Some(id) = self.id {
            self.dropper.drop_gl_object(DropObject::Texture(id));
        }
    }
}

#[derive(Clone)]
pub struct TextureCubeLevels<F> {
    texture_data: Arc<TextureCubeData>,
    _marker: marker::PhantomData<[F]>,
}

impl<F> TextureCubeLevels<F>
where
    F: TextureFormat,
{
    pub fn texture(&self) -> TextureCubeHandle<F> {
        TextureCubeHandle {
            data: self.texture_data.clone(),
            _marker: marker::PhantomData,
        }
    }

    pub fn len(&self) -> usize {
        self.texture_data.levels
    }

    pub fn get(&self, level: usize) -> Option<TextureCubeLevel<F>> {
        let texture_data = &self.texture_data;

        if level < texture_data.levels {
            Some(TextureCubeLevel {
                texture_data: texture_data.clone(),
                level,
                _marker: marker::PhantomData,
            })
        } else {
            None
        }
    }

    pub unsafe fn get_unchecked(&self, level: usize) -> TextureCubeLevel<F> {
        TextureCubeLevel {
            texture_data: self.texture_data.clone(),
            level,
            _marker: marker::PhantomData,
        }
    }

    pub fn iter(&self) -> TextureCubeLevelsIter<F> {
        TextureCubeLevelsIter {
            texture_data: self.texture_data.clone(),
            current_level: 0,
            end_level: self.texture_data.levels,
            _marker: marker::PhantomData,
        }
    }
}

impl<F> IntoIterator for TextureCubeLevels<F>
where
    F: TextureFormat,
{
    type Item = TextureCubeLevel<F>;

    type IntoIter = TextureCubeLevelsIter<F>;

    fn into_iter(self) -> Self::IntoIter {
        TextureCubeLevelsIter {
            current_level: 0,
            end_level: self.texture_data.levels,
            texture_data: self.texture_data,
            _marker: marker::PhantomData,
        }
    }
}

pub struct TextureCubeLevelsIter<F> {
    texture_data: Arc<TextureCubeData>,
    current_level: usize,
    end_level: usize,
    _marker: marker::PhantomData<[F]>,
}

impl<F> Iterator for TextureCubeLevelsIter<F>
where
    F: TextureFormat,
{
    type Item = TextureCubeLevel<F>;

    fn next(&mut self) -> Option<Self::Item> {
        let level = self.current_level;

        if level < self.end_level {
            self.current_level += 1;

            Some(TextureCubeLevel {
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
pub struct TextureCubeLevel<F> {
    texture_data: Arc<TextureCubeData>,
    level: usize,
    _marker: marker::PhantomData<[F]>,
}

impl<F> TextureCubeLevel<F>
where
    F: TextureFormat,
{
    pub fn texture(&self) -> TextureCubeHandle<F> {
        TextureCubeHandle {
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

    pub fn positive_x(&self) -> TextureCubeLevelFace<F> {
        TextureCubeLevelFace {
            texture_data: self.texture_data.clone(),
            level: self.level,
            face: CubeFace::PositiveX,
            _marker: marker::PhantomData,
        }
    }

    pub fn negative_x(&self) -> TextureCubeLevelFace<F> {
        TextureCubeLevelFace {
            texture_data: self.texture_data.clone(),
            level: self.level,
            face: CubeFace::NegativeX,
            _marker: marker::PhantomData,
        }
    }

    pub fn positive_y(&self) -> TextureCubeLevelFace<F> {
        TextureCubeLevelFace {
            texture_data: self.texture_data.clone(),
            level: self.level,
            face: CubeFace::PositiveY,
            _marker: marker::PhantomData,
        }
    }

    pub fn negative_y(&self) -> TextureCubeLevelFace<F> {
        TextureCubeLevelFace {
            texture_data: self.texture_data.clone(),
            level: self.level,
            face: CubeFace::NegativeY,
            _marker: marker::PhantomData,
        }
    }

    pub fn positive_z(&self) -> TextureCubeLevelFace<F> {
        TextureCubeLevelFace {
            texture_data: self.texture_data.clone(),
            level: self.level,
            face: CubeFace::PositiveZ,
            _marker: marker::PhantomData,
        }
    }

    pub fn negative_z(&self) -> TextureCubeLevelFace<F> {
        TextureCubeLevelFace {
            texture_data: self.texture_data.clone(),
            level: self.level,
            face: CubeFace::NegativeZ,
            _marker: marker::PhantomData,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum CubeFace {
    PositiveX,
    NegativeX,
    PositiveY,
    NegativeY,
    PositiveZ,
    NegativeZ,
}

impl CubeFace {
    fn id(&self) -> u32 {
        match self {
            CubeFace::PositiveX => Gl::TEXTURE_CUBE_MAP_POSITIVE_X,
            CubeFace::NegativeX => Gl::TEXTURE_CUBE_MAP_NEGATIVE_X,
            CubeFace::PositiveY => Gl::TEXTURE_CUBE_MAP_POSITIVE_Y,
            CubeFace::NegativeY => Gl::TEXTURE_CUBE_MAP_NEGATIVE_Y,
            CubeFace::PositiveZ => Gl::TEXTURE_CUBE_MAP_POSITIVE_Z,
            CubeFace::NegativeZ => Gl::TEXTURE_CUBE_MAP_NEGATIVE_Z,
        }
    }
}

#[derive(Clone)]
pub struct TextureCubeLevelFace<F> {
    texture_data: Arc<TextureCubeData>,
    level: usize,
    face: CubeFace,
    _marker: marker::PhantomData<[F]>,
}

impl<F> TextureCubeLevelFace<F>
where
    F: TextureFormat,
{
    pub fn texture(&self) -> TextureCubeHandle<F> {
        TextureCubeHandle {
            data: self.texture_data.clone(),
            _marker: marker::PhantomData,
        }
    }

    pub fn level(&self) -> usize {
        self.level
    }

    pub fn face(&self) -> CubeFace {
        self.face
    }

    pub fn width(&self) -> u32 {
        mipmap_size(self.texture_data.width, self.level)
    }

    pub fn height(&self) -> u32 {
        mipmap_size(self.texture_data.height, self.level)
    }

    pub fn sub_image(&self, region: Region2D) -> TextureCubeLevelFaceSubImage<F> {
        TextureCubeLevelFaceSubImage {
            texture_data: self.texture_data.clone(),
            level: self.level,
            face: self.face,
            region,
            _marker: marker::PhantomData,
        }
    }

    pub fn upload_task<D, T>(&self, data: Image2DSource<D, T>) -> TextureCubeUploadTask<D, T, F>
    where
        T: ClientFormat<F>,
    {
        TextureCubeUploadTask {
            data,
            texture_data: self.texture_data.clone(),
            level: self.level,
            face: self.face,
            region: Region2D::Fill,
            _marker: marker::PhantomData,
        }
    }
}

impl<F> AsFramebufferAttachment for TextureCubeLevelFace<F> {
    fn as_framebuffer_attachment(&self) -> FramebufferAttachment {
        FramebufferAttachment {
            internal: FramebufferAttachmentInternal::TextureCubeLevelFace(
                self.texture_data.clone(),
                self.level,
                self.face,
            ),
        }
    }
}

pub struct TextureCubeLevelFaceSubImage<F> {
    texture_data: Arc<TextureCubeData>,
    level: usize,
    face: CubeFace,
    region: Region2D,
    _marker: marker::PhantomData<[F]>,
}

impl<F> TextureCubeLevelFaceSubImage<F>
where
    F: TextureFormat,
{
    pub fn texture(&self) -> TextureCubeHandle<F> {
        TextureCubeHandle {
            data: self.texture_data.clone(),
            _marker: marker::PhantomData,
        }
    }

    pub fn level(&self) -> usize {
        self.level
    }

    pub fn face(&self) -> CubeFace {
        self.face
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

    pub fn sub_image(&self, region: Region2D) -> TextureCubeLevelFaceSubImage<F> {
        TextureCubeLevelFaceSubImage {
            texture_data: self.texture_data.clone(),
            level: self.level,
            face: self.face,
            region: region_2d_sub_image(self.region, region),
            _marker: marker::PhantomData,
        }
    }

    pub fn upload_task<D, T>(&self, data: Image2DSource<D, T>) -> TextureCubeUploadTask<D, T, F>
    where
        T: ClientFormat<F>,
    {
        TextureCubeUploadTask {
            data,
            texture_data: self.texture_data.clone(),
            level: self.level,
            face: self.face,
            region: self.region,
            _marker: marker::PhantomData,
        }
    }
}

struct TextureCubeAllocateTask<F> {
    data: Arc<TextureCubeData>,
    _marker: marker::PhantomData<[F]>,
}

impl<F> GpuTask<Connection> for TextureCubeAllocateTask<F>
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
            .set_bound_texture_2d(Some(&texture_object))
            .apply(gl)
            .unwrap();

        gl.tex_storage_2d(
            Gl::TEXTURE_CUBE_MAP,
            data.levels as i32,
            F::id(),
            data.width as i32,
            data.height as i32,
        );

        data.id = Some(JsId::from_value(texture_object.into()));

        Progress::Finished(())
    }
}

pub struct TextureCubeUploadTask<D, T, F> {
    data: Image2DSource<D, T>,
    texture_data: Arc<TextureCubeData>,
    level: usize,
    face: CubeFace,
    region: Region2D,
    _marker: marker::PhantomData<[F]>,
}

impl<D, T, F> GpuTask<Connection> for TextureCubeUploadTask<D, T, F>
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
                                .set_bound_texture_cube_map(Some(texture_object))
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

                    gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_opt_u8_array(
                        self.face.id(),
                        self.level as i32,
                        offset_x as i32,
                        offset_y as i32,
                        width as i32,
                        height as i32,
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
