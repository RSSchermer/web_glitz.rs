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

pub trait TypedResourceBindingsLayout {
    type Layout: Into<TypedResourceBindingsLayoutDescriptor>;

    const LAYOUT: Self::Layout;
}

pub trait ResourceBindings {
    type Bindings: Borrow<[BindingDescriptor]> + 'static;

    fn encode(
        self,
        encoding_context: &mut ResourceBindingsEncodingContext,
    ) -> ResourceBindingsEncoding<Self::Bindings>;
}

pub unsafe trait TypedResourceBindings: ResourceBindings {
    type Layout: TypedResourceBindingsLayout;
}

#[derive(Clone, Debug)]
pub struct ResourceBindingsLayoutDescriptor {
    pub(crate) bindings: Vec<ResourceSlotDescriptor>,
}

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

#[derive(Clone, Hash, PartialEq, Debug)]
pub struct ResourceSlotDescriptor {
    pub slot_identifier: ResourceSlotIdentifier,
    pub slot_index: u32,
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

#[derive(Clone, Copy, Hash, PartialEq, Debug)]
pub enum ResourceSlotKind {
    // A WebGPU version would add `has_dynamic_offset`.
    UniformBuffer,
    // A WebGPU version would add `dimensionality`, `component_type` and `is_multisampled`.
    SampledTexture,
}

impl ResourceSlotKind {
    pub fn is_uniform_buffer(&self) -> bool {
        if let ResourceSlotKind::UniformBuffer = self {
            true
        } else {
            false
        }
    }

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

#[derive(Clone, PartialEq, Debug)]
pub struct TypedResourceSlotDescriptor {
    pub slot_identifier: ResourceSlotIdentifier,
    pub slot_index: u32,
    pub slot_type: ResourceSlotType,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ResourceSlotType {
    // A WebGPU version would add `has_dynamic_offset`.
    UniformBuffer(&'static [MemoryUnit]),
    // A WebGPU version would add `dimensionality`, `component_type` and `is_multisampled`.
    SampledTexture(SampledTextureType),
}

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
/// pipeline, such that the pipeline may access these resources during execution.
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
///
/// A [GraphicsPipelineDescriptor] must declare the [Resources] type that may be used with pipelines
/// created from it, see [GraphicsPipelineDescriptorBuilder::resources]. The compatibility of the
/// resource bindings declared this type will be checked against the pipeline's resource slots when
/// the pipeline is created, see [RenderingContext::create_graphics_pipeline].
pub unsafe trait Resources {
    type Layout: Into<TypedResourceBindingsLayoutDescriptor>;

    type Bindings: Borrow<[BindingDescriptor]> + 'static;

    const LAYOUT: Self::Layout;

    fn encode_bindings(
        self,
        encoding_context: &mut ResourceBindingsEncodingContext,
    ) -> ResourceBindingsEncoding<Self::Bindings>;
}

impl<T> TypedResourceBindingsLayout for T
where
    T: Resources,
{
    type Layout = T::Layout;

    const LAYOUT: Self::Layout = T::LAYOUT;
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
pub unsafe trait BufferResource {
    /// Encodes a binding for this resource.
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
    fn encode<B>(
        self,
        slot_index: u32,
        encoder: StaticResourceBindingsEncoder<B>,
    ) -> StaticResourceBindingsEncoder<(BindingDescriptor, B)> {
        encoder.add_buffer_view(slot_index, self)
    }
}

/// Trait implemented for types that can be bound to a pipeline as a texture resource.
pub unsafe trait TextureResource {
    /// Encodes a binding for this resource.
    fn encode<B>(
        self,
        slot_index: u32,
        encoder: StaticResourceBindingsEncoder<B>,
    ) -> StaticResourceBindingsEncoder<(BindingDescriptor, B)>;
}

unsafe impl<'a> TextureResource for FloatSampledTexture2D<'a> {
    fn encode<B>(
        self,
        slot_index: u32,
        encoder: StaticResourceBindingsEncoder<B>,
    ) -> StaticResourceBindingsEncoder<(BindingDescriptor, B)> {
        encoder.add_float_sampled_texture_2d(slot_index, self)
    }
}

unsafe impl<'a> TextureResource for FloatSampledTexture2DArray<'a> {
    fn encode<B>(
        self,
        slot_index: u32,
        encoder: StaticResourceBindingsEncoder<B>,
    ) -> StaticResourceBindingsEncoder<(BindingDescriptor, B)> {
        encoder.add_float_sampled_texture_2d_array(slot_index, self)
    }
}

unsafe impl<'a> TextureResource for FloatSampledTexture3D<'a> {
    fn encode<B>(
        self,
        slot_index: u32,
        encoder: StaticResourceBindingsEncoder<B>,
    ) -> StaticResourceBindingsEncoder<(BindingDescriptor, B)> {
        encoder.add_float_sampled_texture_3d(slot_index, self)
    }
}

unsafe impl<'a> TextureResource for FloatSampledTextureCube<'a> {
    fn encode<B>(
        self,
        slot_index: u32,
        encoder: StaticResourceBindingsEncoder<B>,
    ) -> StaticResourceBindingsEncoder<(BindingDescriptor, B)> {
        encoder.add_float_sampled_texture_cube(slot_index, self)
    }
}

unsafe impl<'a> TextureResource for IntegerSampledTexture2D<'a> {
    fn encode<B>(
        self,
        slot_index: u32,
        encoder: StaticResourceBindingsEncoder<B>,
    ) -> StaticResourceBindingsEncoder<(BindingDescriptor, B)> {
        encoder.add_integer_sampled_texture_2d(slot_index, self)
    }
}

unsafe impl<'a> TextureResource for IntegerSampledTexture2DArray<'a> {
    fn encode<B>(
        self,
        slot_index: u32,
        encoder: StaticResourceBindingsEncoder<B>,
    ) -> StaticResourceBindingsEncoder<(BindingDescriptor, B)> {
        encoder.add_integer_sampled_texture_2d_array(slot_index, self)
    }
}

unsafe impl<'a> TextureResource for IntegerSampledTexture3D<'a> {
    fn encode<B>(
        self,
        slot_index: u32,
        encoder: StaticResourceBindingsEncoder<B>,
    ) -> StaticResourceBindingsEncoder<(BindingDescriptor, B)> {
        encoder.add_integer_sampled_texture_3d(slot_index, self)
    }
}

unsafe impl<'a> TextureResource for IntegerSampledTextureCube<'a> {
    fn encode<B>(
        self,
        slot_index: u32,
        encoder: StaticResourceBindingsEncoder<B>,
    ) -> StaticResourceBindingsEncoder<(BindingDescriptor, B)> {
        encoder.add_integer_sampled_texture_cube(slot_index, self)
    }
}

unsafe impl<'a> TextureResource for UnsignedIntegerSampledTexture2D<'a> {
    fn encode<B>(
        self,
        slot_index: u32,
        encoder: StaticResourceBindingsEncoder<B>,
    ) -> StaticResourceBindingsEncoder<(BindingDescriptor, B)> {
        encoder.add_unsigned_integer_sampled_texture_2d(slot_index, self)
    }
}

unsafe impl<'a> TextureResource for UnsignedIntegerSampledTexture2DArray<'a> {
    fn encode<B>(
        self,
        slot_index: u32,
        encoder: StaticResourceBindingsEncoder<B>,
    ) -> StaticResourceBindingsEncoder<(BindingDescriptor, B)> {
        encoder.add_unsigned_integer_sampled_texture_2d_array(slot_index, self)
    }
}

unsafe impl<'a> TextureResource for UnsignedIntegerSampledTexture3D<'a> {
    fn encode<B>(
        self,
        slot_index: u32,
        encoder: StaticResourceBindingsEncoder<B>,
    ) -> StaticResourceBindingsEncoder<(BindingDescriptor, B)> {
        encoder.add_unsigned_integer_sampled_texture_3d(slot_index, self)
    }
}

unsafe impl<'a> TextureResource for UnsignedIntegerSampledTextureCube<'a> {
    fn encode<B>(
        self,
        slot_index: u32,
        encoder: StaticResourceBindingsEncoder<B>,
    ) -> StaticResourceBindingsEncoder<(BindingDescriptor, B)> {
        encoder.add_unsigned_integer_sampled_texture_cube(slot_index, self)
    }
}

unsafe impl<'a> TextureResource for ShadowSampledTexture2D<'a> {
    fn encode<B>(
        self,
        slot_index: u32,
        encoder: StaticResourceBindingsEncoder<B>,
    ) -> StaticResourceBindingsEncoder<(BindingDescriptor, B)> {
        encoder.add_shadow_sampled_texture_2d(slot_index, self)
    }
}

unsafe impl<'a> TextureResource for ShadowSampledTexture2DArray<'a> {
    fn encode<B>(
        self,
        slot_index: u32,
        encoder: StaticResourceBindingsEncoder<B>,
    ) -> StaticResourceBindingsEncoder<(BindingDescriptor, B)> {
        encoder.add_shadow_sampled_texture_2d_array(slot_index, self)
    }
}

unsafe impl<'a> TextureResource for ShadowSampledTextureCube<'a> {
    fn encode<B>(
        self,
        slot_index: u32,
        encoder: StaticResourceBindingsEncoder<B>,
    ) -> StaticResourceBindingsEncoder<(BindingDescriptor, B)> {
        encoder.add_shadow_sampled_texture_cube(slot_index, self)
    }
}
