use crate::pipeline::resources::binding::{Binding, BufferBinding, FloatSampler2DBinding, FloatSampler2DArrayBinding, FloatSampler3DBinding, FloatSamplerCubeBinding, IntegerSampler2DBinding, IntegerSampler2DArrayBinding, IntegerSampler3DBinding, IntegerSamplerCubeBinding, UnsignedIntegerSampler2DBinding, UnsignedIntegerSampler2DArrayBinding, UnsignedIntegerSampler3DBinding, UnsignedIntegerSamplerCubeBinding, ShadowSampler2DBinding, ShadowSampler2DArrayBinding, ShadowSamplerCubeBinding};
use crate::buffer::{Buffer, BufferView};
use crate::pipeline::interface_block::InterfaceBlock;
use std::mem;
use crate::sampler::{FloatSampledTexture2D, FloatSampledTexture2DArray, FloatSampledTexture3D, FloatSampledTextureCube, IntegerSampledTexture2D, IntegerSampledTexture2DArray, IntegerSampledTexture3D, IntegerSampledTextureCube, UnsignedIntegerSampledTexture2D, UnsignedIntegerSampledTexture2DArray, UnsignedIntegerSampledTexture3D, UnsignedIntegerSampledTextureCube, ShadowSampledTexture2D, ShadowSampledTexture2DArray, ShadowSampledTextureCube};
use std::borrow::Borrow;
use crate::pipeline::resources::bind_group_encoding::{BindingDescriptor, BindGroupEncodingContext, BindGroupEncoding};
use crate::pipeline::resources::resource_slot::{ResourceSlotDescriptor, Slot, SlotBindingConfirmer, Identifier};
use crate::pipeline::interface_block;

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
/// use web_glitz::pipeline::resources::Resources;
/// use web_glitz::pipeline::interface_block::InterfaceBlock;
/// use web_glitz::sampler::{FloatSampledTexture2D, FloatSampledTexture2DArray};
/// use web_glitz::buffer::Buffer;
/// use web_glitz::std140;
///
/// #[derive(Resources)]
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
/// #[derive(InterfaceBlock)]
/// struct SomeUniformBlock {
///     some_uniform: std::vec4,
///     some_other_uniform: std::mat4
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
    type Bindings: Borrow<[BindingDescriptor]> + 'static;

    /// Confirms that the [BindGroupEncoding] for these [Resources] as created by
    /// [encode_bind_group] will match a pipeline's resource slots as describe by the [descriptors],
    /// using the [confirmer].
    ///
    /// If this function does not return an error, then calling [encode_bind_group] for any instance
    /// of the type will return a [BindGroupEncoding] that can be safely used with any pipeline
    /// described by the [descriptors].
    fn confirm_slot_bindings<C>(confirmer: &C, descriptors: &[ResourceSlotDescriptor]) -> Result<(), Incompatible> where C: SlotBindingConfirmer;

    /// Encodes these [Resources] into a bind group, so that they can be bound to specific `binding`
    /// indices efficiently before a pipeline is executed.
    fn encode_bind_group<'a>(&self, context: &'a mut BindGroupEncodingContext) -> BindGroupEncoding<'a, Self::Bindings>;
}

pub enum Incompatible {
    MissingResource(Identifier),
    ResourceTypeMismatch(Identifier),
    IncompatibleBlockLayout(Identifier, interface_block::Incompatible)
}

pub trait Resource {
    type Binding: Binding;

    fn into_binding(self, index: u32) -> Self::Binding;
}

impl<'a, T> Resource for &'a Buffer<T> where T: InterfaceBlock {
    type Binding = BufferBinding<'a, T>;

    fn into_binding(self, index: u32) -> Self::Binding {
        BufferBinding {
            index,
            buffer_view: self.view(),
            size_in_bytes: mem::size_of::<T>()
        }
    }
}

impl<'a, T> Resource for &'a BufferView<T> where T: InterfaceBlock {
    type Binding = BufferBinding<'a, T>;

    fn into_binding(self, index: u32) -> Self::Binding {
        BufferBinding {
            index,
            buffer_view: self,
            size_in_bytes: mem::size_of::<T>()
        }
    }
}

impl<'a> Resource for FloatSampledTexture2D<'a> {
    type Binding = FloatSampler2DBinding<'a>;

    fn into_binding(self, index: u32) -> Self::Binding {
        FloatSampler2DBinding {
            texture_unit: index,
            resource: self,
        }
    }
}

impl<'a> Resource for FloatSampledTexture2DArray<'a> {
    type Binding = FloatSampler2DArrayBinding<'a>;

    fn into_binding(self, index: u32) -> Self::Binding {
        FloatSampler2DArrayBinding {
            texture_unit: index,
            resource: self,
        }
    }
}

impl<'a> Resource for FloatSampledTexture3D<'a> {
    type Binding = FloatSampler3DBinding<'a>;

    fn into_binding(self, index: u32) -> Self::Binding {
        FloatSampler3DBinding {
            texture_unit: index,
            resource: self,
        }
    }
}

impl<'a> Resource for FloatSampledTextureCube<'a> {
    type Binding = FloatSamplerCubeBinding<'a>;

    fn into_binding(self, index: u32) -> Self::Binding {
        FloatSamplerCubeBinding {
            texture_unit: index,
            resource: self,
        }
    }
}

impl<'a> Resource for IntegerSampledTexture2D<'a> {
    type Binding = IntegerSampler2DBinding<'a>;

    fn into_binding(self, index: u32) -> Self::Binding {
        IntegerSampler2DBinding {
            texture_unit: index,
            resource: self,
        }
    }
}

impl<'a> Resource for IntegerSampledTexture2DArray<'a> {
    type Binding = IntegerSampler2DArrayBinding<'a>;

    fn into_binding(self, index: u32) -> Self::Binding {
        IntegerSampler2DArrayBinding {
            texture_unit: index,
            resource: self,
        }
    }
}

impl<'a> Resource for IntegerSampledTexture3D<'a> {
    type Binding = IntegerSampler3DBinding<'a>;

    fn into_binding(self, index: u32) -> Self::Binding {
        IntegerSampler3DBinding {
            texture_unit: index,
            resource: self,
        }
    }
}

impl<'a> Resource for IntegerSampledTextureCube<'a> {
    type Binding = IntegerSamplerCubeBinding<'a>;

    fn into_binding(self, index: u32) -> Self::Binding {
        IntegerSamplerCubeBinding {
            texture_unit: index,
            resource: self,
        }
    }
}

impl<'a> Resource for UnsignedIntegerSampledTexture2D<'a> {
    type Binding = UnsignedIntegerSampler2DBinding<'a>;

    fn into_binding(self, index: u32) -> Self::Binding {
        UnsignedIntegerSampler2DBinding {
            texture_unit: index,
            resource: self,
        }
    }
}

impl<'a> Resource for UnsignedIntegerSampledTexture2DArray<'a> {
    type Binding = UnsignedIntegerSampler2DArrayBinding<'a>;

    fn into_binding(self, index: u32) -> Self::Binding {
        UnsignedIntegerSampler2DArrayBinding {
            texture_unit: index,
            resource: self,
        }
    }
}

impl<'a> Resource for UnsignedIntegerSampledTexture3D<'a> {
    type Binding = UnsignedIntegerSampler3DBinding<'a>;

    fn into_binding(self, index: u32) -> Self::Binding {
        UnsignedIntegerSampler3DBinding {
            texture_unit: index,
            resource: self,
        }
    }
}

impl<'a> Resource for UnsignedIntegerSampledTextureCube<'a> {
    type Binding = UnsignedIntegerSamplerCubeBinding<'a>;

    fn into_binding(self, index: u32) -> Self::Binding {
        UnsignedIntegerSamplerCubeBinding {
            texture_unit: index,
            resource: self,
        }
    }
}

impl<'a> Resource for ShadowSampledTexture2D<'a> {
    type Binding = ShadowSampler2DBinding<'a>;

    fn into_binding(self, index: u32) -> Self::Binding {
        ShadowSampler2DBinding {
            texture_unit: index,
            resource: self,
        }
    }
}

impl<'a> Resource for ShadowSampledTexture2DArray<'a> {
    type Binding = ShadowSampler2DArrayBinding<'a>;

    fn into_binding(self, index: u32) -> Self::Binding {
        ShadowSampler2DArrayBinding {
            texture_unit: index,
            resource: self,
        }
    }
}

impl<'a> Resource for ShadowSampledTextureCube<'a> {
    type Binding = ShadowSamplerCubeBinding<'a>;

    fn into_binding(self, index: u32) -> Self::Binding {
        ShadowSamplerCubeBinding {
            texture_unit: index,
            resource: self,
        }
    }
}
