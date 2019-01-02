use std::borrow::Borrow;
use std::marker;
use std::mem;
use std::ops::{Bound, RangeBounds};
use std::slice;
use std::sync::Arc;

use web_sys::{WebGl2RenderingContext as GL, WebGlBuffer};

use super::rendering_context::{Connection, ContextUpdate, RenderingContext};
use super::task::{GpuTask, Progress};
use super::util::JsId;
use rendering_context::BufferRange;
use rendering_context::DropObject;
use rendering_context::Dropper;
use rendering_context::RefCountedDropper;
use util::arc_get_mut_unchecked;

#[derive(Clone, Copy, Debug)]
pub enum BufferUsage {
    StaticDraw,
    DynamicDraw,
    StreamDraw,
    StaticRead,
    DynamicRead,
    StreamRead,
    StaticCopy,
    DynamicCopy,
    StreamCopy,
}

impl BufferUsage {
    fn gl_id(&self) -> u32 {
        match self {
            BufferUsage::StaticDraw => GL::STATIC_DRAW,
            BufferUsage::DynamicDraw => GL::DYNAMIC_DRAW,
            BufferUsage::StreamDraw => GL::STREAM_DRAW,
            BufferUsage::StaticRead => GL::STATIC_READ,
            BufferUsage::DynamicRead => GL::DYNAMIC_READ,
            BufferUsage::StreamRead => GL::STREAM_READ,
            BufferUsage::StaticCopy => GL::STATIC_COPY,
            BufferUsage::DynamicCopy => GL::DYNAMIC_COPY,
            BufferUsage::StreamCopy => GL::STREAM_COPY,
        }
    }
}

#[derive(Clone)]
pub struct BufferHandle<T>
where
    T: ?Sized,
{
    data: Arc<BufferData>,
    _marker: marker::PhantomData<Box<T>>,
}

pub(crate) struct BufferData {
    id: Option<JsId>,
    dropper: RefCountedDropper,
    len: usize,
    size_in_bytes: usize,
    usage_hint: BufferUsage,
    recent_uniform_binding: Option<u32>,
}

impl<T> BufferHandle<T>
where
    T: ?Sized,
{
    pub fn usage_hint(&self) -> BufferUsage {
        self.data.usage_hint
    }
}

impl<T> BufferHandle<T> {
    pub(crate) fn value<Rc>(
        context: &Rc,
        dropper: RefCountedDropper,
        usage_hint: BufferUsage,
    ) -> Self
    where
        Rc: RenderingContext,
    {
        let data = Arc::new(BufferData {
            id: None,
            dropper,
            len: 1,
            usage_hint,
            size_in_bytes: mem::size_of::<T>(),
            recent_uniform_binding: None,
        });

        context.submit(BufferAllocateTask { data: data.clone() });

        BufferHandle {
            data,
            _marker: marker::PhantomData,
        }
    }

    pub(crate) fn bind_uniform(&self, connection: &mut Connection) -> u32 {
        let Connection(gl, state) = connection;

        unsafe {
            let data = arc_get_mut_unchecked(&self.data);
            let most_recent_binding = &mut data.recent_uniform_binding;

            data.id.unwrap().with_value_unchecked(|buffer_object| {
                if most_recent_binding.is_none()
                    || state.bound_uniform_buffer_range(most_recent_binding.unwrap())
                        != BufferRange::Full(buffer_object)
                {
                    state.set_active_uniform_buffer_binding_lru();
                    state
                        .set_bound_uniform_buffer_range(BufferRange::Full(&buffer_object))
                        .apply(gl)
                        .unwrap();

                    let binding = state.active_uniform_buffer_binding();

                    *most_recent_binding = Some(binding);

                    binding
                } else {
                    most_recent_binding.unwrap()
                }
            })
        }
    }

    pub fn upload_task<D>(&self, data: D) -> BufferUploadTask<T, D>
    where
        D: Borrow<T> + Send + Sync + 'static,
    {
        BufferUploadTask {
            buffer_data: self.data.clone(),
            data,
            offset: 0,
            len: 1,
            _marker: marker::PhantomData,
        }
    }

    pub fn download_task(&self) -> BufferDownloadTask<T> {
        BufferDownloadTask {
            data: self.data.clone(),
            state: BufferDownloadState::Initial,
            offset: 0,
            len: 1,
            _marker: marker::PhantomData,
        }
    }
}

impl<T> BufferHandle<[T]> {
    pub(crate) fn array<Rc>(
        context: &Rc,
        dropper: RefCountedDropper,
        len: usize,
        usage_hint: BufferUsage,
    ) -> Self
    where
        Rc: RenderingContext,
    {
        let data = Arc::new(BufferData {
            id: None,
            dropper,
            len,
            usage_hint,
            size_in_bytes: mem::size_of::<T>(),
            recent_uniform_binding: None,
        });

        context.submit(BufferAllocateTask { data: data.clone() });

        BufferHandle {
            data,
            _marker: marker::PhantomData,
        }
    }

    pub fn len(&self) -> usize {
        self.data.len
    }

    pub fn upload_task<D>(&self, data: D) -> BufferUploadTask<[T], D>
    where
        D: Borrow<[T]> + Send + Sync + 'static,
    {
        BufferUploadTask {
            buffer_data: self.data.clone(),
            data,
            offset: 0,
            len: self.data.len,
            _marker: marker::PhantomData,
        }
    }

    pub fn download_task(&self) -> BufferDownloadTask<[T]> {
        BufferDownloadTask {
            data: self.data.clone(),
            state: BufferDownloadState::Initial,
            offset: 0,
            len: self.data.len,
            _marker: marker::PhantomData,
        }
    }

    pub fn slice<R>(&self, range: R) -> BufferSlice<T>
    where
        R: RangeBounds<usize>,
    {
        let len = self.data.len;
        let (start, end) = slice_bounds(range, len);

        if end - start < 1 || end > len {
            panic!(
                "Range must be a positive non-zero range that fits 0..{}",
                len
            );
        }

        BufferSlice {
            data: self.data.clone(),
            offset: start,
            len: end - start,
            _marker: marker::PhantomData,
        }
    }
}

impl Drop for BufferData {
    fn drop(&mut self) {
        if let Some(id) = self.id {
            self.dropper.drop_gl_object(DropObject::Buffer(id));
        }
    }
}

#[derive(Clone)]
pub struct BufferSlice<T> {
    data: Arc<BufferData>,
    offset: usize,
    len: usize,
    _marker: marker::PhantomData<[T]>,
}

impl<T> BufferSlice<T> {
    pub fn usage_hint(&self) -> BufferUsage {
        self.data.usage_hint
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn upload_task<D>(&self, data: D) -> BufferUploadTask<[T], D>
    where
        D: Borrow<[T]> + Send + Sync + 'static,
    {
        BufferUploadTask {
            buffer_data: self.data.clone(),
            data,
            offset: self.offset,
            len: self.len,
            _marker: marker::PhantomData,
        }
    }

    pub fn download_task(&self) -> BufferDownloadTask<[T]> {
        BufferDownloadTask {
            data: self.data.clone(),
            state: BufferDownloadState::Initial,
            offset: self.offset,
            len: self.len,
            _marker: marker::PhantomData,
        }
    }

    pub fn slice<R>(&self, range: R) -> BufferSlice<T>
    where
        R: RangeBounds<usize>,
    {
        let (start, end) = slice_bounds(range, self.len);

        if end - start < 1 || end > self.len {
            panic!(
                "Range must be a positive non-zero range that fits 0..{}",
                self.len
            );
        }

        BufferSlice {
            data: self.data.clone(),
            offset: self.offset + start,
            len: end - start,
            _marker: marker::PhantomData,
        }
    }
}

impl<T> Into<BufferSlice<T>> for BufferHandle<[T]> {
    fn into(self) -> BufferSlice<T> {
        BufferSlice {
            data: self.data.clone(),
            offset: 0,
            len: self.data.len,
            _marker: marker::PhantomData,
        }
    }
}

struct BufferAllocateTask {
    data: Arc<BufferData>,
}

impl GpuTask<Connection> for BufferAllocateTask {
    type Output = ();

    fn progress(&mut self, connection: &mut Connection) -> Progress<Self::Output> {
        let Connection(gl, state) = connection;
        let data = unsafe { arc_get_mut_unchecked(&mut self.data) };

        let buffer_object = gl.create_buffer().unwrap();

        state
            .set_bound_copy_write_buffer(Some(&buffer_object))
            .apply(gl)
            .unwrap();

        gl.buffer_data_with_i32(
            GL::COPY_WRITE_BUFFER,
            data.size_in_bytes as i32,
            data.usage_hint.gl_id(),
        );

        data.id = Some(JsId::from_value(buffer_object.into()));

        Progress::Finished(())
    }
}

pub struct BufferUploadTask<T, D>
where
    T: ?Sized,
{
    buffer_data: Arc<BufferData>,
    data: D,
    offset: usize,
    len: usize,
    _marker: marker::PhantomData<T>,
}

impl<T, D> GpuTask<Connection> for BufferUploadTask<T, D>
where
    D: Borrow<T>,
{
    type Output = ();

    fn progress(&mut self, connection: &mut Connection) -> Progress<Self::Output> {
        let Connection(gl, state) = connection;

        unsafe {
            self.buffer_data
                .id
                .unwrap()
                .with_value_unchecked(|buffer_object| {
                    state
                        .set_bound_copy_write_buffer(Some(&buffer_object))
                        .apply(gl)
                        .unwrap();
                });
        }

        unsafe {
            let data = slice::from_raw_parts(
                self.data.borrow() as *const _ as *const u8,
                mem::size_of::<T>(),
            );

            gl.buffer_sub_data_with_i32_and_u8_array(
                GL::COPY_WRITE_BUFFER,
                0,
                &mut *(data as *const _ as *mut _),
            );
        };

        Progress::Finished(())
    }
}

impl<T, D> GpuTask<Connection> for BufferUploadTask<[T], D>
where
    D: Borrow<[T]>,
{
    type Output = ();

    fn progress(&mut self, connection: &mut Connection) -> Progress<Self::Output> {
        let Connection(gl, state) = connection;

        unsafe {
            self.buffer_data
                .id
                .unwrap()
                .with_value_unchecked(|buffer_object| {
                    state
                        .set_bound_copy_write_buffer(Some(&buffer_object))
                        .apply(gl)
                        .unwrap();
                });
        }

        let data = self.data.borrow();
        let element_size = mem::size_of::<T>();
        let len = data.len();
        let max_len = element_size * self.len;
        let offset = element_size * self.offset;

        unsafe {
            let mut data = slice::from_raw_parts(
                self.data.borrow() as *const _ as *const u8,
                element_size * len,
            );

            if max_len > len {
                data = &data[0..max_len];
            }

            gl.buffer_sub_data_with_i32_and_u8_array(
                GL::COPY_WRITE_BUFFER,
                offset as i32,
                &mut *(data as *const _ as *mut _),
            );
        };

        Progress::Finished(())
    }
}

pub struct BufferDownloadTask<T>
where
    T: ?Sized,
{
    data: Arc<BufferData>,
    state: BufferDownloadState,
    offset: usize,
    len: usize,
    _marker: marker::PhantomData<T>,
}

enum BufferDownloadState {
    Initial,
    Copied(Option<WebGlBuffer>),
}

impl<T> GpuTask<Connection> for BufferDownloadTask<T> {
    type Output = Box<T>;

    fn progress(&mut self, connection: &mut Connection) -> Progress<Self::Output> {
        match self.state {
            BufferDownloadState::Initial => {
                let Connection(gl, state) = connection;
                let read_buffer = gl.create_buffer().unwrap();
                let size_in_bytes = self.data.size_in_bytes;

                state
                    .set_bound_copy_write_buffer(Some(&read_buffer))
                    .apply(gl)
                    .unwrap();

                gl.buffer_data_with_i32(
                    GL::COPY_WRITE_BUFFER,
                    size_in_bytes as i32,
                    GL::STREAM_READ,
                );

                unsafe {
                    self.data.id.unwrap().with_value_unchecked(|buffer_object| {
                        state
                            .set_bound_copy_read_buffer(Some(&buffer_object))
                            .apply(gl)
                            .unwrap();
                    });
                }

                gl.copy_buffer_sub_data_with_i32_and_i32_and_i32(
                    GL::COPY_READ_BUFFER,
                    GL::COPY_WRITE_BUFFER,
                    0,
                    0,
                    size_in_bytes as i32,
                );

                mem::replace(
                    &mut self.state,
                    BufferDownloadState::Copied(Some(read_buffer)),
                );

                Progress::ContinueFenced
            }
            BufferDownloadState::Copied(ref mut read_buffer) => {
                let read_buffer = read_buffer
                    .take()
                    .expect("Cannot make progress on a BufferDownload task after it has finished");
                let Connection(gl, state) = connection;

                state
                    .set_bound_copy_read_buffer(Some(&read_buffer))
                    .apply(gl)
                    .unwrap();

                let mut data = vec![0; self.data.size_in_bytes];

                gl.get_buffer_sub_data_with_i32_and_u8_array(GL::COPY_READ_BUFFER, 0, &mut data);

                gl.delete_buffer(Some(&read_buffer));

                let value = unsafe { Box::from_raw(mem::transmute(data.as_mut_ptr())) };

                mem::forget(data);

                Progress::Finished(value)
            }
        }
    }
}

impl<T> GpuTask<Connection> for BufferDownloadTask<[T]> {
    type Output = Box<[T]>;

    fn progress(&mut self, connection: &mut Connection) -> Progress<Self::Output> {
        match self.state {
            BufferDownloadState::Initial => {
                let Connection(gl, state) = connection;
                let read_buffer = gl.create_buffer().unwrap();
                let element_size = mem::size_of::<T>();
                let size = element_size * self.len;

                state
                    .set_bound_copy_write_buffer(Some(&read_buffer))
                    .apply(gl)
                    .unwrap();

                gl.buffer_data_with_i32(GL::COPY_WRITE_BUFFER, size as i32, GL::STREAM_READ);

                unsafe {
                    self.data.id.unwrap().with_value_unchecked(|buffer_object| {
                        state
                            .set_bound_copy_read_buffer(Some(&buffer_object))
                            .apply(gl)
                            .unwrap();
                    });
                }

                gl.copy_buffer_sub_data_with_i32_and_i32_and_i32(
                    GL::COPY_READ_BUFFER,
                    GL::COPY_WRITE_BUFFER,
                    (self.offset * element_size) as i32,
                    0,
                    size as i32,
                );

                mem::replace(
                    &mut self.state,
                    BufferDownloadState::Copied(Some(read_buffer)),
                );

                Progress::ContinueFenced
            }
            BufferDownloadState::Copied(ref mut read_buffer) => {
                let read_buffer = read_buffer
                    .take()
                    .expect("Cannot make progress on a BufferDownload task after it has finished");
                let Connection(gl, state) = connection;

                state
                    .set_bound_copy_read_buffer(Some(&read_buffer))
                    .apply(gl)
                    .unwrap();

                let mut data = vec![0; self.len * mem::size_of::<T>()];

                gl.get_buffer_sub_data_with_i32_and_u8_array(GL::COPY_READ_BUFFER, 0, &mut data);

                gl.delete_buffer(Some(&read_buffer));

                unsafe {
                    let len = self.len;
                    let ptr = mem::transmute(data.as_mut_ptr());
                    let slice = slice::from_raw_parts_mut(ptr, len);
                    let boxed = Box::from_raw(slice);

                    mem::forget(data);

                    Progress::Finished(boxed)
                }
            }
        }
    }
}

fn slice_bounds<R>(range: R, len: usize) -> (usize, usize)
where
    R: RangeBounds<usize>,
{
    let start = match range.start_bound() {
        Bound::Unbounded => 0,
        Bound::Excluded(b) => b + 1,
        Bound::Included(b) => *b,
    };

    let end = match range.end_bound() {
        Bound::Unbounded => len,
        Bound::Excluded(b) => *b,
        Bound::Included(b) => b - 1,
    };

    (start, end)
}
