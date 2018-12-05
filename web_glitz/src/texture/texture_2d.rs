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
use crate::texture::util::{region_2d_overlap_width, region_2d_overlap_height};
use crate::texture::util::mipmap_size;
use crate::util::JsId;
use texture::util::region_2d_sub_image;

pub struct Texture2DHandle<F, Rc> where Rc: RenderingContext {
    data: Arc<Texture2DData<Rc>>,
    _marker: marker::PhantomData<[F]>,
}

impl<F, Rc> Texture2DHandle<F, Rc>
where
    F: TextureFormat + 'static,
    Rc: RenderingContext + 'static,
{
    pub(crate) fn new(context: &Rc, width: u32, height: u32, levels: usize) -> Self {
        let data = Arc::new(Texture2DData {
            id: None,
            context: context.clone(),
            width,
            height,
            levels,
        });

        context.submit(Texture2DAllocateTask::<F, Rc> {
            data: data.clone(),
            _marker: marker::PhantomData,
        });

        Texture2DHandle {
            data,
            _marker: marker::PhantomData,
        }
    }

    pub fn base_level(&self) -> Texture2DLevel<F, Rc> {
        Texture2DLevel {
            texture_data: self.data.clone(),
            level: 0,
            _marker: marker::PhantomData,
        }
    }

    pub fn levels(&self) -> Texture2DLevels<F, Rc> {
        Texture2DLevels {
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

pub(crate) struct Texture2DData<Rc> where Rc: RenderingContext {
    pub(crate) id: Option<JsId>,
    context: Rc,
    width: u32,
    height: u32,
    levels: usize,
}

impl<Rc> Drop for Texture2DData<Rc>
where
    Rc: RenderingContext,
{
    fn drop(&mut self) {
        if let Some(id) = self.id {
            self.context.submit(Texture2DDropTask { id });
        }
    }
}

pub struct Texture2DLevels<F, Rc> where Rc: RenderingContext {
    texture_data: Arc<Texture2DData<Rc>>,
    _marker: marker::PhantomData<[F]>,
}

impl<F, Rc> Texture2DLevels<F, Rc>
where
    F: TextureFormat,
    Rc: RenderingContext,
{
    pub fn len(&self) -> usize {
        self.texture_data.levels
    }

    pub fn get(&self, level: usize) -> Option<Texture2DLevel<F, Rc>> {
        let texture_data = &self.texture_data;

        if level < texture_data.levels {
            Some(Texture2DLevel {
                texture_data: texture_data.clone(),
                level,
                _marker: marker::PhantomData,
            })
        } else {
            None
        }
    }

    pub unsafe fn get_unchecked(&self, level: usize) -> Texture2DLevel<F, Rc> {
        Texture2DLevel {
            texture_data: self.texture_data.clone(),
            level,
            _marker: marker::PhantomData,
        }
    }

    pub fn iter(&self) -> Texture2DLevelsIter<F, Rc> {
        Texture2DLevelsIter {
            texture_data: self.texture_data.clone(),
            current_level: 0,
            end_level: self.texture_data.levels,
            _marker: marker::PhantomData,
        }
    }
}

impl<F, Rc> IntoIterator for Texture2DLevels<F, Rc>
where
    F: TextureFormat,
    Rc: RenderingContext,
{
    type Item = Texture2DLevel<F, Rc>;

    type IntoIter = Texture2DLevelsIter<F, Rc>;

    fn into_iter(self) -> Self::IntoIter {
        Texture2DLevelsIter {
            current_level: 0,
            end_level: self.texture_data.levels,
            texture_data: self.texture_data,
            _marker: marker::PhantomData,
        }
    }
}

pub struct Texture2DLevelsIter<F, Rc> where Rc: RenderingContext {
    texture_data: Arc<Texture2DData<Rc>>,
    current_level: usize,
    end_level: usize,
    _marker: marker::PhantomData<[F]>,
}

impl<F, Rc> Iterator for Texture2DLevelsIter<F, Rc>
where
    F: TextureFormat,
    Rc: RenderingContext,
{
    type Item = Texture2DLevel<F, Rc>;

    fn next(&mut self) -> Option<Self::Item> {
        let level = self.current_level;

        if level < self.end_level {
            self.current_level += 1;

            Some(Texture2DLevel {
                texture_data: self.texture_data.clone(),
                level,
                _marker: marker::PhantomData,
            })
        } else {
            None
        }
    }
}

pub struct Texture2DLevel<F, Rc> where Rc: RenderingContext {
    texture_data: Arc<Texture2DData<Rc>>,
    level: usize,
    _marker: marker::PhantomData<[F]>,
}

impl<F, Rc> Texture2DLevel<F, Rc>
where
    F: TextureFormat,
    Rc: RenderingContext,
{
    pub fn level(&self) -> usize {
        self.level
    }

    pub fn width(&self) -> u32 {
        mipmap_size(self.texture_data.width, self.level)
    }

    pub fn height(&self) -> u32 {
        mipmap_size(self.texture_data.height, self.level)
    }

    pub fn sub_image(&self, region: Region2D) -> Texture2DLevelSubImage<F, Rc> {
        Texture2DLevelSubImage {
            texture_data: self.texture_data.clone(),
            level: self.level,
            region,
            _marker: marker::PhantomData,
        }
    }

    pub fn upload_task<D, T>(
        &self,
        data: Image2DSource<D, T>,
    ) -> Texture2DUploadTask<D, T, F, Rc>
    where
        T: ClientFormat<F>,
    {
        Texture2DUploadTask {
            data,
            texture_data: self.texture_data.clone(),
            level: self.level,
            region: Region2D::Fill,
            _marker: marker::PhantomData,
        }
    }
}

impl<F, Rc> AsFramebufferAttachment<Rc> for Texture2DLevel<F, Rc>
where
    Rc: RenderingContext,
{
    fn as_framebuffer_attachment(&self) -> FramebufferAttachment<Rc> {
        FramebufferAttachment {
            internal: FramebufferAttachmentInternal::Texture2DLevel(
                self.texture_data.clone(),
                self.level,
            ),
        }
    }
}

pub struct Texture2DLevelSubImage<F, Rc> where Rc: RenderingContext {
    texture_data: Arc<Texture2DData<Rc>>,
    level: usize,
    region: Region2D,
    _marker: marker::PhantomData<[F]>,
}

impl<F, Rc> Texture2DLevelSubImage<F, Rc>
where
    F: TextureFormat,
    Rc: RenderingContext,
{
    pub fn level(&self) -> usize {
        self.level
    }

    pub fn region(&self) -> Region2D {
        self.region
    }

    pub fn width(&self) -> u32 {
        region_2d_overlap_width(self.texture_data.width, self.level, &self.region)
    }

    pub fn height(&self) -> u32 {
        region_2d_overlap_width(self.texture_data.height, self.level, &self.region)
    }

    pub fn sub_image(&self, region: Region2D) -> Texture2DLevelSubImage<F, Rc> {
        Texture2DLevelSubImage {
            texture_data: self.texture_data.clone(),
            level: self.level,
            region: region_2d_sub_image(self.region, region),
            _marker: marker::PhantomData,
        }
    }

    pub fn upload_task<D, T>(
        &self,
        data: Image2DSource<D, T>,
    ) -> Texture2DUploadTask<D, T, F, Rc>
    where
        T: ClientFormat<F>,
    {
        Texture2DUploadTask {
            data,
            texture_data: self.texture_data.clone(),
            level: self.level,
            region: self.region,
            _marker: marker::PhantomData,
        }
    }
}

struct Texture2DAllocateTask<F, Rc> where Rc: RenderingContext {
    data: Arc<Texture2DData<Rc>>,
    _marker: marker::PhantomData<[F]>,
}

impl<F, Rc> GpuTask<Connection> for Texture2DAllocateTask<F, Rc>
where
    F: TextureFormat,
    Rc: RenderingContext,
{
    type Output = ();

    fn progress(&mut self, connection: &mut Connection) -> Progress<Self::Output> {
        let Connection(gl, state) = connection;
        let mut data = Arc::get_mut(&mut self.data).unwrap();

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

struct Texture2DDropTask {
    id: JsId,
}

impl GpuTask<Connection> for Texture2DDropTask {
    type Output = ();

    fn progress(&mut self, connection: &mut Connection) -> Progress<Self::Output> {
        let Connection(gl, _) = connection;
        let texture_object = unsafe { JsId::into_value(self.id).unchecked_into() };

        gl.delete_texture(Some(&texture_object));

        Progress::Finished(())
    }
}

pub struct Texture2DUploadTask<D, T, F, Rc> where Rc: RenderingContext {
    data: Image2DSource<D, T>,
    texture_data: Arc<Texture2DData<Rc>>,
    level: usize,
    region: Region2D,
    _marker: marker::PhantomData<[F]>,
}

impl<D, T, F, Rc> GpuTask<Connection> for Texture2DUploadTask<D, T, F, Rc>
where
    D: Borrow<[T]>,
    T: ClientFormat<F>,
    F: TextureFormat,
    Rc: RenderingContext,
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

                self.texture_data
                    .id
                    .unwrap()
                    .with_value_unchecked(|texture_object| {
                        state
                            .set_bound_texture_2d(Some(texture_object))
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
                    Region2D::Area((offset_x, offset_y), ..) => (offset_x, offset_y)
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
                        Some(&mut *(data as *const _ as *mut _)),
                    ).unwrap();
                }
            }
        }

        Progress::Finished(())
    }
}
