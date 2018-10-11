use gpu_command::GpuCommand;
use rendering_context::{ ContextError, RenderingContext };
use webgl_bindings::{ WebGLBuffer };

use std::borrow::Borrow;
use std::fmt::Debug;
use std::ops::{ Range, RangeBounds };
use std::sync::Arc;

#[derive(Clone)]
pub struct GpuBufferHandle {
    size_in_bytes: usize,
    gl_buffer_object: Arc<Option<WebGLBuffer>>
}

impl GpuBufferHandle {
    fn new(size_in_bytes: usize) -> Self {

    }

    fn size_in_bytes(&self) -> usize {
        self.size_in_bytes
    }

    fn allocate(&self) -> AllocateBuffer {
        AllocateBuffer::new(self.clone())
    }

    fn upload<D>(&self, data: D) -> BufferUpload<D> where D: Borrow<[u8]> + Send + Sync {
        BufferUpload::new(self.clone(), data)
    }

    fn download(&self) -> BufferDownload {
        BufferDownload::new(self.clone())
    }

    fn slice<R>(&self, range: R) -> Result<BufferSlice, RangeError<R>> where R: RangeBounds<usize> + Debug {
        BufferSlice::new(self, range)
    }
}

pub struct BufferSlice {
    buffer_handle: GpuBufferHandle,
    offset_in_bytes: usize,
    size_in_bytes: usize,
}

impl BufferSlice {
    pub fn new<R>(buffer_handle: GpuBufferHandle, range: R) -> Result<BufferSlice, SliceRangeError<R>> where R: RangeBounds<usize> + Debug {

    }
}

#[derive(Debug, Display, Fail)]
#[display = "The slice range must be a sub-range of {}, {} given.", valid_range, range]
pub struct SliceRangeError<R> where R: RangeBounds<usize> + Debug {
    range: R,
    valid_range: Range<usize>
}

struct AllocateBuffer {
    buffer_handle: GpuBufferHandle
}

impl AllocateBuffer {
    pub fn new(buffer_handle: GpuBufferHandle) -> Self {
        AllocateBuffer {
            buffer_handle
        }
    }

    fn execute_internal(&mut self, rendering_context: &mut RenderingContext) -> Result<(), ContextError> {
        unsafe {
            // This is safe because the gl_buffer_object is only ever read or written to while
            // interacting with its RenderingContext and we have a mutable reference to its
            // RenderingContext.

            let ptr = self.buffer_handle.gl_buffer_object.clone().into_raw() as *mut _;

            match *ptr {
                Some(_) => Ok(()),
                None => {}
            }
        }
    }
}

impl GpuCommand<RenderingContext> for AllocateBuffer {
    type Output = ();

    type Error = ContextError;


}

