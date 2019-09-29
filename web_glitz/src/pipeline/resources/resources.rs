use std::borrow::Borrow;
use std::hash::{Hasher, Hash};
use std::ops::Deref;

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
    BindingDescriptor, ResourceBindingsEncoding, ResourceBindingsEncodingContext,
};
use crate::pipeline::resources::resource_slot::IncompatibleInterface;
use crate::pipeline::resources::StaticResourceBindingsEncoder;

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
    type Layout: Into<TypedResourceBindingsLayoutDescriptor>;

    const LAYOUT: Self::Layout;
}

/// Encodes a description of how a set of resources is bound to a pipeline, such that the pipeline
/// may access these resources during its execution.
///
/// # Example
///
/// ```
/// use web_glitz::buffer::Buffer;
/// use web_glitz::image::texture_2d::FloatSampledTexture2D;
/// use web_glitz::pipeline::resources::{ResourceBindings, BindingDescriptor, ResourceBindingsEncodingContext, ResourceBindingsEncoding, StaticResourceBindingsEncoder};
///
/// struct Resources<'a> {
///     buffer_resource: &'a Buffer<f32>,
///     texture_resource: FloatSampledTexture2D<'a>
/// }
///
/// impl ResourceBindings for Resources {
///     type Bindings = [BindingDescriptor; 2];
///
///     fn encode(
///         self,
///         encoding_context: &mut ResourceBindingsEncodingContext
///     ) -> ResourceBindingsEncoding<Self::Bindings> {
///         let encoder = StaticResourceBindingsEncoder::new(encoding_context);
///
///         let encoder = encoder.add_buffer_view(0, self.buffer_resource.into());
///         let encoder = encoder.add_float_sampled_texture_2d(0, self.texture_resource);
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
    type Bindings: Borrow<[BindingDescriptor]> + 'static;

    /// Encodes a description of how this set of resources is bound to a pipeline.
    fn encode(
        self,
        encoding_context: &mut ResourceBindingsEncodingContext,
    ) -> ResourceBindingsEncoding<Self::Bindings>;
}

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

/// A minimal description of the resource binding slots used by a pipeline.
///
/// This type only contains the minimally necessary information for initializing a pipeline. See
/// also [TypedResourceBindingsLayoutDescriptor] for a type that includes information that may be
/// type checked against the resource types defined by the pipeline's shader stages.
#[derive(Clone, Debug)]
pub struct ResourceBindingsLayoutDescriptor {
    pub(crate) bindings: Vec<ResourceSlotDescriptor>,
}

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

/// A typed description of the resource binding slots used by a pipeline.
///
/// This type includes description of the exact resource type used for each resource slot, which may
/// be checked against the resource types defined by the pipeline's shader stages.
///
/// See also [ResourceBindingsLayoutDescriptor] a descriptor that only includes the minimum of
/// information necessary to initialize a pipeline.
#[derive(Clone, PartialEq, Debug)]
pub struct TypedResourceBindingsLayoutDescriptor {
    pub(crate) bindings: &'static [TypedResourceSlotDescriptor],
}

impl From<()> for TypedResourceBindingsLayoutDescriptor {
    fn from(_: ()) -> Self {
        TypedResourceBindingsLayoutDescriptor { bindings: &[] }
    }
}

impl From<&'static [TypedResourceSlotDescriptor]> for TypedResourceBindingsLayoutDescriptor {
    fn from(bindings: &'static [TypedResourceSlotDescriptor]) -> Self {
        TypedResourceBindingsLayoutDescriptor { bindings }
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
    type Bindings: Borrow<[BindingDescriptor]> + 'static;

    const LAYOUT: &'static [TypedResourceSlotDescriptor];

    fn encode_bindings(
        self,
        encoding_context: &mut ResourceBindingsEncodingContext,
    ) -> ResourceBindingsEncoding<Self::Bindings>;
}

impl<T> TypedResourceBindingsLayout for T
where
    T: Resources,
{
    type Layout = &'static [TypedResourceSlotDescriptor];

    const LAYOUT: &'static [TypedResourceSlotDescriptor] = T::LAYOUT;
}

impl<T> ResourceBindings for T
where
    T: Resources,
{
    type Bindings = T::Bindings;

    fn encode(
        self,
        encoding_context: &mut ResourceBindingsEncodingContext,
    ) -> ResourceBindingsEncoding<Self::Bindings> {
        self.encode_bindings(encoding_context)
    }
}

unsafe impl<T> TypedResourceBindings for T
where
    T: Resources,
{
    type Layout = Self;
}

impl TypedResourceBindingsLayout for () {
    type Layout = TypedResourceBindingsLayoutDescriptor;

    const LAYOUT: Self::Layout = TypedResourceBindingsLayoutDescriptor { bindings: &[] };
}

impl ResourceBindings for () {
    type Bindings = [BindingDescriptor; 0];

    fn encode(
        self,
        encoding_context: &mut ResourceBindingsEncodingContext,
    ) -> ResourceBindingsEncoding<Self::Bindings> {
        ResourceBindingsEncoding::empty(encoding_context)
    }
}

unsafe impl TypedResourceBindings for () {
    type Layout = ();
}

/// Error returned by [Resources::confirm_slot_bindings] when the [Resources] don't match the
/// resource slot bindings.
#[derive(Debug)]
pub enum IncompatibleResources {
    MissingResource(ResourceSlotIdentifier),
    ResourceTypeMismatch(ResourceSlotIdentifier),
    IncompatibleInterface(ResourceSlotIdentifier, IncompatibleInterface),
    SlotBindingMismatch { expected: usize, actual: usize },
}

/// Trait implemented for types that can be bound to a pipeline as a buffer resource.
///
/// When automatically deriving the [Resources] trait, fields marked with `#[buffer_resource(...)]`
/// must implement this trait.
pub unsafe trait BufferResource {
    const MEMORY_UNITS: &'static [MemoryUnit];

    /// Encodes a binding for this resource at the specified `slot_index`.
    fn encode<B>(
        self,
        slot_index: u32,
        encoder: StaticResourceBindingsEncoder<B>,
    ) -> StaticResourceBindingsEncoder<(BindingDescriptor, B)>;
}

unsafe impl<'a, T> BufferResource for &'a Buffer<T>
where
    T: InterfaceBlock,
{
    const MEMORY_UNITS: &'static [MemoryUnit] = T::MEMORY_UNITS;

    fn encode<B>(
        self,
        slot_index: u32,
        encoder: StaticResourceBindingsEncoder<B>,
    ) -> StaticResourceBindingsEncoder<(BindingDescriptor, B)> {
        encoder.add_buffer_view(slot_index, self.into())
    }
}

unsafe impl<'a, T> BufferResource for BufferView<'a, T>
where
    T: InterfaceBlock,
{
    const MEMORY_UNITS: &'static [MemoryUnit] = T::MEMORY_UNITS;

    fn encode<B>(
        self,
        slot_index: u32,
        encoder: StaticResourceBindingsEncoder<B>,
    ) -> StaticResourceBindingsEncoder<(BindingDescriptor, B)> {
        encoder.add_buffer_view(slot_index, self)
    }
}

/// Trait implemented for types that can be bound to a pipeline as a texture resource.
///
/// When automatically deriving the [Resources] trait, fields marked with `#[texture_resource(...)]`
/// must implement this trait.
pub unsafe trait TextureResource {
    const SAMPLED_TEXTURE_TYPE: SampledTextureType;

    /// Encodes a binding for this resource at the specified `slot_index`.
    fn encode<B>(
        self,
        slot_index: u32,
        encoder: StaticResourceBindingsEncoder<B>,
    ) -> StaticResourceBindingsEncoder<(BindingDescriptor, B)>;
}

unsafe impl<'a> TextureResource for FloatSampledTexture2D<'a> {
    const SAMPLED_TEXTURE_TYPE: SampledTextureType = SampledTextureType::FloatSampler2D;

    fn encode<B>(
        self,
        slot_index: u32,
        encoder: StaticResourceBindingsEncoder<B>,
    ) -> StaticResourceBindingsEncoder<(BindingDescriptor, B)> {
        encoder.add_float_sampled_texture_2d(slot_index, self)
    }
}

unsafe impl<'a> TextureResource for FloatSampledTexture2DArray<'a> {
    const SAMPLED_TEXTURE_TYPE: SampledTextureType = SampledTextureType::FloatSampler2DArray;

    fn encode<B>(
        self,
        slot_index: u32,
        encoder: StaticResourceBindingsEncoder<B>,
    ) -> StaticResourceBindingsEncoder<(BindingDescriptor, B)> {
        encoder.add_float_sampled_texture_2d_array(slot_index, self)
    }
}

unsafe impl<'a> TextureResource for FloatSampledTexture3D<'a> {
    const SAMPLED_TEXTURE_TYPE: SampledTextureType = SampledTextureType::FloatSampler3D;

    fn encode<B>(
        self,
        slot_index: u32,
        encoder: StaticResourceBindingsEncoder<B>,
    ) -> StaticResourceBindingsEncoder<(BindingDescriptor, B)> {
        encoder.add_float_sampled_texture_3d(slot_index, self)
    }
}

unsafe impl<'a> TextureResource for FloatSampledTextureCube<'a> {
    const SAMPLED_TEXTURE_TYPE: SampledTextureType = SampledTextureType::FloatSamplerCube;

    fn encode<B>(
        self,
        slot_index: u32,
        encoder: StaticResourceBindingsEncoder<B>,
    ) -> StaticResourceBindingsEncoder<(BindingDescriptor, B)> {
        encoder.add_float_sampled_texture_cube(slot_index, self)
    }
}

unsafe impl<'a> TextureResource for IntegerSampledTexture2D<'a> {
    const SAMPLED_TEXTURE_TYPE: SampledTextureType = SampledTextureType::IntegerSampler2D;

    fn encode<B>(
        self,
        slot_index: u32,
        encoder: StaticResourceBindingsEncoder<B>,
    ) -> StaticResourceBindingsEncoder<(BindingDescriptor, B)> {
        encoder.add_integer_sampled_texture_2d(slot_index, self)
    }
}

unsafe impl<'a> TextureResource for IntegerSampledTexture2DArray<'a> {
    const SAMPLED_TEXTURE_TYPE: SampledTextureType = SampledTextureType::IntegerSampler2DArray;

    fn encode<B>(
        self,
        slot_index: u32,
        encoder: StaticResourceBindingsEncoder<B>,
    ) -> StaticResourceBindingsEncoder<(BindingDescriptor, B)> {
        encoder.add_integer_sampled_texture_2d_array(slot_index, self)
    }
}

unsafe impl<'a> TextureResource for IntegerSampledTexture3D<'a> {
    const SAMPLED_TEXTURE_TYPE: SampledTextureType = SampledTextureType::IntegerSampler3D;

    fn encode<B>(
        self,
        slot_index: u32,
        encoder: StaticResourceBindingsEncoder<B>,
    ) -> StaticResourceBindingsEncoder<(BindingDescriptor, B)> {
        encoder.add_integer_sampled_texture_3d(slot_index, self)
    }
}

unsafe impl<'a> TextureResource for IntegerSampledTextureCube<'a> {
    const SAMPLED_TEXTURE_TYPE: SampledTextureType = SampledTextureType::IntegerSamplerCube;

    fn encode<B>(
        self,
        slot_index: u32,
        encoder: StaticResourceBindingsEncoder<B>,
    ) -> StaticResourceBindingsEncoder<(BindingDescriptor, B)> {
        encoder.add_integer_sampled_texture_cube(slot_index, self)
    }
}

unsafe impl<'a> TextureResource for UnsignedIntegerSampledTexture2D<'a> {
    const SAMPLED_TEXTURE_TYPE: SampledTextureType = SampledTextureType::UnsignedIntegerSampler2D;

    fn encode<B>(
        self,
        slot_index: u32,
        encoder: StaticResourceBindingsEncoder<B>,
    ) -> StaticResourceBindingsEncoder<(BindingDescriptor, B)> {
        encoder.add_unsigned_integer_sampled_texture_2d(slot_index, self)
    }
}

unsafe impl<'a> TextureResource for UnsignedIntegerSampledTexture2DArray<'a> {
    const SAMPLED_TEXTURE_TYPE: SampledTextureType = SampledTextureType::UnsignedIntegerSampler2DArray;

    fn encode<B>(
        self,
        slot_index: u32,
        encoder: StaticResourceBindingsEncoder<B>,
    ) -> StaticResourceBindingsEncoder<(BindingDescriptor, B)> {
        encoder.add_unsigned_integer_sampled_texture_2d_array(slot_index, self)
    }
}

unsafe impl<'a> TextureResource for UnsignedIntegerSampledTexture3D<'a> {
    const SAMPLED_TEXTURE_TYPE: SampledTextureType = SampledTextureType::UnsignedIntegerSampler3D;

    fn encode<B>(
        self,
        slot_index: u32,
        encoder: StaticResourceBindingsEncoder<B>,
    ) -> StaticResourceBindingsEncoder<(BindingDescriptor, B)> {
        encoder.add_unsigned_integer_sampled_texture_3d(slot_index, self)
    }
}

unsafe impl<'a> TextureResource for UnsignedIntegerSampledTextureCube<'a> {
    const SAMPLED_TEXTURE_TYPE: SampledTextureType = SampledTextureType::UnsignedIntegerSamplerCube;

    fn encode<B>(
        self,
        slot_index: u32,
        encoder: StaticResourceBindingsEncoder<B>,
    ) -> StaticResourceBindingsEncoder<(BindingDescriptor, B)> {
        encoder.add_unsigned_integer_sampled_texture_cube(slot_index, self)
    }
}

unsafe impl<'a> TextureResource for ShadowSampledTexture2D<'a> {
    const SAMPLED_TEXTURE_TYPE: SampledTextureType = SampledTextureType::Sampler2DShadow;

    fn encode<B>(
        self,
        slot_index: u32,
        encoder: StaticResourceBindingsEncoder<B>,
    ) -> StaticResourceBindingsEncoder<(BindingDescriptor, B)> {
        encoder.add_shadow_sampled_texture_2d(slot_index, self)
    }
}

unsafe impl<'a> TextureResource for ShadowSampledTexture2DArray<'a> {
    const SAMPLED_TEXTURE_TYPE: SampledTextureType = SampledTextureType::Sampler2DArrayShadow;

    fn encode<B>(
        self,
        slot_index: u32,
        encoder: StaticResourceBindingsEncoder<B>,
    ) -> StaticResourceBindingsEncoder<(BindingDescriptor, B)> {
        encoder.add_shadow_sampled_texture_2d_array(slot_index, self)
    }
}

unsafe impl<'a> TextureResource for ShadowSampledTextureCube<'a> {
    const SAMPLED_TEXTURE_TYPE: SampledTextureType = SampledTextureType::SamplerCubeShadow;

    fn encode<B>(
        self,
        slot_index: u32,
        encoder: StaticResourceBindingsEncoder<B>,
    ) -> StaticResourceBindingsEncoder<(BindingDescriptor, B)> {
        encoder.add_shadow_sampled_texture_cube(slot_index, self)
    }
}
