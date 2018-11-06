use std::borrow::Borrow;
use std::marker;
use std::mem;
use std::ops::{ Bound, RangeBounds };
use std::slice;
use std::sync::Arc;

use wasm_bindgen::JsCast;
use web_sys::{
    WebGl2RenderingContext as GL,
    WebGlBuffer
};

use super::task::{ GpuTask, Progress };
use super::rendering_context::{ Connection, RenderingContext, ContextUpdate };
use super::util::JsId;

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
    StreamCopy
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
            BufferUsage::StreamCopy => GL::STREAM_COPY
        }
    }
}

pub struct BufferHandle<T, C> where T: ?Sized, C: RenderingContext {
    internal: Arc<BufferHandleInternal<C>>,
    _marker: marker::PhantomData<Box<T>>,
}

struct BufferHandleInternal<C> where C: RenderingContext {
    gl_object_id: Option<JsId>,
    context: C,
    len: usize,
    size_in_bytes: usize,
    usage_hint: BufferUsage,
}

impl<T, C> BufferHandle<T, C> where T: ?Sized, C: RenderingContext {
    pub fn context(&self) -> &C {
        &self.internal.context
    }

    pub fn usage_hint(&self) -> BufferUsage {
        self.internal.usage_hint
    }

    pub(crate) fn gl_object(&self, connection: &mut Connection) -> WebGlBuffer {
        // It is safe to mutate the handle's JsId here without a Mutex, because by design an
        // &mut Connection should be globally unique (this can only happen on the main graphics
        // thread, not on multiple threads concurrently).

        unsafe {
            let Connection(gl, state) = connection;
            let internal = &self.internal;
            let maybe_buffer = &internal.gl_object_id as *const _ as *mut Option<JsId>;

            if let Some(js_id) = *maybe_buffer {
                JsId::into_value(js_id).unchecked_into()
            } else {
                let buffer_object = gl.create_buffer().unwrap();

                state.set_bound_copy_write_buffer(Some(&buffer_object)).apply(gl).unwrap();
                gl.buffer_data_with_i32(GL::COPY_WRITE_BUFFER, internal.size_in_bytes as i32, internal.usage_hint.gl_id());

                let js_id = JsId::from_value(buffer_object.into());

                *maybe_buffer = Some(js_id);

                JsId::into_value(js_id).unchecked_into()
            }
        }
    }
}

impl<T, C> BufferHandle<T, C> where C: RenderingContext {
    pub(crate) fn value(context: C, usage_hint: BufferUsage) -> Self {
        BufferHandle {
            internal: Arc::new(BufferHandleInternal {
                gl_object_id: None,
                context,
                len: 1,
                usage_hint,
                size_in_bytes: mem::size_of::<T>()
            }),
            _marker: marker::PhantomData
        }
    }

    pub fn upload_task<D>(&self, data: D) -> BufferUploadTask<Self, D> where D: Borrow<[T]> + Send + Sync + 'static {
        BufferUploadTask {
            buffer_rep: self.clone(),
            data
        }
    }

    pub fn download_task(&self) -> BufferDownloadTask<Self> {
        BufferDownloadTask {
            buffer_rep: self.clone(),
            state: BufferDownloadState::Initial
        }
    }
}

impl<T, C> BufferHandle<[T], C> where C: RenderingContext {
    pub(crate) fn array(context: C, len: usize, usage_hint: BufferUsage) -> Self {
        BufferHandle {
            internal: Arc::new(BufferHandleInternal {
                gl_object_id: None,
                context,
                len,
                usage_hint,
                size_in_bytes: mem::size_of::<T>() * len
            }),
            _marker: marker::PhantomData
        }
    }

    pub fn len(&self) -> usize {
        self.internal.len
    }

    pub fn upload_task<D>(&self, data: D) -> BufferUploadTask<ArrayBufferSlice<T, C>, D> where D: Borrow<[T]> + Send + Sync + 'static {
        BufferUploadTask {
            buffer_rep: self.clone().into(),
            data
        }
    }

    pub fn download_task(&self) -> BufferDownloadTask<ArrayBufferSlice<T, C>> {
        BufferDownloadTask {
            buffer_rep: self.clone().into(),
            state: BufferDownloadState::Initial
        }
    }

    pub fn slice<R>(&self, range: R) -> ArrayBufferSlice<T, C> where R: RangeBounds<usize> {
        let len = self.internal.len;
        let (start, end) = slice_bounds(range, len);

        if end - start < 1 || end > len {
            panic!("Range must be a positive non-zero range that fits 0..{}", len);
        }

        ArrayBufferSlice {
            buffer_handle: self.clone(),
            offset: start,
            len: end - start
        }
    }
}

impl<T, C> Clone for BufferHandle<T, C> where C: RenderingContext {
    fn clone(&self) -> Self {
        BufferHandle {
            internal: self.internal.clone(),
            _marker: marker::PhantomData
        }
    }
}

impl<T, C> Clone for BufferHandle<[T], C> where C: RenderingContext {
    fn clone(&self) -> Self {
        BufferHandle {
            internal: self.internal.clone(),
            _marker: marker::PhantomData
        }
    }
}

impl<C> Drop for BufferHandleInternal<C> where C: RenderingContext {
    fn drop(&mut self) {
        if let Some(buffer_id) = self.gl_object_id {
            self.context.submit(DropGlBufferTask {
                buffer_id
            });
        }
    }
}

pub struct ArrayBufferSlice<T, C> where C: RenderingContext {
    buffer_handle: BufferHandle<[T], C>,
    offset: usize,
    len: usize
}

impl<T, C> ArrayBufferSlice<T, C> where C: RenderingContext {
    pub fn context(&self) -> &C {
        &self.buffer_handle.internal.context
    }

    pub fn usage_hint(&self) -> BufferUsage {
        self.buffer_handle.internal.usage_hint
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn upload_task<D>(&self, data: D) -> BufferUploadTask<Self, D> where D: Borrow<[T]> + Send + Sync + 'static {
        BufferUploadTask {
            buffer_rep: self.clone(),
            data
        }
    }

    pub fn download_task(&self) -> BufferDownloadTask<Self> {
        BufferDownloadTask {
            buffer_rep: self.clone(),
            state: BufferDownloadState::Initial
        }
    }

    pub fn slice<R>(&self, range: R) -> ArrayBufferSlice<T, C> where R: RangeBounds<usize> {
        let (start, end) = slice_bounds(range, self.len);

        if end - start < 1 || end > self.len {
            panic!("Range must be a positive non-zero range that fits 0..{}", self.len);
        }

        ArrayBufferSlice {
            buffer_handle: self.buffer_handle.clone(),
            offset: self.offset + start,
            len: end - start
        }
    }
}

impl<T, C> Clone for ArrayBufferSlice<T, C> where C: RenderingContext {
    fn clone(&self) -> Self {
        ArrayBufferSlice {
            buffer_handle: self.buffer_handle.clone(),
            offset: self.offset,
            len: self.len
        }
    }
}

impl<T, C> Into<ArrayBufferSlice<T, C>> for BufferHandle<[T], C> where C: RenderingContext {
    fn into(self) -> ArrayBufferSlice<T, C> {
        ArrayBufferSlice {
            offset: 0,
            len: self.internal.len,
            buffer_handle: self
        }
    }
}

struct DropGlBufferTask {
    buffer_id: JsId
}

impl GpuTask<Connection> for DropGlBufferTask {
    type Output = ();

    type Error = ();

    fn progress(&mut self, _connection: &mut Connection) -> Progress<Self::Output, Self::Error> {
        unsafe {
            JsId::into_value(self.buffer_id);
        }

        Progress::Finished(Ok(()))
    }
}

pub struct BufferUploadTask<B, D> {
    buffer_rep: B,
    data: D
}

impl<D, T, C> GpuTask<Connection> for BufferUploadTask<BufferHandle<T, C>, D> where D: Borrow<T>, C: RenderingContext {
    type Output = ();

    type Error = ();

    fn progress(&mut self, connection: &mut Connection) -> Progress<Self::Output, Self::Error> {
        let buffer_object = self.buffer_rep.gl_object(connection);
        let Connection(gl, state) = connection;

        state.set_bound_copy_write_buffer(Some(&buffer_object)).apply(gl).unwrap();

        unsafe {
            let data = slice::from_raw_parts(self.data.borrow() as *const _ as *const u8, mem::size_of::<T>());

            gl.buffer_sub_data_with_i32_and_u8_array(GL::COPY_WRITE_BUFFER, 0, &mut *(data as *const _ as *mut _));
        };

        mem::forget(buffer_object);

        Progress::Finished(Ok(()))
    }
}

impl<D, T, C> GpuTask<Connection> for BufferUploadTask<ArrayBufferSlice<T, C>, D> where D: Borrow<[T]>, C: RenderingContext {
    type Output = ();

    type Error = ();

    fn progress(&mut self, connection: &mut Connection) -> Progress<Self::Output, Self::Error> {
        let buffer_object = self.buffer_rep.buffer_handle.gl_object(connection);
        let Connection(gl, state) = connection;

        state.set_bound_copy_write_buffer(Some(&buffer_object)).apply(gl).unwrap();

        unsafe {
            let data = self.data.borrow();
            let len = data.len();
            let mut data = slice::from_raw_parts(self.data.borrow() as *const _ as *const u8, mem::size_of::<T>() * len);
            let element_size = mem::size_of::<T>();
            let max_len = element_size * self.buffer_rep.len;

            if max_len > len {
                data = &data[0..max_len];
            }

            let offset = element_size * self.buffer_rep.offset;

            gl.buffer_sub_data_with_i32_and_u8_array(GL::COPY_WRITE_BUFFER, offset as i32, &mut *(data as *const _ as *mut _));
        };

        mem::forget(buffer_object);

        Progress::Finished(Ok(()))
    }
}

pub struct BufferDownloadTask<B> {
    buffer_rep: B,
    state: BufferDownloadState
}

enum BufferDownloadState {
    Initial,
    Copied(Option<WebGlBuffer>)
}

impl<T, C> GpuTask<Connection> for BufferDownloadTask<BufferHandle<T, C>> where C: RenderingContext {
    type Output = Box<T>;

    type Error = ();

    fn progress(&mut self, connection: &mut Connection) -> Progress<Self::Output, Self::Error> {
        match self.state {
            BufferDownloadState::Initial => {
                let buffer = self.buffer_rep.gl_object(connection);
                let Connection(gl, state) = connection;
                let read_buffer = gl.create_buffer().unwrap();
                let size_in_bytes = self.buffer_rep.internal.size_in_bytes;

                state.set_bound_copy_write_buffer(Some(&read_buffer)).apply(gl).unwrap();

                gl.buffer_data_with_i32(GL::COPY_WRITE_BUFFER, size_in_bytes as i32, GL::STREAM_READ);

                state.set_bound_copy_read_buffer(Some(&buffer)).apply(gl).unwrap();

                gl.copy_buffer_sub_data_with_i32_and_i32_and_i32(
                    GL::COPY_READ_BUFFER,
                    GL::COPY_WRITE_BUFFER,
                    0,
                    0,
                    size_in_bytes as i32
                );

                mem::replace(&mut self.state, BufferDownloadState::Copied(Some(read_buffer)));

                mem::forget(buffer);

                Progress::ContinueFenced
            },
            BufferDownloadState::Copied(ref mut read_buffer) => {
                let read_buffer = read_buffer.take().expect("Cannot make progress on a BufferDownload task after it has finished");
                let Connection(gl, state) = connection;

                state.set_bound_copy_read_buffer(Some(&read_buffer)).apply(gl).unwrap();

                let mut data = vec![0; self.buffer_rep.internal.size_in_bytes];

                gl.get_buffer_sub_data_with_i32_and_u8_array(GL::COPY_READ_BUFFER, 0, &mut data);

                gl.delete_buffer(Some(&read_buffer));

                let value = unsafe { Box::from_raw(mem::transmute(data.as_mut_ptr())) };

                mem::forget(data);

                Progress::Finished(Ok(value))
            }
        }
    }
}

impl<T, C> GpuTask<Connection> for BufferDownloadTask<ArrayBufferSlice<T, C>> where C: RenderingContext {
    type Output = Box<[T]>;

    type Error = ();

    fn progress(&mut self, connection: &mut Connection) -> Progress<Self::Output, Self::Error> {
        match self.state {
            BufferDownloadState::Initial => {
                let buffer = self.buffer_rep.buffer_handle.gl_object(connection);
                let Connection(gl, state) = connection;
                let read_buffer = gl.create_buffer().unwrap();
                let element_size = mem::size_of::<T>();
                let size = element_size * self.buffer_rep.len;

                state.set_bound_copy_write_buffer(Some(&read_buffer)).apply(gl).unwrap();

                gl.buffer_data_with_i32(GL::COPY_WRITE_BUFFER, size as i32, GL::STREAM_READ);

                state.set_bound_copy_read_buffer(Some(&buffer)).apply(gl).unwrap();

                gl.copy_buffer_sub_data_with_i32_and_i32_and_i32(
                    GL::COPY_READ_BUFFER,
                    GL::COPY_WRITE_BUFFER,
                    (self.buffer_rep.offset * element_size) as i32,
                    0,
                    size as i32
                );

                mem::replace(&mut self.state, BufferDownloadState::Copied(Some(read_buffer)));

                mem::forget(buffer);

                Progress::ContinueFenced
            },
            BufferDownloadState::Copied(ref mut read_buffer) => {
                let read_buffer = read_buffer.take().expect("Cannot make progress on a BufferDownload task after it has finished");
                let Connection(gl, state) = connection;

                state.set_bound_copy_read_buffer(Some(&read_buffer)).apply(gl).unwrap();

                let mut data = vec![0; self.buffer_rep.len * mem::size_of::<T>()];

                gl.get_buffer_sub_data_with_i32_and_u8_array(GL::COPY_READ_BUFFER, 0, &mut data);

                gl.delete_buffer(Some(&read_buffer));

                unsafe {
                    let len = self.buffer_rep.len;
                    let ptr = mem::transmute(data.as_mut_ptr());
                    let slice = slice::from_raw_parts_mut(ptr, len);
                    let boxed = Box::from_raw(slice);

                    mem::forget(data);

                    Progress::Finished(Ok(boxed))
                }
            }
        }
    }
}

fn slice_bounds<R>(range: R, len: usize) -> (usize, usize) where R: RangeBounds<usize> {
    let start = match range.start_bound() {
        Bound::Unbounded => 0,
        Bound::Excluded(b) => b + 1,
        Bound::Included(b) => *b
    };

    let end = match range.end_bound() {
        Bound::Unbounded => len,
        Bound::Excluded(b) => *b,
        Bound::Included(b) => b - 1
    };

    (start, end)
}
