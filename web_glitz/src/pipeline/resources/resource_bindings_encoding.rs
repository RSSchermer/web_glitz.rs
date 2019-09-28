use std::borrow::Borrow;
use std::sync::Arc;

use crate::buffer::{BufferData, BufferView};
use crate::image::texture_2d::{
    FloatSampledTexture2D, IntegerSampledTexture2D, ShadowSampledTexture2D, Texture2DData,
    UnsignedIntegerSampledTexture2D,
};
use crate::image::texture_2d_array::{
    FloatSampledTexture2DArray, IntegerSampledTexture2DArray, ShadowSampledTexture2DArray,
    Texture2DArrayData, UnsignedIntegerSampledTexture2DArray,
};
use crate::image::texture_3d::{
    FloatSampledTexture3D, IntegerSampledTexture3D, Texture3DData, UnsignedIntegerSampledTexture3D,
};
use crate::image::texture_cube::{
    FloatSampledTextureCube, IntegerSampledTextureCube, ShadowSampledTextureCube, TextureCubeData,
    UnsignedIntegerSampledTextureCube,
};
use crate::runtime::state::{BufferRange, ContextUpdate};
use crate::runtime::Connection;
use crate::sampler::SamplerData;

pub struct ResourceBindingsEncoding<'a, B>
where
    B: Borrow<[BindingDescriptor]> + 'static,
{
    #[allow(dead_code)]
    context: &'a mut ResourceBindingsEncodingContext,
    descriptors: B,
}

impl<'a, B> ResourceBindingsEncoding<'a, B>
where
    B: Borrow<[BindingDescriptor]> + 'static,
{
    pub(crate) fn into_descriptors(self) -> B {
        self.descriptors
    }
}

impl<'a> ResourceBindingsEncoding<'a, [BindingDescriptor; 0]> {
    pub fn empty(context: &'a mut ResourceBindingsEncodingContext) -> Self {
        ResourceBindingsEncoding {
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

pub struct ResourceBindingsEncodingContext {
    context_id: usize,
}

impl ResourceBindingsEncodingContext {
    pub(crate) fn new(context_id: usize) -> Self {
        ResourceBindingsEncodingContext { context_id }
    }
}

pub struct StaticResourceBindingsEncoder<'a, B> {
    context: &'a mut ResourceBindingsEncodingContext,
    bindings: B,
}

impl<'a> StaticResourceBindingsEncoder<'a, ()> {
    pub fn new(context: &'a mut ResourceBindingsEncodingContext) -> Self {
        StaticResourceBindingsEncoder {
            context,
            bindings: (),
        }
    }
}

impl<'a, B> StaticResourceBindingsEncoder<'a, B> {
    pub fn add_buffer_view<T>(
        self,
        slot: u32,
        buffer_view: BufferView<T>,
    ) -> StaticResourceBindingsEncoder<'a, (BindingDescriptor, B)> {
        if buffer_view.buffer_data().context_id() != self.context.context_id {
            panic!("Buffer does not belong to same context as the bind group encoder");
        }

        StaticResourceBindingsEncoder {
            context: self.context,
            bindings: (
                BindingDescriptor {
                    internal: BindingDescriptorInternal::BufferView {
                        index: slot,
                        buffer_data: buffer_view.buffer_data().clone(),
                        offset: buffer_view.offset_in_bytes(),
                        size: buffer_view.size_in_bytes(),
                    },
                },
                self.bindings,
            ),
        }
    }

    pub fn add_float_sampled_texture_2d(
        self,
        slot: u32,
        sampled_texture: FloatSampledTexture2D,
    ) -> StaticResourceBindingsEncoder<'a, (BindingDescriptor, B)> {
        if sampled_texture.texture_data.context_id() != self.context.context_id {
            panic!("Texture does not belong to same context as the bind group encoder");
        }

        StaticResourceBindingsEncoder {
            context: self.context,
            bindings: (
                BindingDescriptor {
                    internal: BindingDescriptorInternal::SampledTexture {
                        unit: slot,
                        sampler_data: sampled_texture.sampler_data.clone(),
                        texture_data: TextureData::Texture2D(sampled_texture.texture_data.clone()),
                    },
                },
                self.bindings,
            ),
        }
    }

    pub fn add_float_sampled_texture_2d_array(
        self,
        slot: u32,
        sampled_texture: FloatSampledTexture2DArray,
    ) -> StaticResourceBindingsEncoder<'a, (BindingDescriptor, B)> {
        if sampled_texture.texture_data.context_id() != self.context.context_id {
            panic!("Texture does not belong to same context as the bind group encoder");
        }

        StaticResourceBindingsEncoder {
            context: self.context,
            bindings: (
                BindingDescriptor {
                    internal: BindingDescriptorInternal::SampledTexture {
                        unit: slot,
                        sampler_data: sampled_texture.sampler_data.clone(),
                        texture_data: TextureData::Texture2DArray(
                            sampled_texture.texture_data.clone(),
                        ),
                    },
                },
                self.bindings,
            ),
        }
    }

    pub fn add_float_sampled_texture_3d(
        self,
        slot: u32,
        sampled_texture: FloatSampledTexture3D,
    ) -> StaticResourceBindingsEncoder<'a, (BindingDescriptor, B)> {
        if sampled_texture.texture_data.context_id() != self.context.context_id {
            panic!("Texture does not belong to same context as the bind group encoder");
        }

        StaticResourceBindingsEncoder {
            context: self.context,
            bindings: (
                BindingDescriptor {
                    internal: BindingDescriptorInternal::SampledTexture {
                        unit: slot,
                        sampler_data: sampled_texture.sampler_data.clone(),
                        texture_data: TextureData::Texture3D(sampled_texture.texture_data.clone()),
                    },
                },
                self.bindings,
            ),
        }
    }

    pub fn add_float_sampled_texture_cube(
        self,
        slot: u32,
        sampled_texture: FloatSampledTextureCube,
    ) -> StaticResourceBindingsEncoder<'a, (BindingDescriptor, B)> {
        if sampled_texture.texture_data.context_id() != self.context.context_id {
            panic!("Texture does not belong to same context as the bind group encoder");
        }

        StaticResourceBindingsEncoder {
            context: self.context,
            bindings: (
                BindingDescriptor {
                    internal: BindingDescriptorInternal::SampledTexture {
                        unit: slot,
                        sampler_data: sampled_texture.sampler_data.clone(),
                        texture_data: TextureData::TextureCube(
                            sampled_texture.texture_data.clone(),
                        ),
                    },
                },
                self.bindings,
            ),
        }
    }

    pub fn add_integer_sampled_texture_2d(
        self,
        slot: u32,
        sampled_texture: IntegerSampledTexture2D,
    ) -> StaticResourceBindingsEncoder<'a, (BindingDescriptor, B)> {
        if sampled_texture.texture_data.context_id() != self.context.context_id {
            panic!("Texture does not belong to same context as the bind group encoder");
        }

        StaticResourceBindingsEncoder {
            context: self.context,
            bindings: (
                BindingDescriptor {
                    internal: BindingDescriptorInternal::SampledTexture {
                        unit: slot,
                        sampler_data: sampled_texture.sampler_data.clone(),
                        texture_data: TextureData::Texture2D(sampled_texture.texture_data.clone()),
                    },
                },
                self.bindings,
            ),
        }
    }

    pub fn add_integer_sampled_texture_2d_array(
        self,
        slot: u32,
        sampled_texture: IntegerSampledTexture2DArray,
    ) -> StaticResourceBindingsEncoder<'a, (BindingDescriptor, B)> {
        if sampled_texture.texture_data.context_id() != self.context.context_id {
            panic!("Texture does not belong to same context as the bind group encoder");
        }

        StaticResourceBindingsEncoder {
            context: self.context,
            bindings: (
                BindingDescriptor {
                    internal: BindingDescriptorInternal::SampledTexture {
                        unit: slot,
                        sampler_data: sampled_texture.sampler_data.clone(),
                        texture_data: TextureData::Texture2DArray(
                            sampled_texture.texture_data.clone(),
                        ),
                    },
                },
                self.bindings,
            ),
        }
    }

    pub fn add_integer_sampled_texture_3d(
        self,
        slot: u32,
        sampled_texture: IntegerSampledTexture3D,
    ) -> StaticResourceBindingsEncoder<'a, (BindingDescriptor, B)> {
        if sampled_texture.texture_data.context_id() != self.context.context_id {
            panic!("Texture does not belong to same context as the bind group encoder");
        }

        StaticResourceBindingsEncoder {
            context: self.context,
            bindings: (
                BindingDescriptor {
                    internal: BindingDescriptorInternal::SampledTexture {
                        unit: slot,
                        sampler_data: sampled_texture.sampler_data.clone(),
                        texture_data: TextureData::Texture3D(sampled_texture.texture_data.clone()),
                    },
                },
                self.bindings,
            ),
        }
    }

    pub fn add_integer_sampled_texture_cube(
        self,
        slot: u32,
        sampled_texture: IntegerSampledTextureCube,
    ) -> StaticResourceBindingsEncoder<'a, (BindingDescriptor, B)> {
        if sampled_texture.texture_data.context_id() != self.context.context_id {
            panic!("Texture does not belong to same context as the bind group encoder");
        }

        StaticResourceBindingsEncoder {
            context: self.context,
            bindings: (
                BindingDescriptor {
                    internal: BindingDescriptorInternal::SampledTexture {
                        unit: slot,
                        sampler_data: sampled_texture.sampler_data.clone(),
                        texture_data: TextureData::TextureCube(
                            sampled_texture.texture_data.clone(),
                        ),
                    },
                },
                self.bindings,
            ),
        }
    }

    pub fn add_unsigned_integer_sampled_texture_2d(
        self,
        slot: u32,
        sampled_texture: UnsignedIntegerSampledTexture2D,
    ) -> StaticResourceBindingsEncoder<'a, (BindingDescriptor, B)> {
        if sampled_texture.texture_data.context_id() != self.context.context_id {
            panic!("Texture does not belong to same context as the bind group encoder");
        }

        StaticResourceBindingsEncoder {
            context: self.context,
            bindings: (
                BindingDescriptor {
                    internal: BindingDescriptorInternal::SampledTexture {
                        unit: slot,
                        sampler_data: sampled_texture.sampler_data.clone(),
                        texture_data: TextureData::Texture2D(sampled_texture.texture_data.clone()),
                    },
                },
                self.bindings,
            ),
        }
    }

    pub fn add_unsigned_integer_sampled_texture_2d_array(
        self,
        slot: u32,
        sampled_texture: UnsignedIntegerSampledTexture2DArray,
    ) -> StaticResourceBindingsEncoder<'a, (BindingDescriptor, B)> {
        if sampled_texture.texture_data.context_id() != self.context.context_id {
            panic!("Texture does not belong to same context as the bind group encoder");
        }

        StaticResourceBindingsEncoder {
            context: self.context,
            bindings: (
                BindingDescriptor {
                    internal: BindingDescriptorInternal::SampledTexture {
                        unit: slot,
                        sampler_data: sampled_texture.sampler_data.clone(),
                        texture_data: TextureData::Texture2DArray(
                            sampled_texture.texture_data.clone(),
                        ),
                    },
                },
                self.bindings,
            ),
        }
    }

    pub fn add_unsigned_integer_sampled_texture_3d(
        self,
        slot: u32,
        sampled_texture: UnsignedIntegerSampledTexture3D,
    ) -> StaticResourceBindingsEncoder<'a, (BindingDescriptor, B)> {
        if sampled_texture.texture_data.context_id() != self.context.context_id {
            panic!("Texture does not belong to same context as the bind group encoder");
        }

        StaticResourceBindingsEncoder {
            context: self.context,
            bindings: (
                BindingDescriptor {
                    internal: BindingDescriptorInternal::SampledTexture {
                        unit: slot,
                        sampler_data: sampled_texture.sampler_data.clone(),
                        texture_data: TextureData::Texture3D(sampled_texture.texture_data.clone()),
                    },
                },
                self.bindings,
            ),
        }
    }

    pub fn add_unsigned_integer_sampled_texture_cube(
        self,
        slot: u32,
        sampled_texture: UnsignedIntegerSampledTextureCube,
    ) -> StaticResourceBindingsEncoder<'a, (BindingDescriptor, B)> {
        if sampled_texture.texture_data.context_id() != self.context.context_id {
            panic!("Texture does not belong to same context as the bind group encoder");
        }

        StaticResourceBindingsEncoder {
            context: self.context,
            bindings: (
                BindingDescriptor {
                    internal: BindingDescriptorInternal::SampledTexture {
                        unit: slot,
                        sampler_data: sampled_texture.sampler_data.clone(),
                        texture_data: TextureData::TextureCube(
                            sampled_texture.texture_data.clone(),
                        ),
                    },
                },
                self.bindings,
            ),
        }
    }

    pub fn add_shadow_sampled_texture_2d(
        self,
        slot: u32,
        sampled_texture: ShadowSampledTexture2D,
    ) -> StaticResourceBindingsEncoder<'a, (BindingDescriptor, B)> {
        if sampled_texture.texture_data.context_id() != self.context.context_id {
            panic!("Texture does not belong to same context as the bind group encoder");
        }

        StaticResourceBindingsEncoder {
            context: self.context,
            bindings: (
                BindingDescriptor {
                    internal: BindingDescriptorInternal::SampledTexture {
                        unit: slot,
                        sampler_data: sampled_texture.sampler_data.clone(),
                        texture_data: TextureData::Texture2D(sampled_texture.texture_data.clone()),
                    },
                },
                self.bindings,
            ),
        }
    }

    pub fn add_shadow_sampled_texture_2d_array(
        self,
        slot: u32,
        sampled_texture: ShadowSampledTexture2DArray,
    ) -> StaticResourceBindingsEncoder<'a, (BindingDescriptor, B)> {
        if sampled_texture.texture_data.context_id() != self.context.context_id {
            panic!("Texture does not belong to same context as the bind group encoder");
        }

        StaticResourceBindingsEncoder {
            context: self.context,
            bindings: (
                BindingDescriptor {
                    internal: BindingDescriptorInternal::SampledTexture {
                        unit: slot,
                        sampler_data: sampled_texture.sampler_data.clone(),
                        texture_data: TextureData::Texture2DArray(
                            sampled_texture.texture_data.clone(),
                        ),
                    },
                },
                self.bindings,
            ),
        }
    }

    pub fn add_shadow_sampled_texture_cube(
        self,
        slot: u32,
        sampled_texture: ShadowSampledTextureCube,
    ) -> StaticResourceBindingsEncoder<'a, (BindingDescriptor, B)> {
        if sampled_texture.texture_data.context_id() != self.context.context_id {
            panic!("Texture does not belong to same context as the bind group encoder");
        }

        StaticResourceBindingsEncoder {
            context: self.context,
            bindings: (
                BindingDescriptor {
                    internal: BindingDescriptorInternal::SampledTexture {
                        unit: slot,
                        sampler_data: sampled_texture.sampler_data.clone(),
                        texture_data: TextureData::TextureCube(
                            sampled_texture.texture_data.clone(),
                        ),
                    },
                },
                self.bindings,
            ),
        }
    }
}

impl<'a> StaticResourceBindingsEncoder<'a, ()> {
    pub fn finish(self) -> ResourceBindingsEncoding<'a, [BindingDescriptor; 0]> {
        ResourceBindingsEncoding {
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
        impl<'a> StaticResourceBindingsEncoder<'a, nest_pairs_reverse!([(), $($C),*])> {
            pub fn finish(self) -> ResourceBindingsEncoding<'a, [BindingDescriptor;$n]> {
                let nest_pairs_reverse!([_, $($I),*]) = self.bindings;

                ResourceBindingsEncoding {
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
