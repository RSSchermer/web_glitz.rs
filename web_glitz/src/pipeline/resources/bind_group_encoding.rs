use std::borrow::Borrow;
use std::sync::Arc;

use crate::buffer::BufferData;
use crate::image::texture_2d::Texture2DData;
use crate::image::texture_2d_array::Texture2DArrayData;
use crate::image::texture_3d::Texture3DData;
use crate::image::texture_cube::TextureCubeData;
use crate::pipeline::resources::binding::{
    BufferBinding, FloatSampler2DArrayBinding, FloatSampler2DBinding, FloatSampler3DBinding,
    FloatSamplerCubeBinding, IntegerSampler2DArrayBinding, IntegerSampler2DBinding,
    IntegerSampler3DBinding, IntegerSamplerCubeBinding, ShadowSampler2DArrayBinding,
    ShadowSampler2DBinding, ShadowSamplerCubeBinding, UnsignedIntegerSampler2DArrayBinding,
    UnsignedIntegerSampler2DBinding, UnsignedIntegerSampler3DBinding,
    UnsignedIntegerSamplerCubeBinding,
};
use crate::runtime::state::{BufferRange, ContextUpdate};
use crate::runtime::Connection;
use crate::sampler::SamplerData;

pub struct BindGroupEncoding<'a, B>
where
    B: Borrow<[BindingDescriptor]> + 'static,
{
    #[allow(dead_code)]
    context: &'a mut BindGroupEncodingContext,
    descriptors: B,
}

impl<'a, B> BindGroupEncoding<'a, B>
where
    B: Borrow<[BindingDescriptor]> + 'static,
{
    pub(crate) fn into_descriptors(self) -> B {
        self.descriptors
    }
}

impl<'a> BindGroupEncoding<'a, [BindingDescriptor; 0]> {
    pub fn empty(context: &'a mut BindGroupEncodingContext) -> Self {
        BindGroupEncoding {
            context,
            descriptors: [],
        }
    }
}

pub struct BindingDescriptor {
    internal: BindingDescriptorInternal,
}

impl BindingDescriptor {
    pub(crate) fn bind(&self, connection: &mut Connection) {
        let (gl, state) = unsafe { connection.unpack_mut() };

        match &self.internal {
            BindingDescriptorInternal::BufferView {
                index,
                buffer_data,
                offset,
                size,
            } => unsafe {
                buffer_data
                    .id()
                    .unwrap()
                    .with_value_unchecked(|buffer_object| {
                        state.set_active_uniform_buffer_index(*index);

                        state
                            .set_bound_uniform_buffer_range(BufferRange::OffsetSize(
                                buffer_object,
                                *offset as u32,
                                *size as u32,
                            ))
                            .apply(gl)
                            .unwrap();
                    });
            },
            BindingDescriptorInternal::SampledTexture {
                unit,
                sampler_data,
                texture_data,
            } => {
                state.set_active_texture(*unit).apply(gl).unwrap();

                match texture_data {
                    TextureData::Texture2D(data) => unsafe {
                        data.id().unwrap().with_value_unchecked(|texture_object| {
                            state
                                .set_bound_texture_2d(Some(texture_object))
                                .apply(gl)
                                .unwrap();
                        });
                    },
                    TextureData::Texture2DArray(data) => unsafe {
                        data.id().unwrap().with_value_unchecked(|texture_object| {
                            state
                                .set_bound_texture_2d_array(Some(texture_object))
                                .apply(gl)
                                .unwrap();
                        });
                    },
                    TextureData::Texture3D(data) => unsafe {
                        data.id().unwrap().with_value_unchecked(|texture_object| {
                            state
                                .set_bound_texture_3d(Some(texture_object))
                                .apply(gl)
                                .unwrap();
                        });
                    },
                    TextureData::TextureCube(data) => unsafe {
                        data.id().unwrap().with_value_unchecked(|texture_object| {
                            state
                                .set_bound_texture_cube_map(Some(texture_object))
                                .apply(gl)
                                .unwrap();
                        });
                    },
                }

                unsafe {
                    sampler_data
                        .id()
                        .unwrap()
                        .with_value_unchecked(|sampler_object| {
                            state
                                .set_bound_sampler(*unit, Some(sampler_object))
                                .apply(gl)
                                .unwrap();
                        });
                }
            }
        }
    }
}

enum BindingDescriptorInternal {
    BufferView {
        index: u32,
        buffer_data: Arc<BufferData>,
        offset: usize,
        size: usize,
    },
    SampledTexture {
        unit: u32,
        sampler_data: Arc<SamplerData>,
        texture_data: TextureData,
    },
}

enum TextureData {
    Texture2D(Arc<Texture2DData>),
    Texture2DArray(Arc<Texture2DArrayData>),
    Texture3D(Arc<Texture3DData>),
    TextureCube(Arc<TextureCubeData>),
}

pub struct BindGroupEncodingContext {
    context_id: usize,
}

impl BindGroupEncodingContext {
    pub(crate) fn new(context_id: usize) -> Self {
        BindGroupEncodingContext { context_id }
    }
}

pub struct BindGroupEncoder<'a, B> {
    context: &'a mut BindGroupEncodingContext,
    bindings: B,
}

impl<'a> BindGroupEncoder<'a, ()> {
    pub fn new(context: &'a mut BindGroupEncodingContext) -> Self {
        BindGroupEncoder {
            context,
            bindings: ()
        }
    }
}

impl<'a, B> BindGroupEncoder<'a, B> {
    pub fn add_buffer<T>(
        self,
        binding: &BufferBinding<T>,
    ) -> BindGroupEncoder<'a, (BindingDescriptor, B)> {
        if binding.buffer_view.buffer_data().context_id() != self.context.context_id {
            panic!("Buffer does not belong to same context as the bind group encoder");
        }

        BindGroupEncoder {
            context: self.context,
            bindings: (
                BindingDescriptor {
                    internal: BindingDescriptorInternal::BufferView {
                        index: binding.index,
                        buffer_data: binding.buffer_view.buffer_data().clone(),
                        offset: binding.buffer_view.offset_in_bytes(),
                        size: binding.size_in_bytes,
                    },
                },
                self.bindings,
            ),
        }
    }

    pub fn add_float_sampler_2d(
        self,
        binding: &FloatSampler2DBinding,
    ) -> BindGroupEncoder<'a, (BindingDescriptor, B)> {
        if binding.resource.texture_data.context_id() != self.context.context_id {
            panic!("Texture does not belong to same context as the bind group encoder");
        }

        BindGroupEncoder {
            context: self.context,
            bindings: (
                BindingDescriptor {
                    internal: BindingDescriptorInternal::SampledTexture {
                        unit: binding.texture_unit,
                        sampler_data: binding.resource.sampler_data.clone(),
                        texture_data: TextureData::Texture2D(binding.resource.texture_data.clone()),
                    },
                },
                self.bindings,
            ),
        }
    }

    pub fn add_float_sampler_2d_array(
        self,
        binding: &FloatSampler2DArrayBinding,
    ) -> BindGroupEncoder<'a, (BindingDescriptor, B)> {
        if binding.resource.texture_data.context_id() != self.context.context_id {
            panic!("Texture does not belong to same context as the bind group encoder");
        }

        BindGroupEncoder {
            context: self.context,
            bindings: (
                BindingDescriptor {
                    internal: BindingDescriptorInternal::SampledTexture {
                        unit: binding.texture_unit,
                        sampler_data: binding.resource.sampler_data.clone(),
                        texture_data: TextureData::Texture2DArray(
                            binding.resource.texture_data.clone(),
                        ),
                    },
                },
                self.bindings,
            ),
        }
    }

    pub fn add_float_sampler_3d(
        self,
        binding: &FloatSampler3DBinding,
    ) -> BindGroupEncoder<'a, (BindingDescriptor, B)> {
        if binding.resource.texture_data.context_id() != self.context.context_id {
            panic!("Texture does not belong to same context as the bind group encoder");
        }

        BindGroupEncoder {
            context: self.context,
            bindings: (
                BindingDescriptor {
                    internal: BindingDescriptorInternal::SampledTexture {
                        unit: binding.texture_unit,
                        sampler_data: binding.resource.sampler_data.clone(),
                        texture_data: TextureData::Texture3D(binding.resource.texture_data.clone()),
                    },
                },
                self.bindings,
            ),
        }
    }

    pub fn add_float_sampler_cube(
        self,
        binding: &FloatSamplerCubeBinding,
    ) -> BindGroupEncoder<'a, (BindingDescriptor, B)> {
        if binding.resource.texture_data.context_id() != self.context.context_id {
            panic!("Texture does not belong to same context as the bind group encoder");
        }

        BindGroupEncoder {
            context: self.context,
            bindings: (
                BindingDescriptor {
                    internal: BindingDescriptorInternal::SampledTexture {
                        unit: binding.texture_unit,
                        sampler_data: binding.resource.sampler_data.clone(),
                        texture_data: TextureData::TextureCube(
                            binding.resource.texture_data.clone(),
                        ),
                    },
                },
                self.bindings,
            ),
        }
    }

    pub fn add_integer_sampler_2d(
        self,
        binding: &IntegerSampler2DBinding,
    ) -> BindGroupEncoder<'a, (BindingDescriptor, B)> {
        if binding.resource.texture_data.context_id() != self.context.context_id {
            panic!("Texture does not belong to same context as the bind group encoder");
        }

        BindGroupEncoder {
            context: self.context,
            bindings: (
                BindingDescriptor {
                    internal: BindingDescriptorInternal::SampledTexture {
                        unit: binding.texture_unit,
                        sampler_data: binding.resource.sampler_data.clone(),
                        texture_data: TextureData::Texture2D(binding.resource.texture_data.clone()),
                    },
                },
                self.bindings,
            ),
        }
    }

    pub fn add_integer_sampler_2d_array(
        self,
        binding: &IntegerSampler2DArrayBinding,
    ) -> BindGroupEncoder<'a, (BindingDescriptor, B)> {
        if binding.resource.texture_data.context_id() != self.context.context_id {
            panic!("Texture does not belong to same context as the bind group encoder");
        }

        BindGroupEncoder {
            context: self.context,
            bindings: (
                BindingDescriptor {
                    internal: BindingDescriptorInternal::SampledTexture {
                        unit: binding.texture_unit,
                        sampler_data: binding.resource.sampler_data.clone(),
                        texture_data: TextureData::Texture2DArray(
                            binding.resource.texture_data.clone(),
                        ),
                    },
                },
                self.bindings,
            ),
        }
    }

    pub fn add_integer_sampler_3d(
        self,
        binding: &IntegerSampler3DBinding,
    ) -> BindGroupEncoder<'a, (BindingDescriptor, B)> {
        if binding.resource.texture_data.context_id() != self.context.context_id {
            panic!("Texture does not belong to same context as the bind group encoder");
        }

        BindGroupEncoder {
            context: self.context,
            bindings: (
                BindingDescriptor {
                    internal: BindingDescriptorInternal::SampledTexture {
                        unit: binding.texture_unit,
                        sampler_data: binding.resource.sampler_data.clone(),
                        texture_data: TextureData::Texture3D(binding.resource.texture_data.clone()),
                    },
                },
                self.bindings,
            ),
        }
    }

    pub fn add_integer_sampler_cube(
        self,
        binding: &IntegerSamplerCubeBinding,
    ) -> BindGroupEncoder<'a, (BindingDescriptor, B)> {
        if binding.resource.texture_data.context_id() != self.context.context_id {
            panic!("Texture does not belong to same context as the bind group encoder");
        }

        BindGroupEncoder {
            context: self.context,
            bindings: (
                BindingDescriptor {
                    internal: BindingDescriptorInternal::SampledTexture {
                        unit: binding.texture_unit,
                        sampler_data: binding.resource.sampler_data.clone(),
                        texture_data: TextureData::TextureCube(
                            binding.resource.texture_data.clone(),
                        ),
                    },
                },
                self.bindings,
            ),
        }
    }

    pub fn add_unsigned_integer_sampler_2d(
        self,
        binding: &UnsignedIntegerSampler2DBinding,
    ) -> BindGroupEncoder<'a, (BindingDescriptor, B)> {
        if binding.resource.texture_data.context_id() != self.context.context_id {
            panic!("Texture does not belong to same context as the bind group encoder");
        }

        BindGroupEncoder {
            context: self.context,
            bindings: (
                BindingDescriptor {
                    internal: BindingDescriptorInternal::SampledTexture {
                        unit: binding.texture_unit,
                        sampler_data: binding.resource.sampler_data.clone(),
                        texture_data: TextureData::Texture2D(binding.resource.texture_data.clone()),
                    },
                },
                self.bindings,
            ),
        }
    }

    pub fn add_unsigned_integer_sampler_2d_array(
        self,
        binding: &UnsignedIntegerSampler2DArrayBinding,
    ) -> BindGroupEncoder<'a, (BindingDescriptor, B)> {
        if binding.resource.texture_data.context_id() != self.context.context_id {
            panic!("Texture does not belong to same context as the bind group encoder");
        }

        BindGroupEncoder {
            context: self.context,
            bindings: (
                BindingDescriptor {
                    internal: BindingDescriptorInternal::SampledTexture {
                        unit: binding.texture_unit,
                        sampler_data: binding.resource.sampler_data.clone(),
                        texture_data: TextureData::Texture2DArray(
                            binding.resource.texture_data.clone(),
                        ),
                    },
                },
                self.bindings,
            ),
        }
    }

    pub fn add_unsigned_integer_sampler_3d(
        self,
        binding: &UnsignedIntegerSampler3DBinding,
    ) -> BindGroupEncoder<'a, (BindingDescriptor, B)> {
        if binding.resource.texture_data.context_id() != self.context.context_id {
            panic!("Texture does not belong to same context as the bind group encoder");
        }

        BindGroupEncoder {
            context: self.context,
            bindings: (
                BindingDescriptor {
                    internal: BindingDescriptorInternal::SampledTexture {
                        unit: binding.texture_unit,
                        sampler_data: binding.resource.sampler_data.clone(),
                        texture_data: TextureData::Texture3D(binding.resource.texture_data.clone()),
                    },
                },
                self.bindings,
            ),
        }
    }

    pub fn add_unsigned_integer_sampler_cube(
        self,
        binding: &UnsignedIntegerSamplerCubeBinding,
    ) -> BindGroupEncoder<'a, (BindingDescriptor, B)> {
        if binding.resource.texture_data.context_id() != self.context.context_id {
            panic!("Texture does not belong to same context as the bind group encoder");
        }

        BindGroupEncoder {
            context: self.context,
            bindings: (
                BindingDescriptor {
                    internal: BindingDescriptorInternal::SampledTexture {
                        unit: binding.texture_unit,
                        sampler_data: binding.resource.sampler_data.clone(),
                        texture_data: TextureData::TextureCube(
                            binding.resource.texture_data.clone(),
                        ),
                    },
                },
                self.bindings,
            ),
        }
    }

    pub fn add_shadow_sampler_2d(
        self,
        binding: &ShadowSampler2DBinding,
    ) -> BindGroupEncoder<'a, (BindingDescriptor, B)> {
        if binding.resource.texture_data.context_id() != self.context.context_id {
            panic!("Texture does not belong to same context as the bind group encoder");
        }

        BindGroupEncoder {
            context: self.context,
            bindings: (
                BindingDescriptor {
                    internal: BindingDescriptorInternal::SampledTexture {
                        unit: binding.texture_unit,
                        sampler_data: binding.resource.sampler_data.clone(),
                        texture_data: TextureData::Texture2D(binding.resource.texture_data.clone()),
                    },
                },
                self.bindings,
            ),
        }
    }

    pub fn add_shadow_sampler_2d_array(
        self,
        binding: &ShadowSampler2DArrayBinding,
    ) -> BindGroupEncoder<'a, (BindingDescriptor, B)> {
        if binding.resource.texture_data.context_id() != self.context.context_id {
            panic!("Texture does not belong to same context as the bind group encoder");
        }

        BindGroupEncoder {
            context: self.context,
            bindings: (
                BindingDescriptor {
                    internal: BindingDescriptorInternal::SampledTexture {
                        unit: binding.texture_unit,
                        sampler_data: binding.resource.sampler_data.clone(),
                        texture_data: TextureData::Texture2DArray(
                            binding.resource.texture_data.clone(),
                        ),
                    },
                },
                self.bindings,
            ),
        }
    }

    pub fn add_shadow_sampler_cube(
        self,
        binding: &ShadowSamplerCubeBinding,
    ) -> BindGroupEncoder<'a, (BindingDescriptor, B)> {
        if binding.resource.texture_data.context_id() != self.context.context_id {
            panic!("Texture does not belong to same context as the bind group encoder");
        }

        BindGroupEncoder {
            context: self.context,
            bindings: (
                BindingDescriptor {
                    internal: BindingDescriptorInternal::SampledTexture {
                        unit: binding.texture_unit,
                        sampler_data: binding.resource.sampler_data.clone(),
                        texture_data: TextureData::TextureCube(
                            binding.resource.texture_data.clone(),
                        ),
                    },
                },
                self.bindings,
            ),
        }
    }
}

impl<'a> BindGroupEncoder<'a, ()> {
    pub fn finish(self) -> BindGroupEncoding<'a, [BindingDescriptor; 0]> {
        BindGroupEncoding {
            context: self.context,
            descriptors: [],
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
    ($n:tt, $($C:ident|$I:ident),*) => {
        impl<'a> BindGroupEncoder<'a, nest_pairs_reverse!([(), $($C),*])> {
            pub fn finish(self) -> BindGroupEncoding<'a, [BindingDescriptor;$n]> {
                let nest_pairs_reverse!([_, $($I),*]) = self.bindings;

                BindGroupEncoding {
                    context: self.context,
                    descriptors: [$($I),*]
                }
            }
        }
    }
}

generate_encoder_finish!(1, BindingDescriptor | b0);
generate_encoder_finish!(2, BindingDescriptor | b0, BindingDescriptor | b1);
generate_encoder_finish!(
    3,
    BindingDescriptor | b0,
    BindingDescriptor | b1,
    BindingDescriptor | b2
);
generate_encoder_finish!(
    4,
    BindingDescriptor | b0,
    BindingDescriptor | b1,
    BindingDescriptor | b2,
    BindingDescriptor | b3
);
generate_encoder_finish!(
    5,
    BindingDescriptor | b0,
    BindingDescriptor | b1,
    BindingDescriptor | b2,
    BindingDescriptor | b3,
    BindingDescriptor | b4
);
generate_encoder_finish!(
    6,
    BindingDescriptor | b0,
    BindingDescriptor | b1,
    BindingDescriptor | b2,
    BindingDescriptor | b3,
    BindingDescriptor | b4,
    BindingDescriptor | b5
);
generate_encoder_finish!(
    7,
    BindingDescriptor | b0,
    BindingDescriptor | b1,
    BindingDescriptor | b2,
    BindingDescriptor | b3,
    BindingDescriptor | b4,
    BindingDescriptor | b5,
    BindingDescriptor | b6
);
generate_encoder_finish!(
    8,
    BindingDescriptor | b0,
    BindingDescriptor | b1,
    BindingDescriptor | b2,
    BindingDescriptor | b3,
    BindingDescriptor | b4,
    BindingDescriptor | b5,
    BindingDescriptor | b6,
    BindingDescriptor | b7
);
generate_encoder_finish!(
    9,
    BindingDescriptor | b0,
    BindingDescriptor | b1,
    BindingDescriptor | b2,
    BindingDescriptor | b3,
    BindingDescriptor | b4,
    BindingDescriptor | b5,
    BindingDescriptor | b6,
    BindingDescriptor | b7,
    BindingDescriptor | b8
);
generate_encoder_finish!(
    10,
    BindingDescriptor | b0,
    BindingDescriptor | b1,
    BindingDescriptor | b2,
    BindingDescriptor | b3,
    BindingDescriptor | b4,
    BindingDescriptor | b5,
    BindingDescriptor | b6,
    BindingDescriptor | b7,
    BindingDescriptor | b8,
    BindingDescriptor | b9
);
generate_encoder_finish!(
    11,
    BindingDescriptor | b0,
    BindingDescriptor | b1,
    BindingDescriptor | b2,
    BindingDescriptor | b3,
    BindingDescriptor | b4,
    BindingDescriptor | b5,
    BindingDescriptor | b6,
    BindingDescriptor | b7,
    BindingDescriptor | b8,
    BindingDescriptor | b9,
    BindingDescriptor | b10
);
generate_encoder_finish!(
    12,
    BindingDescriptor | b0,
    BindingDescriptor | b1,
    BindingDescriptor | b2,
    BindingDescriptor | b3,
    BindingDescriptor | b4,
    BindingDescriptor | b5,
    BindingDescriptor | b6,
    BindingDescriptor | b7,
    BindingDescriptor | b8,
    BindingDescriptor | b9,
    BindingDescriptor | b10,
    BindingDescriptor | b11
);
generate_encoder_finish!(
    13,
    BindingDescriptor | b0,
    BindingDescriptor | b1,
    BindingDescriptor | b2,
    BindingDescriptor | b3,
    BindingDescriptor | b4,
    BindingDescriptor | b5,
    BindingDescriptor | b6,
    BindingDescriptor | b7,
    BindingDescriptor | b8,
    BindingDescriptor | b9,
    BindingDescriptor | b10,
    BindingDescriptor | b11,
    BindingDescriptor | b12
);
generate_encoder_finish!(
    14,
    BindingDescriptor | b0,
    BindingDescriptor | b1,
    BindingDescriptor | b2,
    BindingDescriptor | b3,
    BindingDescriptor | b4,
    BindingDescriptor | b5,
    BindingDescriptor | b6,
    BindingDescriptor | b7,
    BindingDescriptor | b8,
    BindingDescriptor | b9,
    BindingDescriptor | b10,
    BindingDescriptor | b11,
    BindingDescriptor | b12,
    BindingDescriptor | b13
);
generate_encoder_finish!(
    15,
    BindingDescriptor | b0,
    BindingDescriptor | b1,
    BindingDescriptor | b2,
    BindingDescriptor | b3,
    BindingDescriptor | b4,
    BindingDescriptor | b5,
    BindingDescriptor | b6,
    BindingDescriptor | b7,
    BindingDescriptor | b8,
    BindingDescriptor | b9,
    BindingDescriptor | b10,
    BindingDescriptor | b11,
    BindingDescriptor | b12,
    BindingDescriptor | b13,
    BindingDescriptor | b14
);
generate_encoder_finish!(
    16,
    BindingDescriptor | b0,
    BindingDescriptor | b1,
    BindingDescriptor | b2,
    BindingDescriptor | b3,
    BindingDescriptor | b4,
    BindingDescriptor | b5,
    BindingDescriptor | b6,
    BindingDescriptor | b7,
    BindingDescriptor | b8,
    BindingDescriptor | b9,
    BindingDescriptor | b10,
    BindingDescriptor | b11,
    BindingDescriptor | b12,
    BindingDescriptor | b13,
    BindingDescriptor | b14,
    BindingDescriptor | b15
);
