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

/// Represents a group of bindable resources that may be bound to a pipeline and are shared by all
/// invocations during the pipeline's execution.
///
/// See [RenderingContext::create_bind_group] for details on how a bind group is created.
///
/// More than one bind group may be bound to a pipeline, see
/// [GraphicsPipelineTaskBuilder::bind_resources] and
/// [GraphicsPipelineTaskBuilder::bind_resources_untyped] for details.
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

/// A minimal description of the resource binding slots used by a pipeline.
///
/// This type only contains the minimally necessary information for initializing a pipeline. See
/// also [TypedResourceBindingsLayoutDescriptor] for a type that includes information that may be
/// type checked against the resource types defined by the pipeline's shader stages.
#[derive(Clone, Debug)]
pub struct ResourceBindingsLayoutDescriptor {
    layout: Vec<LayoutElement>,
    bind_groups: usize,
}

impl ResourceBindingsLayoutDescriptor {
    /// A collection of descriptors for the bind group layouts associated with the resource bindings
    /// layout.
    ///
    /// See also [BindGroupLayout].
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

/// Returned from [ResourceBindingsLayoutDescriptor::bind_groups], a collection of bind group
/// layouts associated with a [ResourceBindingsLayoutDescriptor].
#[derive(Clone, Copy)]
pub struct BindGroupLayouts<'a> {
    layout: &'a ResourceBindingsLayoutDescriptor,
}

impl<'a> BindGroupLayouts<'a> {
    /// The number of bind group layouts associated with the [ResourceBindingsLayoutDescriptor].
    pub fn len(&self) -> usize {
        self.layout.bind_groups
    }

    /// Returns an iterator of the bind group associated with the
    /// [ResourceBindingsLayoutDescriptor].
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

/// Returned from [BindGroupLayouts::iter], an iterator over the bind group layouts associated with
/// a [ResourceBindingsLayoutDescriptor].
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

/// Describes the layout for a bind group at a specific bind group index in a
/// [ResourceBindingsLayoutDescriptor].
pub struct BindGroupLayout<'a> {
    slots: &'a [LayoutElement],
    bind_group_index: u32,
}

impl<'a> BindGroupLayout<'a> {
    /// The bind group index for the bind group.
    pub fn bind_group_index(&self) -> u32 {
        self.bind_group_index
    }

    /// A collection of descriptors for the resource slots associated with the bind group.
    ///
    /// See also [ResourceSlotDescriptor].
    pub fn slots(&self) -> BindGroupLayoutSlots {
        BindGroupLayoutSlots { slots: self.slots }
    }
}

/// Returned from [BindGroupLayout::slots], represents the resource slots associated with the
/// bind group layout.
pub struct BindGroupLayoutSlots<'a> {
    slots: &'a [LayoutElement],
}

impl<'a> BindGroupLayoutSlots<'a> {
    /// Number of resource slots associated with the bind group layout.
    pub fn len(&self) -> usize {
        self.slots.len()
    }

    /// Returns an iterator over descriptors of the resource slots associated with the bind group
    /// layout.
    pub fn iter(&self) -> BindGroupLayoutSlotsIter {
        BindGroupLayoutSlotsIter {
            iter: self.slots.iter(),
        }
    }
}

impl<'a> IntoIterator for BindGroupLayoutSlots<'a> {
    type Item = &'a ResourceSlotDescriptor;
    type IntoIter = BindGroupLayoutSlotsIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        BindGroupLayoutSlotsIter {
            iter: self.slots.into_iter(),
        }
    }
}

/// Returned from [BindGroupLayoutSlots::iter], an iterator over the slots in the bind group layout.
pub struct BindGroupLayoutSlotsIter<'a> {
    iter: std::slice::Iter<'a, LayoutElement>,
}

impl<'a> Iterator for BindGroupLayoutSlotsIter<'a> {
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

impl<'a> ExactSizeIterator for BindGroupLayoutSlotsIter<'a> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

/// Hint for a [ResourceBindingsLayoutDescriptorBuilder] that indicates how much memory is required
/// to accommodate the result.
///
/// See [ResourceBindingsLayoutDescriptorBuilder::new].
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct LayoutAllocationHint {
    /// The number of bind groups the layout will declare.
    pub bind_groups: usize,

    /// The total number of resource slots the layout will declare across all bind groups.
    pub total_resource_slots: usize,
}

/// Enumerates the errors that may occur when building a [ResourceBindingsLayoutDescriptor].
///
/// See [ResourceBindingsLayoutDescriptorBuilder].
pub enum ResourceBindingsLayoutBuilderError {
    InvalidBindGroupSequence(InvalidBindGroupSequence),
    InvalidResourceSlotSequence(InvalidResourceSlotSequence)
}

/// Error returned when adding bind groups to a [ResourceBindingsLayoutDescriptorBuilder] out of
/// order or when adding multiple bind groups that declare the same bind group index.
///
/// See [ResourceBindingsLayoutDescriptorBuilder::add_bind_group].
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct InvalidBindGroupSequence {
    pub current_index: u32,
    pub previous_index: u32,
}

/// Builds a [ResourceBindingsLayoutDescriptor].
///
/// # Example
///
/// ```
/// use web_glitz::pipeline::resources::{ResourceSlotDescriptor, ResourceBindingsLayoutBuilder, LayoutAllocationHint, ResourceSlotIdentifier, ResourceSlotKind};
///
/// let mut builder = ResourceBindingsLayoutBuilder::new(Some(LayoutAllocationHint {
///     bind_groups: 2,
///     total_resource_slots: 3,
/// }));
///
/// let resource_bindings_layout = builder
///     .add_bind_group(0)?
///         .add_resource_slot(ResourceSlotDescriptor {
///             slot_index: 0,
///             slot_identifier: ResourceSlotIdentifier::Static("buffer_0"),
///             slot_kind: ResourceSlotKind::UniformBuffer
///         })?
///         .finish()
///     .add_bind_group(1)?
///         .add_resource_slot(ResourceSlotDescriptor {
///             slot_index: 0,
///             slot_identifier: ResourceSlotIdentifier::Static("texture_0"),
///             slot_kind: ResourceSlotKind::SampledTexture
///         })?
///         .add_resource_slot(ResourceSlotDescriptor {
///             slot_index: 1,
///             slot_identifier: ResourceSlotIdentifier::Static("texture_1"),
///             slot_kind: ResourceSlotKind::SampledTexture
///         })?
///         .finish()
///     .finish();
/// ```
///
/// Note that all bind groups must be added in increasing order of bind group index and no
/// 2 bind groups may declare the same bind group index (but bind group indices do not have to be
/// contiguous); all resource slots must be added to their respective bind groups in increaseing
/// order of slot index and no 2 resource slots within the same bind group may declare the same
/// slot index (but slot indices do not have to be contiguous).
pub struct ResourceBindingsLayoutBuilder {
    layout: Vec<LayoutElement>,
    last_bind_group_index: Option<u32>,
    bind_groups: usize,
}

impl ResourceBindingsLayoutBuilder {
    /// Creates a new [ResourceBindingsLayoutDescriptorBuilder].
    ///
    /// An optional [LayoutAllocationHint] to indicate to the builder how much memory should be
    /// allocated to accommodate the result. Note that providing an inaccurate hint will not result
    /// in any errors, but may result in more than 1 allocation or the allocation of extraneous
    /// memory.
    pub fn new(allocation_hint: Option<LayoutAllocationHint>) -> Self {
        let layout = if let Some(hint) = allocation_hint {
            Vec::with_capacity(hint.bind_groups + hint.total_resource_slots)
        } else {
            Vec::new()
        };

        ResourceBindingsLayoutBuilder {
            layout,
            last_bind_group_index: None,
            bind_groups: 0,
        }
    }

    /// Adds a bind group to the layout with the given `bind_group_index`.
    ///
    /// Returns a builder that may be used to add resource slots to the bind group's layout, or
    /// returns an [InvalidBindGroupIndex] error if the `bind_group_index` is not greater than the
    /// previous bind group index (if any): bind groups must be added to the layout in ascending
    /// order of bind group index and no two bind groups may declare the same bind group index.
    pub fn add_bind_group(
        mut self,
        bind_group_index: u32,
    ) -> Result<BindGroupLayoutBuilder, InvalidBindGroupSequence> {
        if let Some(last_bind_group_index) = self.last_bind_group_index {
            if bind_group_index <= last_bind_group_index {
                return Err(InvalidBindGroupSequence {
                    current_index: bind_group_index,
                    previous_index: last_bind_group_index
                });
            }
        }

        let start = self.layout.len();

        self.layout
            .push(LayoutElement::NextBindGroup(BindGroupElement {
                bind_group_index,
                len: 0,
            }));

        self.bind_groups += 1;

        Ok(BindGroupLayoutBuilder {
            builder: self,
            bind_group_index,
            start,
            last_slot_index: None,
        })
    }

    /// Finishes this builder and returns the resulting [ResourceBindingsLayoutDescriptor].
    pub fn finish(self) -> ResourceBindingsLayoutDescriptor {
        ResourceBindingsLayoutDescriptor {
            layout: self.layout,
            bind_groups: self.bind_groups,
        }
    }
}

/// Error returned when adding resource slots to a [BindGroupLayoutBuilder] out of order or when
/// adding multiple resource slots that declare the same slot index.
///
/// See [BindGroupLayoutBuilder::add_resource_slot].
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct InvalidResourceSlotSequence {
    pub bind_group_index: u32,
    pub current_index: u32,
    pub previous_index: u32
}

/// Returned from [ResourceBindingsLayoutDescriptorBuilder::add_bind_group], accumulates the
/// resource slot descriptors for a bind group layout description.
pub struct BindGroupLayoutBuilder {
    builder: ResourceBindingsLayoutBuilder,
    bind_group_index: u32,
    start: usize,
    last_slot_index: Option<u32>,
}

impl BindGroupLayoutBuilder {
    /// Adds a resource slot descriptor to the bind group layout.
    ///
    /// The slot index declared by the descriptor must be greater than the previous resource slot
    /// added to this builder (if any), otherwise an [InvalidResourceSlotSequence] error is
    /// returned: all resource slots must be added to the builder in ascending order of slot index
    /// and no 2 resource slots must declare the same slot index.
    pub fn add_resource_slot(
        mut self,
        descriptor: ResourceSlotDescriptor,
    ) -> Result<Self, InvalidResourceSlotSequence > {
        if let Some(last_slot_index) = self.last_slot_index {
            if descriptor.slot_index <= last_slot_index {
                return Err(InvalidResourceSlotSequence {
                    bind_group_index: self.bind_group_index,
                    current_index: descriptor.slot_index,
                    previous_index: last_slot_index
                } );
            }
        }

        self.builder.layout.push(LayoutElement::Slot(descriptor));

        Ok(self)
    }

    /// Finishes the layout for this bind group and returns the [ResourceBindingsLayoutBuilder].
    pub fn finish(mut self) -> ResourceBindingsLayoutBuilder {
        let len = self.builder.layout.len() - self.start;

        self.builder.layout[self.start] = LayoutElement::NextBindGroup(BindGroupElement {
            bind_group_index: self.bind_group_index,
            len,
        });

        self.builder
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

/// Describes the resource slot layout of a bind group in a [TypedResourceBindingsLayoutDescriptor].
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

/// A description of the resource slot layout of a bind group, attached to a type.
///
/// See also [TypedResourceSlotDescriptor].
///
/// This trait becomes useful in combination with the [TypedBindableResourceGroup] and
/// [TypedResourceBindings] traits, the later of which is implemented for any tuple of [BindGroup]s
/// where each [BindGroup] holds resources that implement [TypedBindableResourceGroup]. If a
/// pipeline (or a tuple of such bind groups) declares a typed resource bindings layout, then a
/// (tuple of) bind group(s) with a matching resource layout may be safely bound to the pipeline
/// without additional runtime checks.
///
/// Note that [TypedBindGroupLayout] is safe to implement, but implementing
/// [TypedBindableResourceGroup] is unsafe: the resource bindings encoded by the
/// [TypedBindableResourceGroup] implementation must always be compatible with the bind group
/// layout specified by its [TypedBindableResourceGroup::Layout], see [TypedBindableResourceGroup]
/// for details.
pub trait TypedBindGroupLayout {
    const LAYOUT: &'static [TypedResourceSlotDescriptor];
}

/// A group of resources that may be used to form a [BindGroup].
///
/// See also [RenderingContext::create_bind_group] for details on how a [BindGroup] is created.
///
/// This trait is implemented for any type that implements the [Resources] trait. The [Resources]
/// trait may automatically derived (see the documentation for [Resources] for details).
///
/// # Example
///
/// ```
/// use web_glitz::buffer::Buffer;
/// use web_glitz::image::texture_2d::FloatSampledTexture2D;
/// use web_glitz::pipeline::resources::{BindableResourceGroup, BindGroupEncodingContext, BindGroupEncoding, BindGroupEncoder};
///
/// struct Resources<'a, 'b> {
///     uniform_buffer: &'a Buffer<std140::mat4x4>,
///     texture: FloatSampledTexture2D<'b>
/// }
///
/// impl<'a, 'b> BindableResourceGroup for Resources<'a, 'b> {
///     fn encode_bind_group(
///         self,
///         encoding_context: &mut BindGroupEncodingContext
///     ) -> BindGroupEncoding<'a> {
///         let mut encoder = BindGroupEncoder::new(encoding_context, Some(2));
///
///         encoder.add_buffer_view(0, self.uniform_buffer.into());
///         encoder.add_float_sampled_texture_2d(1, self.texture);
///
///         encoder.finish()
///     }
/// }
/// ```
pub trait BindableResourceGroup {
    /// Encodes a description of the bindings for the resources in the group.
    fn encode_bind_group(
        self,
        encoding_context: &mut BindGroupEncodingContext,
    ) -> BindGroupEncoding;
}

/// Sub-trait of [BindableResourceGroup], where a type statically describes the layout for the
/// bind group.
///
/// A [BindGroup] for resources that implement this trait may be bound to pipeline's with a matching
/// typed resource bindings layout without additional runtime checks.
///
/// # Unsafe
///
/// Any instance of a type that implements this trait must encode a bind group (see
/// [BindableResourceGroup::encode_bind_group]) with a layout that matches the bind group layout
/// declared by [Layout].
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
/// See also [StaticResourceBindingsEncoder].
///
/// Note that when multiple bindings of the same type bind to the same slot-index, then only the
/// binding that was added last will be used. However, buffer resources and texture resources belong
/// to distinct bind groups, their slot-indices do not interact.
///
/// This trait is automatically implemented for any type that derives the [Resources] trait.
pub trait ResourceBindings {
    /// Type that describes the collection of bindings.
    type BindGroups: Borrow<[BindGroupDescriptor]> + 'static;

    /// Encodes a description of how this set of resources is bound to a pipeline.
    fn encode(
        self,
        encoding_context: &mut ResourceBindingsEncodingContext,
    ) -> ResourceBindingsEncoding<Self::BindGroups>;
}

impl ResourceBindings for () {
    type BindGroups = [BindGroupDescriptor; 0];

    fn encode(
        self,
        encoding_context: &mut ResourceBindingsEncodingContext,
    ) -> ResourceBindingsEncoding<Self::BindGroups> {
        ResourceBindingsEncoding::empty(encoding_context)
    }
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

unsafe impl TypedResourceBindings for () {
    type Layout = ();
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

/// Provides a group of resources (uniform block buffers, sampled textures) that can initialize a
/// [BindGroup].
///
/// See [RendinderingContext::create_bind_group] for details on how a [BindGroup] is created.
///
/// The resulting bind group may be bound to a pipeline, such that all pipeline invokations may
/// access these resources when the pipeline is executed. See
/// [GraphicsPipelineTaskBuilder::bind_resources] for details on how bind groups are bound to a
/// pipeline.
///
/// This type acts as an automatically derivable trait for a [TypedBindableResourceGroup] type that
/// acts as its own [TypedBindGroupLayout]. This trait is only intended to be derived automatically;
/// if your set of resources cannot be adequatly described by automatically deriving this trait,
/// rather than manually implementing this trait, instead consider manually implementing the
/// [TypedBindableResourceGroup] and [TypedBindGroupLayout] traits separately.
///
/// Note that a [BindGroup] may also be initialized from an untyped [BindableResourceGroup]. The
/// resulting bind group will be untyped and binding it to a pipeline is unsafe: you are responsible
/// for ensuring that the resources you bind match the resources expected by the shader stages of
/// the pipeline, see [GraphicsPipelineTaskBuilder::bind_resources_untyped] for details.
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
/// Note that GLSL ES 3.0 does not allow you to define your own explicit bind groups. Instead, bind
/// groups are implicitly defined for each kind of resource:
///
/// - Uniform buffers: this bind group uses bind group index `0`.
/// - Sampled textures: this bind group uses bind group index `1`.
///
/// This trait may be safely derived automatically on a type to define how specific resource
/// instances should be bound to the pipeline:
///
/// ```
/// use web_glitz::image::texture_2d::FloatSampledTexture2D;
/// use web_glitz::image::texture_2d_array::FloatSampledTexture2DArray;
/// use web_glitz::buffer::Buffer;
/// use web_glitz::std140;
/// use web_glitz::std140::repr_std140;
///
/// #[derive(web_glitz::derive::Resources)]
/// struct BufferResources<'a> {
///     #[resource(binding=0, name="SomeUniformBlock")]
///     some_uniform_block: &'a Buffer<SomeUniformBlock>,
/// }
///
/// #[derive(web_glitz::derive::Resources)]
/// struct TextureResources<'a> {
///     #[resource(binding=0)]
///     some_texture: FloatSampledTexture2D<'a>,
///
///     #[resource(binding=1, name="some_other_texture")]
///     second_texture: FloatSampledTexture2DArray<'a>,
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
/// Any field marked with a `#[resource(...)]` attribute defines a resource binding; unmarked fields
/// are ignored when initializing a bind group.
///
/// A `#[resource(...)]` attribute must always declare a `binding` index; for a texture this should
/// be a positive integer that is smaller than the [RenderingContext::max_texture_resource_index] of
/// the [RenderingContext] with which you intend to use the [Resources] (hardware dependent, at
/// least `32`); for a uniform buffer this should be a positive integer that is smaller than the
/// [RenderingContext::max_buffer_resource_index] for the [RenderingContext] with which you intend
/// to use the [Resources] (hardware dependent, at least `24`).
///
/// If the field name does not match the resource name used in the shader code, then the attribute
/// should also declare a `name` with a string that does match the resource name used in the shader
/// code; if the field name does match the name used in the shader, then `name` may be omitted.
///
/// The field's type must implement [Resource]; marking a field that does not implement [Resource]
/// with `#[resource(...)]` will result in a compilation error. If multiple `#[resource(...)]`
/// fields are defined, then all fields must declare a unique `binding` index; 2 or more
/// `#[resource(...)]` fields with the same `binding` index will also result in a compilation error.
pub unsafe trait Resources {
    const LAYOUT: &'static [TypedResourceSlotDescriptor];

    fn encode_bind_group(
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
        <T as Resources>::encode_bind_group(self, encoding_context)
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
    fn encode(self, slot_index: u32, encoder: &mut BindGroupEncoder);
}

unsafe impl<'a, T> Resource for &'a Buffer<T>
where
    T: InterfaceBlock,
{
    const TYPE: ResourceSlotType = ResourceSlotType::UniformBuffer(T::MEMORY_UNITS);

    fn encode(self, slot_index: u32, encoder: &mut BindGroupEncoder) {
        encoder.add_buffer_view(slot_index, self.into());
    }
}

unsafe impl<'a, T> Resource for BufferView<'a, T>
where
    T: InterfaceBlock,
{
    const TYPE: ResourceSlotType = ResourceSlotType::UniformBuffer(T::MEMORY_UNITS);

    fn encode(self, slot_index: u32, encoder: &mut BindGroupEncoder) {
        encoder.add_buffer_view(slot_index, self);
    }
}

unsafe impl<'a> Resource for FloatSampledTexture2D<'a> {
    const TYPE: ResourceSlotType =
        ResourceSlotType::SampledTexture(SampledTextureType::FloatSampler2D);

    fn encode(self, slot_index: u32, encoder: &mut BindGroupEncoder) {
        encoder.add_float_sampled_texture_2d(slot_index, self);
    }
}

unsafe impl<'a> Resource for FloatSampledTexture2DArray<'a> {
    const TYPE: ResourceSlotType =
        ResourceSlotType::SampledTexture(SampledTextureType::FloatSampler2DArray);

    fn encode(self, slot_index: u32, encoder: &mut BindGroupEncoder) {
        encoder.add_float_sampled_texture_2d_array(slot_index, self);
    }
}

unsafe impl<'a> Resource for FloatSampledTexture3D<'a> {
    const TYPE: ResourceSlotType =
        ResourceSlotType::SampledTexture(SampledTextureType::FloatSampler3D);

    fn encode(self, slot_index: u32, encoder: &mut BindGroupEncoder) {
        encoder.add_float_sampled_texture_3d(slot_index, self);
    }
}

unsafe impl<'a> Resource for FloatSampledTextureCube<'a> {
    const TYPE: ResourceSlotType =
        ResourceSlotType::SampledTexture(SampledTextureType::FloatSamplerCube);

    fn encode(self, slot_index: u32, encoder: &mut BindGroupEncoder) {
        encoder.add_float_sampled_texture_cube(slot_index, self);
    }
}

unsafe impl<'a> Resource for IntegerSampledTexture2D<'a> {
    const TYPE: ResourceSlotType =
        ResourceSlotType::SampledTexture(SampledTextureType::IntegerSampler2D);

    fn encode(self, slot_index: u32, encoder: &mut BindGroupEncoder) {
        encoder.add_integer_sampled_texture_2d(slot_index, self);
    }
}

unsafe impl<'a> Resource for IntegerSampledTexture2DArray<'a> {
    const TYPE: ResourceSlotType =
        ResourceSlotType::SampledTexture(SampledTextureType::IntegerSampler2DArray);

    fn encode(self, slot_index: u32, encoder: &mut BindGroupEncoder) {
        encoder.add_integer_sampled_texture_2d_array(slot_index, self);
    }
}

unsafe impl<'a> Resource for IntegerSampledTexture3D<'a> {
    const TYPE: ResourceSlotType =
        ResourceSlotType::SampledTexture(SampledTextureType::IntegerSampler3D);

    fn encode(self, slot_index: u32, encoder: &mut BindGroupEncoder) {
        encoder.add_integer_sampled_texture_3d(slot_index, self);
    }
}

unsafe impl<'a> Resource for IntegerSampledTextureCube<'a> {
    const TYPE: ResourceSlotType =
        ResourceSlotType::SampledTexture(SampledTextureType::IntegerSamplerCube);

    fn encode(self, slot_index: u32, encoder: &mut BindGroupEncoder) {
        encoder.add_integer_sampled_texture_cube(slot_index, self);
    }
}

unsafe impl<'a> Resource for UnsignedIntegerSampledTexture2D<'a> {
    const TYPE: ResourceSlotType =
        ResourceSlotType::SampledTexture(SampledTextureType::UnsignedIntegerSampler2D);

    fn encode(self, slot_index: u32, encoder: &mut BindGroupEncoder) {
        encoder.add_unsigned_integer_sampled_texture_2d(slot_index, self);
    }
}

unsafe impl<'a> Resource for UnsignedIntegerSampledTexture2DArray<'a> {
    const TYPE: ResourceSlotType =
        ResourceSlotType::SampledTexture(SampledTextureType::UnsignedIntegerSampler2DArray);

    fn encode(self, slot_index: u32, encoder: &mut BindGroupEncoder) {
        encoder.add_unsigned_integer_sampled_texture_2d_array(slot_index, self);
    }
}

unsafe impl<'a> Resource for UnsignedIntegerSampledTexture3D<'a> {
    const TYPE: ResourceSlotType =
        ResourceSlotType::SampledTexture(SampledTextureType::UnsignedIntegerSampler3D);

    fn encode(self, slot_index: u32, encoder: &mut BindGroupEncoder) {
        encoder.add_unsigned_integer_sampled_texture_3d(slot_index, self);
    }
}

unsafe impl<'a> Resource for UnsignedIntegerSampledTextureCube<'a> {
    const TYPE: ResourceSlotType =
        ResourceSlotType::SampledTexture(SampledTextureType::UnsignedIntegerSamplerCube);

    fn encode(self, slot_index: u32, encoder: &mut BindGroupEncoder) {
        encoder.add_unsigned_integer_sampled_texture_cube(slot_index, self);
    }
}

unsafe impl<'a> Resource for ShadowSampledTexture2D<'a> {
    const TYPE: ResourceSlotType =
        ResourceSlotType::SampledTexture(SampledTextureType::Sampler2DShadow);

    fn encode(self, slot_index: u32, encoder: &mut BindGroupEncoder) {
        encoder.add_shadow_sampled_texture_2d(slot_index, self);
    }
}

unsafe impl<'a> Resource for ShadowSampledTexture2DArray<'a> {
    const TYPE: ResourceSlotType =
        ResourceSlotType::SampledTexture(SampledTextureType::Sampler2DArrayShadow);

    fn encode(self, slot_index: u32, encoder: &mut BindGroupEncoder) {
        encoder.add_shadow_sampled_texture_2d_array(slot_index, self);
    }
}

unsafe impl<'a> Resource for ShadowSampledTextureCube<'a> {
    const TYPE: ResourceSlotType =
        ResourceSlotType::SampledTexture(SampledTextureType::SamplerCubeShadow);

    fn encode(self, slot_index: u32, encoder: &mut BindGroupEncoder) {
        encoder.add_shadow_sampled_texture_cube(slot_index, self);
    }
}
