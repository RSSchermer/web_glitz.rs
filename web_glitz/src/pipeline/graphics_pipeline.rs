
use js_sys::{Uint32Array, Uint8Array};
use crate::pipeline::interface_block::MemoryUnitDescriptor;
use crate::pipeline::interface_block::InterfaceBlock;
use std::sync::Arc;
use crate::buffer::BufferView;
use std::marker;
use crate::vertex_input::VertexInputAttributeDescriptor;
use crate::util::JsId;
use crate::buffer::Buffer;
use crate::pipeline::interface_block::MatrixOrder;
use crate::pipeline::interface_block;

pub unsafe trait IndexFormat {
    fn id() -> u32;
}

unsafe impl IndexFormat for u8 {
    fn id() -> u32 {
        Gl::UNSIGNED_BYTE
    }
}

unsafe impl IndexFormat for u16 {
    fn id() -> u32 {
        Gl::UNSIGNED_SHORT
    }
}

unsafe impl IndexFormat for u32 {
    fn id() -> u32 {
        Gl::UNSIGNED_INT
    }
}

pub struct VertexArray<V, I> {
    id: Arc<Option<JsId>>,
    len: usize,
    _vertex_layout_marker: marker::PhantomData<V>,
    _indices_marker: marker::PhantomData<Buffer<I>>
}

impl<V, I> VertexArray<V, I> {
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn range<R>(&self, range: R) -> Option<VertexArraySlice<V, I>> where R: VertexArrayRange {
        rante.range(VertexArraySlice {
            vertex_array: self,
            offset: 0,
            len: self.len,
        })
    }

    pub unsafe fn range_unchecked<R>(&self, range: R) -> Option<VertexArraySlice<V, I>> where R: VertexArrayRange {
        rante.range_unchecked(VertexArraySlice {
            vertex_array: self,
            offset: 0,
            len: self.len,
        })
    }

    pub fn instanced(&self, instance_count: usize) -> Instanced<VertexArraySlice<V, I>> {
        Instanced {
            vertex_array: VertexArraySlice {
                vertex_array: self,
                offset: 0,
                len: self.len,
            }
        }
    }
}

pub trait VertexArrayRange {
    fn range<'a, V, I>(self, vertex_array: VertexArraySlice<I, V>) -> Option<VertexArraySlice<V, I>>;

    unsafe fn range_unchecked<'a, V, I>(self, vertex_array: VertexArraySlice<I, V>) -> VertexArraySlice<V, I>;
}

#[derive(Clone, Copy)]
pub struct VertexArraySlice<'a, V, I> {
    vertex_array: &'a VertexArray<V, I>,
    offset: usize,
    len: usize,
}

impl<V, I> VertexArraySlice<V, I> {
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn range<R>(&self, range: R) -> Option<VertexArraySlice<V, I>> where R: VertexArrayRange {
        range.range(self)
    }

    pub unsafe fn range_unchecked<R>(&self, range: R) -> Option<VertexArraySlice<V, I>> where R: VertexArrayRange {
        range.range_unchecked(self)
    }

    pub fn instanced(&self, instance_count: usize) -> InstancedVertexArraySlice<V, I> {
        InstancedVertexArraySlice {
            vertex_array: self,
            offset: self.offset,
            len: self.len,
            instance_count: usize
        }
    }
}

#[derive(Clone, Copy)]
pub struct InstancedVertexArraySlice<'a, V, I> {
    vertex_array: &'a VertexArray<V, I>,
    offset: usize,
    len: usize,
    instance_count: usize,
}

pub trait VertexInputStreamDescription<V, I> {
    fn descriptor(&self) -> VertexInputStreamDescriptor;
}

pub struct VertexInputStreamDescriptor {
    vertex_array_id: Arc<Option<JsId>>,
    offset: usize,
    count: usize,
    instance_count: usize
}

impl<V, I> VertexInputStreamDescription<V, I> for VertexArray<V, I> {
    fn descriptor(&self) -> VertexInputStreamDescriptor {
        VertexInputStreamDescriptor {
            vertex_array_id: self.id.clone(),
            offset: 0,
            count: self.len,
            instance_count: 1
        }
    }
}

impl<'a, V, I> VertexInputStreamDescription<V, I> for VertexArraySlice<'a, V, I> {
    fn descriptor(&self) -> VertexInputStreamDescriptor {
        VertexInputStreamDescriptor {
            vertex_array_id: self.id.clone(),
            offset: self.offset,
            count: self.len,
            instance_count: 1
        }
    }
}

impl<'a, V, I> VertexInputStreamDescription<V, I> for InstancedVertexArraySlice<'a, V, I> {
    fn descriptor(&self) -> VertexInputStreamDescriptor {
        VertexInputStreamDescriptor {
            vertex_array_id: self.id.clone(),
            offset: self.offset,
            count: self.len,
            instance_count: self.instance_count
        }
    }
}

pub struct DrawCommand<R> {
    pipeline_data: Arc<GraphicsPipelineData>,
    vertex_stream_data: VertexArrayData,
    element_offset: usize,
    element_count: usize,
    instance_count: usize,
    resources: R
}

pub struct GraphicsPipeline<V, R, F> {
    data: Arc<GraphicsPipelineData>,
    _vertex_input_marker: marker::PhantomData<V>,
    _resource_layout: marker::PhantomData<R>
}

pub struct ActiveGraphicsPipeline<V, R, F> {
    framebuffer_size: (u32, u32),
    data: Arc<GraphicsPipelineData>,
    _vertex_input_marker: marker::PhantomData<V>,
    _resource_layout: marker::PhantomData<R>
}

impl<V, R, F> ActiveGraphicsPipeline<V, R, F> where V: StaticVertexInputLayout, R: ResourceBindings {
    pub fn draw_command<S, I>(&self, vertex_input_stream: S, resources: R) -> DrawCommand<R::Encoding> where S: VertexInputStreamDescription, I: IndexFormat {
        unimplemented!()
    }
}

pub unsafe trait StaticVertexInputLayout {
    fn attribute_descriptors() -> &'static [VertexInputAttributeDescriptor];
}



pub enum ConfirmSlotBindings {
    Check,
    Update
}
