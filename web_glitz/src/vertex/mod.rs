mod vertex_input_state_description;
pub use self::vertex_input_state_description::{VertexInputStateDescription, VertexInputDescriptor, VertexAttributeDescriptor, PerInstance};

mod index_buffer_description;
pub use self::index_buffer_description::{IndexBufferDescription, IndexBufferDescriptor, IndexFormat, IndexType};

mod vertex_array;
pub use self::vertex_array::{VertexArray, VertexArrayDescriptor, VertexArrayRange, VertexArraySlice, Instanced};

mod vertex_attribute_layout;
pub use self::vertex_attribute_layout::VertexAttributeLayout;

mod vertex_stream_description;
pub use self::vertex_stream_description::{VertexStreamDescription, VertexStreamDescriptor};

pub mod attribute_format;

pub unsafe trait Vertex: Sized {
    fn attribute_descriptors() -> &'static [VertexAttributeDescriptor];
}
