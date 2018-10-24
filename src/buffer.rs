use std::borrow::Borrow;
use std::fmt::Debug;
use std::mem;
use std::ops::{ Bound, RangeBounds };
use std::sync::Arc;

use web_sys::{
    WebGl2RenderingContext as GL,
    WebGlBuffer
};

use super::task::{ GpuTask, Progress };
use super::rendering_context::{ Connection, RenderingContext, ContextUpdate };

#[derive(Clone, Copy)]
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

#[derive(Clone)]
pub struct GpuBufferHandle<C> {
    context: C,
    size_in_bytes: usize,
    usage_hint: BufferUsage,
    gl_buffer_object: Arc<Option<WebGlBuffer>>
}

impl<C> GpuBufferHandle<C> where C: RenderingContext {
    pub fn new(context: &C, size_in_bytes: usize, usage_hint: BufferUsage) -> Self {
        GpuBufferHandle {
            context: context.clone(),
            size_in_bytes,
            usage_hint,
            gl_buffer_object: Arc::new(None)
        }
    }

    pub fn context(&self) -> &C {
        &self.context
    }

    pub fn size_in_bytes(&self) -> usize {
        self.size_in_bytes
    }

    pub fn usage_hint(&self) -> BufferUsage {
        self.usage_hint
    }

    pub fn upload<D>(&self, data: D) -> BufferUpload<C, D> where D: Borrow<[u8]> + Send + Sync {
        BufferUpload {
            buffer_slice: BufferSlice {
                buffer_handle: self.clone(),
                offset_in_bytes: 0,
                size_in_bytes: self.size_in_bytes
            },
            data
        }
    }

    pub fn download(&self) -> BufferDownload<C> {
        BufferDownload {
            buffer_slice: BufferSlice {
                buffer_handle: self.clone(),
                offset_in_bytes: 0,
                size_in_bytes: self.size_in_bytes
            },
            state: BufferDownloadState::Initial
        }
    }

    pub fn slice<R>(&self, range: R) -> BufferSlice<C> where R: RangeBounds<usize> + Debug {
        let offset_in_bytes = match range.start_bound() {
            Bound::Included(start) => *start,
            Bound::Excluded(start) => start + 1,
            Bound::Unbounded => 0
        };

        let size_in_bytes = match range.end_bound() {
            Bound::Included(end) => end - offset_in_bytes,
            Bound::Excluded(end) => end - 1 - offset_in_bytes,
            Bound::Unbounded => self.size_in_bytes
        };

        if offset_in_bytes + size_in_bytes >= self.size_in_bytes {
            panic!("Range out of bounds");
        }

        BufferSlice {
            buffer_handle: self.clone(),
            offset_in_bytes,
            size_in_bytes
        }
    }

    fn gl_object(&self, connection: &mut Connection) -> &WebGlBuffer {
        unsafe {
            let Connection(gl, state) = connection;

            let maybe_buffer = Arc::into_raw(self.gl_buffer_object.clone()) as *mut Option<WebGlBuffer>;

            if (*maybe_buffer).is_none() {
                let buffer_object = gl.create_buffer().unwrap();

                state.set_bound_copy_write_buffer(Some(&buffer_object)).apply(gl).unwrap();
                gl.buffer_data_with_i32(GL::COPY_WRITE_BUFFER, self.size_in_bytes as i32, self.usage_hint.gl_id());

                *maybe_buffer = Some(buffer_object);
            };

            let buffer_object = (*maybe_buffer).as_ref().unwrap();

            Arc::from_raw(maybe_buffer);

            buffer_object
        }
    }
}

#[derive(Clone)]
pub struct BufferSlice<C> {
    buffer_handle: GpuBufferHandle<C>,
    offset_in_bytes: usize,
    size_in_bytes: usize,
}

impl<C> BufferSlice<C> where C: RenderingContext {
    pub fn context(&self) -> &C {
        &self.buffer_handle.context
    }

    pub fn buffer(&self) -> &GpuBufferHandle<C> {
        &self.buffer_handle
    }

    pub fn offset_in_bytes(&self) -> usize {
        self.offset_in_bytes
    }

    pub fn size_in_bytes(&self) -> usize {
        self.size_in_bytes
    }

    pub fn upload<D>(&self, data: D) -> BufferUpload<C, D> where D: Borrow<[u8]> {
        BufferUpload {
            buffer_slice: self.clone(),
            data
        }
    }

    pub fn download(&self) -> BufferDownload<C> {
        BufferDownload {
            buffer_slice: self.clone(),
            state: BufferDownloadState::Initial
        }
    }

    pub fn slice<R>(&self, range: R) -> BufferSlice<C> where R: RangeBounds<usize> + Debug {
        let offset_in_bytes = match range.start_bound() {
            Bound::Included(start) => *start,
            Bound::Excluded(start) => start + 1,
            Bound::Unbounded => 0
        };

        let size_in_bytes = match range.end_bound() {
            Bound::Included(end) => end - offset_in_bytes,
            Bound::Excluded(end) => end - 1 - offset_in_bytes,
            Bound::Unbounded => self.size_in_bytes
        };

        if offset_in_bytes + size_in_bytes >= self.size_in_bytes {
            panic!("Range out of bounds");
        }

        BufferSlice {
            buffer_handle: self.buffer_handle.clone(),
            offset_in_bytes: self.offset_in_bytes + offset_in_bytes,
            size_in_bytes
        }
    }
}

pub struct BufferUpload<C, D> {
    buffer_slice: BufferSlice<C>,
    data: D
}

impl<C, D> GpuTask<Connection> for BufferUpload<C, D> where C: RenderingContext, D: Borrow<[u8]> {
    type Output = ();

    type Error = ();

    fn progress(&mut self, connection: &mut Connection) -> Progress<Self::Output, Self::Error> {
        let buffer_object = self.buffer_slice.buffer_handle.gl_object(connection);
        let Connection(gl, state) = connection;
        let mut data = self.data.borrow();

        if data.len() > self.buffer_slice.size_in_bytes {
            data = &data[0..self.buffer_slice.size_in_bytes]
        }

        state.set_bound_copy_write_buffer(Some(&buffer_object)).apply(gl).unwrap();

        unsafe {
            let data = &mut *((data as *const _) as *mut _);

            gl.buffer_sub_data_with_i32_and_u8_array(GL::COPY_WRITE_BUFFER, self.buffer_slice.offset_in_bytes as i32, data);
        };

        Progress::Finished(Ok(()))
    }
}

pub struct BufferDownload<C> {
    buffer_slice: BufferSlice<C>,
    state: BufferDownloadState
}

enum BufferDownloadState {
    Initial,
    Copied(Option<WebGlBuffer>)
}

impl<C> GpuTask<Connection> for BufferDownload<C> where C: RenderingContext {
    type Output = Vec<u8>;

    type Error = ();

    fn progress(&mut self, connection: &mut Connection) -> Progress<Self::Output, Self::Error> {
        match self.state {
            BufferDownloadState::Initial => {
                let buffer = self.buffer_slice.buffer_handle.gl_object(connection);
                let Connection(gl, state) = connection;
                let read_buffer = gl.create_buffer().unwrap();

                state.set_bound_copy_write_buffer(Some(&read_buffer)).apply(gl).unwrap();

                gl.buffer_data_with_i32(GL::COPY_WRITE_BUFFER, self.buffer_slice.size_in_bytes as i32, GL::STREAM_READ);

                state.set_bound_copy_read_buffer(Some(buffer)).apply(gl).unwrap();

                gl.copy_buffer_sub_data_with_i32_and_i32_and_i32(
                    GL::COPY_READ_BUFFER,
                    GL::COPY_WRITE_BUFFER,
                    self.buffer_slice.offset_in_bytes as i32,
                    0,
                    self.buffer_slice.size_in_bytes as i32
                );

                mem::replace(&mut self.state, BufferDownloadState::Copied(Some(read_buffer)));

                Progress::ContinueFenced
            },
            BufferDownloadState::Copied(ref mut read_buffer) => {
                let read_buffer = read_buffer.take().expect("Cannot make progress on a BufferDownload task after it has finished");
                let Connection(gl, state) = connection;

                state.set_bound_copy_read_buffer(Some(&read_buffer)).apply(gl).unwrap();

                let mut data = vec![0; self.buffer_slice.size_in_bytes];

                gl.get_buffer_sub_data_with_i32_and_u8_array(GL::COPY_READ_BUFFER, 0, &mut data);

                gl.delete_buffer(Some(&read_buffer));

                Progress::Finished(Ok(data))
            }
        }
    }
}

