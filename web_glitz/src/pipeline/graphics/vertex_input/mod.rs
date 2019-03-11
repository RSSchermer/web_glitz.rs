mod vertex;
pub use self::vertex::{Vertex, VertexAttribute};

mod vertex_buffer_descriptor;
pub use self::vertex_buffer_descriptor::{
    FormatKind, InputRate, VertexBufferDescription, VertexInputAttributeDescriptor,
};

mod input_attribute_layout;
pub use self::input_attribute_layout::{
    AttributeSlotDescriptor, AttributeType, Incompatible, InputAttributeLayout,
};

mod vertex_array;
pub use self::vertex_array::{
    IndexBufferDescription, IndexBufferDescriptor, IndexFormat, InstancedVertexArraySlice,
    VertexArray, VertexArraySlice, VertexBuffersDescription, VertexBuffersDescriptor,
    VertexInputStreamDescription, VertexInputStreamDescriptor,
};

pub mod attribute_format;
