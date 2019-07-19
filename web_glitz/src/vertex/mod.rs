mod vertex_input_state_description;
pub use self::vertex_input_state_description::{
    InputRate, VertexAttributeDescriptor, VertexBufferDescriptor, VertexBuffersDescription,
};

mod index_buffer_description;
pub use self::index_buffer_description::{
    IndexBufferDescription, IndexBufferDescriptor, IndexFormat, IndexType,
};

mod vertex_attribute_layout;
pub use self::vertex_attribute_layout::{
    BindSlotRef, TypedVertexAttributeLayout, VertexAttributeLayoutDescriptor,
};

use std::borrow::Borrow;

pub mod attribute_format;

/// Trait implemented for types that provide attribute data for a [VertexArray].
///
/// [Buffer]s that contain an array of a type that implements this trait can act as vertex input
/// state descriptions for [VertexArray]s, see [RenderingContext::create_vertex_array] and
/// [VertexArrayDescriptor].
///
/// # Unsafe
///
/// If this trait is implemented for a type, then the [attribute_descriptors] must return a set
/// of [VertexAttributeDescriptor]s that can be validly applied to any instance of the type: for
/// every [VertexAttributeDescriptor] there must be data that can be interpreted the
/// [VertexAttributeDescriptor::format] at the [VertexAttributeDescriptor::offset_in_bytes] relative
/// to start of the instance's memory.
///
/// # Deriving
///
/// This trait may be automatically derived safely for struct types. Any field that defines an
/// attribute must be marked with `#[vertex_attribute(...)]`, other fields will be ignored. A vertex
/// attribute has to declare a `location` and a `format`. The `location` defines the location of the
/// attribute slot that will be bound to in a graphics pipeline. The `format` must be the name of
/// one of the [AttributeFormatIdentifier] types defined in the [attribute_format] module:
///
/// ```rust
/// # #![feature(const_fn)]
/// #[derive(web_glitz::derive::Vertex, Clone, Copy)]
/// struct Vertex {
///     #[vertex_attribute(location = 0, format = "Float2_f32")]
///     position: [f32; 2],
///
///     #[vertex_attribute(location = 1, format = "Float3_u8_norm")]
///     color: [u8; 3],
/// }
/// ```
///
/// The type of a struct field marked with `#[vertex_attribute(...)]` must be [FormatCompatible]
/// with the [AttributeFormatIdentifier] `format` declared for the attribute, otherwise the struct
/// will fail to compile. For example, in the example above `[f32; 2]` must implement
/// `FormatCompatible<Float2_f32>` (which it does) and `[u8;3 ]` must implement
/// `FormatCompatible<Float3_u8_norm>` (which it does).
///
/// Note that in this example we also derive `Clone` and `Copy`. This is not strictly required to
/// derive the [Vertex] trait, however, a [Buffer] can only store an array of a type that implements
/// the `Copy` trait. Therefor if we intend to create [Buffer] with our [Vertex] type, then we must
/// also derive `Copy`. As `Clone` is a supertrait of `Copy`, we must also derive `Clone`.
pub unsafe trait Vertex: Sized {
    const INPUT_RATE: InputRate = InputRate::PerVertex;

    /// A set of [VertexAttributeDescriptor]s that describe how attribute data for this type is to
    /// be bound to the attribute slots of a graphics pipeline.
    const ATTRIBUTE_DESCRIPTORS: &'static [VertexAttributeDescriptor];
}
