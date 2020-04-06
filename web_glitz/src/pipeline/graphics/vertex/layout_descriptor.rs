use std::hash::{Hash, Hasher};
use std::mem;

use fnv::FnvHasher;
use web_sys::WebGl2RenderingContext as Gl;

use crate::pipeline::graphics::attribute_format::VertexAttributeFormat;
use crate::pipeline::graphics::Vertex;

/// A vertex input layout description attached to a type.
///
/// See also [VertexInputLayoutDescriptor].
///
/// This trait becomes useful in combination with the [TypedVertexBuffers] trait. If a
/// [TypedVertexInputLayout] is attached to a [GraphicsPipeline] (see
/// [GraphicsPipelineDescriptorBuilder::typed_vertex_input_layout]), then [TypedVertexBuffers] with
/// a matching [TypedVertexBuffers::Layout] may be bound to the pipeline without further runtime
/// checks.
///
/// Note that [TypedVertexInputLayout] is safe to implement, but implementing [TypedVertexBuffers]
/// is unsafe: the data in the buffer representation for which [TypedVertexBuffers] is implemented
/// must always be compatible with the vertex input layout specified by the
/// [TypedVertexBuffers::Layout], see [TypedVertexBuffers] for details.
pub trait TypedVertexInputLayout {
    type LayoutDescription: Into<VertexInputLayoutDescriptor>;

    const LAYOUT_DESCRIPTION: Self::LayoutDescription;
}

impl TypedVertexInputLayout for () {
    type LayoutDescription = ();

    const LAYOUT_DESCRIPTION: Self::LayoutDescription = ();
}

macro_rules! impl_typed_vertex_input_layout {
    ($n:tt, $($T:ident),*) => {
        #[allow(unused_parens)]
        impl<$($T),*> TypedVertexInputLayout for ($($T),*) where $($T: Vertex),* {
            type LayoutDescription = [StaticVertexBufferSlotDescriptor; $n];

            const LAYOUT_DESCRIPTION: Self::LayoutDescription = [
                $(
                    StaticVertexBufferSlotDescriptor {
                        stride: mem::size_of::<$T>() as u8,
                        input_rate: $T::INPUT_RATE,
                        attributes: $T::ATTRIBUTE_DESCRIPTORS
                    }
                ),*
            ];
        }
    }
}

impl_typed_vertex_input_layout!(1, T0);
impl_typed_vertex_input_layout!(2, T0, T1);
impl_typed_vertex_input_layout!(3, T0, T1, T2);
impl_typed_vertex_input_layout!(4, T0, T1, T2, T3);
impl_typed_vertex_input_layout!(5, T0, T1, T2, T3, T4);
impl_typed_vertex_input_layout!(6, T0, T1, T2, T3, T4, T5);
impl_typed_vertex_input_layout!(7, T0, T1, T2, T3, T4, T5, T6);
impl_typed_vertex_input_layout!(8, T0, T1, T2, T3, T4, T5, T6, T7);
impl_typed_vertex_input_layout!(9, T0, T1, T2, T3, T4, T5, T6, T7, T8);
impl_typed_vertex_input_layout!(10, T0, T1, T2, T3, T4, T5, T6, T7, T8, T9);
impl_typed_vertex_input_layout!(11, T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
impl_typed_vertex_input_layout!(12, T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);
impl_typed_vertex_input_layout!(13, T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12);
impl_typed_vertex_input_layout!(14, T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13);
impl_typed_vertex_input_layout!(
    15, T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14
);
impl_typed_vertex_input_layout!(
    16, T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15
);

/// Helper type for implementing [TypedVertexInputLayout] for (tuples of) [Vertex] types.
pub struct StaticVertexBufferSlotDescriptor {
    /// The stride in bytes between successive vertices in the bind slot.
    pub stride: u8,

    /// The [InputRate] for the bind slot.
    pub input_rate: InputRate,

    /// The set of [VertexAttributeDescriptor]s defined on the bind slot.
    pub attributes: &'static [VertexAttributeDescriptor],
}

impl Into<VertexInputLayoutDescriptor> for () {
    fn into(self) -> VertexInputLayoutDescriptor {
        VertexInputLayoutDescriptor {
            initial_bind_slot: None,
            layout: Vec::new(), // Won't allocate, see [Vec::new].
            hash_code: 0,
        }
    }
}

macro_rules! impl_into_vertex_input_layout_descriptor {
    ($n:tt) => {
        impl Into<VertexInputLayoutDescriptor> for [StaticVertexBufferSlotDescriptor; $n] {
            fn into(self) -> VertexInputLayoutDescriptor {
                let mut attribute_count = 0;

                for i in 0..$n {
                    attribute_count += self[i].attributes.len();
                }

                let mut builder = VertexInputLayoutDescriptorBuilder::new(Some(
                    VertexInputLayoutAllocationHint {
                        bind_slot_count: $n,
                        attribute_count: attribute_count as u8,
                    },
                ));

                for i in 0..$n {
                    let mut slot = builder.add_buffer_slot(self[i].stride, self[i].input_rate);

                    for attribute in self[i].attributes {
                        slot.add_attribute(*attribute);
                    }
                }

                builder.finish()
            }
        }
    };
}

impl_into_vertex_input_layout_descriptor!(1);
impl_into_vertex_input_layout_descriptor!(2);
impl_into_vertex_input_layout_descriptor!(3);
impl_into_vertex_input_layout_descriptor!(4);
impl_into_vertex_input_layout_descriptor!(5);
impl_into_vertex_input_layout_descriptor!(6);
impl_into_vertex_input_layout_descriptor!(7);
impl_into_vertex_input_layout_descriptor!(8);
impl_into_vertex_input_layout_descriptor!(9);
impl_into_vertex_input_layout_descriptor!(10);
impl_into_vertex_input_layout_descriptor!(11);
impl_into_vertex_input_layout_descriptor!(12);
impl_into_vertex_input_layout_descriptor!(13);
impl_into_vertex_input_layout_descriptor!(14);
impl_into_vertex_input_layout_descriptor!(15);
impl_into_vertex_input_layout_descriptor!(16);

/// Describes the input rate for a vertex buffer in a [VertexInputLayoutDescriptor].
///
/// See also [VertexInputLayoutDescriptorBuilder::add_buffer_slot] and
/// [VertexBufferSlotRef::input_rate].
#[derive(Clone, Copy, PartialEq, Hash, Debug)]
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

/// Describes a layout of vertex buffers bind slots and the vertex attributes defined on these bind
/// slots.
///
/// See [VertexInputLayoutDescriptorBuilder] for details on how a layout is constructed.
#[derive(Clone, PartialEq, Debug)]
pub struct VertexInputLayoutDescriptor {
    initial_bind_slot: Option<BindSlot>,
    layout: Vec<LayoutElement>,
    hash_code: u64,
}

impl VertexInputLayoutDescriptor {
    pub(crate) fn check_compatibility(
        &self,
        slot_descriptors: &[VertexAttributeSlotDescriptor],
    ) -> Result<(), IncompatibleVertexInputLayout> {
        'outer: for slot in slot_descriptors.iter() {
            for element in self.layout.iter() {
                if let LayoutElement::NextAttribute(attribute_descriptor) = element {
                    if attribute_descriptor.location == slot.location {
                        if !attribute_descriptor
                            .format
                            .is_compatible(slot.attribute_type)
                        {
                            return Err(IncompatibleVertexInputLayout::TypeMismatch {
                                location: slot.location,
                            });
                        }

                        continue 'outer;
                    }
                }
            }

            return Err(IncompatibleVertexInputLayout::MissingAttribute {
                location: slot.location,
            });
        }

        Ok(())
    }

    /// Returns an iterator over the vertex buffer binding slots described by this descriptor.
    pub fn buffer_slots(&self) -> VertexBufferSlots {
        VertexBufferSlots {
            layout: self,
            cursor: -1,
        }
    }
}

impl Hash for VertexInputLayoutDescriptor {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        state.write_u64(self.hash_code);
    }
}

/// Returned from [VertexAttributeLayoutDescriptor::buffer_slots].
pub struct VertexBufferSlots<'a> {
    layout: &'a VertexInputLayoutDescriptor,
    cursor: isize,
}

impl<'a> Iterator for VertexBufferSlots<'a> {
    type Item = VertexBufferSlotRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor < 0 {
            self.cursor += 1;

            self.layout.initial_bind_slot.map(|slot| {
                let BindSlot { stride, input_rate } = slot;

                VertexBufferSlotRef {
                    layout: self.layout,
                    start: 0,
                    stride,
                    input_rate,
                }
            })
        } else {
            while let Some(element) = self.layout.layout.get(self.cursor as usize) {
                self.cursor += 1;

                if let LayoutElement::NextBindSlot(slot) = element {
                    let BindSlot { stride, input_rate } = *slot;

                    return Some(VertexBufferSlotRef {
                        layout: self.layout,
                        start: self.cursor as usize,
                        stride,
                        input_rate,
                    });
                }
            }

            None
        }
    }
}

/// Reference to a bind slot description in a [VertexAttributeLayoutDescriptor].
///
/// See [VertexAttributeLayoutDescriptor::bind_slots].
pub struct VertexBufferSlotRef<'a> {
    layout: &'a VertexInputLayoutDescriptor,
    start: usize,
    stride: u8,
    input_rate: InputRate,
}

impl<'a> VertexBufferSlotRef<'a> {
    /// Returns the stride in bytes between successive vertex entries.
    pub fn stride_in_bytes(&self) -> u8 {
        self.stride
    }

    /// Returns the [InputRate] used for this bind slot.
    pub fn input_rate(&self) -> InputRate {
        self.input_rate
    }

    /// Returns an iterator over the [VertexAttributeDescriptor]s defined on this bind slot.
    pub fn attributes(&self) -> VertexBufferSlotAttributes {
        VertexBufferSlotAttributes {
            layout: &self.layout.layout,
            cursor: self.start,
        }
    }
}

/// Returned from [BindSlotRef::attributes].
pub struct VertexBufferSlotAttributes<'a> {
    layout: &'a Vec<LayoutElement>,
    cursor: usize,
}

impl<'a> Iterator for VertexBufferSlotAttributes<'a> {
    type Item = &'a VertexAttributeDescriptor;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(LayoutElement::NextAttribute(attribute)) = self.layout.get(self.cursor) {
            self.cursor += 1;

            Some(attribute)
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, PartialEq, Hash, Debug)]
enum LayoutElement {
    NextAttribute(VertexAttributeDescriptor),
    NextBindSlot(BindSlot),
}

#[derive(Clone, Copy, PartialEq, Hash, Debug)]
struct BindSlot {
    stride: u8,
    input_rate: InputRate,
}

/// Allocation hint for a [VertexInputLayoutDescriptor], see
/// [VertexInputLayoutDescriptorBuilder::new].
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct VertexInputLayoutAllocationHint {
    /// The number of bind slots the descriptor describes.
    pub bind_slot_count: u8,

    /// The total number of attributes the descriptor describes across all bind slots.
    pub attribute_count: u8,
}

/// Builds a new [VertexInputLayoutDescriptor].
///
/// # Example
///
/// ```rust
/// use web_glitz::pipeline::graphics::{VertexInputLayoutDescriptorBuilder, VertexInputLayoutAllocationHint, InputRate, VertexAttributeDescriptor};
/// use web_glitz::pipeline::graphics::attribute_format::VertexAttributeFormat;
///
/// let mut builder = VertexInputLayoutDescriptorBuilder::new(Some(VertexInputLayoutAllocationHint {
///     bind_slot_count: 2,
///     attribute_count: 3
/// }));
///
/// builder.add_buffer_slot(28, InputRate::PerVertex)
///     .add_attribute(VertexAttributeDescriptor {
///         location: 0,
///         offset_in_bytes: 0,
///         format: VertexAttributeFormat::Float4_f32
///     })
///     .add_attribute(VertexAttributeDescriptor {
///         location: 1,
///         offset_in_bytes: 16,
///         format: VertexAttributeFormat::Float3_f32
///     });
///
/// builder.add_buffer_slot(16, InputRate::PerInstance)
///     .add_attribute(VertexAttributeDescriptor {
///         location: 2,
///         offset_in_bytes: 0,
///         format: VertexAttributeFormat::Float4_f32
///     });
///
/// let layout_descriptor = builder.finish();
/// ```
pub struct VertexInputLayoutDescriptorBuilder {
    initial_bind_slot: Option<BindSlot>,
    layout: Vec<LayoutElement>,
}

impl VertexInputLayoutDescriptorBuilder {
    /// Creates a new builder.
    ///
    /// Takes an optional `allocation_hint` to help reduce the number of allocations without over-
    /// allocating. With an accurate allocation hint the builder will only allocate once. See
    /// [VertexInputLayoutAllocationHint] for details.
    pub fn new(allocation_hint: Option<VertexInputLayoutAllocationHint>) -> Self {
        let layout = if let Some(hint) = allocation_hint {
            Vec::with_capacity((hint.bind_slot_count - 1 + hint.attribute_count) as usize)
        } else {
            Vec::new()
        };

        VertexInputLayoutDescriptorBuilder {
            initial_bind_slot: None,
            layout,
        }
    }

    /// Adds a vertex buffer binding slot to the layout.
    pub fn add_buffer_slot(
        &mut self,
        stride: u8,
        input_rate: InputRate,
    ) -> VertexBufferSlotAttributeAttacher {
        let bind_slot = BindSlot { stride, input_rate };

        if self.initial_bind_slot.is_none() {
            self.initial_bind_slot = Some(bind_slot);
        } else {
            self.layout.push(LayoutElement::NextBindSlot(bind_slot))
        }

        VertexBufferSlotAttributeAttacher {
            stride,
            layout_builder: self,
        }
    }

    /// Finishes building and returns the resulting [VertexInputLayoutDescriptor].
    pub fn finish(self) -> VertexInputLayoutDescriptor {
        // Setup a hashcode once at pipeline creation time so that future hashing is cheap.
        let hash_code = if let Some(slot) = self.initial_bind_slot {
            let mut hasher = FnvHasher::default();

            slot.hash(&mut hasher);
            self.layout.hash(&mut hasher);

            hasher.finish()
        } else {
            0
        };

        VertexInputLayoutDescriptor {
            initial_bind_slot: self.initial_bind_slot,
            layout: self.layout,
            hash_code,
        }
    }
}

/// Returned from [VertexInputLayoutDescriptorBuilder::add_buffer_slot], attaches
/// [VertexAttributeDescriptor]s to attribute layout bind slots.
pub struct VertexBufferSlotAttributeAttacher<'a> {
    stride: u8,
    layout_builder: &'a mut VertexInputLayoutDescriptorBuilder,
}

impl<'a> VertexBufferSlotAttributeAttacher<'a> {
    /// Adds an attribute descriptor to the current bind slot.
    ///
    /// # Panics
    ///
    /// Panics if the attribute does not fit within one stride (attribute offset + size is greater
    /// bind slot stride).
    pub fn add_attribute(
        &mut self,
        attribute_descriptor: VertexAttributeDescriptor,
    ) -> &mut VertexBufferSlotAttributeAttacher<'a> {
        let size = attribute_descriptor.format.size_in_bytes();

        if attribute_descriptor.offset_in_bytes + size > self.stride {
            panic!("Attribute does not fit within stride.");
        }

        self.layout_builder
            .layout
            .push(LayoutElement::NextAttribute(attribute_descriptor));

        self
    }
}

/// Error returned by [AttributeSlotLayoutCompatible::check_compatibility].
#[derive(Debug)]
pub enum IncompatibleVertexInputLayout {
    /// Variant returned if no attribute data is available for the [AttributeSlotDescriptor] with
    /// at the `location`.
    MissingAttribute { location: u32 },

    /// Variant returned if attribute data is available for the [AttributeSlotDescriptor] with
    /// at the `location`. but attribute data is not compatible with the [AttributeType] of the
    /// [AttributeSlotDescriptor] (see [AttributeSlotDescriptor::attribute_type]).
    TypeMismatch { location: u32 },
}

/// Describes an input slot on a [GraphicsPipeline].
pub(crate) struct VertexAttributeSlotDescriptor {
    /// The shader location of the attribute slot.
    pub(crate) location: u32,

    /// The type of attribute required to fill the slot.
    pub(crate) attribute_type: VertexAttributeType,
}

/// Enumerates the possible attribute types that might be required to fill an attribute slot.
///
/// See also [AttributeSlotDescriptor].
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum VertexAttributeType {
    Float,
    FloatVector2,
    FloatVector3,
    FloatVector4,
    FloatMatrix2x2,
    FloatMatrix2x3,
    FloatMatrix2x4,
    FloatMatrix3x2,
    FloatMatrix3x3,
    FloatMatrix3x4,
    FloatMatrix4x2,
    FloatMatrix4x3,
    FloatMatrix4x4,
    Integer,
    IntegerVector2,
    IntegerVector3,
    IntegerVector4,
    UnsignedInteger,
    UnsignedIntegerVector2,
    UnsignedIntegerVector3,
    UnsignedIntegerVector4,
}

impl VertexAttributeType {
    pub(crate) fn from_type_id(id: u32) -> Self {
        match id {
            Gl::FLOAT => VertexAttributeType::Float,
            Gl::FLOAT_VEC2 => VertexAttributeType::FloatVector2,
            Gl::FLOAT_VEC3 => VertexAttributeType::FloatVector3,
            Gl::FLOAT_VEC4 => VertexAttributeType::FloatVector4,
            Gl::FLOAT_MAT2 => VertexAttributeType::FloatMatrix2x2,
            Gl::FLOAT_MAT3 => VertexAttributeType::FloatMatrix3x3,
            Gl::FLOAT_MAT4 => VertexAttributeType::FloatMatrix4x4,
            Gl::FLOAT_MAT2X3 => VertexAttributeType::FloatMatrix2x3,
            Gl::FLOAT_MAT2X4 => VertexAttributeType::FloatMatrix2x4,
            Gl::FLOAT_MAT3X2 => VertexAttributeType::FloatMatrix3x2,
            Gl::FLOAT_MAT3X4 => VertexAttributeType::FloatMatrix3x4,
            Gl::FLOAT_MAT4X2 => VertexAttributeType::FloatMatrix4x2,
            Gl::FLOAT_MAT4X3 => VertexAttributeType::FloatMatrix4x3,
            Gl::INT => VertexAttributeType::Integer,
            Gl::INT_VEC2 => VertexAttributeType::IntegerVector2,
            Gl::INT_VEC3 => VertexAttributeType::IntegerVector3,
            Gl::INT_VEC4 => VertexAttributeType::IntegerVector4,
            Gl::UNSIGNED_INT => VertexAttributeType::UnsignedInteger,
            Gl::UNSIGNED_INT_VEC2 => VertexAttributeType::UnsignedIntegerVector2,
            Gl::UNSIGNED_INT_VEC3 => VertexAttributeType::UnsignedIntegerVector3,
            Gl::UNSIGNED_INT_VEC4 => VertexAttributeType::UnsignedIntegerVector4,
            id => panic!("Invalid attribute type id: {}", id),
        }
    }
}

/// Describes how the data for an input attribute in a [VertexShader] is sourced from vertex
/// buffers.
///
/// See also [VertexInputLayoutDescriptor].
#[derive(Clone, Copy, PartialEq, Hash, Debug)]
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
    /// values are obtained by adding a stride to the base offset.
    pub offset_in_bytes: u8,

    /// The data format in which the attribute values are stored.
    ///
    /// Should be a format that is compatible with the type used for the attribute in the shader,
    /// see also [VertexAttributeFormat::is_compatible].
    pub format: VertexAttributeFormat,
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
            VertexAttributeFormat::Float_f32 => {
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
            VertexAttributeFormat::Float_i8_fixed => {
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
            VertexAttributeFormat::Float_i8_norm => {
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
            VertexAttributeFormat::Float_i16_fixed => {
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
            VertexAttributeFormat::Float_i16_norm => {
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
            VertexAttributeFormat::Float_u8_fixed => {
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
            VertexAttributeFormat::Float_u8_norm => {
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
            VertexAttributeFormat::Float_u16_fixed => {
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
            VertexAttributeFormat::Float_u16_norm => {
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
            VertexAttributeFormat::Float2_f32 => {
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
            VertexAttributeFormat::Float2_i8_fixed => {
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
            VertexAttributeFormat::Float2_i8_norm => {
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
            VertexAttributeFormat::Float2_i16_fixed => {
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
            VertexAttributeFormat::Float2_i16_norm => {
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
            VertexAttributeFormat::Float2_u8_fixed => {
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
            VertexAttributeFormat::Float2_u8_norm => {
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
            VertexAttributeFormat::Float2_u16_fixed => {
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
            VertexAttributeFormat::Float2_u16_norm => {
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
            VertexAttributeFormat::Float3_f32 => {
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
            VertexAttributeFormat::Float3_i8_fixed => {
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
            VertexAttributeFormat::Float3_i8_norm => {
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
            VertexAttributeFormat::Float3_i16_fixed => {
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
            VertexAttributeFormat::Float3_i16_norm => {
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
            VertexAttributeFormat::Float3_u8_fixed => {
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
            VertexAttributeFormat::Float3_u8_norm => {
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
            VertexAttributeFormat::Float3_u16_fixed => {
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
            VertexAttributeFormat::Float3_u16_norm => {
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
            VertexAttributeFormat::Float4_f32 => {
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
            VertexAttributeFormat::Float4_i8_fixed => {
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
            VertexAttributeFormat::Float4_i8_norm => {
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
            VertexAttributeFormat::Float4_i16_fixed => {
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
            VertexAttributeFormat::Float4_i16_norm => {
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
            VertexAttributeFormat::Float4_u8_fixed => {
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
            VertexAttributeFormat::Float4_u8_norm => {
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
            VertexAttributeFormat::Float4_u16_fixed => {
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
            VertexAttributeFormat::Float4_u16_norm => {
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
            VertexAttributeFormat::Float2x2_f32 => {
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
            VertexAttributeFormat::Float2x2_i8_fixed => {
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
            VertexAttributeFormat::Float2x2_i8_norm => {
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
            VertexAttributeFormat::Float2x2_i16_fixed => {
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
            VertexAttributeFormat::Float2x2_i16_norm => {
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
            VertexAttributeFormat::Float2x2_u8_fixed => {
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
            VertexAttributeFormat::Float2x2_u8_norm => {
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
            VertexAttributeFormat::Float2x2_u16_fixed => {
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
            VertexAttributeFormat::Float2x2_u16_norm => {
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
            VertexAttributeFormat::Float2x3_f32 => {
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
            VertexAttributeFormat::Float2x3_i8_fixed => {
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
            VertexAttributeFormat::Float2x3_i8_norm => {
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
            VertexAttributeFormat::Float2x3_i16_fixed => {
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
            VertexAttributeFormat::Float2x3_i16_norm => {
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
            VertexAttributeFormat::Float2x3_u8_fixed => {
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
            VertexAttributeFormat::Float2x3_u8_norm => {
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
            VertexAttributeFormat::Float2x3_u16_fixed => {
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
            VertexAttributeFormat::Float2x3_u16_norm => {
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
            VertexAttributeFormat::Float2x4_f32 => {
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
            VertexAttributeFormat::Float2x4_i8_fixed => {
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
            VertexAttributeFormat::Float2x4_i8_norm => {
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
            VertexAttributeFormat::Float2x4_i16_fixed => {
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
            VertexAttributeFormat::Float2x4_i16_norm => {
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
            VertexAttributeFormat::Float2x4_u8_fixed => {
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
            VertexAttributeFormat::Float2x4_u8_norm => {
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
            VertexAttributeFormat::Float2x4_u16_fixed => {
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
            VertexAttributeFormat::Float2x4_u16_norm => {
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
            VertexAttributeFormat::Float3x2_f32 => {
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
            VertexAttributeFormat::Float3x2_i8_fixed => {
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
            VertexAttributeFormat::Float3x2_i8_norm => {
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
            VertexAttributeFormat::Float3x2_i16_fixed => {
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
            VertexAttributeFormat::Float3x2_i16_norm => {
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
            VertexAttributeFormat::Float3x2_u8_fixed => {
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
            VertexAttributeFormat::Float3x2_u8_norm => {
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
            VertexAttributeFormat::Float3x2_u16_fixed => {
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
            VertexAttributeFormat::Float3x2_u16_norm => {
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
            VertexAttributeFormat::Float3x3_f32 => {
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
            VertexAttributeFormat::Float3x3_i8_fixed => {
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
            VertexAttributeFormat::Float3x3_i8_norm => {
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
            VertexAttributeFormat::Float3x3_i16_fixed => {
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
            VertexAttributeFormat::Float3x3_i16_norm => {
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
            VertexAttributeFormat::Float3x3_u8_fixed => {
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
            VertexAttributeFormat::Float3x3_u8_norm => {
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
            VertexAttributeFormat::Float3x3_u16_fixed => {
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
            VertexAttributeFormat::Float3x3_u16_norm => {
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
            VertexAttributeFormat::Float3x4_f32 => {
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
            VertexAttributeFormat::Float3x4_i8_fixed => {
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
            VertexAttributeFormat::Float3x4_i8_norm => {
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
            VertexAttributeFormat::Float3x4_i16_fixed => {
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
            VertexAttributeFormat::Float3x4_i16_norm => {
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
            VertexAttributeFormat::Float3x4_u8_fixed => {
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
            VertexAttributeFormat::Float3x4_u8_norm => {
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
            VertexAttributeFormat::Float3x4_u16_fixed => {
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
            VertexAttributeFormat::Float3x4_u16_norm => {
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
            VertexAttributeFormat::Float4x2_f32 => {
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
            VertexAttributeFormat::Float4x2_i8_fixed => {
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
            VertexAttributeFormat::Float4x2_i8_norm => {
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
            VertexAttributeFormat::Float4x2_i16_fixed => {
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
            VertexAttributeFormat::Float4x2_i16_norm => {
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
            VertexAttributeFormat::Float4x2_u8_fixed => {
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
            VertexAttributeFormat::Float4x2_u8_norm => {
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
            VertexAttributeFormat::Float4x2_u16_fixed => {
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
            VertexAttributeFormat::Float4x2_u16_norm => {
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
            VertexAttributeFormat::Float4x3_f32 => {
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
            VertexAttributeFormat::Float4x3_i8_fixed => {
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
            VertexAttributeFormat::Float4x3_i8_norm => {
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
            VertexAttributeFormat::Float4x3_i16_fixed => {
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
            VertexAttributeFormat::Float4x3_i16_norm => {
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
            VertexAttributeFormat::Float4x3_u8_fixed => {
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
            VertexAttributeFormat::Float4x3_u8_norm => {
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
            VertexAttributeFormat::Float4x3_u16_fixed => {
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
            VertexAttributeFormat::Float4x3_u16_norm => {
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
            VertexAttributeFormat::Float4x4_f32 => {
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
            VertexAttributeFormat::Float4x4_i8_fixed => {
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
            VertexAttributeFormat::Float4x4_i8_norm => {
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
            VertexAttributeFormat::Float4x4_i16_fixed => {
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
            VertexAttributeFormat::Float4x4_i16_norm => {
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
            VertexAttributeFormat::Float4x4_u8_fixed => {
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
            VertexAttributeFormat::Float4x4_u8_norm => {
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
            VertexAttributeFormat::Float4x4_u16_fixed => {
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
            VertexAttributeFormat::Float4x4_u16_norm => {
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
            VertexAttributeFormat::Integer_i8 => {
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
            VertexAttributeFormat::Integer_u8 => {
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
            VertexAttributeFormat::Integer_i16 => {
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
            VertexAttributeFormat::Integer_u16 => {
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
            VertexAttributeFormat::Integer_i32 => {
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
            VertexAttributeFormat::Integer_u32 => {
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
            VertexAttributeFormat::Integer2_i8 => {
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
            VertexAttributeFormat::Integer2_u8 => {
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
            VertexAttributeFormat::Integer2_i16 => {
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
            VertexAttributeFormat::Integer2_u16 => {
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
            VertexAttributeFormat::Integer2_i32 => {
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
            VertexAttributeFormat::Integer2_u32 => {
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
            VertexAttributeFormat::Integer3_i8 => {
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
            VertexAttributeFormat::Integer3_u8 => {
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
            VertexAttributeFormat::Integer3_i16 => {
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
            VertexAttributeFormat::Integer3_u16 => {
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
            VertexAttributeFormat::Integer3_i32 => {
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
            VertexAttributeFormat::Integer3_u32 => {
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
            VertexAttributeFormat::Integer4_i8 => {
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
            VertexAttributeFormat::Integer4_u8 => {
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
            VertexAttributeFormat::Integer4_i16 => {
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
            VertexAttributeFormat::Integer4_u16 => {
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
            VertexAttributeFormat::Integer4_i32 => {
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
            VertexAttributeFormat::Integer4_u32 => {
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
