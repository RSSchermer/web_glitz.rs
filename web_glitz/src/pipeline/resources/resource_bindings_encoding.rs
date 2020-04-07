use std::borrow::Borrow;
use std::marker;
use std::sync::Arc;

use crate::buffer::{BufferData, BufferView};
use crate::image::sampler::SamplerData;
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
use crate::pipeline::resources::resources::{BindGroup, BindGroupInternal};
use crate::runtime::state::{BufferRange, ContextUpdate};
use crate::runtime::Connection;

pub struct BindGroupEncoding<'a, E> {
    #[allow(dead_code)]
    pub(crate) context: &'a mut BindGroupEncodingContext,
    pub(crate) bindings: Vec<ResourceBindingDescriptor>,
    _marker: marker::PhantomData<E>,
}

impl<'a> BindGroupEncoding<'a, ()> {
    pub fn empty(context: &'a mut BindGroupEncodingContext) -> Self {
        BindGroupEncoding {
            context,
            bindings: Vec::new(),
            _marker: marker::PhantomData,
        }
    }
}

// TODO: separate "internal" type no longer needed now that resource binding descriptor is not
// public.
pub(crate) struct ResourceBindingDescriptor {
    internal: BindingDescriptorInternal,
}

impl ResourceBindingDescriptor {
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
                            .bind_uniform_buffer_range(BufferRange::OffsetSize(
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
                                .bind_texture_2d(Some(texture_object))
                                .apply(gl)
                                .unwrap();
                        });
                    },
                    TextureData::Texture2DArray(data) => unsafe {
                        data.id().unwrap().with_value_unchecked(|texture_object| {
                            state
                                .bind_texture_2d_array(Some(texture_object))
                                .apply(gl)
                                .unwrap();
                        });
                    },
                    TextureData::Texture3D(data) => unsafe {
                        data.id().unwrap().with_value_unchecked(|texture_object| {
                            state
                                .bind_texture_3d(Some(texture_object))
                                .apply(gl)
                                .unwrap();
                        });
                    },
                    TextureData::TextureCube(data) => unsafe {
                        data.id().unwrap().with_value_unchecked(|texture_object| {
                            state
                                .bind_texture_cube_map(Some(texture_object))
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
                                .bind_sampler(*unit, Some(sampler_object))
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

pub struct BindGroupEncoder<'a, E> {
    context: &'a mut BindGroupEncodingContext,
    bindings: Vec<ResourceBindingDescriptor>,
    _marker: marker::PhantomData<E>,
}

impl<'a> BindGroupEncoder<'a, ()> {
    pub fn new(context: &'a mut BindGroupEncodingContext, size_hint: Option<usize>) -> Self {
        let bindings = if let Some(size_hint) = size_hint {
            Vec::with_capacity(size_hint)
        } else {
            Vec::new()
        };

        BindGroupEncoder {
            context,
            bindings,
            _marker: marker::PhantomData,
        }
    }
}

impl<'a, E> BindGroupEncoder<'a, E> {
    pub fn add_buffer_view<'b, T>(
        self,
        slot: u32,
        buffer_view: BufferView<'b, T>,
    ) -> BindGroupEncoder<'a, (BufferView<'b, T>, E)> {
        let BindGroupEncoder {
            context,
            mut bindings,
            ..
        } = self;

        if buffer_view.buffer_data().context_id() != context.context_id {
            panic!("Buffer does not belong to same context as the bind group encoder");
        }

        bindings.push(ResourceBindingDescriptor {
            internal: BindingDescriptorInternal::BufferView {
                index: slot,
                buffer_data: buffer_view.buffer_data().clone(),
                offset: buffer_view.offset_in_bytes(),
                size: buffer_view.size_in_bytes(),
            },
        });

        BindGroupEncoder {
            context,
            bindings,
            _marker: marker::PhantomData,
        }
    }

    pub fn add_float_sampled_texture_2d<'b>(
        self,
        slot: u32,
        sampled_texture: FloatSampledTexture2D<'b>,
    ) -> BindGroupEncoder<'a, (FloatSampledTexture2D<'b>, E)> {
        let BindGroupEncoder {
            context,
            mut bindings,
            ..
        } = self;

        if sampled_texture.texture_data.context_id() != context.context_id {
            panic!("Texture does not belong to same context as the bind group encoder");
        }

        bindings.push(ResourceBindingDescriptor {
            internal: BindingDescriptorInternal::SampledTexture {
                unit: slot,
                sampler_data: sampled_texture.sampler_data.clone(),
                texture_data: TextureData::Texture2D(sampled_texture.texture_data.clone()),
            },
        });

        BindGroupEncoder {
            context,
            bindings,
            _marker: marker::PhantomData,
        }
    }

    pub fn add_float_sampled_texture_2d_array<'b>(
        self,
        slot: u32,
        sampled_texture: FloatSampledTexture2DArray<'b>,
    ) -> BindGroupEncoder<'a, (FloatSampledTexture2DArray<'b>, E)> {
        let BindGroupEncoder {
            context,
            mut bindings,
            ..
        } = self;

        if sampled_texture.texture_data.context_id() != context.context_id {
            panic!("Texture does not belong to same context as the bind group encoder");
        }

        bindings.push(ResourceBindingDescriptor {
            internal: BindingDescriptorInternal::SampledTexture {
                unit: slot,
                sampler_data: sampled_texture.sampler_data.clone(),
                texture_data: TextureData::Texture2DArray(sampled_texture.texture_data.clone()),
            },
        });

        BindGroupEncoder {
            context,
            bindings,
            _marker: marker::PhantomData,
        }
    }

    pub fn add_float_sampled_texture_3d<'b>(
        self,
        slot: u32,
        sampled_texture: FloatSampledTexture3D<'b>,
    ) -> BindGroupEncoder<'a, (FloatSampledTexture3D<'b>, E)> {
        let BindGroupEncoder {
            context,
            mut bindings,
            ..
        } = self;

        if sampled_texture.texture_data.context_id() != context.context_id {
            panic!("Texture does not belong to same context as the bind group encoder");
        }

        bindings.push(ResourceBindingDescriptor {
            internal: BindingDescriptorInternal::SampledTexture {
                unit: slot,
                sampler_data: sampled_texture.sampler_data.clone(),
                texture_data: TextureData::Texture3D(sampled_texture.texture_data.clone()),
            },
        });

        BindGroupEncoder {
            context,
            bindings,
            _marker: marker::PhantomData,
        }
    }

    pub fn add_float_sampled_texture_cube<'b>(
        self,
        slot: u32,
        sampled_texture: FloatSampledTextureCube<'b>,
    ) -> BindGroupEncoder<'a, (FloatSampledTextureCube<'b>, E)> {
        let BindGroupEncoder {
            context,
            mut bindings,
            ..
        } = self;

        if sampled_texture.texture_data.context_id() != context.context_id {
            panic!("Texture does not belong to same context as the bind group encoder");
        }

        bindings.push(ResourceBindingDescriptor {
            internal: BindingDescriptorInternal::SampledTexture {
                unit: slot,
                sampler_data: sampled_texture.sampler_data.clone(),
                texture_data: TextureData::TextureCube(sampled_texture.texture_data.clone()),
            },
        });

        BindGroupEncoder {
            context,
            bindings,
            _marker: marker::PhantomData,
        }
    }

    pub fn add_integer_sampled_texture_2d<'b>(
        self,
        slot: u32,
        sampled_texture: IntegerSampledTexture2D<'b>,
    ) -> BindGroupEncoder<'a, (IntegerSampledTexture2D<'b>, E)> {
        let BindGroupEncoder {
            context,
            mut bindings,
            ..
        } = self;

        if sampled_texture.texture_data.context_id() != context.context_id {
            panic!("Texture does not belong to same context as the bind group encoder");
        }

        bindings.push(ResourceBindingDescriptor {
            internal: BindingDescriptorInternal::SampledTexture {
                unit: slot,
                sampler_data: sampled_texture.sampler_data.clone(),
                texture_data: TextureData::Texture2D(sampled_texture.texture_data.clone()),
            },
        });

        BindGroupEncoder {
            context,
            bindings,
            _marker: marker::PhantomData,
        }
    }

    pub fn add_integer_sampled_texture_2d_array<'b>(
        self,
        slot: u32,
        sampled_texture: IntegerSampledTexture2DArray<'b>,
    ) -> BindGroupEncoder<'a, (IntegerSampledTexture2DArray<'b>, E)> {
        let BindGroupEncoder {
            context,
            mut bindings,
            ..
        } = self;

        if sampled_texture.texture_data.context_id() != context.context_id {
            panic!("Texture does not belong to same context as the bind group encoder");
        }

        bindings.push(ResourceBindingDescriptor {
            internal: BindingDescriptorInternal::SampledTexture {
                unit: slot,
                sampler_data: sampled_texture.sampler_data.clone(),
                texture_data: TextureData::Texture2DArray(sampled_texture.texture_data.clone()),
            },
        });

        BindGroupEncoder {
            context,
            bindings,
            _marker: marker::PhantomData,
        }
    }

    pub fn add_integer_sampled_texture_3d<'b>(
        self,
        slot: u32,
        sampled_texture: IntegerSampledTexture3D<'b>,
    ) -> BindGroupEncoder<'a, (IntegerSampledTexture3D<'b>, E)> {
        let BindGroupEncoder {
            context,
            mut bindings,
            ..
        } = self;

        if sampled_texture.texture_data.context_id() != context.context_id {
            panic!("Texture does not belong to same context as the bind group encoder");
        }

        bindings.push(ResourceBindingDescriptor {
            internal: BindingDescriptorInternal::SampledTexture {
                unit: slot,
                sampler_data: sampled_texture.sampler_data.clone(),
                texture_data: TextureData::Texture3D(sampled_texture.texture_data.clone()),
            },
        });

        BindGroupEncoder {
            context,
            bindings,
            _marker: marker::PhantomData,
        }
    }

    pub fn add_integer_sampled_texture_cube<'b>(
        self,
        slot: u32,
        sampled_texture: IntegerSampledTextureCube<'b>,
    ) -> BindGroupEncoder<'a, (IntegerSampledTextureCube<'b>, E)> {
        let BindGroupEncoder {
            context,
            mut bindings,
            ..
        } = self;

        if sampled_texture.texture_data.context_id() != context.context_id {
            panic!("Texture does not belong to same context as the bind group encoder");
        }

        bindings.push(ResourceBindingDescriptor {
            internal: BindingDescriptorInternal::SampledTexture {
                unit: slot,
                sampler_data: sampled_texture.sampler_data.clone(),
                texture_data: TextureData::TextureCube(sampled_texture.texture_data.clone()),
            },
        });

        BindGroupEncoder {
            context,
            bindings,
            _marker: marker::PhantomData,
        }
    }

    pub fn add_unsigned_integer_sampled_texture_2d<'b>(
        self,
        slot: u32,
        sampled_texture: UnsignedIntegerSampledTexture2D<'b>,
    ) -> BindGroupEncoder<'a, (UnsignedIntegerSampledTexture2D<'b>, E)> {
        let BindGroupEncoder {
            context,
            mut bindings,
            ..
        } = self;

        if sampled_texture.texture_data.context_id() != context.context_id {
            panic!("Texture does not belong to same context as the bind group encoder");
        }

        bindings.push(ResourceBindingDescriptor {
            internal: BindingDescriptorInternal::SampledTexture {
                unit: slot,
                sampler_data: sampled_texture.sampler_data.clone(),
                texture_data: TextureData::Texture2D(sampled_texture.texture_data.clone()),
            },
        });

        BindGroupEncoder {
            context,
            bindings,
            _marker: marker::PhantomData,
        }
    }

    pub fn add_unsigned_integer_sampled_texture_2d_array<'b>(
        self,
        slot: u32,
        sampled_texture: UnsignedIntegerSampledTexture2DArray<'b>,
    ) -> BindGroupEncoder<'a, (UnsignedIntegerSampledTexture2DArray<'b>, E)> {
        let BindGroupEncoder {
            context,
            mut bindings,
            ..
        } = self;

        if sampled_texture.texture_data.context_id() != context.context_id {
            panic!("Texture does not belong to same context as the bind group encoder");
        }

        bindings.push(ResourceBindingDescriptor {
            internal: BindingDescriptorInternal::SampledTexture {
                unit: slot,
                sampler_data: sampled_texture.sampler_data.clone(),
                texture_data: TextureData::Texture2DArray(sampled_texture.texture_data.clone()),
            },
        });

        BindGroupEncoder {
            context,
            bindings,
            _marker: marker::PhantomData,
        }
    }

    pub fn add_unsigned_integer_sampled_texture_3d<'b>(
        self,
        slot: u32,
        sampled_texture: UnsignedIntegerSampledTexture3D<'b>,
    ) -> BindGroupEncoder<'a, (UnsignedIntegerSampledTexture3D<'b>, E)> {
        let BindGroupEncoder {
            context,
            mut bindings,
            ..
        } = self;

        if sampled_texture.texture_data.context_id() != context.context_id {
            panic!("Texture does not belong to same context as the bind group encoder");
        }

        bindings.push(ResourceBindingDescriptor {
            internal: BindingDescriptorInternal::SampledTexture {
                unit: slot,
                sampler_data: sampled_texture.sampler_data.clone(),
                texture_data: TextureData::Texture3D(sampled_texture.texture_data.clone()),
            },
        });

        BindGroupEncoder {
            context,
            bindings,
            _marker: marker::PhantomData,
        }
    }

    pub fn add_unsigned_integer_sampled_texture_cube<'b>(
        self,
        slot: u32,
        sampled_texture: UnsignedIntegerSampledTextureCube<'b>,
    ) -> BindGroupEncoder<'a, (UnsignedIntegerSampledTextureCube<'b>, E)> {
        let BindGroupEncoder {
            context,
            mut bindings,
            ..
        } = self;

        if sampled_texture.texture_data.context_id() != context.context_id {
            panic!("Texture does not belong to same context as the bind group encoder");
        }

        bindings.push(ResourceBindingDescriptor {
            internal: BindingDescriptorInternal::SampledTexture {
                unit: slot,
                sampler_data: sampled_texture.sampler_data.clone(),
                texture_data: TextureData::TextureCube(sampled_texture.texture_data.clone()),
            },
        });

        BindGroupEncoder {
            context,
            bindings,
            _marker: marker::PhantomData,
        }
    }

    pub fn add_shadow_sampled_texture_2d<'b>(
        self,
        slot: u32,
        sampled_texture: ShadowSampledTexture2D<'b>,
    ) -> BindGroupEncoder<'a, (ShadowSampledTexture2D<'b>, E)> {
        let BindGroupEncoder {
            context,
            mut bindings,
            ..
        } = self;

        if sampled_texture.texture_data.context_id() != context.context_id {
            panic!("Texture does not belong to same context as the bind group encoder");
        }

        bindings.push(ResourceBindingDescriptor {
            internal: BindingDescriptorInternal::SampledTexture {
                unit: slot,
                sampler_data: sampled_texture.sampler_data.clone(),
                texture_data: TextureData::Texture2D(sampled_texture.texture_data.clone()),
            },
        });

        BindGroupEncoder {
            context,
            bindings,
            _marker: marker::PhantomData,
        }
    }

    pub fn add_shadow_sampled_texture_2d_array<'b>(
        self,
        slot: u32,
        sampled_texture: ShadowSampledTexture2DArray<'b>,
    ) -> BindGroupEncoder<'a, (ShadowSampledTexture2DArray<'b>, E)> {
        let BindGroupEncoder {
            context,
            mut bindings,
            ..
        } = self;

        if sampled_texture.texture_data.context_id() != context.context_id {
            panic!("Texture does not belong to same context as the bind group encoder");
        }

        bindings.push(ResourceBindingDescriptor {
            internal: BindingDescriptorInternal::SampledTexture {
                unit: slot,
                sampler_data: sampled_texture.sampler_data.clone(),
                texture_data: TextureData::Texture2DArray(sampled_texture.texture_data.clone()),
            },
        });

        BindGroupEncoder {
            context,
            bindings,
            _marker: marker::PhantomData,
        }
    }

    pub fn add_shadow_sampled_texture_cube<'b>(
        self,
        slot: u32,
        sampled_texture: ShadowSampledTextureCube<'b>,
    ) -> BindGroupEncoder<'a, (ShadowSampledTextureCube<'b>, E)> {
        let BindGroupEncoder {
            context,
            mut bindings,
            ..
        } = self;

        if sampled_texture.texture_data.context_id() != context.context_id {
            panic!("Texture does not belong to same context as the bind group encoder");
        }

        bindings.push(ResourceBindingDescriptor {
            internal: BindingDescriptorInternal::SampledTexture {
                unit: slot,
                sampler_data: sampled_texture.sampler_data.clone(),
                texture_data: TextureData::TextureCube(sampled_texture.texture_data.clone()),
            },
        });

        BindGroupEncoder {
            context,
            bindings,
            _marker: marker::PhantomData,
        }
    }
}

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

macro_rules! impl_finish_bind_group_encoding(
    ($($B:ident),*) => {
        impl<'a, $($B),*> BindGroupEncoder<'a, nest_pairs_reverse!([(), $($B),*])> {
            pub fn finish(self) -> BindGroupEncoding<'a, ($($B,)*)> {
                let BindGroupEncoder {
                    context,
                    bindings,
                    ..
                } = self;

                BindGroupEncoding {
                    context,
                    bindings,
                    _marker: marker::PhantomData
                }
            }
        }
    }
);

impl_finish_bind_group_encoding!(R0);
impl_finish_bind_group_encoding!(R0, R1);
impl_finish_bind_group_encoding!(R0, R1, R2);
impl_finish_bind_group_encoding!(R0, R1, R2, R3);
impl_finish_bind_group_encoding!(R0, R1, R2, R3, R4);
impl_finish_bind_group_encoding!(R0, R1, R2, R3, R4, R5);
impl_finish_bind_group_encoding!(R0, R1, R2, R3, R4, R5, R6);
impl_finish_bind_group_encoding!(R0, R1, R2, R3, R4, R5, R6, R7);
impl_finish_bind_group_encoding!(R0, R1, R2, R3, R4, R5, R6, R7, R8);
impl_finish_bind_group_encoding!(R0, R1, R2, R3, R4, R5, R6, R7, R8, R9);
impl_finish_bind_group_encoding!(R0, R1, R2, R3, R4, R5, R6, R7, R8, R9, R10);
impl_finish_bind_group_encoding!(R0, R1, R2, R3, R4, R5, R6, R7, R8, R9, R10, R11);
impl_finish_bind_group_encoding!(R0, R1, R2, R3, R4, R5, R6, R7, R8, R9, R10, R11, R12);
impl_finish_bind_group_encoding!(R0, R1, R2, R3, R4, R5, R6, R7, R8, R9, R10, R11, R12, R13);
impl_finish_bind_group_encoding!(R0, R1, R2, R3, R4, R5, R6, R7, R8, R9, R10, R11, R12, R13, R14);
impl_finish_bind_group_encoding!(
    R0, R1, R2, R3, R4, R5, R6, R7, R8, R9, R10, R11, R12, R13, R14, R16
);

#[derive(Clone)]
pub struct BindGroupDescriptor {
    #[allow(dead_code)]
    pub(crate) bind_group_index: u32,
    pub(crate) bindings: Option<Arc<Vec<ResourceBindingDescriptor>>>,
}

impl BindGroupDescriptor {
    pub(crate) fn bind(&self, connection: &mut Connection) {
        if let Some(bindings) = &self.bindings {
            for binding in bindings.iter() {
                binding.bind(connection);
            }
        }
    }
}

pub struct ResourceBindingsEncodingContext {
    context_id: usize,
}

impl ResourceBindingsEncodingContext {
    pub(crate) fn new(context_id: usize) -> Self {
        ResourceBindingsEncodingContext { context_id }
    }
}

pub struct ResourceBindingsEncoding<'a, B>
where
    B: Borrow<[BindGroupDescriptor]>,
{
    #[allow(dead_code)]
    pub(crate) context: &'a mut ResourceBindingsEncodingContext,
    pub(crate) bind_groups: B,
}

impl<'a> ResourceBindingsEncoding<'a, [BindGroupDescriptor; 0]> {
    pub fn empty(context: &'a mut ResourceBindingsEncodingContext) -> Self {
        ResourceBindingsEncoding {
            context,
            bind_groups: [],
        }
    }
}

pub struct StaticResourceBindingsEncoder<'a, B> {
    context: &'a mut ResourceBindingsEncodingContext,
    bind_groups: B,
}

impl<'a> StaticResourceBindingsEncoder<'a, ()> {
    pub fn new(context: &'a mut ResourceBindingsEncodingContext) -> Self {
        StaticResourceBindingsEncoder {
            context,
            bind_groups: (),
        }
    }
}

impl<'a, B> StaticResourceBindingsEncoder<'a, B> {
    pub fn add_bind_group<T>(
        self,
        bind_group_index: u32,
        bind_group: &BindGroup<T>,
    ) -> StaticResourceBindingsEncoder<'a, (BindGroupDescriptor, B)> {
        let bindings = match &bind_group.internal {
            BindGroupInternal::Empty => None,
            BindGroupInternal::NotEmpty {
                context_id,
                encoding,
            } => {
                if self.context.context_id != *context_id {
                    panic!("Bind group belongs to a different context than the current pipeline.");
                }

                Some(encoding.clone())
            }
        };

        StaticResourceBindingsEncoder {
            context: self.context,
            bind_groups: (
                BindGroupDescriptor {
                    bind_group_index,
                    bindings,
                },
                self.bind_groups,
            ),
        }
    }
}

// TODO: implement this with const generics when possible

macro_rules! generate_encoder_finish {
    ($n:tt, $($C:ident|$I:ident),*) => {
        impl<'a> StaticResourceBindingsEncoder<'a, nest_pairs_reverse!([(), $($C),*])> {
            pub fn finish(self) -> ResourceBindingsEncoding<'a, [BindGroupDescriptor;$n]> {
                let nest_pairs_reverse!([_, $($I),*]) = self.bind_groups;

                ResourceBindingsEncoding {
                    context: self.context,
                    bind_groups: [$($I),*]
                }
            }
        }
    }
}

generate_encoder_finish!(1, BindGroupDescriptor | b0);
generate_encoder_finish!(2, BindGroupDescriptor | b0, BindGroupDescriptor | b1);
generate_encoder_finish!(
    3,
    BindGroupDescriptor | b0,
    BindGroupDescriptor | b1,
    BindGroupDescriptor | b2
);
generate_encoder_finish!(
    4,
    BindGroupDescriptor | b0,
    BindGroupDescriptor | b1,
    BindGroupDescriptor | b2,
    BindGroupDescriptor | b3
);
generate_encoder_finish!(
    5,
    BindGroupDescriptor | b0,
    BindGroupDescriptor | b1,
    BindGroupDescriptor | b2,
    BindGroupDescriptor | b3,
    BindGroupDescriptor | b4
);
generate_encoder_finish!(
    6,
    BindGroupDescriptor | b0,
    BindGroupDescriptor | b1,
    BindGroupDescriptor | b2,
    BindGroupDescriptor | b3,
    BindGroupDescriptor | b4,
    BindGroupDescriptor | b5
);
generate_encoder_finish!(
    7,
    BindGroupDescriptor | b0,
    BindGroupDescriptor | b1,
    BindGroupDescriptor | b2,
    BindGroupDescriptor | b3,
    BindGroupDescriptor | b4,
    BindGroupDescriptor | b5,
    BindGroupDescriptor | b6
);
generate_encoder_finish!(
    8,
    BindGroupDescriptor | b0,
    BindGroupDescriptor | b1,
    BindGroupDescriptor | b2,
    BindGroupDescriptor | b3,
    BindGroupDescriptor | b4,
    BindGroupDescriptor | b5,
    BindGroupDescriptor | b6,
    BindGroupDescriptor | b7
);
generate_encoder_finish!(
    9,
    BindGroupDescriptor | b0,
    BindGroupDescriptor | b1,
    BindGroupDescriptor | b2,
    BindGroupDescriptor | b3,
    BindGroupDescriptor | b4,
    BindGroupDescriptor | b5,
    BindGroupDescriptor | b6,
    BindGroupDescriptor | b7,
    BindGroupDescriptor | b8
);
generate_encoder_finish!(
    10,
    BindGroupDescriptor | b0,
    BindGroupDescriptor | b1,
    BindGroupDescriptor | b2,
    BindGroupDescriptor | b3,
    BindGroupDescriptor | b4,
    BindGroupDescriptor | b5,
    BindGroupDescriptor | b6,
    BindGroupDescriptor | b7,
    BindGroupDescriptor | b8,
    BindGroupDescriptor | b9
);
generate_encoder_finish!(
    11,
    BindGroupDescriptor | b0,
    BindGroupDescriptor | b1,
    BindGroupDescriptor | b2,
    BindGroupDescriptor | b3,
    BindGroupDescriptor | b4,
    BindGroupDescriptor | b5,
    BindGroupDescriptor | b6,
    BindGroupDescriptor | b7,
    BindGroupDescriptor | b8,
    BindGroupDescriptor | b9,
    BindGroupDescriptor | b10
);
generate_encoder_finish!(
    12,
    BindGroupDescriptor | b0,
    BindGroupDescriptor | b1,
    BindGroupDescriptor | b2,
    BindGroupDescriptor | b3,
    BindGroupDescriptor | b4,
    BindGroupDescriptor | b5,
    BindGroupDescriptor | b6,
    BindGroupDescriptor | b7,
    BindGroupDescriptor | b8,
    BindGroupDescriptor | b9,
    BindGroupDescriptor | b10,
    BindGroupDescriptor | b11
);
generate_encoder_finish!(
    13,
    BindGroupDescriptor | b0,
    BindGroupDescriptor | b1,
    BindGroupDescriptor | b2,
    BindGroupDescriptor | b3,
    BindGroupDescriptor | b4,
    BindGroupDescriptor | b5,
    BindGroupDescriptor | b6,
    BindGroupDescriptor | b7,
    BindGroupDescriptor | b8,
    BindGroupDescriptor | b9,
    BindGroupDescriptor | b10,
    BindGroupDescriptor | b11,
    BindGroupDescriptor | b12
);
generate_encoder_finish!(
    14,
    BindGroupDescriptor | b0,
    BindGroupDescriptor | b1,
    BindGroupDescriptor | b2,
    BindGroupDescriptor | b3,
    BindGroupDescriptor | b4,
    BindGroupDescriptor | b5,
    BindGroupDescriptor | b6,
    BindGroupDescriptor | b7,
    BindGroupDescriptor | b8,
    BindGroupDescriptor | b9,
    BindGroupDescriptor | b10,
    BindGroupDescriptor | b11,
    BindGroupDescriptor | b12,
    BindGroupDescriptor | b13
);
generate_encoder_finish!(
    15,
    BindGroupDescriptor | b0,
    BindGroupDescriptor | b1,
    BindGroupDescriptor | b2,
    BindGroupDescriptor | b3,
    BindGroupDescriptor | b4,
    BindGroupDescriptor | b5,
    BindGroupDescriptor | b6,
    BindGroupDescriptor | b7,
    BindGroupDescriptor | b8,
    BindGroupDescriptor | b9,
    BindGroupDescriptor | b10,
    BindGroupDescriptor | b11,
    BindGroupDescriptor | b12,
    BindGroupDescriptor | b13,
    BindGroupDescriptor | b14
);
generate_encoder_finish!(
    16,
    BindGroupDescriptor | b0,
    BindGroupDescriptor | b1,
    BindGroupDescriptor | b2,
    BindGroupDescriptor | b3,
    BindGroupDescriptor | b4,
    BindGroupDescriptor | b5,
    BindGroupDescriptor | b6,
    BindGroupDescriptor | b7,
    BindGroupDescriptor | b8,
    BindGroupDescriptor | b9,
    BindGroupDescriptor | b10,
    BindGroupDescriptor | b11,
    BindGroupDescriptor | b12,
    BindGroupDescriptor | b13,
    BindGroupDescriptor | b14,
    BindGroupDescriptor | b15
);
