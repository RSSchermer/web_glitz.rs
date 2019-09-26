use std::borrow::Borrow;
use std::mem;

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
use crate::pipeline::interface_block;
use crate::pipeline::interface_block::InterfaceBlock;
use crate::pipeline::resources::bind_group_encoding::{
    BindGroupEncoding, BindGroupEncodingContext, BindingDescriptor,
};
use crate::pipeline::resources::binding::{
    Binding, BufferBinding, FloatSampler2DArrayBinding, FloatSampler2DBinding,
    FloatSampler3DBinding, FloatSamplerCubeBinding, IntegerSampler2DArrayBinding,
    IntegerSampler2DBinding, IntegerSampler3DBinding, IntegerSamplerCubeBinding,
    ShadowSampler2DArrayBinding, ShadowSampler2DBinding, ShadowSamplerCubeBinding,
    UnsignedIntegerSampler2DArrayBinding, UnsignedIntegerSampler2DBinding,
    UnsignedIntegerSampler3DBinding, UnsignedIntegerSamplerCubeBinding,
};
use crate::pipeline::resources::resource_slot::{Identifier, ResourceSlotDescriptor, SlotBindingConfirmer, SlotBindingMismatch, IncompatibleInterface};

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
    type Bindings: Borrow<[BindingDescriptor]> + 'static;

    /// Confirms that the [BindGroupEncoding] for these [Resources] as created by
    /// [encode_bind_group] will match a pipeline's resource slots as describe by the [descriptors],
    /// using the [confirmer].
    ///
    /// If this function does not return an error, then calling [encode_bind_group] for any instance
    /// of the type will return a [BindGroupEncoding] that can be safely used with any pipeline
    /// described by the [descriptors].
    fn confirm_slot_bindings<C>(
        confirmer: &C,
        descriptors: &[ResourceSlotDescriptor],
    ) -> Result<(), IncompatibleResources>
    where
        C: SlotBindingConfirmer;

    /// Encodes these [Resources] into a bind group, so that they can be bound to specific `binding`
    /// indices efficiently before a pipeline is executed.
    fn into_bind_group(
        self,
        context: &mut BindGroupEncodingContext,
    ) -> BindGroupEncoding<Self::Bindings>;
}

unsafe impl Resources for () {
    type Bindings = [BindingDescriptor; 0];

    fn confirm_slot_bindings<C>(
        _confirmer: &C,
        descriptors: &[ResourceSlotDescriptor],
    ) -> Result<(), IncompatibleResources>
    where
        C: SlotBindingConfirmer,
    {
        if let Some(descriptor) = descriptors.first() {
            Err(IncompatibleResources::MissingResource(
                descriptor.identifier().clone(),
            ))
        } else {
            Ok(())
        }
    }

    fn into_bind_group(
        self,
        context: &mut BindGroupEncodingContext,
    ) -> BindGroupEncoding<Self::Bindings> {
        BindGroupEncoding::empty(context)
    }
}

/// Error returned by [Resources::confirm_slot_bindings] when the [Resources] don't match the
/// resource slot bindings.
#[derive(Debug)]
pub enum IncompatibleResources {
    MissingResource(Identifier),
    ResourceTypeMismatch(Identifier),
    IncompatibleInterface(Identifier, IncompatibleInterface),
    SlotBindingMismatch { expected: usize, actual: usize },
}

impl From<SlotBindingMismatch> for IncompatibleResources {
    fn from(err: SlotBindingMismatch) -> Self {
        IncompatibleResources::SlotBindingMismatch {
            expected: err.expected,
            actual: err.actual,
        }
    }
}

/// Trait implemented for types that can be bound to a pipeline as a buffer resource.
pub unsafe trait BufferResource {
    /// The type of binding for this resource.
    type Binding: Binding;

    /// Turns this buffer resource into a binding for the given `index`.
    fn into_binding(self, index: u32) -> Self::Binding;
}

unsafe impl<'a, T> BufferResource for &'a Buffer<T>
where
    T: InterfaceBlock,
{
    type Binding = BufferBinding<'a, T>;

    fn into_binding(self, index: u32) -> Self::Binding {
        BufferBinding {
            index,
            buffer_view: self.into(),
            size_in_bytes: mem::size_of::<T>(),
        }
    }
}

unsafe impl<'a, T> BufferResource for BufferView<'a, T>
where
    T: InterfaceBlock,
{
    type Binding = BufferBinding<'a, T>;

    fn into_binding(self, index: u32) -> Self::Binding {
        BufferBinding {
            index,
            buffer_view: self,
            size_in_bytes: mem::size_of::<T>(),
        }
    }
}

/// Trait implemented for types that can be bound to a pipeline as a texture resource.
pub unsafe trait TextureResource {
    /// The type of binding for this resource.
    type Binding: Binding;

    /// Turns this texture resource into a binding for the given `index`.
    fn into_binding(self, index: u32) -> Self::Binding;
}

unsafe impl<'a> TextureResource for FloatSampledTexture2D<'a> {
    type Binding = FloatSampler2DBinding<'a>;

    fn into_binding(self, index: u32) -> Self::Binding {
        FloatSampler2DBinding {
            texture_unit: index,
            resource: self,
        }
    }
}

unsafe impl<'a> TextureResource for FloatSampledTexture2DArray<'a> {
    type Binding = FloatSampler2DArrayBinding<'a>;

    fn into_binding(self, index: u32) -> Self::Binding {
        FloatSampler2DArrayBinding {
            texture_unit: index,
            resource: self,
        }
    }
}

unsafe impl<'a> TextureResource for FloatSampledTexture3D<'a> {
    type Binding = FloatSampler3DBinding<'a>;

    fn into_binding(self, index: u32) -> Self::Binding {
        FloatSampler3DBinding {
            texture_unit: index,
            resource: self,
        }
    }
}

unsafe impl<'a> TextureResource for FloatSampledTextureCube<'a> {
    type Binding = FloatSamplerCubeBinding<'a>;

    fn into_binding(self, index: u32) -> Self::Binding {
        FloatSamplerCubeBinding {
            texture_unit: index,
            resource: self,
        }
    }
}

unsafe impl<'a> TextureResource for IntegerSampledTexture2D<'a> {
    type Binding = IntegerSampler2DBinding<'a>;

    fn into_binding(self, index: u32) -> Self::Binding {
        IntegerSampler2DBinding {
            texture_unit: index,
            resource: self,
        }
    }
}

unsafe impl<'a> TextureResource for IntegerSampledTexture2DArray<'a> {
    type Binding = IntegerSampler2DArrayBinding<'a>;

    fn into_binding(self, index: u32) -> Self::Binding {
        IntegerSampler2DArrayBinding {
            texture_unit: index,
            resource: self,
        }
    }
}

unsafe impl<'a> TextureResource for IntegerSampledTexture3D<'a> {
    type Binding = IntegerSampler3DBinding<'a>;

    fn into_binding(self, index: u32) -> Self::Binding {
        IntegerSampler3DBinding {
            texture_unit: index,
            resource: self,
        }
    }
}

unsafe impl<'a> TextureResource for IntegerSampledTextureCube<'a> {
    type Binding = IntegerSamplerCubeBinding<'a>;

    fn into_binding(self, index: u32) -> Self::Binding {
        IntegerSamplerCubeBinding {
            texture_unit: index,
            resource: self,
        }
    }
}

unsafe impl<'a> TextureResource for UnsignedIntegerSampledTexture2D<'a> {
    type Binding = UnsignedIntegerSampler2DBinding<'a>;

    fn into_binding(self, index: u32) -> Self::Binding {
        UnsignedIntegerSampler2DBinding {
            texture_unit: index,
            resource: self,
        }
    }
}

unsafe impl<'a> TextureResource for UnsignedIntegerSampledTexture2DArray<'a> {
    type Binding = UnsignedIntegerSampler2DArrayBinding<'a>;

    fn into_binding(self, index: u32) -> Self::Binding {
        UnsignedIntegerSampler2DArrayBinding {
            texture_unit: index,
            resource: self,
        }
    }
}

unsafe impl<'a> TextureResource for UnsignedIntegerSampledTexture3D<'a> {
    type Binding = UnsignedIntegerSampler3DBinding<'a>;

    fn into_binding(self, index: u32) -> Self::Binding {
        UnsignedIntegerSampler3DBinding {
            texture_unit: index,
            resource: self,
        }
    }
}

unsafe impl<'a> TextureResource for UnsignedIntegerSampledTextureCube<'a> {
    type Binding = UnsignedIntegerSamplerCubeBinding<'a>;

    fn into_binding(self, index: u32) -> Self::Binding {
        UnsignedIntegerSamplerCubeBinding {
            texture_unit: index,
            resource: self,
        }
    }
}

unsafe impl<'a> TextureResource for ShadowSampledTexture2D<'a> {
    type Binding = ShadowSampler2DBinding<'a>;

    fn into_binding(self, index: u32) -> Self::Binding {
        ShadowSampler2DBinding {
            texture_unit: index,
            resource: self,
        }
    }
}

unsafe impl<'a> TextureResource for ShadowSampledTexture2DArray<'a> {
    type Binding = ShadowSampler2DArrayBinding<'a>;

    fn into_binding(self, index: u32) -> Self::Binding {
        ShadowSampler2DArrayBinding {
            texture_unit: index,
            resource: self,
        }
    }
}

unsafe impl<'a> TextureResource for ShadowSampledTextureCube<'a> {
    type Binding = ShadowSamplerCubeBinding<'a>;

    fn into_binding(self, index: u32) -> Self::Binding {
        ShadowSamplerCubeBinding {
            texture_unit: index,
            resource: self,
        }
    }
}
