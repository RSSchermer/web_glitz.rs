use std::borrow::Borrow;
use std::sync::Arc;
use crate::buffer::BufferData;
use crate::sampler::SamplerData;
use crate::image::texture_2d::Texture2DData;
use crate::image::texture_2d_array::Texture2DArrayData;
use crate::image::texture_3d::Texture3DData;
use crate::image::texture_cube::TextureCubeData;
use crate::pipeline::resources::binding::{BufferBinding, FloatSampler2DBinding, FloatSampler2DArrayBinding, FloatSampler3DBinding, FloatSamplerCubeBinding, IntegerSampler2DBinding, IntegerSampler2DArrayBinding, IntegerSampler3DBinding, IntegerSamplerCubeBinding, UnsignedIntegerSampler2DBinding, UnsignedIntegerSampler2DArrayBinding, UnsignedIntegerSampler3DBinding, UnsignedIntegerSamplerCubeBinding, ShadowSampler2DBinding, ShadowSampler2DArrayBinding, ShadowSamplerCubeBinding};

pub struct BindGroupEncoding<'a, B> where B: Borrow<[BindingDescriptor]> + 'static {
    context: &'a mut BindGroupEncodingContext,
    descriptors: B
}

pub struct BindingDescriptor {
    internal: BindingDescriptorInternal
}

enum BindingDescriptorInternal {
    BufferView {
        index: u32,
        buffer_data: Arc<BufferData>,
        offset: usize,
        size: usize
    },
    SampledTexture {
        unit: u32,
        sampler_data: Arc<SamplerData>,
        texture_data: TextureData
    }
}

enum TextureData {
    Texture2D(Arc<Texture2DData>),
    Texture2DArray(Arc<Texture2DArrayData>),
    Texture3D(Arc<Texture3DData>),
    TextureCube(Arc<TextureCubeData>)
}

pub struct BindGroupEncodingContext {
    context_id: usize
}

pub struct BindGroupEncoder<'a, B> {
    context: &'a mut BindGroupEncodingContext,
    bindings: B
}

impl<'a, B> BindGroupEncoder<'a, B> {
    pub fn add_buffer<T>(self, binding: &BufferBinding<T>) -> BindGroupEncoder<'a, (BindingDescriptor, B)> {
        BindGroupEncoder {
            context: self.context,
            bindings: (BindingDescriptor {
                internal: BindingDescriptorInternal::BufferView {
                    index: binding.index,
                    buffer_data: binding.buffer_view.buffer_data().clone(),
                    offset: binding.buffer_view.offset_in_bytes(),
                    size: binding.size_in_bytes
                }
            })
        }
    }

    pub fn add_float_sampler_2d(self, binding: &FloatSampler2DBinding) -> BindGroupEncoder<'a, (BindingDescriptor, B)> {
        BindGroupEncoder {
            context: self.context,
            bindings: (BindingDescriptor {
                internal: BindingDescriptorInternal::SampledTexture {
                    unit: binding.index,
                    sampler_data: binding.resource.sampler_data.clone(),
                    texture_data: TextureData::Texture2D(binding.resource.texture_data.clone())
                }
            })
        }
    }

    pub fn add_float_sampler_2d_array(self, binding: &FloatSampler2DArrayBinding) -> BindGroupEncoder<'a, (BindingDescriptor, B)> {
        BindGroupEncoder {
            context: self.context,
            bindings: (BindingDescriptor {
                internal: BindingDescriptorInternal::SampledTexture {
                    unit: binding.index,
                    sampler_data: binding.resource.sampler_data.clone(),
                    texture_data: TextureData::Texture2DArray(binding.resource.texture_data.clone())
                }
            })
        }
    }

    pub fn add_float_sampler_3d(self, binding: &FloatSampler3DBinding) -> BindGroupEncoder<'a, (BindingDescriptor, B)> {
        BindGroupEncoder {
            context: self.context,
            bindings: (BindingDescriptor {
                internal: BindingDescriptorInternal::SampledTexture {
                    unit: binding.index,
                    sampler_data: binding.resource.sampler_data.clone(),
                    texture_data: TextureData::Texture3D(binding.resource.texture_data.clone())
                }
            })
        }
    }

    pub fn add_float_sampler_cube(self, binding: &FloatSamplerCubeBinding) -> BindGroupEncoder<'a, (BindingDescriptor, B)> {
        BindGroupEncoder {
            context: self.context,
            bindings: (BindingDescriptor {
                internal: BindingDescriptorInternal::SampledTexture {
                    unit: binding.index,
                    sampler_data: binding.resource.sampler_data.clone(),
                    texture_data: TextureData::TextureCube(binding.resource.texture_data.clone())
                }
            })
        }
    }

    pub fn add_integer_sampler_2d(self, binding: &IntegerSampler2DBinding) -> BindGroupEncoder<'a, (BindingDescriptor, B)> {
        BindGroupEncoder {
            context: self.context,
            bindings: (BindingDescriptor {
                internal: BindingDescriptorInternal::SampledTexture {
                    unit: binding.index,
                    sampler_data: binding.resource.sampler_data.clone(),
                    texture_data: TextureData::Texture2D(binding.resource.texture_data.clone())
                }
            })
        }
    }

    pub fn add_integer_sampler_2d_array(self, binding: &IntegerSampler2DArrayBinding) -> BindGroupEncoder<'a, (BindingDescriptor, B)> {
        BindGroupEncoder {
            context: self.context,
            bindings: (BindingDescriptor {
                internal: BindingDescriptorInternal::SampledTexture {
                    unit: binding.index,
                    sampler_data: binding.resource.sampler_data.clone(),
                    texture_data: TextureData::Texture2DArray(binding.resource.texture_data.clone())
                }
            })
        }
    }

    pub fn add_integer_sampler_3d(self, binding: &IntegerSampler3DBinding) -> BindGroupEncoder<'a, (BindingDescriptor, B)> {
        BindGroupEncoder {
            context: self.context,
            bindings: (BindingDescriptor {
                internal: BindingDescriptorInternal::SampledTexture {
                    unit: binding.index,
                    sampler_data: binding.resource.sampler_data.clone(),
                    texture_data: TextureData::Texture3D(binding.resource.texture_data.clone())
                }
            })
        }
    }

    pub fn add_integer_sampler_cube(self, binding: &IntegerSamplerCubeBinding) -> BindGroupEncoder<'a, (BindingDescriptor, B)> {
        BindGroupEncoder {
            context: self.context,
            bindings: (BindingDescriptor {
                internal: BindingDescriptorInternal::SampledTexture {
                    unit: binding.index,
                    sampler_data: binding.resource.sampler_data.clone(),
                    texture_data: TextureData::TextureCube(binding.resource.texture_data.clone())
                }
            })
        }
    }

    pub fn add_unsigned_integer_sampler_2d(self, binding: &UnsignedIntegerSampler2DBinding) -> BindGroupEncoder<'a, (BindingDescriptor, B)> {
        BindGroupEncoder {
            context: self.context,
            bindings: (BindingDescriptor {
                internal: BindingDescriptorInternal::SampledTexture {
                    unit: binding.index,
                    sampler_data: binding.resource.sampler_data.clone(),
                    texture_data: TextureData::Texture2D(binding.resource.texture_data.clone())
                }
            })
        }
    }

    pub fn add_unsigned_integer_sampler_2d_array(self, binding: &UnsignedIntegerSampler2DArrayBinding) -> BindGroupEncoder<'a, (BindingDescriptor, B)> {
        BindGroupEncoder {
            context: self.context,
            bindings: (BindingDescriptor {
                internal: BindingDescriptorInternal::SampledTexture {
                    unit: binding.index,
                    sampler_data: binding.resource.sampler_data.clone(),
                    texture_data: TextureData::Texture2DArray(binding.resource.texture_data.clone())
                }
            })
        }
    }

    pub fn add_unsigned_integer_sampler_3d(self, binding: &UnsignedIntegerSampler3DBinding) -> BindGroupEncoder<'a, (BindingDescriptor, B)> {
        BindGroupEncoder {
            context: self.context,
            bindings: (BindingDescriptor {
                internal: BindingDescriptorInternal::SampledTexture {
                    unit: binding.index,
                    sampler_data: binding.resource.sampler_data.clone(),
                    texture_data: TextureData::Texture3D(binding.resource.texture_data.clone())
                }
            })
        }
    }

    pub fn add_unsigned_integer_sampler_cube(self, binding: &UnsignedIntegerSamplerCubeBinding) -> BindGroupEncoder<'a, (BindingDescriptor, B)> {
        BindGroupEncoder {
            context: self.context,
            bindings: (BindingDescriptor {
                internal: BindingDescriptorInternal::SampledTexture {
                    unit: binding.index,
                    sampler_data: binding.resource.sampler_data.clone(),
                    texture_data: TextureData::TextureCube(binding.resource.texture_data.clone())
                }
            })
        }
    }


    pub fn add_shadow_sampler_2d(self, binding: &ShadowSampler2DBinding) -> BindGroupEncoder<'a, (BindingDescriptor, B)> {
        BindGroupEncoder {
            context: self.context,
            bindings: (BindingDescriptor {
                internal: BindingDescriptorInternal::SampledTexture {
                    unit: binding.index,
                    sampler_data: binding.resource.sampler_data.clone(),
                    texture_data: TextureData::Texture2D(binding.resource.texture_data.clone())
                }
            })
        }
    }

    pub fn add_shadow_sampler_2d_array(self, binding: &ShadowSampler2DArrayBinding) -> BindGroupEncoder<'a, (BindingDescriptor, B)> {
        BindGroupEncoder {
            context: self.context,
            bindings: (BindingDescriptor {
                internal: BindingDescriptorInternal::SampledTexture {
                    unit: binding.index,
                    sampler_data: binding.resource.sampler_data.clone(),
                    texture_data: TextureData::Texture2DArray(binding.resource.texture_data.clone())
                }
            })
        }
    }

    pub fn add_shadow_sampler_cube(self, binding: &ShadowSamplerCubeBinding) -> BindGroupEncoder<'a, (BindingDescriptor, B)> {
        BindGroupEncoder {
            context: self.context,
            bindings: (BindingDescriptor {
                internal: BindingDescriptorInternal::SampledTexture {
                    unit: binding.index,
                    sampler_data: binding.resource.sampler_data.clone(),
                    texture_data: TextureData::TextureCube(binding.resource.texture_data.clone())
                }
            })
        }
    }
}

impl<'a> BindGroupEncoder<'a, ()> {
    pub fn finish(self) -> BindGroupEncoding<'a, [BindingDescriptor;0]> {
        BindGroupEncoding {
            context: self.context,
            descriptors: []
        }
    }
}

// TODO: implement this with const generics when possible

macro_rules! nest_pairs {
    ($head:tt) => ($head);
    ($head:tt, $($tail:tt),*) => (($head, nest_pairs!($($tail),*)));
}

macro_rules! nest_pairs_reverse {
    ([$head:tt] $($reverse:tt)*) => (nest_pairs!($head, $($reverse),*));
    ([$head:tt, $($tail:tt),*] $($reverse:tt)*) => {
        nest_pairs_reverse!([$($tail),*] $head $($reverse)*)
    }
}

macro_rules! generate_encoder_finish {
    ($n:tt, $($C:ident),*) => {
        impl<'a> BindGroupEncoder<'a, nest_pairs_reverse!([(), $($C),*])> {
            pub fn finish(self) -> BindGroupEncoding<'a, [BindingDescriptor;$n]> {
                let nest_pairs_reverse!([_, $($C),*]) = self.bindings;

                BindGroupEncoding {
                    context: self.context,
                    descriptors: [$($C),*]
                }
            }
        }
    }
}

generate_encoder_finish!(1, BindingDescriptor);
generate_encoder_finish!(2, BindingDescriptor, BindingDescriptor);
generate_encoder_finish!(3, BindingDescriptor, BindingDescriptor, BindingDescriptor);
generate_encoder_finish!(4, BindingDescriptor, BindingDescriptor, BindingDescriptor, BindingDescriptor);
generate_encoder_finish!(
    5,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
);
generate_encoder_finish!(
    6,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
);
generate_encoder_finish!(
    7,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
);
generate_encoder_finish!(
    8,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
);
generate_encoder_finish!(
    9,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
);
generate_encoder_finish!(
    10,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
);
generate_encoder_finish!(
    11,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
);
generate_encoder_finish!(
    12,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
);
generate_encoder_finish!(
    13,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
);
generate_encoder_finish!(
    14,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
);
generate_encoder_finish!(
    15,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
);
generate_encoder_finish!(
    16,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
    BindingDescriptor,
);
