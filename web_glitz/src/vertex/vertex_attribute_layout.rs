use std::borrow::Borrow;
use std::mem;

use crate::pipeline::graphics::{AttributeSlotDescriptor, IncompatibleAttributeLayout};
use crate::vertex::{InputRate, Vertex, VertexAttributeDescriptor};
use fnv::FnvHasher;
use std::hash::{Hash, Hasher};

/// Describes a [VertexAttributeLayout] attached to a type.
///
/// The attribute layout is described by sequence of grouped [VertexAttributeDescriptor]s.
/// Together with a sequence of [VertexInputDescriptor]s, these may be used to describe the vertex
/// input state for a [VertexArray], see [VertexInputStateDescription].
///
/// # Unsafe
///
/// The value returned by [input_attribute_bindings] must describe the same attribute layout on
/// every invocation.
pub unsafe trait TypedVertexAttributeLayout {
    type LayoutDescription: Into<VertexAttributeLayoutDescriptor>;

    /// Returns a sequence of grouped [VertexAttributeDescriptor]s.
    ///
    /// Together with a sequence of [VertexInputDescriptor]s, these may be used to describe the
    /// vertex input state for a [VertexArray], see [VertexInputStateDescription].
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

pub struct StaticBindSlotDescriptor {
    pub stride: u8,
    pub input_rate: InputRate,
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

                let mut builder =
                    VertexAttributeLayoutDescriptorBuilder::new(Some(AllocationHint {
                        bind_slot_count: $n,
                        attribute_count: attribute_count as u8,
                    }));

                for i in 0..$n {
                    let mut slot = builder.add_bind_slot(self[i].stride, self[i].input_rate);

                    for attribute in self[i].attributes {
                        slot.add_attribute(*attribute)
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

pub struct BindSlotRef<'a> {
    layout: &'a VertexAttributeLayoutDescriptor,
    start: usize,
    stride: u8,
    input_rate: InputRate,
}

impl<'a> BindSlotRef<'a> {
    pub fn stride_in_bytes(&self) -> u8 {
        self.stride
    }

    pub fn input_rate(&self) -> InputRate {
        self.input_rate
    }

    pub fn attributes(&self) -> BindSlotAttributes {
        BindSlotAttributes {
            layout: &self.layout.layout,
            cursor: self.start,
        }
    }
}

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

pub struct VertexAttributeLayoutDescriptorBuilder {
    initial_bind_slot: Option<BindSlot>,
    layout: Vec<LayoutElement>,
}

#[derive(Clone, Copy, PartialEq, Hash, Debug)]
struct BindSlot {
    stride: u8,
    input_rate: InputRate,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct AllocationHint {
    pub bind_slot_count: u8,
    pub attribute_count: u8,
}

impl VertexAttributeLayoutDescriptorBuilder {
    pub fn new(allocation_hint: Option<AllocationHint>) -> Self {
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

pub struct BindSlotAttributeAttacher<'a> {
    stride: u8,
    layout_builder: &'a mut VertexAttributeLayoutDescriptorBuilder,
}

impl<'a> BindSlotAttributeAttacher<'a> {
    pub fn add_attribute(&mut self, attribute_descriptor: VertexAttributeDescriptor) {
        let size = attribute_descriptor.format.size_in_bytes();

        if attribute_descriptor.offset_in_bytes + size > self.stride {
            panic!("Attribute does not fit within stride.");
        }

        self.layout_builder
            .layout
            .push(LayoutElement::NextAttribute(attribute_descriptor));
    }

    pub unsafe fn add_attribute_unchecked(
        &mut self,
        attribute_descriptor: VertexAttributeDescriptor,
    ) {
        self.layout_builder
            .layout
            .push(LayoutElement::NextAttribute(attribute_descriptor));
    }

    pub fn finish(self) -> &'a mut VertexAttributeLayoutDescriptorBuilder {
        self.layout_builder
    }
}
