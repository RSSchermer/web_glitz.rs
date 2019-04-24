use std::borrow::Borrow;
use std::mem;
use std::sync::Arc;

use web_sys::WebGl2RenderingContext as Gl;

use crate::buffer::{Buffer, BufferData, BufferView};
use crate::vertex::attribute_format::AttributeFormat;
use crate::vertex::{Vertex, VertexAttributeLayout};

/// Describes the attribute layout and input data sources for a [VertexArray].
///
/// The [AttributeLayout] describes how attribute values are to be constructed from the
/// [vertex_input_descriptors]. The groups of [VertexAttributeDescriptor]s in the [AttributeLayout]
/// are applied to the [vertex_input_descriptors] in order: the first group of
/// [VertexAttributeDescriptor]s is applied to the first [VertexInputDescriptor], the second group
/// of [VertexAttributeDescriptor]s is applied to the second [VertexInputDescriptor], etc.
///
/// # Unsafe
///
/// It must be valid to apply a group of [VertexAttributeDescriptor]s to the [VertexInputDescriptor]
/// it is paired with: if a group contains a [VertexAttributeDescriptor] describes a certain
/// [AttributeFormat] (see [VertexAttributeDescriptor::format]) at a certain offset (see
/// [VertexAttributeDescriptor::offset_in_bytes]), then the [Buffer] region described by the
/// [VertexInputDescriptor] must contain data that can be validly interpreted as that format
/// starting at that offset, and at every multiple of [VertexInputDescriptor::stride_in_bytes] bytes
/// added to that offset for the entire size of the input region (see
/// [VertexInputDescriptor::size_in_bytes]).
///
/// # Example
///
/// ```
/// use web_glitz::vertex::{
///     VertexAttributeDescriptor, VertexAttributeLayout, VertexInputDescriptor,
///     VertexInputStateDescription
/// };
/// use web_glitz::vertex::attribute_format::AttributeFormat;
/// use web_glitz::buffer::Buffer;
///
/// struct CustomAttributeLayout;
///
/// unsafe impl VertexAttributeLayout for CustomAttributeLayout {
///     type InputAttributeBindings = [&'static [VertexAttributeDescriptor]; 2];
///
///     fn input_attribute_bindings() -> Self::InputAttributeBindings {
///         const PER_VERTEX_ATTRIBUTES: [VertexAttributeDescriptor; 2] = [
///             VertexAttributeDescriptor {
///                 location: 0,
///                 offset_in_bytes: 0,
///                 format: AttributeFormat::Float4_f32
///             },
///             VertexAttributeDescriptor {
///                 location: 1,
///                 offset_in_bytes: 4,
///                 format: AttributeFormat::Float3_f32
///             }
///         ];
///
///         const PER_INSTANCE_ATTRIBUTES: [VertexAttributeDescriptor; 1] = [
///             VertexAttributeDescriptor {
///                 location: 2,
///                 offset_in_bytes: 0,
///                 format: AttributeFormat::Float4_f32
///             },
///         ];
///
///         [&PER_VERTEX_ATTRIBUTES, &PER_INSTANCE_ATTRIBUTES]
///     }
/// }
///
/// struct CustomVertexInput {
///     per_vertex_buffer: Buffer<[(f32, f32, f32, f32, f32, f32, f32)]>,
///     per_instance_buffer: Buffer<[(f32, f32, f32, f32)]>
/// }
///
/// unsafe impl VertexInputStateDescription for CustomVertexInput {
///     type AttributeLayout = CustomAttributeLayout;
///
///     type InputDescriptors = [VertexInputDescriptor; 2];
///
///     fn vertex_input_descriptors(&self) -> Self::InputDescriptors {
///         [
///             VertexInputDescriptor::from_buffer_view(
///                 self.per_vertex_buffer.view(),
///                 InputRate::PerVertex
///             ),
///             VertexInputDescriptor::from_buffer_view(
///                 self.per_instance_buffer.view(),
///                 InputRate::PerInstance
///             ),
///         ]
///     }
/// }
/// ```
pub unsafe trait VertexInputStateDescription {
    /// The type that defines the attribute layout for this [VertexInputStateDescription].
    ///
    /// If this [VertexInputStateDescription] is used to create a [VertexArray], then this type will
    /// also be used at the element type marker for the [VertexArray].
    type AttributeLayout: VertexAttributeLayout;

    /// Type returned by [vertex_input_descriptors].
    ///
    /// Typically an array:
    ///
    /// ```
    /// # use web_glitz::vertex::VertexInputDescriptor;
    /// type InputDescriptors = [VertexInputDescriptor; 3];
    /// ```
    type InputDescriptors: Borrow<[VertexInputDescriptor]> + 'static;

    /// Returns the [VertexInputDescriptor]s that describe the data input sources for the attribute
    /// data.
    fn vertex_input_descriptors(&self) -> Self::InputDescriptors;
}

/// Describes the input rate for a [VertexInputDescriptor] when it is used as a data source for
/// a vertex stream.
///
/// See also [VertexStreamDescription] and [VertexStreamDescriptor].
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum InputRate {
    /// Any attribute defined on the [VertexInputDescriptor] will advance to its next value for
    /// every vertex in a vertex stream; the attribute resets to its first value for every instance
    /// in a vertex stream that defines more than 1 instance.
    PerVertex,

    /// Any attribute defined on the [VertexInputDescriptor] will advance to its next value for
    /// every instance in a vertex stream that defines more than 1 instance; all vertices for the
    /// instance will use the same attribute value.
    PerInstance,
}

/// Describes an input source for vertex attribute data.
#[derive(Clone)]
pub struct VertexInputDescriptor {
    pub(crate) buffer_data: Arc<BufferData>,
    pub(crate) stride_in_bytes: u8,
    pub(crate) offset_in_bytes: u32,
    pub(crate) size_in_bytes: u32,
    pub(crate) input_rate: InputRate,
}

impl VertexInputDescriptor {
    /// Creates a [VertexInputDescriptor] from a [BufferView] of a typed slice, with the given
    /// [InputRate].
    ///
    /// The [offset_in_bytes] of the [VertexInputDescriptor] is the offset in bytes of the [Buffer]
    /// region viewed by the [BufferView] relative to the start of the buffer. The [size_in_bytes]
    /// of the [VertexInputDescriptor] is the size in bytes of the buffer region viewed by the
    /// [BufferView]. The [stride_in_bytes] of the [VertexInputDescriptor] is
    /// `std::mem::size_of::<T>`, where `T` is the element type of the slice viewed by the
    /// [BufferView].
    pub fn from_buffer_view<T>(buffer_view: BufferView<[T]>, input_rate: InputRate) -> Self {
        VertexInputDescriptor {
            buffer_data: buffer_view.buffer_data().clone(),
            stride_in_bytes: mem::size_of::<T>() as u8,
            offset_in_bytes: buffer_view.offset_in_bytes() as u32,
            size_in_bytes: (mem::size_of::<T>() * buffer_view.len()) as u32,
            input_rate,
        }
    }

    /// The stride in bytes between different values in the value sequence for an attribute defined
    /// on this [VertexInputDescriptor].
    ///
    /// If a [VertexAttributeDescriptor] is applied to this [VertexInputDescriptor] (see also
    /// [VertexInputStateDescription), then the byte-sequence for the first value for that attribute
    /// will start at [VertexAttributeDescriptor::offset_in_bytes] bytes relative to the start of
    /// memory region described by the [VertexInputDescriptor] (see [offset_in_bytes]); subsequent
    /// values in the attribute sequence are obtained by repeatedly adding the stride to this
    /// offset.
    ///
    /// For example, given a [VertexAttributeDescriptor] with an offset in bytes of `offset`, and
    /// a [VertexInputDescriptor] with a stride in bytes of `stride`, then the first attribute value
    /// starts at `offset` bytes, the second attribute value starts at `offset + stride`, the third
    /// attribute value start at `offset + 2 * stride`, ..., the `n`th attribute value start at
    /// `offset + n * stride`. The sequence of attribute values ends when `offset + n * stride` is
    /// greater than the [size_in_bytes] defined for the [VertexInputDescriptor].
    pub fn stride_in_bytes(&self) -> u8 {
        self.stride_in_bytes
    }

    /// The offset in bytes of the memory region described by this [VertexInputDescriptor], relative
    /// to the start of the [Buffer] it is defined on.
    pub fn offset_in_bytes(&self) -> u32 {
        self.offset_in_bytes
    }

    /// The size in bytes of the memory region described by this [VertexInputDescriptor].
    pub fn size_in_bytes(&self) -> u32 {
        self.size_in_bytes
    }

    /// Whether the attributes defined on this [VertexInputDescriptor] will be "per vertex"
    /// attributes or "per instance" attributes.
    ///
    /// See also [InputRate].
    pub fn input_rate(&self) -> InputRate {
        self.input_rate
    }
}

unsafe impl<'a, T> VertexInputStateDescription for &'a Buffer<[T]>
where
    T: Vertex,
{
    type AttributeLayout = T;

    type InputDescriptors = [VertexInputDescriptor; 1];

    fn vertex_input_descriptors(&self) -> Self::InputDescriptors {
        [VertexInputDescriptor::from_buffer_view(
            self.view(),
            InputRate::PerVertex,
        )]
    }
}

unsafe impl<'a, T> VertexInputStateDescription for BufferView<'a, [T]>
where
    T: Vertex,
{
    type AttributeLayout = T;

    type InputDescriptors = [VertexInputDescriptor; 1];

    fn vertex_input_descriptors(&self) -> Self::InputDescriptors {
        [VertexInputDescriptor::from_buffer_view(
            self.clone(),
            InputRate::PerVertex,
        )]
    }
}

/// Marker type that can be used to indicate that a vertex [Buffer] reference or vertex [BufferView]
/// should be used as a "per instance" [VertexInputDescriptor].
///
/// See the documentation for [VertexArray] for an example.
pub struct PerInstance<T>(pub T);

unsafe impl<'a, T> VertexInputStateDescription for PerInstance<&'a Buffer<[T]>>
where
    T: Vertex,
{
    type AttributeLayout = T;

    type InputDescriptors = [VertexInputDescriptor; 1];

    fn vertex_input_descriptors(&self) -> Self::InputDescriptors {
        [VertexInputDescriptor::from_buffer_view(
            self.0.view(),
            InputRate::PerInstance,
        )]
    }
}

unsafe impl<'a, T> VertexInputStateDescription for PerInstance<BufferView<'a, [T]>>
where
    T: Vertex,
{
    type AttributeLayout = T;

    type InputDescriptors = [VertexInputDescriptor; 1];

    fn vertex_input_descriptors(&self) -> Self::InputDescriptors {
        [VertexInputDescriptor::from_buffer_view(
            self.0.clone(),
            InputRate::PerInstance,
        )]
    }
}

macro_rules! impl_vertex_input_state_description {
    ($($T:ident),*) => {
        unsafe impl<$($T),*> VertexInputStateDescription for ($($T),*)
            where
                $($T: VertexInputStateDescription, $T::AttributeLayout: Vertex),*
        {
            type AttributeLayout = ($($T::AttributeLayout),*);

            type InputDescriptors = Vec<VertexInputDescriptor>;

            #[allow(unused_assignments)]
            fn vertex_input_descriptors(&self) -> Self::InputDescriptors {
                let mut vec = Vec::new();

                #[allow(unused_parens, non_snake_case)]
                let ($($T),*) = self;

                $(
                    for descriptor in $T.vertex_input_descriptors().borrow().iter() {
                        vec.push(descriptor.clone());
                    }
                )*

                vec
            }
        }
    }
}

impl_vertex_input_state_description!(T0, T1);
impl_vertex_input_state_description!(T0, T1, T2);
impl_vertex_input_state_description!(T0, T1, T2, T3);
impl_vertex_input_state_description!(T0, T1, T2, T3, T4);
impl_vertex_input_state_description!(T0, T1, T2, T3, T4, T5);
impl_vertex_input_state_description!(T0, T1, T2, T3, T4, T5, T6);
impl_vertex_input_state_description!(T0, T1, T2, T3, T4, T5, T6, T7);
impl_vertex_input_state_description!(T0, T1, T2, T3, T4, T5, T6, T7, T8);
impl_vertex_input_state_description!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9);
impl_vertex_input_state_description!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
impl_vertex_input_state_description!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);
impl_vertex_input_state_description!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12);
impl_vertex_input_state_description!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13);
impl_vertex_input_state_description!(
    T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14
);
impl_vertex_input_state_description!(
    T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15
);

/// Describes how the data for an `in` attribute in a [VertexShader] is sourced from a
/// [VertexInputDescriptor].
///
/// See also [VertexInputStateDescription].
#[derive(Clone, PartialEq, Debug)]
pub struct VertexAttributeDescriptor {
    /// The shader location of the attribute described by this [VertexAttributeDescriptor].
    ///
    /// For example, if the location is `1`, then this [VertexAttributeDescriptor] describes how
    /// data is sourced for values of the attribute at shader attribute location `1`. In GLSL, the
    /// location for an attribute may be specified with the `layout` qualifier:
    ///
    /// ```glsl
    /// layout(location=1) in vec4 position;
    /// ```
    pub location: u32,

    /// The offset in bytes of the first value in the attribute value sequence relative to the start
    /// of a [VertexInputDescriptor].
    ///
    /// The byte-sequence for the first attribute value begins at this offset. Subsequent attribute
    /// values are obtained by adding a stride to the base offset, see
    /// [VertexInputDescriptor::stride_in_bytes].
    pub offset_in_bytes: u8,

    /// The data format in which the attribute values are stored.
    ///
    /// Should be a format that is compatible with the type used for the attribute in the shader,
    /// see also [AttributeFormat::is_compatible].
    pub format: AttributeFormat,
}

impl VertexAttributeDescriptor {
    pub(crate) fn apply(
        &self,
        gl: &Gl,
        stride_in_bytes: i32,
        base_offset_in_bytes: i32,
        input_rate: InputRate,
    ) {
        match self.format {
            AttributeFormat::Float_f32 => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    1,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Float_i8_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    1,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Float_i8_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    1,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);
            }
            AttributeFormat::Float_i16_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    1,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Float_i16_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    1,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Float_u8_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    1,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Float_u8_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    1,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Float_u16_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    1,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Float_u16_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    1,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Float2_f32 => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Float2_i8_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Float2_i8_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Float2_i16_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Float2_i16_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Float2_u8_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Float2_u8_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Float2_u16_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Float2_u16_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Float3_f32 => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Float3_i8_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Float3_i8_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Float3_i16_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Float3_i16_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Float3_u8_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Float3_u8_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Float3_u16_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Float3_u16_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Float4_f32 => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Float4_i8_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Float4_i8_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Float4_i16_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Float4_i16_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Float4_u8_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Float4_u8_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Float4_u16_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Float4_u16_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Float2x2_f32 => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 4 * 2,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                }
            }
            AttributeFormat::Float2x2_i8_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 2,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                }
            }
            AttributeFormat::Float2x2_i8_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 2,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                }
            }
            AttributeFormat::Float2x2_i16_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 2,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                }
            }
            AttributeFormat::Float2x2_i16_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 2,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                }
            }
            AttributeFormat::Float2x2_u8_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 2,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                }
            }
            AttributeFormat::Float2x2_u8_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 2,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                }
            }
            AttributeFormat::Float2x2_u16_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 2,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                }
            }
            AttributeFormat::Float2x2_u16_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 2,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                }
            }
            AttributeFormat::Float2x3_f32 => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 4 * 3,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                }
            }
            AttributeFormat::Float2x3_i8_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 3,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                }
            }
            AttributeFormat::Float2x3_i8_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 3,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                }
            }
            AttributeFormat::Float2x3_i16_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 3,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                }
            }
            AttributeFormat::Float2x3_i16_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 3,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                }
            }
            AttributeFormat::Float2x3_u8_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 3,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                }
            }
            AttributeFormat::Float2x3_u8_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 3,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                }
            }
            AttributeFormat::Float2x3_u16_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 3,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                }
            }
            AttributeFormat::Float2x3_u16_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 3,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                }
            }
            AttributeFormat::Float2x4_f32 => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 4 * 4,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                }
            }
            AttributeFormat::Float2x4_i8_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 4,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                }
            }
            AttributeFormat::Float2x4_i8_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 4,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                }
            }
            AttributeFormat::Float2x4_i16_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 4,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                }
            }
            AttributeFormat::Float2x4_i16_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 4,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                }
            }
            AttributeFormat::Float2x4_u8_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 4,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                }
            }
            AttributeFormat::Float2x4_u8_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 4,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                }
            }
            AttributeFormat::Float2x4_u16_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 4,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                }
            }
            AttributeFormat::Float2x4_u16_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 4,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                }
            }
            AttributeFormat::Float3x2_f32 => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 4 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    2,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 4 * 2 * 2,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                }
            }
            AttributeFormat::Float3x2_i8_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    2,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 2 * 2,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                }
            }
            AttributeFormat::Float3x2_i8_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    2,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 2 * 2,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                }
            }
            AttributeFormat::Float3x2_i16_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    2,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 2 * 2,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                }
            }
            AttributeFormat::Float3x2_i16_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    2,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 2 * 2,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                }
            }
            AttributeFormat::Float3x2_u8_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    2,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 2 * 2,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                }
            }
            AttributeFormat::Float3x2_u8_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    2,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 2 * 2,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                }
            }
            AttributeFormat::Float3x2_u16_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    2,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 2 * 2,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                }
            }
            AttributeFormat::Float3x2_u16_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    2,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 2 * 2,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                }
            }
            AttributeFormat::Float3x3_f32 => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 4 * 3,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    3,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 4 * 3 * 2,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                }
            }
            AttributeFormat::Float3x3_i8_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 3,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    3,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 3 * 2,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                }
            }
            AttributeFormat::Float3x3_i8_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 3,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    3,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 3 * 2,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                }
            }
            AttributeFormat::Float3x3_i16_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 3,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    3,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 3 * 2,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                }
            }
            AttributeFormat::Float3x3_i16_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 3,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    3,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 3 * 2,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                }
            }
            AttributeFormat::Float3x3_u8_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 3,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    3,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 3 * 2,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                }
            }
            AttributeFormat::Float3x3_u8_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 3,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    3,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 3 * 2,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                }
            }
            AttributeFormat::Float3x3_u16_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 3,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    3,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 3 * 2,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                }
            }
            AttributeFormat::Float3x3_u16_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 3,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    3,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 3 * 2,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                }
            }
            AttributeFormat::Float3x4_f32 => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 4 * 4,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    4,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 4 * 4 * 2,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                }
            }
            AttributeFormat::Float3x4_i8_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 4,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    4,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 4 * 2,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                }
            }
            AttributeFormat::Float3x4_i8_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 4,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    4,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 4 * 2,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                }
            }
            AttributeFormat::Float3x4_i16_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 4,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    4,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 4 * 2,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                }
            }
            AttributeFormat::Float3x4_i16_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 4,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    4,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 4 * 2,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                }
            }
            AttributeFormat::Float3x4_u8_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 4,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    4,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 4 * 2,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                }
            }
            AttributeFormat::Float3x4_u8_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 4,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    4,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 4 * 2,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                }
            }
            AttributeFormat::Float3x4_u16_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 4,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    4,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 4 * 2,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                }
            }
            AttributeFormat::Float3x4_u16_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 4,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    4,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 4 * 2,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                }
            }
            AttributeFormat::Float4x2_f32 => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 4 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    2,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 4 * 2 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    2,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 4 * 2 * 3,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
                gl.enable_vertex_attrib_array(self.location + 3);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                    gl.vertex_attrib_divisor(self.location + 3, 1);
                }
            }
            AttributeFormat::Float4x2_i8_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    2,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 2 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    2,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 2 * 3,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
                gl.enable_vertex_attrib_array(self.location + 3);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                    gl.vertex_attrib_divisor(self.location + 3, 1);
                }
            }
            AttributeFormat::Float4x2_i8_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    2,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 2 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    2,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 2 * 3,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
                gl.enable_vertex_attrib_array(self.location + 3);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                    gl.vertex_attrib_divisor(self.location + 3, 1);
                }
            }
            AttributeFormat::Float4x2_i16_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    2,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 2 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    2,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 2 * 3,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
                gl.enable_vertex_attrib_array(self.location + 3);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                    gl.vertex_attrib_divisor(self.location + 3, 1);
                }
            }
            AttributeFormat::Float4x2_i16_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    2,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 2 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    2,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 2 * 3,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
                gl.enable_vertex_attrib_array(self.location + 3);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                    gl.vertex_attrib_divisor(self.location + 3, 1);
                }
            }
            AttributeFormat::Float4x2_u8_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    2,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 2 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    2,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 2 * 3,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
                gl.enable_vertex_attrib_array(self.location + 3);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                    gl.vertex_attrib_divisor(self.location + 3, 1);
                }
            }
            AttributeFormat::Float4x2_u8_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    2,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 2 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    2,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 2 * 3,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
                gl.enable_vertex_attrib_array(self.location + 3);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                    gl.vertex_attrib_divisor(self.location + 3, 1);
                }
            }
            AttributeFormat::Float4x2_u16_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    2,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 2 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    2,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 2 * 3,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
                gl.enable_vertex_attrib_array(self.location + 3);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                    gl.vertex_attrib_divisor(self.location + 3, 1);
                }
            }
            AttributeFormat::Float4x2_u16_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    2,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 2 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    2,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 2 * 3,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
                gl.enable_vertex_attrib_array(self.location + 3);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                    gl.vertex_attrib_divisor(self.location + 3, 1);
                }
            }
            AttributeFormat::Float4x3_f32 => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 4 * 3,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    3,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 4 * 3 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    3,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 4 * 3 * 3,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
                gl.enable_vertex_attrib_array(self.location + 3);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                    gl.vertex_attrib_divisor(self.location + 3, 1);
                }
            }
            AttributeFormat::Float4x3_i8_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 3,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    3,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 3 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    3,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 3 * 3,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
                gl.enable_vertex_attrib_array(self.location + 3);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                    gl.vertex_attrib_divisor(self.location + 3, 1);
                }
            }
            AttributeFormat::Float4x3_i8_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 3,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    3,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 3 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    3,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 3 * 3,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
                gl.enable_vertex_attrib_array(self.location + 3);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                    gl.vertex_attrib_divisor(self.location + 3, 1);
                }
            }
            AttributeFormat::Float4x3_i16_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 3,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    3,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 3 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    3,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 3 * 3,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
                gl.enable_vertex_attrib_array(self.location + 3);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                    gl.vertex_attrib_divisor(self.location + 3, 1);
                }
            }
            AttributeFormat::Float4x3_i16_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 3,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    3,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 3 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    3,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 3 * 3,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
                gl.enable_vertex_attrib_array(self.location + 3);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                    gl.vertex_attrib_divisor(self.location + 3, 1);
                }
            }
            AttributeFormat::Float4x3_u8_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 3,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    3,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 3 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    3,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 3 * 3,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
                gl.enable_vertex_attrib_array(self.location + 3);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                    gl.vertex_attrib_divisor(self.location + 3, 1);
                }
            }
            AttributeFormat::Float4x3_u8_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 3,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    3,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 3 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    3,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 3 * 3,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
                gl.enable_vertex_attrib_array(self.location + 3);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                    gl.vertex_attrib_divisor(self.location + 3, 1);
                }
            }
            AttributeFormat::Float4x3_u16_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 3,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    3,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 3 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    3,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 3 * 3,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
                gl.enable_vertex_attrib_array(self.location + 3);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                    gl.vertex_attrib_divisor(self.location + 3, 1);
                }
            }
            AttributeFormat::Float4x3_u16_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 3,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    3,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 3 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    3,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 3 * 3,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
                gl.enable_vertex_attrib_array(self.location + 3);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                    gl.vertex_attrib_divisor(self.location + 3, 1);
                }
            }
            AttributeFormat::Float4x4_f32 => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 4 * 4,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    4,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 4 * 4 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    4,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 4 * 4 * 3,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
                gl.enable_vertex_attrib_array(self.location + 3);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                    gl.vertex_attrib_divisor(self.location + 3, 1);
                }
            }
            AttributeFormat::Float4x4_i8_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 4,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    4,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 4 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    4,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 4 * 3,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
                gl.enable_vertex_attrib_array(self.location + 3);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                    gl.vertex_attrib_divisor(self.location + 3, 1);
                }
            }
            AttributeFormat::Float4x4_i8_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 4,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    4,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 4 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    4,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 4 * 3,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
                gl.enable_vertex_attrib_array(self.location + 3);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                    gl.vertex_attrib_divisor(self.location + 3, 1);
                }
            }
            AttributeFormat::Float4x4_i16_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 4,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    4,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 4 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    4,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 4 * 3,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
                gl.enable_vertex_attrib_array(self.location + 3);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                    gl.vertex_attrib_divisor(self.location + 3, 1);
                }
            }
            AttributeFormat::Float4x4_i16_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 4,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    4,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 4 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    4,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 4 * 3,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
                gl.enable_vertex_attrib_array(self.location + 3);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                    gl.vertex_attrib_divisor(self.location + 3, 1);
                }
            }
            AttributeFormat::Float4x4_u8_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 4,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    4,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 4 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    4,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 4 * 3,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
                gl.enable_vertex_attrib_array(self.location + 3);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                    gl.vertex_attrib_divisor(self.location + 3, 1);
                }
            }
            AttributeFormat::Float4x4_u8_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 4,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    4,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 4 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    4,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 4 * 3,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
                gl.enable_vertex_attrib_array(self.location + 3);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                    gl.vertex_attrib_divisor(self.location + 3, 1);
                }
            }
            AttributeFormat::Float4x4_u16_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 4,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    4,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 4 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    4,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 4 * 3,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
                gl.enable_vertex_attrib_array(self.location + 3);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                    gl.vertex_attrib_divisor(self.location + 3, 1);
                }
            }
            AttributeFormat::Float4x4_u16_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 4,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    4,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 4 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    4,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 4 * 3,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
                gl.enable_vertex_attrib_array(self.location + 3);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                    gl.vertex_attrib_divisor(self.location + 3, 1);
                }
            }
            AttributeFormat::Integer_i8 => {
                gl.vertex_attrib_i_pointer_with_i32(
                    self.location,
                    1,
                    Gl::BYTE,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Integer_u8 => {
                gl.vertex_attrib_i_pointer_with_i32(
                    self.location,
                    1,
                    Gl::UNSIGNED_BYTE,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Integer_i16 => {
                gl.vertex_attrib_i_pointer_with_i32(
                    self.location,
                    1,
                    Gl::SHORT,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Integer_u16 => {
                gl.vertex_attrib_i_pointer_with_i32(
                    self.location,
                    1,
                    Gl::UNSIGNED_SHORT,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Integer_i32 => {
                gl.vertex_attrib_i_pointer_with_i32(
                    self.location,
                    1,
                    Gl::INT,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Integer_u32 => {
                gl.vertex_attrib_i_pointer_with_i32(
                    self.location,
                    1,
                    Gl::UNSIGNED_INT,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Integer2_i8 => {
                gl.vertex_attrib_i_pointer_with_i32(
                    self.location,
                    2,
                    Gl::BYTE,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Integer2_u8 => {
                gl.vertex_attrib_i_pointer_with_i32(
                    self.location,
                    2,
                    Gl::UNSIGNED_BYTE,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Integer2_i16 => {
                gl.vertex_attrib_i_pointer_with_i32(
                    self.location,
                    2,
                    Gl::SHORT,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Integer2_u16 => {
                gl.vertex_attrib_i_pointer_with_i32(
                    self.location,
                    2,
                    Gl::UNSIGNED_SHORT,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Integer2_i32 => {
                gl.vertex_attrib_i_pointer_with_i32(
                    self.location,
                    2,
                    Gl::INT,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Integer2_u32 => {
                gl.vertex_attrib_i_pointer_with_i32(
                    self.location,
                    2,
                    Gl::UNSIGNED_INT,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Integer3_i8 => {
                gl.vertex_attrib_i_pointer_with_i32(
                    self.location,
                    3,
                    Gl::BYTE,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Integer3_u8 => {
                gl.vertex_attrib_i_pointer_with_i32(
                    self.location,
                    3,
                    Gl::UNSIGNED_BYTE,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Integer3_i16 => {
                gl.vertex_attrib_i_pointer_with_i32(
                    self.location,
                    3,
                    Gl::SHORT,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Integer3_u16 => {
                gl.vertex_attrib_i_pointer_with_i32(
                    self.location,
                    3,
                    Gl::UNSIGNED_SHORT,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Integer3_i32 => {
                gl.vertex_attrib_i_pointer_with_i32(
                    self.location,
                    3,
                    Gl::INT,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Integer3_u32 => {
                gl.vertex_attrib_i_pointer_with_i32(
                    self.location,
                    3,
                    Gl::UNSIGNED_INT,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Integer4_i8 => {
                gl.vertex_attrib_i_pointer_with_i32(
                    self.location,
                    4,
                    Gl::BYTE,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Integer4_u8 => {
                gl.vertex_attrib_i_pointer_with_i32(
                    self.location,
                    4,
                    Gl::UNSIGNED_BYTE,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Integer4_i16 => {
                gl.vertex_attrib_i_pointer_with_i32(
                    self.location,
                    4,
                    Gl::SHORT,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Integer4_u16 => {
                gl.vertex_attrib_i_pointer_with_i32(
                    self.location,
                    4,
                    Gl::UNSIGNED_SHORT,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Integer4_i32 => {
                gl.vertex_attrib_i_pointer_with_i32(
                    self.location,
                    4,
                    Gl::INT,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Integer4_u32 => {
                gl.vertex_attrib_i_pointer_with_i32(
                    self.location,
                    4,
                    Gl::UNSIGNED_INT,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
        }
    }
}
