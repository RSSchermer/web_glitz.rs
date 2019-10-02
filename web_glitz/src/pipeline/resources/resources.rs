use std::borrow::Borrow;
use std::hash::{Hash, Hasher};
use std::ops::{Deref, Range};

use crate::buffer::{Buffer, BufferView};
use crate::image::texture_2d::{
    FloatSampledTexture2D, IntegerSampledTexture2D, ShadowSampledTexture2D,
    UnsignedIntegerSampledTexture2D,
};
use crate::image::texture_2d_array::{
    FloatSampledTexture2DArray, IntegerSampledTexture2DArray, ShadowSampledTexture2DArray,
    UnsignedIntegerSampledTexture2DArray,
};
use crate::image::texture_3d::{
    FloatSampledTexture3D, IntegerSampledTexture3D, UnsignedIntegerSampledTexture3D,
};
use crate::image::texture_cube::{
    FloatSampledTextureCube, IntegerSampledTextureCube, ShadowSampledTextureCube,
    UnsignedIntegerSampledTextureCube,
};
use crate::pipeline::interface_block::{InterfaceBlock, MemoryUnit};
use crate::pipeline::resources::resource_bindings_encoding::{
    BindGroupEncoding, BindGroupEncodingContext, ResourceBindingDescriptor,
};
use crate::pipeline::resources::resource_slot::{IncompatibleInterface, SlotType};
use crate::pipeline::resources::{
    BindGroupDescriptor, BindGroupEncoder, ResourceBindingsEncoding,
    ResourceBindingsEncodingContext, StaticResourceBindingsEncoder,
};
use fnv::FnvHasher;
use std::marker;
use std::sync::Arc;

pub struct BindGroup<T> {
    pub(crate) context_id: usize,
    pub(crate) encoding: Arc<Vec<ResourceBindingDescriptor>>,
    _marker: marker::PhantomData<T>,
}

impl<T> BindGroup<T>
where
    T: BindableResourceGroup,
{
    pub(crate) fn new(context_id: usize, resources: T) -> Self {
        let mut encoding_context = BindGroupEncodingContext::new(context_id);
        let encoding = resources.encode_bind_group(&mut encoding_context);

        BindGroup {
            context_id,
            encoding: Arc::new(encoding.bindings),
            _marker: marker::PhantomData,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ResourceBindingsLayoutDescriptor {
    layout: Vec<LayoutElement>,
    bind_groups: usize,
}

impl ResourceBindingsLayoutDescriptor {
    pub fn bind_groups(&self) -> BindGroupLayouts {
        BindGroupLayouts { layout: self }
    }

    pub(crate) fn key(&self) -> u64 {
        let mut hasher = FnvHasher::default();

        for element in self.layout.iter() {
            match element {
                LayoutElement::Slot(descriptor) => descriptor.hash(&mut hasher),
                LayoutElement::NextBindGroup(element) => element.bind_group_index.hash(&mut hasher),
            }
        }

        hasher.finish()
    }
}

#[derive(Clone, Debug)]
enum LayoutElement {
    Slot(ResourceSlotDescriptor),
    NextBindGroup(BindGroupElement),
}

#[derive(Clone, Debug)]
struct BindGroupElement {
    bind_group_index: u32,
    len: usize,
}

#[derive(Clone, Copy)]
pub struct BindGroupLayouts<'a> {
    layout: &'a ResourceBindingsLayoutDescriptor,
}

impl<'a> BindGroupLayouts<'a> {
    pub fn len(&self) -> usize {
        self.layout.bind_groups
    }

    pub fn iter(&self) -> BindGroupLayoutsIter {
        BindGroupLayoutsIter {
            layout: &self.layout.layout,
            cursor: 0,
            len: self.layout.bind_groups,
        }
    }
}

impl<'a> IntoIterator for BindGroupLayouts<'a> {
    type Item = BindGroupLayout<'a>;
    type IntoIter = BindGroupLayoutsIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        BindGroupLayoutsIter {
            layout: &self.layout.layout,
            cursor: 0,
            len: self.layout.bind_groups,
        }
    }
}

pub struct BindGroupLayoutsIter<'a> {
    layout: &'a [LayoutElement],
    cursor: usize,
    len: usize,
}

impl<'a> Iterator for BindGroupLayoutsIter<'a> {
    type Item = BindGroupLayout<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.layout.get(self.cursor).map(|element| {
            if let LayoutElement::NextBindGroup(element) = element {
                let start = self.cursor as usize + 1;
                let end = start + element.len;

                self.cursor = end;
                self.len -= 1;

                BindGroupLayout {
                    slots: &self.layout[start..end],
                    bind_group_index: element.bind_group_index,
                }
            } else {
                unreachable!()
            }
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<'a> ExactSizeIterator for BindGroupLayoutsIter<'a> {
    fn len(&self) -> usize {
        self.len
    }
}

pub struct BindGroupLayout<'a> {
    slots: &'a [LayoutElement],
    bind_group_index: u32,
}

impl<'a> BindGroupLayout<'a> {
    pub fn bind_group_index(&self) -> u32 {
        self.bind_group_index
    }

    pub fn slots(&self) -> BindGroupLayoutSlots {
        BindGroupLayoutSlots { slots: self.slots }
    }
}

pub struct BindGroupLayoutSlots<'a> {
    slots: &'a [LayoutElement],
}

impl<'a> BindGroupLayoutSlots<'a> {
    pub fn len(&self) -> usize {
        self.slots.len()
    }

    pub fn iter(&self) -> BindGroupSlotsIter {
        BindGroupSlotsIter {
            iter: self.slots.iter(),
        }
    }
}

impl<'a> IntoIterator for BindGroupLayoutSlots<'a> {
    type Item = &'a ResourceSlotDescriptor;
    type IntoIter = BindGroupSlotsIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        BindGroupSlotsIter {
            iter: self.slots.into_iter(),
        }
    }
}

pub struct BindGroupSlotsIter<'a> {
    iter: std::slice::Iter<'a, LayoutElement>,
}

impl<'a> Iterator for BindGroupSlotsIter<'a> {
    type Item = &'a ResourceSlotDescriptor;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|element| {
            if let LayoutElement::Slot(slot) = element {
                slot
            } else {
                unreachable!()
            }
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a> ExactSizeIterator for BindGroupSlotsIter<'a> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ResourceBindingsLayoutDescriptorAllocationHint {
    pub bind_groups: usize,
    pub total_resource_slots: usize,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct InvalidBindGroupIndex;

pub struct ResourceBindingsLayoutDescriptorBuilder {
    layout: Vec<LayoutElement>,
    last_bind_group_index: Option<u32>,
}

impl ResourceBindingsLayoutDescriptorBuilder {
    pub fn new(allocation_hint: Option<ResourceBindingsLayoutDescriptorAllocationHint>) -> Self {
        let layout = if let Some(hint) = allocation_hint {
            Vec::with_capacity(hint.bind_groups + hint.total_resource_slots)
        } else {
            Vec::new()
        };

        ResourceBindingsLayoutDescriptorBuilder {
            layout,
            last_bind_group_index: None,
        }
    }

    pub fn add_bind_group(
        &mut self,
        bind_group_index: u32,
    ) -> Result<BindGroupLayoutBuilder, InvalidBindGroupIndex> {
        if let Some(last_bind_group_index) = self.last_bind_group_index {
            if bind_group_index <= last_bind_group_index {
                return Err(InvalidBindGroupIndex);
            }
        }

        let start = self.layout.len();

        self.layout
            .push(LayoutElement::NextBindGroup(BindGroupElement {
                bind_group_index,
                len: 0,
            }));

        Ok(BindGroupLayoutBuilder {
            layout: &mut self.layout,
            bind_group_index,
            start,
            last_slot_index: None,
        })
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct InvalidSlotIndex;

pub struct BindGroupLayoutBuilder<'a> {
    layout: &'a mut Vec<LayoutElement>,
    bind_group_index: u32,
    start: usize,
    last_slot_index: Option<u32>,
}

impl<'a> BindGroupLayoutBuilder<'a> {
    pub fn add_resource_slot(
        mut self,
        descriptor: ResourceSlotDescriptor,
    ) -> Result<Self, InvalidSlotIndex> {
        if let Some(last_slot_index) = self.last_slot_index {
            if descriptor.slot_index <= last_slot_index {
                return Err(InvalidSlotIndex);
            }
        }

        self.layout.push(LayoutElement::Slot(descriptor));

        Ok(self)
    }
}

impl<'a> Drop for BindGroupLayoutBuilder<'a> {
    fn drop(&mut self) {
        let len = self.layout.len() - self.start;

        self.layout[self.start] = LayoutElement::NextBindGroup(BindGroupElement {
            bind_group_index: self.bind_group_index,
            len,
        });
    }
}

/// A typed description of the resource binding slots used by a pipeline.
///
/// This type includes description of the exact resource type used for each resource slot, which may
/// be checked against the resource types defined by the pipeline's shader stages.
///
/// See also [ResourceBindingsLayoutDescriptor] a descriptor that only includes the minimum of
/// information necessary to initialize a pipeline.
#[derive(Clone, Debug)]
pub struct TypedResourceBindingsLayoutDescriptor {
    bind_groups: &'static [TypedBindGroupLayoutDescriptor],
}

impl TypedResourceBindingsLayoutDescriptor {
    /// Creates a new [TypedBindGroupLayoutDescriptor] contains the specified `bind_groups`.
    ///
    /// # Unsafe
    ///
    /// The bind groups must be ordered by their bind group index (see
    /// [TypedBindGroupLayoutDescriptor::bind_group_index]) in ascending order and there must not
    /// be multiple bind groups that use the same bind group index.
    pub const unsafe fn new(bind_groups: &'static [TypedBindGroupLayoutDescriptor]) -> Self {
        TypedResourceBindingsLayoutDescriptor { bind_groups }
    }

    /// Creates a new empty [TypedResourceBindingsLayoutDescriptor] without any bind groups.
    pub const fn empty() -> Self {
        TypedResourceBindingsLayoutDescriptor { bind_groups: &[] }
    }

    /// The bind groups specified for this layout.
    pub fn bind_groups(&self) -> &[TypedBindGroupLayoutDescriptor] {
        &self.bind_groups
    }

    pub(crate) fn key(&self) -> u64 {
        let mut hasher = FnvHasher::default();

        for bind_group in self.bind_groups.iter() {
            bind_group.bind_group_index.hash(&mut hasher);

            for slot in bind_group.resource_slots.iter() {
                let minimal: ResourceSlotDescriptor = slot.clone().into();

                minimal.hash(&mut hasher)
            }
        }

        hasher.finish()
    }
}

/// Describes the layout of a single bind group in a [TypedResourceBindingsLayoutDescriptor].
#[derive(Clone, Debug)]
pub struct TypedBindGroupLayoutDescriptor {
    bind_group_index: u32,
    resource_slots: &'static [TypedResourceSlotDescriptor],
}

impl TypedBindGroupLayoutDescriptor {
    /// Creates a new [TypedBindGroupLayoutDescriptor] that will use the specified
    /// `bind_group_index` and defines the specified `resource_slots`.
    ///
    /// # Unsafe
    ///
    /// The resource slots must by ordered by their slot index (see
    /// [TypedResourceSlotDescriptor::slot_index]) in ascending order and there must not be 2
    /// descriptors for the same slot index.
    pub const unsafe fn new(
        bind_group_index: u32,
        resource_slots: &'static [TypedResourceSlotDescriptor],
    ) -> Self {
        TypedBindGroupLayoutDescriptor {
            bind_group_index,
            resource_slots,
        }
    }

    /// The bind group index used to attach the bind group.
    pub fn bind_group_index(&self) -> u32 {
        self.bind_group_index
    }

    /// The resource slots provided by the bind group.
    pub fn slots(&self) -> &[TypedResourceSlotDescriptor] {
        &self.resource_slots
    }
}

/// A resource bindings layout description attached to a type.
///
/// See also [TypedResourceBindingsLayoutDescriptor].
///
/// This trait becomes useful in combination with the [TypedResourceBindings] trait. If a
/// [TypedResourceBindingsLayout] is attached to a [GraphicsPipeline] (see
/// [GraphicsPipelineDescriptorBuilder::typed_resource_bindings_layout]), then
/// [TypedResourceBindings] with a matching [TypedResourceBindings::Layout] may be bound to the
/// pipeline without further runtime checks.
///
/// Note that [TypedResourceBindingsLayout] is safe to implement, but implementing
/// [TypedResourceBindings] is unsafe: the resource bindings encoded by a [TypedResourceBindings]
/// implementation must always be compatible with the bindings layout specified by its
/// [TypedResourceBindings::Layout], see [TypedResourceBindings] for details.
pub trait TypedResourceBindingsLayout {
    const LAYOUT: TypedResourceBindingsLayoutDescriptor;
}

macro_rules! implement_typed_resource_bindings_layout {
    ($($T:ident: $i:tt),*) => {
        impl<$($T),*> TypedResourceBindingsLayout for ($($T),*)
        where
            $($T: TypedBindGroupLayout),*
        {
            const LAYOUT: TypedResourceBindingsLayoutDescriptor = unsafe {
                TypedResourceBindingsLayoutDescriptor::new(&[
                    $(TypedBindGroupLayoutDescriptor::new($i, $T::LAYOUT)),*
                ])
            };
        }
    }
}

implement_typed_resource_bindings_layout!(T0: 0);
implement_typed_resource_bindings_layout!(T0: 0, T1: 1);
implement_typed_resource_bindings_layout!(T0: 0, T1: 1, T2: 2);
implement_typed_resource_bindings_layout!(T0: 0, T1: 1, T2: 2, T3: 3);
implement_typed_resource_bindings_layout!(T0: 0, T1: 1, T2: 2, T3: 3, T4: 4);
implement_typed_resource_bindings_layout!(T0: 0, T1: 1, T2: 2, T3: 3, T4: 4, T5: 5);
implement_typed_resource_bindings_layout!(T0: 0, T1: 1, T2: 2, T3: 3, T4: 4, T5: 5, T6: 6);
implement_typed_resource_bindings_layout!(T0: 0, T1: 1, T2: 2, T3: 3, T4: 4, T5: 5, T6: 6, T7: 7);
implement_typed_resource_bindings_layout!(T0: 0, T1: 1, T2: 2, T3: 3, T4: 4, T5: 5, T6: 6, T7: 7, T8: 8);
implement_typed_resource_bindings_layout!(T0: 0, T1: 1, T2: 2, T3: 3, T4: 4, T5: 5, T6: 6, T7: 7, T8: 8, T9: 9);
implement_typed_resource_bindings_layout!(T0: 0, T1: 1, T2: 2, T3: 3, T4: 4, T5: 5, T6: 6, T7: 7, T8: 8, T9: 9, T10: 10);
implement_typed_resource_bindings_layout!(T0: 0, T1: 1, T2: 2, T3: 3, T4: 4, T5: 5, T6: 6, T7: 7, T8: 8, T9: 9, T10: 10, T11: 11);
implement_typed_resource_bindings_layout!(T0: 0, T1: 1, T2: 2, T3: 3, T4: 4, T5: 5, T6: 6, T7: 7, T8: 8, T9: 9, T10: 10, T11: 11, T12: 12);
implement_typed_resource_bindings_layout!(T0: 0, T1: 1, T2: 2, T3: 3, T4: 4, T5: 5, T6: 6, T7: 7, T8: 8, T9: 9, T10: 10, T11: 11, T12: 12, T13: 13);
implement_typed_resource_bindings_layout!(T0: 0, T1: 1, T2: 2, T3: 3, T4: 4, T5: 5, T6: 6, T7: 7, T8: 8, T9: 9, T10: 10, T11: 11, T12: 12, T13: 13, T14: 14);
implement_typed_resource_bindings_layout!(T0: 0, T1: 1, T2: 2, T3: 3, T4: 4, T5: 5, T6: 6, T7: 7, T8: 8, T9: 9, T10: 10, T11: 11, T12: 12, T13: 13, T14: 14, T15: 15);

pub trait TypedBindGroupLayout {
    const LAYOUT: &'static [TypedResourceSlotDescriptor];
}

/// A group of resources that may be used to form a [BindGroup].
///
/// See also [RenderingContext::create_bind_group] for details on how a [BindGroup] is created.
pub trait BindableResourceGroup {
    /// Encodes a description of the bindings for the resources in the group.
    fn encode_bind_group(
        self,
        encoding_context: &mut BindGroupEncodingContext,
    ) -> BindGroupEncoding;
}


pub unsafe trait TypedBindableResourceGroup {
    type Layout: TypedBindGroupLayout;
}

/// Encodes a description of how a set of resources is bound to a pipeline, such that the pipeline
/// may access these resources during its execution.
///
/// # Example
///
/// ```
/// use web_glitz::buffer::Buffer;
/// use web_glitz::image::texture_2d::FloatSampledTexture2D;
/// use web_glitz::pipeline::resources::{ResourceBindings, ResourceBindingsEncodingContext, ResourceBindingsEncoding, StaticResourceBindingsEncoder, BindGroup, BindGroupDescriptor};
///
/// struct Resources<T0, T1> {
///     bind_group_0: BindGroup<T0>,
///     bind_group_1: BindGroup<T1>,
/// }
///
/// impl<T0, T1> ResourceBindings for &'_ Resources<T0, T1> {
///     type BindGroups = [BindGroupDescriptor; 2];
///
///     fn encode(
///         self,
///         encoding_context: &mut ResourceBindingsEncodingContext
///     ) -> ResourceBindingsEncoding<Self::BindGroups> {
///         let encoder = StaticResourceBindingsEncoder::new(encoding_context);
///
///         let encoder = encoder.add_bind_group(0, &self.bind_group_0);
///         let encoder = encoder.add_bind_group(1, &self.bind_group_1);
///
///         encoder.finish()
///     }
/// }
/// ```
///
/// See also [StaticResourceBindingsEncoder]. Note that when multiple bindings of the same type bind
/// to the same slot-index, then only the binding that was added last will be used. However, buffer
/// resources and texture resources belong to distinct bind groups, their slot-indices do not
/// interact.
///
/// This trait is automically implemented for any type that derives the [Resources] trait.
pub trait ResourceBindings {
    /// Type that describes the collection of bindings.
    type BindGroups: Borrow<[BindGroupDescriptor]> + 'static;

    /// Encodes a description of how this set of resources is bound to a pipeline.
    fn encode(
        self,
        encoding_context: &mut ResourceBindingsEncodingContext,
    ) -> ResourceBindingsEncoding<Self::BindGroups>;
}

macro_rules! implement_resource_bindings {
    ($n:tt, $($T:ident: $i:tt),*) => {
        impl<$($T),*> ResourceBindings for ($(&'_ BindGroup<$T>),*) {
            type BindGroups = [BindGroupDescriptor; $n];

            fn encode(self, encoding_context: &mut ResourceBindingsEncodingContext) -> ResourceBindingsEncoding<Self::BindGroups> {
                let encoder = StaticResourceBindingsEncoder::new(encoding_context);

                #[allow(unused_parens, non_snake_case)]
                let ($($T),*) = self;

                $(let encoder = encoder.add_bind_group($i, $T);)*

                encoder.finish()
            }
        }
    }
}

implement_resource_bindings!(1, T0: 0);
implement_resource_bindings!(2, T0: 0, T1: 1);
implement_resource_bindings!(3, T0: 0, T1: 1, T2: 2);
implement_resource_bindings!(4, T0: 0, T1: 1, T2: 2, T3: 3);
implement_resource_bindings!(5, T0: 0, T1: 1, T2: 2, T3: 3, T4: 4);
implement_resource_bindings!(6, T0: 0, T1: 1, T2: 2, T3: 3, T4: 4, T5: 5);
implement_resource_bindings!(7, T0: 0, T1: 1, T2: 2, T3: 3, T4: 4, T5: 5, T6: 6);
implement_resource_bindings!(8, T0: 0, T1: 1, T2: 2, T3: 3, T4: 4, T5: 5, T6: 6, T7: 7);
implement_resource_bindings!(9, T0: 0, T1: 1, T2: 2, T3: 3, T4: 4, T5: 5, T6: 6, T7: 7, T8: 8);
implement_resource_bindings!(10, T0: 0, T1: 1, T2: 2, T3: 3, T4: 4, T5: 5, T6: 6, T7: 7, T8: 8, T9: 9);
implement_resource_bindings!(11, T0: 0, T1: 1, T2: 2, T3: 3, T4: 4, T5: 5, T6: 6, T7: 7, T8: 8, T9: 9, T10: 10);
implement_resource_bindings!(12, T0: 0, T1: 1, T2: 2, T3: 3, T4: 4, T5: 5, T6: 6, T7: 7, T8: 8, T9: 9, T10: 10, T11: 11);
implement_resource_bindings!(13, T0: 0, T1: 1, T2: 2, T3: 3, T4: 4, T5: 5, T6: 6, T7: 7, T8: 8, T9: 9, T10: 10, T11: 11, T12: 12);
implement_resource_bindings!(14, T0: 0, T1: 1, T2: 2, T3: 3, T4: 4, T5: 5, T6: 6, T7: 7, T8: 8, T9: 9, T10: 10, T11: 11, T12: 12, T13: 13);
implement_resource_bindings!(15, T0: 0, T1: 1, T2: 2, T3: 3, T4: 4, T5: 5, T6: 6, T7: 7, T8: 8, T9: 9, T10: 10, T11: 11, T12: 12, T13: 13, T14: 14);
implement_resource_bindings!(16, T0: 0, T1: 1, T2: 2, T3: 3, T4: 4, T5: 5, T6: 6, T7: 7, T8: 8, T9: 9, T10: 10, T11: 11, T12: 12, T13: 13, T14: 14, T15: 15);

/// Sub-trait of [ResourceBindings], where a type statically describes its resource bindings layout.
///
/// Resource bindings that implement this trait may be bound to graphics pipelines with a matching
/// [TypedResourceBindingsLayout] without further runtime checks.
///
/// # Unsafe
///
/// This trait must only by implemented for [ResourceBindings] types if the recourse bindings
/// encoding for any instance of the the type is guaranteed to compatible with the resource slots
/// on a pipeline that matches the [Layout].
pub unsafe trait TypedResourceBindings: ResourceBindings {
    /// A type statically associated with a resource bindings layout with which the encoding of
    /// any instance of these [TypedResourceBindings] is compatible.
    type Layout: TypedResourceBindingsLayout;
}

macro_rules! impl_typed_resource_bindings {
    ($($T:ident),*) => {
        unsafe impl<$($T),*> TypedResourceBindings for ($(&'_ BindGroup<$T>),*)
        where
            $($T: TypedBindableResourceGroup),*
        {
            type Layout = ($($T::Layout),*);
        }
    }
}

impl_typed_resource_bindings!(T0);
impl_typed_resource_bindings!(T0, T1);
impl_typed_resource_bindings!(T0, T1, T2);
impl_typed_resource_bindings!(T0, T1, T2, T3);
impl_typed_resource_bindings!(T0, T1, T2, T3, T4);
impl_typed_resource_bindings!(T0, T1, T2, T3, T4, T5);
impl_typed_resource_bindings!(T0, T1, T2, T3, T4, T5, T6);
impl_typed_resource_bindings!(T0, T1, T2, T3, T4, T5, T6, T7);
impl_typed_resource_bindings!(T0, T1, T2, T3, T4, T5, T6, T7, T8);
impl_typed_resource_bindings!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9);
impl_typed_resource_bindings!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
impl_typed_resource_bindings!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);
impl_typed_resource_bindings!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12);
impl_typed_resource_bindings!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13);
impl_typed_resource_bindings!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14);
impl_typed_resource_bindings!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15);

/// A minimal description of the resource binding slots used by a pipeline.
///
/// This type only contains the minimally necessary information for initializing a pipeline. See
/// also [TypedResourceBindingsLayoutDescriptor] for a type that includes information that may be
/// type checked against the resource types defined by the pipeline's shader stages.
//#[derive(Clone, Debug)]
//pub struct ResourceBindingsLayoutDescriptor {
//    pub(crate) bindings: Vec<ResourceSlotDescriptor>,
//}

/// Identifies a resource slot in a pipeline.
#[derive(Clone, Debug)]
pub enum ResourceSlotIdentifier {
    Static(&'static str),
    Dynamic(String),
}

impl From<&'static str> for ResourceSlotIdentifier {
    fn from(value: &'static str) -> Self {
        ResourceSlotIdentifier::Static(value)
    }
}

impl From<String> for ResourceSlotIdentifier {
    fn from(value: String) -> Self {
        ResourceSlotIdentifier::Dynamic(value)
    }
}

impl PartialEq for ResourceSlotIdentifier {
    fn eq(&self, other: &Self) -> bool {
        self.deref() == other.deref()
    }
}

impl Hash for ResourceSlotIdentifier {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let as_str: &str = self.deref();

        as_str.hash(state);
    }
}

impl Deref for ResourceSlotIdentifier {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        match self {
            ResourceSlotIdentifier::Static(s) => s,
            ResourceSlotIdentifier::Dynamic(s) => s,
        }
    }
}

/// Describes a single resource slot in a pipeline.
///
/// See also [ResourceBindingsLayoutDescriptor].
#[derive(Clone, Hash, PartialEq, Debug)]
pub struct ResourceSlotDescriptor {
    /// The identifier for the slot.
    pub slot_identifier: ResourceSlotIdentifier,

    /// The index of the slot.
    pub slot_index: u32,

    /// The kind of resource slot.
    pub slot_kind: ResourceSlotKind,
}

impl From<TypedResourceSlotDescriptor> for ResourceSlotDescriptor {
    fn from(descriptor: TypedResourceSlotDescriptor) -> Self {
        let TypedResourceSlotDescriptor {
            slot_identifier,
            slot_index,
            slot_type,
        } = descriptor;

        ResourceSlotDescriptor {
            slot_identifier,
            slot_index,
            slot_kind: slot_type.into(),
        }
    }
}

/// Enumerates the different kinds of resource slots a pipeline can define.
///
/// See also [ResourceSlotDescriptor].
#[derive(Clone, Copy, Hash, PartialEq, Debug)]
pub enum ResourceSlotKind {
    // A WebGPU version would add `has_dynamic_offset`.
    UniformBuffer,
    // A WebGPU version would add `dimensionality`, `component_type` and `is_multisampled`.
    SampledTexture,
}

impl ResourceSlotKind {
    /// Whether or not this is a uniform buffer slot.
    pub fn is_uniform_buffer(&self) -> bool {
        if let ResourceSlotKind::UniformBuffer = self {
            true
        } else {
            false
        }
    }

    /// Whether or not this is a sampled-texture slot.
    pub fn is_sampled_texture(&self) -> bool {
        if let ResourceSlotKind::UniformBuffer = self {
            true
        } else {
            false
        }
    }
}

impl From<ResourceSlotType> for ResourceSlotKind {
    fn from(slot_type: ResourceSlotType) -> Self {
        match slot_type {
            ResourceSlotType::UniformBuffer(_) => ResourceSlotKind::UniformBuffer,
            ResourceSlotType::SampledTexture(_) => ResourceSlotKind::SampledTexture,
        }
    }
}

/// Describes a single resource slot in a pipeline and its type.
///
/// See also [TypedResourceBindingsLayoutDescriptor].
#[derive(Clone, PartialEq, Debug)]
pub struct TypedResourceSlotDescriptor {
    /// The identifier for the slot.
    pub slot_identifier: ResourceSlotIdentifier,

    /// The index of the slot.
    pub slot_index: u32,

    /// The type of the slot.
    pub slot_type: ResourceSlotType,
}

/// Enumerates the slot types for a [TypedResourceSlotDescriptor].
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ResourceSlotType {
    /// A uniform buffer slot and its memory layout as a collection of [MemoryUnit]s.
    // A WebGPU version would add `has_dynamic_offset`.
    UniformBuffer(&'static [MemoryUnit]),
    /// A sampled-texture slot and it's [SampledTextureType].
    // A WebGPU version would add `dimensionality`, `component_type` and `is_multisampled`.
    SampledTexture(SampledTextureType),
}

/// Enumerates the types available for sampled-texture resource slot.
#[derive(Clone, Copy, Hash, PartialEq, Debug)]
pub enum SampledTextureType {
    FloatSampler2D,
    IntegerSampler2D,
    UnsignedIntegerSampler2D,
    FloatSampler2DArray,
    IntegerSampler2DArray,
    UnsignedIntegerSampler2DArray,
    FloatSampler3D,
    IntegerSampler3D,
    UnsignedIntegerSampler3D,
    FloatSamplerCube,
    IntegerSamplerCube,
    UnsignedIntegerSamplerCube,
    Sampler2DShadow,
    Sampler2DArrayShadow,
    SamplerCubeShadow,
}

/// Provides a group of resources (uniform block buffers, sampled textures) that may be bound to a
/// pipeline, such that the pipeline may access these resources during its execution.
///
/// This type acts as an automatically derivable trait for a [TypedResourceBindings] type that acts
/// as its own [TypedResourceBindingsLayout]. This trait is only intended to be derived
/// automatically; if your set of resources cannot be adequatly described by automatically deriving
/// this trait, rather than manually implementing this trait, instead consider manually implementing
/// the [TypedResourceBindings] and [TypedResourceBindingsLayout] traits separately.
///
/// # Usage
///
/// The programmable stages of a pipeline may require access to various resources during pipeline
/// execution. To this end, the code for a programmable stage may define slots for such resources
/// (e.g. uniform  blocks, texture samplers):
///
/// ```glsl
/// #version 300 es
///
/// uniform sampler2D some_texture;
///
/// uniform sampler2DArray some_other_texture;
///
/// layout(std140) uniform SomeUniformBlock {
///     vec4 some_uniform;
///     mat4 some_other_uniform;
/// };
///
/// main () {
///     ...
/// }
/// ```
///
/// This trait may be safely derived automatically on a type to define how specific resource
/// instances should be bound to the pipeline before it is executed:
///
/// ```
/// use web_glitz::image::texture_2d::FloatSampledTexture2D;
/// use web_glitz::image::texture_2d_array::FloatSampledTexture2DArray;
/// use web_glitz::buffer::Buffer;
/// use web_glitz::std140;
/// use web_glitz::std140::repr_std140;
///
/// #[derive(web_glitz::derive::Resources)]
/// struct MyResources<'a> {
///     #[texture_resource(binding=0)]
///     some_texture: FloatSampledTexture2D<'a>,
///
///     #[texture_resource(binding=1, name="some_other_texture")]
///     second_texture: FloatSampledTexture2DArray<'a>,
///
///     #[buffer_resource(binding=0, name="SomeUniformBlock")]
///     some_uniform_block: &'a Buffer<SomeUniformBlock>
/// }
///
/// #[repr_std140]
/// #[derive(web_glitz::derive::InterfaceBlock)]
/// struct SomeUniformBlock {
///     some_uniform: std140::vec4,
///     some_other_uniform: std140::mat4x4
/// }
/// ```
///
/// Any field marked with a `#[texture_resource(...)]` attribute defines a texture binding; any
/// field marked with a `#[buffer_resource(...)]` attribute defines a buffer binding.
///
/// A `#[texture_resource(...)]` attribute must always declare a `binding` index; this should be a
/// positive integer that is smaller than the [RenderingContext::max_texture_resource_index] for the
/// [RenderingContext] with which you intend to use the [Resources] (hardware dependent, at least
/// `32`). If the field name does not match the resource name used in the shader code, then the
/// attribute should also declare a `name` with a string that does match the resource name used in
/// the shader code; if the field name does match the name used in the shader, then `name` may be
/// omitted. The field's type must implement [TextureResource]; a `#[texture_resource(...)]` field
/// that does not implement [TextureResource] will result in a compilation error. If multiple
/// `#[texture_resource(...)]` fields are defined, then all fields must declare a unique `binding`
/// index; 2 or more `#[texture_resource(...)]` fields with the same `binding` index will result in
/// a compilation error.
///
/// A `#[buffer_resource(...)]` attribute must always declare a `binding` index; this should be a
/// positive integer that is smaller than the [RenderingContext::max_buffer_resource_index] for the
/// [RenderingContext] with which you intend to use the [Resources] (hardware dependent, at least
/// `24`). If the field name does not match the block name used in the shader code, then the
/// attribute should also declare a `name` with a string that does match the block name used in the
/// shader code; if the field name does match the name used in the shader, then `name` may be
/// omitted. The field's type must implement [BufferResource]; a `#[buffer_resource(...)]` field
/// that does not implement [BufferResource] will result in a compilation error. The type contained
/// in the resource must implement [InterfaceBlock] and its memory layout should match the layout
/// expected by the pipeline (see [InterfaceBlock] and [web_glitz::std140]). If multiple
/// `#[buffer_resource(...)]` fields are defined, then all fields must declare a unique `binding`
/// index; 2 or more `#[buffer_resource(...)]` fields with the same `binding` index will result in a
/// compilation error.
///
/// Note that while `binding` indices must be internally unique amongst `#[texture_resource(...)]`
/// fields and must be internally unique amongst `#[buffer_resource(...)]` fields, both binding
/// types use a separate set of bindings: a `#[texture_resource(...)]` field may declare the same
/// `binding` index as a `#[buffer_resource(...)]`.
pub unsafe trait Resources {
    const LAYOUT: &'static [TypedResourceSlotDescriptor];

    fn encode_binding_group(
        self,
        encoding_context: &mut BindGroupEncodingContext,
    ) -> BindGroupEncoding;
}

impl<T> TypedBindGroupLayout for T
where
    T: Resources,
{
    const LAYOUT: &'static [TypedResourceSlotDescriptor] = T::LAYOUT;
}

impl<T> BindableResourceGroup for T
where
    T: Resources,
{
    fn encode_bind_group(
        self,
        encoding_context: &mut BindGroupEncodingContext,
    ) -> BindGroupEncoding {
        <T as Resources>::encode_binding_group(self, encoding_context)
    }
}

unsafe impl<T> TypedBindableResourceGroup for T
where
    T: Resources,
{
    type Layout = T;
}

impl TypedBindGroupLayout for () {
    const LAYOUT: &'static [TypedResourceSlotDescriptor] = &[];
}

impl BindableResourceGroup for () {
    fn encode_bind_group(
        self,
        encoding_context: &mut BindGroupEncodingContext,
    ) -> BindGroupEncoding {
        BindGroupEncoding::empty(encoding_context)
    }
}

unsafe impl TypedBindableResourceGroup for () {
    type Layout = ();
}

/// Error returned when a [ResourceBindingsLayoutDescriptor] or
/// [TypedResourceBindingsLayoutDescriptor] does not match resource slots declared in a pipeline's
/// shader stages.
#[derive(Debug)]
pub enum IncompatibleResources {
    MissingBindGroup(u32),
    MissingResource(ResourceSlotIdentifier),
    ResourceTypeMismatch(ResourceSlotIdentifier),
    IncompatibleInterface(ResourceSlotIdentifier, IncompatibleInterface),
    SlotBindingMismatch { expected: usize, actual: usize },
}

/// Trait implemented for types that can be bound to a pipeline as a resource.
///
/// When automatically deriving the [Resources] trait, fields marked with `#[resource(...)]` must
/// implement this trait.
pub unsafe trait Resource {
    const TYPE: ResourceSlotType;

    /// Encodes a binding for this resource at the specified `slot_index`.
    fn encode<B>(self, slot_index: u32, encoder: &mut BindGroupEncoder);
}

unsafe impl<'a, T> Resource for &'a Buffer<T>
where
    T: InterfaceBlock,
{
    const TYPE: ResourceSlotType = ResourceSlotType::UniformBuffer(T::MEMORY_UNITS);

    fn encode<B>(self, slot_index: u32, encoder: &mut BindGroupEncoder) {
        encoder.add_buffer_view(slot_index, self.into());
    }
}

unsafe impl<'a, T> Resource for BufferView<'a, T>
where
    T: InterfaceBlock,
{
    const TYPE: ResourceSlotType = ResourceSlotType::UniformBuffer(T::MEMORY_UNITS);

    fn encode<B>(self, slot_index: u32, encoder: &mut BindGroupEncoder) {
        encoder.add_buffer_view(slot_index, self);
    }
}

unsafe impl<'a> Resource for FloatSampledTexture2D<'a> {
    const TYPE: ResourceSlotType =
        ResourceSlotType::SampledTexture(SampledTextureType::FloatSampler2D);

    fn encode<B>(self, slot_index: u32, encoder: &mut BindGroupEncoder) {
        encoder.add_float_sampled_texture_2d(slot_index, self);
    }
}

unsafe impl<'a> Resource for FloatSampledTexture2DArray<'a> {
    const TYPE: ResourceSlotType =
        ResourceSlotType::SampledTexture(SampledTextureType::FloatSampler2DArray);

    fn encode<B>(self, slot_index: u32, encoder: &mut BindGroupEncoder) {
        encoder.add_float_sampled_texture_2d_array(slot_index, self);
    }
}

unsafe impl<'a> Resource for FloatSampledTexture3D<'a> {
    const TYPE: ResourceSlotType =
        ResourceSlotType::SampledTexture(SampledTextureType::FloatSampler3D);

    fn encode<B>(self, slot_index: u32, encoder: &mut BindGroupEncoder) {
        encoder.add_float_sampled_texture_3d(slot_index, self);
    }
}

unsafe impl<'a> Resource for FloatSampledTextureCube<'a> {
    const TYPE: ResourceSlotType =
        ResourceSlotType::SampledTexture(SampledTextureType::FloatSamplerCube);

    fn encode<B>(self, slot_index: u32, encoder: &mut BindGroupEncoder) {
        encoder.add_float_sampled_texture_cube(slot_index, self);
    }
}

unsafe impl<'a> Resource for IntegerSampledTexture2D<'a> {
    const TYPE: ResourceSlotType =
        ResourceSlotType::SampledTexture(SampledTextureType::IntegerSampler2D);

    fn encode<B>(self, slot_index: u32, encoder: &mut BindGroupEncoder) {
        encoder.add_integer_sampled_texture_2d(slot_index, self);
    }
}

unsafe impl<'a> Resource for IntegerSampledTexture2DArray<'a> {
    const TYPE: ResourceSlotType =
        ResourceSlotType::SampledTexture(SampledTextureType::IntegerSampler2DArray);

    fn encode<B>(self, slot_index: u32, encoder: &mut BindGroupEncoder) {
        encoder.add_integer_sampled_texture_2d_array(slot_index, self);
    }
}

unsafe impl<'a> Resource for IntegerSampledTexture3D<'a> {
    const TYPE: ResourceSlotType =
        ResourceSlotType::SampledTexture(SampledTextureType::IntegerSampler3D);

    fn encode<B>(self, slot_index: u32, encoder: &mut BindGroupEncoder) {
        encoder.add_integer_sampled_texture_3d(slot_index, self);
    }
}

unsafe impl<'a> Resource for IntegerSampledTextureCube<'a> {
    const TYPE: ResourceSlotType =
        ResourceSlotType::SampledTexture(SampledTextureType::IntegerSamplerCube);

    fn encode<B>(self, slot_index: u32, encoder: &mut BindGroupEncoder) {
        encoder.add_integer_sampled_texture_cube(slot_index, self);
    }
}

unsafe impl<'a> Resource for UnsignedIntegerSampledTexture2D<'a> {
    const TYPE: ResourceSlotType =
        ResourceSlotType::SampledTexture(SampledTextureType::UnsignedIntegerSampler2D);

    fn encode<B>(self, slot_index: u32, encoder: &mut BindGroupEncoder) {
        encoder.add_unsigned_integer_sampled_texture_2d(slot_index, self);
    }
}

unsafe impl<'a> Resource for UnsignedIntegerSampledTexture2DArray<'a> {
    const TYPE: ResourceSlotType =
        ResourceSlotType::SampledTexture(SampledTextureType::UnsignedIntegerSampler2DArray);

    fn encode<B>(self, slot_index: u32, encoder: &mut BindGroupEncoder) {
        encoder.add_unsigned_integer_sampled_texture_2d_array(slot_index, self);
    }
}

unsafe impl<'a> Resource for UnsignedIntegerSampledTexture3D<'a> {
    const TYPE: ResourceSlotType =
        ResourceSlotType::SampledTexture(SampledTextureType::UnsignedIntegerSampler3D);

    fn encode<B>(self, slot_index: u32, encoder: &mut BindGroupEncoder) {
        encoder.add_unsigned_integer_sampled_texture_3d(slot_index, self);
    }
}

unsafe impl<'a> Resource for UnsignedIntegerSampledTextureCube<'a> {
    const TYPE: ResourceSlotType =
        ResourceSlotType::SampledTexture(SampledTextureType::UnsignedIntegerSamplerCube);

    fn encode<B>(self, slot_index: u32, encoder: &mut BindGroupEncoder) {
        encoder.add_unsigned_integer_sampled_texture_cube(slot_index, self);
    }
}

unsafe impl<'a> Resource for ShadowSampledTexture2D<'a> {
    const TYPE: ResourceSlotType =
        ResourceSlotType::SampledTexture(SampledTextureType::Sampler2DShadow);

    fn encode<B>(self, slot_index: u32, encoder: &mut BindGroupEncoder) {
        encoder.add_shadow_sampled_texture_2d(slot_index, self);
    }
}

unsafe impl<'a> Resource for ShadowSampledTexture2DArray<'a> {
    const TYPE: ResourceSlotType =
        ResourceSlotType::SampledTexture(SampledTextureType::Sampler2DArrayShadow);

    fn encode<B>(self, slot_index: u32, encoder: &mut BindGroupEncoder) {
        encoder.add_shadow_sampled_texture_2d_array(slot_index, self);
    }
}

unsafe impl<'a> Resource for ShadowSampledTextureCube<'a> {
    const TYPE: ResourceSlotType =
        ResourceSlotType::SampledTexture(SampledTextureType::SamplerCubeShadow);

    fn encode<B>(self, slot_index: u32, encoder: &mut BindGroupEncoder) {
        encoder.add_shadow_sampled_texture_cube(slot_index, self);
    }
}
