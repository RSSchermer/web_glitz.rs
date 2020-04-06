pub(crate) mod vertex_buffers;
pub use self::vertex_buffers::{
    TypedVertexBuffer, TypedVertexBuffers, VertexBuffer, VertexBuffers, VertexBuffersEncoding,
    VertexBuffersEncodingContext,
};

pub(crate) mod index_buffer;
pub use self::index_buffer::{
    IndexBuffer, IndexBufferSliceRange, IndexBufferView, IndexBufferViewSliceIndex, IndexData,
    IndexDataDescriptor, IndexFormat, IndexType,
};

pub(crate) mod layout_descriptor;
pub use self::layout_descriptor::{
    IncompatibleVertexInputLayout, InputRate, TypedVertexInputLayout, VertexAttributeDescriptor,
    VertexAttributeType, VertexBufferSlotAttributeAttacher, VertexBufferSlotRef,
    VertexInputLayoutAllocationHint, VertexInputLayoutDescriptor,
    VertexInputLayoutDescriptorBuilder,
};

pub mod attribute_format;

/// Trait implemented for types that provide attribute data for a vertex buffer.
///
/// [Buffer]s that contain an array of a type that implements this trait can act as vertex buffers
/// for a graphics pipeline without additional runtime checks, see also [TypedVertexBuffers].
///
/// # Unsafe
///
/// If this trait is implemented for a type, then the [ATTRIBUTE_DESCRIPTORS] must be a set
/// of [VertexAttributeDescriptor]s that can be validly applied to any instance of the type: for
/// every [VertexAttributeDescriptor] there must be data that can be interpreted as the
/// [VertexAttributeDescriptor::format] at the [VertexAttributeDescriptor::offset_in_bytes] relative
/// to start of the instance's memory.
///
/// # Deriving
///
/// This trait may be automatically derived safely for struct types. Any field that defines an
/// attribute must be marked with `#[vertex_attribute(...)]`, other fields will be ignored. A vertex
/// attribute has to declare a `location` and a `format`. The `location` defines the location of the
/// attribute slot that will be bound to in a graphics pipeline. The `format` must be the name of
/// one of the [VertexAttributeFormatIdentifier] types defined in the [attribute_format] module:
///
/// ```rust
/// # #![feature(const_fn, const_transmute, const_ptr_offset_from, ptr_offset_from)]
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
/// The type of a struct field marked with `#[vertex_attribute(...)]` must be
/// [VertexAttributeFormatCompatible] with the [AttributeFormatIdentifier] `format` declared for the
/// attribute, otherwise the struct will fail to compile. For example, in the example above
/// `[f32; 2]` must implement `VertexAttributeFormatCompatible<Float2_f32>` (which it does) and
/// `[u8;3 ]` must implement `VertexAttributeFormatCompatible<Float3_u8_norm>` (which it does).
///
/// Note that in this example we also derive `Clone` and `Copy`. This is not strictly required to
/// derive the [Vertex] trait, however, a [Buffer] can only store an array of a type that implements
/// the `Copy` trait. Therefor if we intend to create [Buffer] with our [Vertex] type, then we must
/// derive `Copy`. As `Clone` is a supertrait of `Copy`, we must also derive `Clone`.
pub unsafe trait Vertex: Sized {
    const INPUT_RATE: InputRate = InputRate::PerVertex;

    /// A set of [VertexAttributeDescriptor]s that describe how attribute data for this type is to
    /// be bound to the attribute slots of a graphics pipeline.
    const ATTRIBUTE_DESCRIPTORS: &'static [VertexAttributeDescriptor];
}
