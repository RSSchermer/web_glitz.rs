use crate::pipeline::resources::resource_slot::{Slot, SamplerKind};
use crate::pipeline::resources::test::{BindGroupEncoder, BindingDescriptor};
use crate::pipeline::interface_block;
use crate::buffer::BufferView;
use crate::pipeline::interface_block::InterfaceBlock;
use crate::sampler::{FloatSampledTexture2D, FloatSampledTexture2DArray, FloatSampledTexture3D, FloatSampledTextureCube, IntegerSampledTexture2D, IntegerSampledTexture2DArray, IntegerSampledTexture3D, IntegerSampledTextureCube, UnsignedIntegerSampledTexture2D, UnsignedIntegerSampledTexture2DArray, UnsignedIntegerSampledTexture3D, UnsignedIntegerSampledTextureCube, ShadowSampledTexture2D, ShadowSampledTexture2DArray, ShadowSampledTextureCube};

pub unsafe trait Binding {
    fn compatibility(slot: &Slot) -> Result<(), Incompatible>;

    fn encode<T>(&self, encoder: BindGroupEncoder<T>) -> BindGroupEncoder<(BindingDescriptor, T)>;
}

pub enum Incompatible {
    TypeMismatch,
    LayoutMismatch(interface_block::Incompatible)
}

pub struct BufferBinding<'a, T> {
    pub(crate) index: u32,
    pub(crate) buffer_view: &'a BufferView<T>,
    pub(crate) size_in_bytes: usize
}

unsafe impl<'a, T> Binding for BufferBinding<'a, T> where T: InterfaceBlock {
    fn compatibility(slot: &Slot) -> Result<(), Incompatible> {
        match slot {
            Slot::UniformBlock(slot) => {
                slot.compatibility::<T>().map_err(|e| Incompatible::LayoutMismatch(e))
            }
            _ => {
                Err(Incompatible::TypeMismatch)
            }
        }
    }

    fn encode<B>(&self, encoder: BindGroupEncoder<B>) -> BindGroupEncoder<(BindingDescriptor, B)> {
        encoder.add_buffer(self)
    }
}

pub struct FloatSampler2DBinding<'a> {
    pub(crate) texture_unit: u32,
    pub(crate) resource: FloatSampledTexture2D<'a>
}

unsafe impl<'a> Binding for FloatSampler2DBinding<'a> {
    fn compatibility(slot: &Slot) -> Result<(), Incompatible> {
        if let Slot::TextureSampler(slot) = slot {
            match slot.kind() {
                SamplerKind::FloatSampler2D => Ok(()),
                _ => Err(Incompatible::TypeMismatch)
            }
        } else {
            Err(Incompatible::TypeMismatch)
        }
    }

    fn encode<T>(&self, encoder: BindGroupEncoder<T>) -> BindGroupEncoder<(BindingDescriptor, T)> {
        encoder.add_float_sampler_2d(self)
    }
}

pub struct FloatSampler2DArrayBinding<'a> {
    pub(crate) texture_unit: u32,
    pub(crate) resource: FloatSampledTexture2DArray<'a>
}

unsafe impl<'a> Binding for FloatSampler2DArrayBinding<'a> {
    fn compatibility(slot: &Slot) -> Result<(), Incompatible> {
        if let Slot::TextureSampler(slot) = slot {
            match slot.kind() {
                SamplerKind::FloatSampler2DArray => Ok(()),
                _ => Err(Incompatible::TypeMismatch)
            }
        } else {
            Err(Incompatible::TypeMismatch)
        }
    }

    fn encode<T>(&self, encoder: BindGroupEncoder<T>) -> BindGroupEncoder<(BindingDescriptor, T)> {
        encoder.add_float_sampler_2d_array(self)
    }
}

pub struct FloatSampler3DBinding<'a> {
    pub(crate) texture_unit: u32,
    pub(crate) resource: FloatSampledTexture3D<'a>
}

unsafe impl<'a> Binding for FloatSampler3DBinding<'a> {
    fn compatibility(slot: &Slot) -> Result<(), Incompatible> {
        if let Slot::TextureSampler(slot) = slot {
            match slot.kind() {
                SamplerKind::FloatSampler3D => Ok(()),
                _ => Err(Incompatible::TypeMismatch)
            }
        } else {
            Err(Incompatible::TypeMismatch)
        }
    }

    fn encode<T>(&self, encoder: BindGroupEncoder<T>) -> BindGroupEncoder<(BindingDescriptor, T)> {
        encoder.add_float_sampler_3d(self)
    }
}

pub struct FloatSamplerCubeBinding<'a> {
    pub(crate) texture_unit: u32,
    pub(crate) resource: FloatSampledTextureCube<'a>
}

unsafe impl<'a> Binding for FloatSamplerCubeBinding<'a> {
    fn compatibility(slot: &Slot) -> Result<(), Incompatible> {
        if let Slot::TextureSampler(slot) = slot {
            match slot.kind() {
                SamplerKind::FloatSamplerCube => Ok(()),
                _ => Err(Incompatible::TypeMismatch)
            }
        } else {
            Err(Incompatible::TypeMismatch)
        }
    }

    fn encode<T>(&self, encoder: BindGroupEncoder<T>) -> BindGroupEncoder<(BindingDescriptor, T)> {
        encoder.add_float_sampler_cube(self)
    }
}

pub struct IntegerSampler2DBinding<'a> {
    pub(crate) texture_unit: u32,
    pub(crate) resource: IntegerSampledTexture2D<'a>
}

unsafe impl<'a> Binding for IntegerSampler2DBinding<'a> {
    fn compatibility(slot: &Slot) -> Result<(), Incompatible> {
        if let Slot::TextureSampler(slot) = slot {
            match slot.kind() {
                SamplerKind::IntegerSampler2D => Ok(()),
                _ => Err(Incompatible::TypeMismatch)
            }
        } else {
            Err(Incompatible::TypeMismatch)
        }
    }

    fn encode<T>(&self, encoder: BindGroupEncoder<T>) -> BindGroupEncoder<(BindingDescriptor, T)> {
        encoder.add_integer_sampler_2d(self)
    }
}

pub struct IntegerSampler2DArrayBinding<'a> {
    pub(crate) texture_unit: u32,
    pub(crate) resource: IntegerSampledTexture2DArray<'a>
}

unsafe impl<'a> Binding for IntegerSampler2DArrayBinding<'a> {
    fn compatibility(slot: &Slot) -> Result<(), Incompatible> {
        if let Slot::TextureSampler(slot) = slot {
            match slot.kind() {
                SamplerKind::IntegerSampler2DArray => Ok(()),
                _ => Err(Incompatible::TypeMismatch)
            }
        } else {
            Err(Incompatible::TypeMismatch)
        }
    }

    fn encode<T>(&self, encoder: BindGroupEncoder<T>) -> BindGroupEncoder<(BindingDescriptor, T)> {
        encoder.add_integer_sampler_2d_array(self)
    }
}

pub struct IntegerSampler3DBinding<'a> {
    pub(crate) texture_unit: u32,
    pub(crate) resource: IntegerSampledTexture3D<'a>
}

unsafe impl<'a> Binding for IntegerSampler3DBinding<'a> {
    fn compatibility(slot: &Slot) -> Result<(), Incompatible> {
        if let Slot::TextureSampler(slot) = slot {
            match slot.kind() {
                SamplerKind::IntegerSampler3D => Ok(()),
                _ => Err(Incompatible::TypeMismatch)
            }
        } else {
            Err(Incompatible::TypeMismatch)
        }
    }

    fn encode<T>(&self, encoder: BindGroupEncoder<T>) -> BindGroupEncoder<(BindingDescriptor, T)> {
        encoder.add_integer_sampler_3d(self)
    }
}

pub struct IntegerSamplerCubeBinding<'a> {
    pub(crate) texture_unit: u32,
    pub(crate) resource: IntegerSampledTextureCube<'a>
}

unsafe impl<'a> Binding for IntegerSamplerCubeBinding<'a> {
    fn compatibility(slot: &Slot) -> Result<(), Incompatible> {
        if let Slot::TextureSampler(slot) = slot {
            match slot.kind() {
                SamplerKind::IntegerSamplerCube => Ok(()),
                _ => Err(Incompatible::TypeMismatch)
            }
        } else {
            Err(Incompatible::TypeMismatch)
        }
    }

    fn encode<T>(&self, encoder: BindGroupEncoder<T>) -> BindGroupEncoder<(BindingDescriptor, T)> {
        encoder.add_integer_sampler_cube(self)
    }
}

pub struct UnsignedIntegerSampler2DBinding<'a> {
    pub(crate) texture_unit: u32,
    pub(crate) resource: UnsignedIntegerSampledTexture2D<'a>
}

unsafe impl<'a> Binding for UnsignedIntegerSampler2DBinding<'a> {
    fn compatibility(slot: &Slot) -> Result<(), Incompatible> {
        if let Slot::TextureSampler(slot) = slot {
            match slot.kind() {
                SamplerKind::UnsignedIntegerSampler2D => Ok(()),
                _ => Err(Incompatible::TypeMismatch)
            }
        } else {
            Err(Incompatible::TypeMismatch)
        }
    }

    fn encode<T>(&self, encoder: BindGroupEncoder<T>) -> BindGroupEncoder<(BindingDescriptor, T)> {
        encoder.add_unsigned_integer_sampler_2d(self)
    }
}

pub struct UnsignedIntegerSampler2DArrayBinding<'a> {
    pub(crate) texture_unit: u32,
    pub(crate) resource: UnsignedIntegerSampledTexture2DArray<'a>
}

unsafe impl<'a> Binding for UnsignedIntegerSampler2DArrayBinding<'a> {
    fn compatibility(slot: &Slot) -> Result<(), Incompatible> {
        if let Slot::TextureSampler(slot) = slot {
            match slot.kind() {
                SamplerKind::UnsignedIntegerSampler2DArray => Ok(()),
                _ => Err(Incompatible::TypeMismatch)
            }
        } else {
            Err(Incompatible::TypeMismatch)
        }
    }

    fn encode<T>(&self, encoder: BindGroupEncoder<T>) -> BindGroupEncoder<(BindingDescriptor, T)> {
        encoder.add_unsigned_integer_sampler_2d_array(self)
    }
}

pub struct UnsignedIntegerSampler3DBinding<'a> {
    pub(crate) texture_unit: u32,
    pub(crate) resource: UnsignedIntegerSampledTexture3D<'a>
}

unsafe impl<'a> Binding for UnsignedIntegerSampler3DBinding<'a> {
    fn compatibility(slot: &Slot) -> Result<(), Incompatible> {
        if let Slot::TextureSampler(slot) = slot {
            match slot.kind() {
                SamplerKind::UnsignedIntegerSampler3D => Ok(()),
                _ => Err(Incompatible::TypeMismatch)
            }
        } else {
            Err(Incompatible::TypeMismatch)
        }
    }

    fn encode<T>(&self, encoder: BindGroupEncoder<T>) -> BindGroupEncoder<(BindingDescriptor, T)> {
        encoder.add_unsigned_integer_sampler_3d(self)
    }
}

pub struct UnsignedIntegerSamplerCubeBinding<'a> {
    pub(crate) texture_unit: u32,
    pub(crate) resource: UnsignedIntegerSampledTextureCube<'a>
}

unsafe impl<'a> Binding for UnsignedIntegerSamplerCubeBinding<'a> {
    fn compatibility(slot: &Slot) -> Result<(), Incompatible> {
        if let Slot::TextureSampler(slot) = slot {
            match slot.kind() {
                SamplerKind::UnsignedIntegerSamplerCube => Ok(()),
                _ => Err(Incompatible::TypeMismatch)
            }
        } else {
            Err(Incompatible::TypeMismatch)
        }
    }

    fn encode<T>(&self, encoder: BindGroupEncoder<T>) -> BindGroupEncoder<(BindingDescriptor, T)> {
        encoder.add_unsigned_integer_sampler_cube(self)
    }
}

pub struct ShadowSampler2DBinding<'a> {
    pub(crate) texture_unit: u32,
    pub(crate) resource: ShadowSampledTexture2D<'a>
}

unsafe impl<'a> Binding for ShadowSampler2DBinding<'a> {
    fn compatibility(slot: &Slot) -> Result<(), Incompatible> {
        if let Slot::TextureSampler(slot) = slot {
            match slot.kind() {
                SamplerKind::ShadowSampler2D => Ok(()),
                _ => Err(Incompatible::TypeMismatch)
            }
        } else {
            Err(Incompatible::TypeMismatch)
        }
    }

    fn encode<T>(&self, encoder: BindGroupEncoder<T>) -> BindGroupEncoder<(BindingDescriptor, T)> {
        encoder.add_shadow_sampler_2d(self)
    }
}

pub struct ShadowSampler2DArrayBinding<'a> {
    pub(crate) texture_unit: u32,
    pub(crate) resource: ShadowSampledTexture2DArray<'a>
}

unsafe impl<'a> Binding for ShadowSampler2DArrayBinding<'a> {
    fn compatibility(slot: &Slot) -> Result<(), Incompatible> {
        if let Slot::TextureSampler(slot) = slot {
            match slot.kind() {
                SamplerKind::ShadowSampler2DArray => Ok(()),
                _ => Err(Incompatible::TypeMismatch)
            }
        } else {
            Err(Incompatible::TypeMismatch)
        }
    }

    fn encode<T>(&self, encoder: BindGroupEncoder<T>) -> BindGroupEncoder<(BindingDescriptor, T)> {
        encoder.add_shadow_sampler_2d_array(self)
    }
}

pub struct ShadowSamplerCubeBinding<'a> {
    pub(crate) texture_unit: u32,
    pub(crate) resource: ShadowSampledTextureCube<'a>
}

unsafe impl<'a> Binding for ShadowSamplerCubeBinding<'a> {
    fn compatibility(slot: &Slot) -> Result<(), Incompatible> {
        if let Slot::TextureSampler(slot) = slot {
            match slot.kind() {
                SamplerKind::ShadowSamplerCube => Ok(()),
                _ => Err(Incompatible::TypeMismatch)
            }
        } else {
            Err(Incompatible::TypeMismatch)
        }
    }

    fn encode<T>(&self, encoder: BindGroupEncoder<T>) -> BindGroupEncoder<(BindingDescriptor, T)> {
        encoder.add_shadow_sampler_cube(self)
    }
}
