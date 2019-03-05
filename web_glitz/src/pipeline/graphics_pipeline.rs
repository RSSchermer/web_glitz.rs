
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

impl<V, R, F> ActiveGraphicsPipeline<V, R, F> where V: VertexInputLayout, R: ResourceBindings {
    pub fn draw_command<S>(&self, vertex_input_stream: S, resources: R) -> DrawCommand<R::Encoding> where S: VertexInputStreamDescription<Layout=V> {
        unimplemented!()
    }
}

pub unsafe trait VertexInputLayout {
    fn compatibility(descriptors: &[VertexInputAttributeDescriptor]) -> Compatibility;
}

pub enum ConfirmSlotBindings {
    Check,
    Update
}
