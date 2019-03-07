mod vertex;
pub use self::vertex::{Vertex, VertexAttribute};

mod vertex_buffer_descriptor;
pub use self::vertex_buffer_descriptor::{
    AttributeFormat, InputRate, VertexBufferDescription, VertexInputAttributeDescriptor,
};

mod input_attribute_layout;
mod vertex_array;