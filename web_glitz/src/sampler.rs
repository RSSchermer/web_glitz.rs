use rendering_context::RefCountedDropper;
use std::sync::Arc;
use texture::texture_2d::Texture2DHandle;
use texture::texture_2d_array::Texture2DArrayHandle;
use texture::texture_3d::Texture3DHandle;
use texture::texture_cube::TextureCubeHandle;
use util::JsId;
use rendering_context::Connection;
use util::arc_get_mut_unchecked;
use util::identical;
use rendering_context::ContextUpdate;
use texture::TextureFormat;
use image_format::DepthRenderable;

pub struct SamplerHandle<T> {
    pub(crate) data: Arc<SamplerData<T>>,
}

impl<T> SamplerHandle<T> {
    fn bind_to_unit(&self, connection: &mut Connection, unit: u32) {
        unsafe {
            arc_get_mut_unchecked(&self.data).bind_to_unit(connection, unit)
        }
    }
}

impl<F> SamplerHandle<Texture2DHandle<F>> where F: TextureFormat + 'static {
    pub(crate) fn bind(&self, connection: &mut Connection) -> u32 {
        let unit = self.data.texture.bind(connection);

        self.bind_to_unit(connection, unit);

        unit
    }
}

impl<F> SamplerHandle<Texture2DArrayHandle<F>> where F: TextureFormat + 'static {
    pub(crate) fn bind(&self, connection: &mut Connection) -> u32 {
        let unit = self.data.texture.bind(connection);

        self.bind_to_unit(connection, unit);

        unit
    }
}

impl<F> SamplerHandle<Texture3DHandle<F>> where F: TextureFormat + 'static {
    pub(crate) fn bind(&self, connection: &mut Connection) -> u32 {
        let unit = self.data.texture.bind(connection);

        self.bind_to_unit(connection, unit);

        unit
    }
}

impl<F> SamplerHandle<TextureCubeHandle<F>> where F: TextureFormat + 'static {
    pub(crate) fn bind(&self, connection: &mut Connection) -> u32 {
        let unit = self.data.texture.bind(connection);

        self.bind_to_unit(connection, unit);

        unit
    }
}

pub struct SamplerShadowHandle<T>
{
    pub(crate) data: Arc<SamplerData<T>>,
}

impl<T> SamplerShadowHandle<T> {
    fn bind_to_unit(&self, connection: &mut Connection, unit: u32) {
        unsafe {
            arc_get_mut_unchecked(&self.data).bind_to_unit(connection, unit)
        }
    }
}

impl<F> SamplerShadowHandle<Texture2DHandle<F>> where F: TextureFormat + DepthRenderable + 'static {
    pub(crate) fn bind(&self, connection: &mut Connection) -> u32 {
        let unit = self.data.texture.bind(connection);

        self.bind_to_unit(connection, unit);

        unit
    }
}

impl<F> SamplerShadowHandle<Texture2DArrayHandle<F>> where F: TextureFormat + DepthRenderable + 'static {
    pub(crate) fn bind(&self, connection: &mut Connection) -> u32 {
        let unit = self.data.texture.bind(connection);

        self.bind_to_unit(connection, unit);

        unit
    }
}

pub(crate) struct SamplerData<T>
{
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
