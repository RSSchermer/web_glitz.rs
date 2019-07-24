use std::hash::{Hash, Hasher};
use std::mem;

use fnv::FnvHasher;
use web_sys::WebGl2RenderingContext as Gl;

use crate::pipeline::graphics::{InputRate, Vertex, VertexAttributeDescriptor};

// TODO: could this trait potentially be safe now? The `unsafe` contract on the `TypedVertexBuffers`
// trait might be sufficient.

/// A vertex attribute layout description attached to a type.
///
/// See also [VertexAttributeLayoutDescriptor].
pub unsafe trait TypedVertexAttributeLayout {
    type LayoutDescription: Into<VertexAttributeLayoutDescriptor>;

    const LAYOUT_DESCRIPTION: Self::LayoutDescription;
}

unsafe impl TypedVertexAttributeLayout for () {
    type LayoutDescription = ();

    const LAYOUT_DESCRIPTION: Self::LayoutDescription = ();
}

macro_rules! impl_typed_vertex_attribute_layout {
    ($n:tt, $($T:ident),*) => {
        unsafe impl<$($T),*> TypedVertexAttributeLayout for ($($T),*) where $($T: Vertex),* {
            type LayoutDescription = [StaticBindSlotDescriptor; $n];

            const LAYOUT_DESCRIPTION: Self::LayoutDescription = [
                $(
                    StaticBindSlotDescriptor {
                        stride: mem::size_of::<$T>() as u8,
                        input_rate: $T::INPUT_RATE,
                        attributes: $T::ATTRIBUTE_DESCRIPTORS
                    }
                ),*
            ];
        }
    }
}

impl_typed_vertex_attribute_layout!(1, T0);
impl_typed_vertex_attribute_layout!(2, T0, T1);
impl_typed_vertex_attribute_layout!(3, T0, T1, T2);
impl_typed_vertex_attribute_layout!(4, T0, T1, T2, T3);
impl_typed_vertex_attribute_layout!(5, T0, T1, T2, T3, T4);
impl_typed_vertex_attribute_layout!(6, T0, T1, T2, T3, T4, T5);
impl_typed_vertex_attribute_layout!(7, T0, T1, T2, T3, T4, T5, T6);
impl_typed_vertex_attribute_layout!(8, T0, T1, T2, T3, T4, T5, T6, T7);
impl_typed_vertex_attribute_layout!(9, T0, T1, T2, T3, T4, T5, T6, T7, T8);
impl_typed_vertex_attribute_layout!(10, T0, T1, T2, T3, T4, T5, T6, T7, T8, T9);
impl_typed_vertex_attribute_layout!(11, T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
impl_typed_vertex_attribute_layout!(12, T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);
impl_typed_vertex_attribute_layout!(13, T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12);
impl_typed_vertex_attribute_layout!(14, T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13);
impl_typed_vertex_attribute_layout!(
    15, T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14
);
impl_typed_vertex_attribute_layout!(
    16, T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15
);

/// Helper type for implementing [TypedVertexAttributeLayout] for (tuples of) [Vertex] types.
pub struct StaticBindSlotDescriptor {
    /// The stride in bytes between successive vertices in the bind slot.
    pub stride: u8,

    /// The [InputRate] for the bind slot.
    pub input_rate: InputRate,

    /// The set of [VertexAttributeDescriptor]s defined on the bind slot.
    pub attributes: &'static [VertexAttributeDescriptor],
}

impl Into<VertexAttributeLayoutDescriptor> for () {
    fn into(self) -> VertexAttributeLayoutDescriptor {
        VertexAttributeLayoutDescriptor {
            initial_bind_slot: None,
            layout: Vec::new(), // Won't allocate, see [Vec::new].
            hash_code: 0,
        }
    }
}

macro_rules! impl_into_vertex_attribute_layout_descriptor {
    ($n:tt) => {
        impl Into<VertexAttributeLayoutDescriptor> for [StaticBindSlotDescriptor; $n] {
            fn into(self) -> VertexAttributeLayoutDescriptor {
                let mut attribute_count = 0;

                for i in 0..$n {
                    attribute_count += self[i].attributes.len();
                }

                let mut builder = VertexAttributeLayoutDescriptorBuilder::new(Some(
                    AttributeLayoutAllocationHint {
                        bind_slot_count: $n,
                        attribute_count: attribute_count as u8,
                    },
                ));

                for i in 0..$n {
                    let mut slot = builder.add_bind_slot(self[i].stride, self[i].input_rate);

                    for attribute in self[i].attributes {
                        slot.add_attribute(*attribute);
                    }
                }

                builder.finish()
            }
        }
    };
}

impl_into_vertex_attribute_layout_descriptor!(1);
impl_into_vertex_attribute_layout_descriptor!(2);
impl_into_vertex_attribute_layout_descriptor!(3);
impl_into_vertex_attribute_layout_descriptor!(4);
impl_into_vertex_attribute_layout_descriptor!(5);
impl_into_vertex_attribute_layout_descriptor!(6);
impl_into_vertex_attribute_layout_descriptor!(7);
impl_into_vertex_attribute_layout_descriptor!(8);
impl_into_vertex_attribute_layout_descriptor!(9);
impl_into_vertex_attribute_layout_descriptor!(10);
impl_into_vertex_attribute_layout_descriptor!(11);
impl_into_vertex_attribute_layout_descriptor!(12);
impl_into_vertex_attribute_layout_descriptor!(13);
impl_into_vertex_attribute_layout_descriptor!(14);
impl_into_vertex_attribute_layout_descriptor!(15);
impl_into_vertex_attribute_layout_descriptor!(16);

/// Describes a layout of vertex buffers bind slots and the vertex attributes defined on these bind
/// slots.
///
/// See [VertexAttributeLayoutDescriptorBuilder] for details on how a layout is constructed.
#[derive(Clone, PartialEq, Debug)]
pub struct VertexAttributeLayoutDescriptor {
    initial_bind_slot: Option<BindSlot>,
    layout: Vec<LayoutElement>,
    hash_code: u64,
}

impl VertexAttributeLayoutDescriptor {
    pub(crate) fn check_compatibility(
        &self,
        slot_descriptors: &[AttributeSlotDescriptor],
    ) -> Result<(), IncompatibleAttributeLayout> {
        'outer: for slot in slot_descriptors.iter() {
            for element in self.layout.iter() {
                if let LayoutElement::NextAttribute(attribute_descriptor) = element {
                    if attribute_descriptor.location == slot.location() {
                        if !attribute_descriptor
                            .format
                            .is_compatible(slot.attribute_type)
                        {
                            return Err(IncompatibleAttributeLayout::TypeMismatch {
                                location: slot.location(),
                            });
                        }

                        continue 'outer;
                    }
                }
            }

            return Err(IncompatibleAttributeLayout::MissingAttribute {
                location: slot.location(),
            });
        }

        Ok(())
    }

    /// Returns an iterator over the bind slots described by this descriptor.
    pub fn bind_slots(&self) -> BindSlots {
        BindSlots {
            layout: self,
            cursor: -1,
        }
    }
}

impl Hash for VertexAttributeLayoutDescriptor {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        state.write_u64(self.hash_code);
    }
}

/// Returned from [VertexAttributeLayoutDescriptor::bind_slots].
pub struct BindSlots<'a> {
    layout: &'a VertexAttributeLayoutDescriptor,
    cursor: isize,
}

impl<'a> Iterator for BindSlots<'a> {
    type Item = BindSlotRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor < 0 {
            self.cursor += 1;

            self.layout.initial_bind_slot.map(|slot| {
                let BindSlot { stride, input_rate } = slot;

                BindSlotRef {
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

                    return Some(BindSlotRef {
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
pub struct BindSlotRef<'a> {
    layout: &'a VertexAttributeLayoutDescriptor,
    start: usize,
    stride: u8,
    input_rate: InputRate,
}

impl<'a> BindSlotRef<'a> {
    /// Returns the stride in bytes between successive vertex entries.
    pub fn stride_in_bytes(&self) -> u8 {
        self.stride
    }

    /// Returns the [InputRate] used for this bind slot.
    pub fn input_rate(&self) -> InputRate {
        self.input_rate
    }

    /// Returns an iterator over the [VertexAttributeDescriptor]s defined on this bind slot.
    pub fn attributes(&self) -> BindSlotAttributes {
        BindSlotAttributes {
            layout: &self.layout.layout,
            cursor: self.start,
        }
    }
}

/// Returned from [BindSlotRef::attributes].
pub struct BindSlotAttributes<'a> {
    layout: &'a Vec<LayoutElement>,
    cursor: usize,
}

impl<'a> Iterator for BindSlotAttributes<'a> {
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

/// Allocation hint for a [VertexAttributeDescriptor], see
/// [VertexAttributeLayoutDescriptorBuilder::new].
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct AttributeLayoutAllocationHint {
    /// The number of bind slots the descriptor describes.
    pub bind_slot_count: u8,

    /// The total number of attributes the descriptor describes across all bind slots.
    pub attribute_count: u8,
}

/// Builds a new [VertexAttributeLayoutDescriptor].
///
/// # Example
///
/// ```rust
/// use web_glitz::pipeline::graphics::{AttibuteLayoutAllocationHint, VertexAttributeLayoutDescriptorBuilder, AttributeLayoutAllocationHint, InputRate, VertexAttributeDescriptor};
/// use web_glitz::pipeline::graphics::attribute_format::AttributeFormat;
///
/// let mut builder = VertexAttributeLayoutDescriptorBuilder::new(Some(AttributeLayoutAllocationHint {
///     bind_slot_count: 2,
///     attribute_count: 3
/// }));
///
/// builder.add_bind_slot(28, InputRate::PerVertex)
///     .add_attribute(VertexAttributeDescriptor {
///         location: 0,
///         offset_in_bytes: 0,
///         format: AttributeFormat::Float4_f32
///     })
///     .add_attribute(VertexAttributeDescriptor {
///         location: 1,
///         offset_in_bytes: 16,
///         format: AttributeFormat::Float3_f32
///     });
///
/// builder.add_bind_slot(16, InputRate::PerInstance)
///     .add_attribute(VertexAttributeDescriptor {
///         location: 2,
///         offset_in_bytes: 0,
///         format: AttributeFormat::Float4_f32
///     });
///
/// let layout_descriptor = builder.finish();
/// ```
pub struct VertexAttributeLayoutDescriptorBuilder {
    initial_bind_slot: Option<BindSlot>,
    layout: Vec<LayoutElement>,
}

impl VertexAttributeLayoutDescriptorBuilder {
    /// Creates a new builder.
    ///
    /// Takes an optional `allocation_hint` to help reduce the number of allocations without over-
    /// allocating. With an accurate allocation hint the builder will only allocate once. See
    /// [AttributeLayoutAllocationHint] for details.
    pub fn new(allocation_hint: Option<AttributeLayoutAllocationHint>) -> Self {
        let layout = if let Some(hint) = allocation_hint {
            Vec::with_capacity((hint.bind_slot_count - 1 + hint.attribute_count) as usize)
        } else {
            Vec::new()
        };

        VertexAttributeLayoutDescriptorBuilder {
            initial_bind_slot: None,
            layout,
        }
    }

    /// Adds a vertex buffer binding slot to the layout.
    pub fn add_bind_slot(
        &mut self,
        stride: u8,
        input_rate: InputRate,
    ) -> BindSlotAttributeAttacher {
        let bind_slot = BindSlot { stride, input_rate };

        if self.initial_bind_slot.is_none() {
            self.initial_bind_slot = Some(bind_slot);
        } else {
            self.layout.push(LayoutElement::NextBindSlot(bind_slot))
        }

        BindSlotAttributeAttacher {
            stride,
            layout_builder: self,
        }
    }

    /// Finishes building and returns the resulting [VertexAttributeLayoutDescriptor].
    pub fn finish(self) -> VertexAttributeLayoutDescriptor {
        // Setup a hashcode once at pipeline creation time so that future hashing is cheap.
        let hash_code = if let Some(slot) = self.initial_bind_slot {
            let mut hasher = FnvHasher::default();

            slot.hash(&mut hasher);
            self.layout.hash(&mut hasher);

            hasher.finish()
        } else {
            0
        };

        VertexAttributeLayoutDescriptor {
            initial_bind_slot: self.initial_bind_slot,
            layout: self.layout,
            hash_code,
        }
    }
}

/// Returned from [VertexAttributeLayoutDescriptorBuilder::add_bind_slot], attaches
/// [VertexAttributeDescriptor]s to attribute layout bind slots.
pub struct BindSlotAttributeAttacher<'a> {
    stride: u8,
    layout_builder: &'a mut VertexAttributeLayoutDescriptorBuilder,
}

impl<'a> BindSlotAttributeAttacher<'a> {
    /// Adds an attribute descriptor to the current bind slot.
    ///
    /// # Panics
    ///
    /// Panics if the attribute does not fit within one stride (attribute offset + size is greater
    /// bind slot stride).
    pub fn add_attribute(
        &mut self,
        attribute_descriptor: VertexAttributeDescriptor,
    ) -> &mut BindSlotAttributeAttacher<'a> {
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
pub enum IncompatibleAttributeLayout {
    /// Variant returned if no attribute data is available for the [AttributeSlotDescriptor] with
    /// at the `location`.
    MissingAttribute { location: u32 },

    /// Variant returned if attribute data is available for the [AttributeSlotDescriptor] with
    /// at the `location`. but attribute data is not compatible with the [AttributeType] of the
    /// [AttributeSlotDescriptor] (see [AttributeSlotDescriptor::attribute_type]).
    TypeMismatch { location: u32 },
}

/// Describes an input slot on a [GraphicsPipeline].
pub struct AttributeSlotDescriptor {
    pub(crate) location: u32,
    pub(crate) attribute_type: AttributeType,
}

impl AttributeSlotDescriptor {
    /// The shader location of the attribute slot.
    pub fn location(&self) -> u32 {
        self.location
    }

    /// The type of attribute required to fill the slot.
    pub fn attribute_type(&self) -> AttributeType {
        self.attribute_type
    }
}

/// Enumerates the possible attribute types that might be required to fill an attribute slot.
///
/// See also [AttributeSlotDescriptor].
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum AttributeType {
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

impl AttributeType {
    pub(crate) fn from_type_id(id: u32) -> Self {
        match id {
            Gl::FLOAT => AttributeType::Float,
            Gl::FLOAT_VEC2 => AttributeType::FloatVector2,
            Gl::FLOAT_VEC3 => AttributeType::FloatVector3,
            Gl::FLOAT_VEC4 => AttributeType::FloatVector4,
            Gl::FLOAT_MAT2 => AttributeType::FloatMatrix2x2,
            Gl::FLOAT_MAT3 => AttributeType::FloatMatrix3x3,
            Gl::FLOAT_MAT4 => AttributeType::FloatMatrix4x4,
            Gl::FLOAT_MAT2X3 => AttributeType::FloatMatrix2x3,
            Gl::FLOAT_MAT2X4 => AttributeType::FloatMatrix2x4,
            Gl::FLOAT_MAT3X2 => AttributeType::FloatMatrix3x2,
            Gl::FLOAT_MAT3X4 => AttributeType::FloatMatrix3x4,
            Gl::FLOAT_MAT4X2 => AttributeType::FloatMatrix4x2,
            Gl::FLOAT_MAT4X3 => AttributeType::FloatMatrix4x3,
            Gl::INT => AttributeType::Integer,
            Gl::INT_VEC2 => AttributeType::IntegerVector2,
            Gl::INT_VEC3 => AttributeType::IntegerVector3,
            Gl::INT_VEC4 => AttributeType::IntegerVector4,
            Gl::UNSIGNED_INT => AttributeType::UnsignedInteger,
            Gl::UNSIGNED_INT_VEC2 => AttributeType::UnsignedIntegerVector2,
            Gl::UNSIGNED_INT_VEC3 => AttributeType::UnsignedIntegerVector3,
            Gl::UNSIGNED_INT_VEC4 => AttributeType::UnsignedIntegerVector4,
            id => panic!("Invalid attribute type id: {}", id),
        }
    }
}
