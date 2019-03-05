use crate::pipeline::resources::binding::{Binding, BufferBinding, FloatSampler2DBinding, FloatSampler2DArrayBinding, FloatSampler3DBinding, FloatSamplerCubeBinding, IntegerSampler2DBinding, IntegerSampler2DArrayBinding, IntegerSampler3DBinding, IntegerSamplerCubeBinding, UnsignedIntegerSampler2DBinding, UnsignedIntegerSampler2DArrayBinding, UnsignedIntegerSampler3DBinding, UnsignedIntegerSamplerCubeBinding, ShadowSampler2DBinding, ShadowSampler2DArrayBinding, ShadowSamplerCubeBinding};
use crate::buffer::{Buffer, BufferView};
use crate::pipeline::interface_block::InterfaceBlock;
use std::mem;
use crate::sampler::{FloatSampledTexture2D, FloatSampledTexture2DArray, FloatSampledTexture3D, FloatSampledTextureCube, IntegerSampledTexture2D, IntegerSampledTexture2DArray, IntegerSampledTexture3D, IntegerSampledTextureCube, UnsignedIntegerSampledTexture2D, UnsignedIntegerSampledTexture2DArray, UnsignedIntegerSampledTexture3D, UnsignedIntegerSampledTextureCube, ShadowSampledTexture2D, ShadowSampledTexture2DArray, ShadowSampledTextureCube};
use std::borrow::Borrow;
use crate::pipeline::resources::bind_group_encoding::{BindingDescriptor, BindGroupEncodingContext, BindGroupEncoding};
use crate::pipeline::resources::resource_slot::{ResourceSlotDescriptor, Slot, SlotBindingConfirmer, Identifier};
use crate::pipeline::interface_block;

pub unsafe trait Resources {
    type Bindings: Borrow<[BindingDescriptor]> + 'static;

    fn confirm_binding_indices<C>(confirmer: &C, descriptors: &[ResourceSlotDescriptor]) -> Result<(), Incompatible> where C: SlotBindingConfirmer;

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
