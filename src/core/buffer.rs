use std::borrow::Borrow;
use std::ops::{Range, RangeBounds};

use super::base_rendering_context::BaseRenderingContext;
use super::gpu_command::GpuCommand;

pub trait GpuBuffer<Rc> {
    fn new(rendering_context: Rc, size_in_bytes: usize) -> Self;

    fn size_in_bytes(&self) -> usize;

    fn allocate(&self) -> AllocateBuffer<Rc, Self>;

    fn upload<D>(&self, data: D) -> BufferUpload<Rc, D> where D: Borrow<[u8]> + Send + Sync;

    fn download(&self) -> BufferDownload<Rc>;

    fn slice<R>(&self, range: R) -> Result<BufferSlice<Rc, Self>, RangeError<R>> where R: RangeBounds<usize> {
        BufferSlice::new(self, range)
    }
}

pub struct TypedBuffer<Rc, T> where Rc: RenderingContext {
    byte_buffer: Rc::BufferHandle
}

impl<Rc, T> Borrow<Rc::BufferHandle> TypedBuffer<Rc, T>

#[derive(Clone, Debug)]
pub struct BufferSlice<Rc, B> {
    buffer: B,
    offset: usize,
    len: usize
}

impl<Rc, B> GpuBufferView<Rc> for BufferSlice<Rc, B> where B: GpuBuffer<Rc> {
    type Slice = BufferSlice<Rc, B>;

    fn offset(&self) -> usize {
        self.offset
    }

    fn len(&self) -> usize {
        self.len
    }

    fn upload<D>(&self, data: D) -> BufferUploadCommand<Rc, B, D> where D: Borrow<[u8]> + Send + Sync {
        BufferUploadCommand::new(self.clone(), data)
    }

    fn download(&self) -> BufferDownloadCommand<Rc>;

    fn slice<R>(&self, range: R) -> Result<Self::Slice, RangeError<R>> where R: RangeBounds<usize> {
        let additional_offset = ;
        let len = ;

        BufferSlice {
            buffer: self.buffer.clone(),
            offset: self.offset + additional_offset,
            len
        }
    }
}

struct RangeError<R> where R: RangeBounds<usize> {
    range: R,
    valid_range: Range<usize>,
}

pub struct BufferUploadCommand<Rc, B, D> {
    buffer_view: B,
    data: D
}

impl<Rc, B, D> BufferUploadCommand<Rc, B, D> where D: Borrow<[u8]> + Send + Sync {
    pub fn new(buffer_view: B, data: D) -> Self {
        BufferUploadCommand {
            buffer_view,
            data
        }
    }
}

impl<Rc, D> BufferUploadCommand<Rc, Rc::BufferView, D> where Rc: BaseRenderingContext, D: Borrow<[u8]> + Send + Sync {
    fn execute_internal(&self, rendering_context: &mut Rc) -> Result<(), ContextError> {
        rendering_context.buffer_upload(self.buffer_view, self.data.borrow())
    }
}

impl<Rc, D> BufferUploadCommand<Rc, BufferSlice<Rc, Rc::BufferView>, D> where Rc: BaseRenderingContext, D: Borrow<[u8]> + Send + Sync {
    fn execute_internal(&self, rendering_context: &mut Rc) -> Result<(), ContextError> {
        rendering_context.buffer_upload(self.buffer_view.buffer, self.data.borrow())
    }
}

impl<Rc, D> GpuCommand<Rc> for BufferUploadCommand<Rc, Rc::BufferView, D> where Rc: BaseRenderingContext, D: Borrow<[u8]> + Send + Sync {
    type Output = ();

    type Error = ContextError;

    fn execute_static(self, rendering_context: &mut Rc) -> Result<Self::Output, Self::Error> {
        self.execute_internal(rendering_context)
    }

    fn execute_dynamic(self: Box<Self>, rendering_context: &mut Rc) -> Result<Self::Output, Self::Error> {
        self.execute_internal(rendering_context)
    }
}

impl<Rc, D> GpuCommand<Rc> for BufferUploadCommand<Rc, BufferSlice<Rc, Rc::BufferView>, D> where Rc: BaseRenderingContext, D: Borrow<[u8]> + Send + Sync {
    type Output = ();

    type Error = ContextError;

    fn execute_static(self, rendering_context: &mut Rc) -> Result<Self::Output, Self::Error> {
        self.execute_internal(rendering_context)
    }

    fn execute_dynamic(self: Box<Self>, rendering_context: &mut Rc) -> Result<Self::Output, Self::Error> {
        self.execute_internal(rendering_context)
    }
}
