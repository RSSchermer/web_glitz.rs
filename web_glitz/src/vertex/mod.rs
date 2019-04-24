mod vertex_input_state_description;
pub use self::vertex_input_state_description::{
    PerInstance, VertexAttributeDescriptor, VertexInputDescriptor, VertexInputStateDescription,
};

mod index_buffer_description;
pub use self::index_buffer_description::{
    IndexBufferDescription, IndexBufferDescriptor, IndexFormat, IndexType,
};

mod vertex_array;
pub use self::vertex_array::{
    Instanced, VertexArray, VertexArrayDescriptor, VertexArrayRange, VertexArraySlice,
};

mod vertex_attribute_layout;
pub use self::vertex_attribute_layout::VertexAttributeLayout;

mod vertex_stream_description;
pub use self::vertex_stream_description::{VertexStreamDescription, VertexStreamDescriptor};

pub mod attribute_format;

pub unsafe trait Vertex: Sized {
    fn attribute_descriptors() -> &'static [VertexAttributeDescriptor];
}
