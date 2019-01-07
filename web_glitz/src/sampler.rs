use std::sync::Arc;

use crate::image_format::{FloatSamplable, IntegerSamplable, UnsignedIntegerSamplable, ShadowSamplable};
use crate::runtime::{Connection, RenderingContext};
use crate::runtime::dropper::{DropObject, Dropper, RefCountedDropper};
use crate::runtime::dynamic_state::ContextUpdate;
use crate::texture::texture_2d::Texture2DHandle;
use crate::texture::texture_2d_array::Texture2DArrayHandle;
use crate::texture::texture_3d::Texture3DHandle;
use crate::texture::texture_cube::TextureCubeHandle;
use crate::texture::TextureFormat;
use crate::util::{JsId, identical, arc_get_mut_unchecked};

pub struct FloatSampler2DHandle<F> {
    data: Arc<SamplerData<Texture2DHandle<F>>>,
}

impl<F> FloatSampler2DHandle<F> {
    pub(crate) fn bind(&self, connection: &mut Connection) -> u32 {
        let unit = self.data.texture.bind(connection);

        unsafe {
            arc_get_mut_unchecked(&self.data).bind_to_unit(connection, unit);
        }

        unit
    }
}

pub struct IntegerSampler2DHandle<F> {
    data: Arc<SamplerData<Texture2DHandle<F>>>,
}

impl<F> IntegerSampler2DHandle<F> {
    pub(crate) fn bind(&self, connection: &mut Connection) -> u32 {
        let unit = self.data.texture.bind(connection);

        unsafe {
            arc_get_mut_unchecked(&self.data).bind_to_unit(connection, unit);
        }

        unit
    }
}

pub struct UnsignedIntegerSampler2DHandle<F> {
    data: Arc<SamplerData<Texture2DHandle<F>>>,
}

impl<F> UnsignedIntegerSampler2DHandle<F> {
    pub(crate) fn bind(&self, connection: &mut Connection) -> u32 {
        let unit = self.data.texture.bind(connection);

        unsafe {
            arc_get_mut_unchecked(&self.data).bind_to_unit(connection, unit);
        }

        unit
    }
}

pub struct FloatSampler2DArrayHandle<F> {
    data: Arc<SamplerData<Texture2DArrayHandle<F>>>,
}

impl<F> FloatSampler2DArrayHandle<F> {
    pub(crate) fn bind(&self, connection: &mut Connection) -> u32 {
        let unit = self.data.texture.bind(connection);

        unsafe {
            arc_get_mut_unchecked(&self.data).bind_to_unit(connection, unit);
        }

        unit
    }
}

pub struct IntegerSampler2DArrayHandle<F> {
    data: Arc<SamplerData<Texture2DArrayHandle<F>>>,
}

impl<F> IntegerSampler2DArrayHandle<F> {
    pub(crate) fn bind(&self, connection: &mut Connection) -> u32 {
        let unit = self.data.texture.bind(connection);

        unsafe {
            arc_get_mut_unchecked(&self.data).bind_to_unit(connection, unit);
        }

        unit
    }
}

pub struct UnsignedIntegerSampler2DArrayHandle<F> {
    data: Arc<SamplerData<Texture2DArrayHandle<F>>>,
}

impl<F> UnsignedIntegerSampler2DArrayHandle<F> {
    pub(crate) fn bind(&self, connection: &mut Connection) -> u32 {
        let unit = self.data.texture.bind(connection);

        unsafe {
            arc_get_mut_unchecked(&self.data).bind_to_unit(connection, unit);
        }

        unit
    }
}

pub struct FloatSampler3DHandle<F> {
    data: Arc<SamplerData<Texture3DHandle<F>>>,
}

impl<F> FloatSampler3DHandle<F> {
    pub(crate) fn bind(&self, connection: &mut Connection) -> u32 {
        let unit = self.data.texture.bind(connection);

        unsafe {
            arc_get_mut_unchecked(&self.data).bind_to_unit(connection, unit);
        }

        unit
    }
}

pub struct IntegerSampler3DHandle<F> {
    data: Arc<SamplerData<Texture3DHandle<F>>>,
}

impl<F> IntegerSampler3DHandle<F> {
    pub(crate) fn bind(&self, connection: &mut Connection) -> u32 {
        let unit = self.data.texture.bind(connection);

        unsafe {
            arc_get_mut_unchecked(&self.data).bind_to_unit(connection, unit);
        }

        unit
    }
}

pub struct UnsignedIntegerSampler3DHandle<F> {
    data: Arc<SamplerData<Texture3DHandle<F>>>,
}

impl<F> UnsignedIntegerSampler3DHandle<F> {
    pub(crate) fn bind(&self, connection: &mut Connection) -> u32 {
        let unit = self.data.texture.bind(connection);

        unsafe {
            arc_get_mut_unchecked(&self.data).bind_to_unit(connection, unit);
        }

        unit
    }
}

pub struct FloatSamplerCubeHandle<F> {
    data: Arc<SamplerData<TextureCubeHandle<F>>>,
}

impl<F> FloatSamplerCubeHandle<F> {
    pub(crate) fn bind(&self, connection: &mut Connection) -> u32 {
        let unit = self.data.texture.bind(connection);

        unsafe {
            arc_get_mut_unchecked(&self.data).bind_to_unit(connection, unit);
        }

        unit
    }
}

pub struct IntegerSamplerCubeHandle<F> {
    data: Arc<SamplerData<TextureCubeHandle<F>>>,
}

impl<F> IntegerSamplerCubeHandle<F> {
    pub(crate) fn bind(&self, connection: &mut Connection) -> u32 {
        let unit = self.data.texture.bind(connection);

        unsafe {
            arc_get_mut_unchecked(&self.data).bind_to_unit(connection, unit);
        }

        unit
    }
}

pub struct UnsignedIntegerSamplerCubeHandle<F> {
    data: Arc<SamplerData<TextureCubeHandle<F>>>,
}

impl<F> UnsignedIntegerSamplerCubeHandle<F> {
    pub(crate) fn bind(&self, connection: &mut Connection) -> u32 {
        let unit = self.data.texture.bind(connection);

        unsafe {
            arc_get_mut_unchecked(&self.data).bind_to_unit(connection, unit);
        }

        unit
    }
}

pub struct Sampler2DShadowHandle<F> {
    data: Arc<SamplerData<Texture2DHandle<F>>>,
}

impl<F> Sampler2DShadowHandle<F> {
    pub(crate) fn bind(&self, connection: &mut Connection) -> u32 {
        let unit = self.data.texture.bind(connection);

        unsafe {
            arc_get_mut_unchecked(&self.data).bind_to_unit(connection, unit);
        }

        unit
    }
}

pub struct Sampler2DArrayShadowHandle<F> {
    data: Arc<SamplerData<Texture2DArrayHandle<F>>>,
}

impl<F> Sampler2DArrayShadowHandle<F> {
    pub(crate) fn bind(&self, connection: &mut Connection) -> u32 {
        let unit = self.data.texture.bind(connection);

        unsafe {
            arc_get_mut_unchecked(&self.data).bind_to_unit(connection, unit);
        }

        unit
    }
}

pub struct SamplerCubeShadowHandle<F> {
    data: Arc<SamplerData<TextureCubeHandle<F>>>,
}

impl<F> SamplerCubeShadowHandle<F> {
    pub(crate) fn bind(&self, connection: &mut Connection) -> u32 {
        let unit = self.data.texture.bind(connection);

        unsafe {
            arc_get_mut_unchecked(&self.data).bind_to_unit(connection, unit);
        }

        unit
    }
}

pub(crate) struct SamplerData<T> {
    pub(crate) id: Option<JsId>,
    dropper: RefCountedDropper,
    pub(crate) texture: T,
}

impl<T> SamplerData<T> {
    fn bind_to_unit(&mut self, connection: &mut Connection, unit: u32) {
        let Connection(gl, state) = connection;

        unsafe {
            self.id.unwrap().with_value_unchecked(|sampler_object| {
                if !identical(state.bound_sampler(unit).as_ref(), Some(&sampler_object)) {
                    state
                        .set_bound_sampler(unit, Some(&sampler_object))
                        .apply(gl)
                        .unwrap();
                }
            });
        }
    }
}
