mod vertex;
pub use self::vertex::{FormatCompatible, Vertex};

mod vertex_buffer_descriptor;
pub use self::vertex_buffer_descriptor::{
    FormatKind, InputRate, VertexBufferDescription, VertexBufferDescriptor,
    VertexInputAttributeDescriptor,
};

mod input_attribute_layout;
pub use self::input_attribute_layout::{
    AttributeSlotDescriptor, AttributeType, Incompatible, InputAttributeLayout,
};

pub(crate) mod vertex_array;
pub use self::vertex_array::{
    IndexBufferDescription, IndexBufferDescriptor, IndexFormat, Instanced, VertexArray,
    VertexArraySlice, VertexBuffersDescription, VertexBuffersDescriptor,
    VertexInputStreamDescription, VertexInputStreamDescriptor,
};

pub mod attribute_format;
