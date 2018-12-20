use rendering_context::RefCountedDropper;
use std::sync::Arc;
use texture::texture_2d::Texture2DData;
use texture::texture_2d_array::Texture2DArrayData;
use texture::texture_3d::Texture3DData;
use texture::texture_cube::TextureCubeData;
use texture::Texture2DHandle;
use util::JsId;
use rendering_context::Connection;
use util::arc_get_mut_unchecked;
use util::identical;
use rendering_context::ContextUpdate;

pub struct Sampler<T>
where
    T: AsSampled,
{
    pub(crate) data: Arc<SamplerData<T>>,
}

impl<T> Sampler<T> where T: AsSampled {
    pub(crate) fn bind(&self, connection: &mut Connection) -> u32 {
        let Connection(gl, state) = connection;
        let data = unsafe { arc_get_mut_unchecked(&self.data) };

        let unit = match data.texture.as_sampled().internal {
            SampledInternal::Texture2D(ref mut data) => unsafe {
                let data = unsafe { arc_get_mut_unchecked(data) };
                let most_recent_unit = &mut data.most_recent_unit;

                data.id.unwrap().with_value_unchecked(|texture_object| {
                    if most_recent_unit.is_none()
                        || !identical(
                        state.texture_units_textures()[most_recent_unit.unwrap() as usize]
                            .as_ref(),
                        Some(&texture_object),
                    ) {
                        state.set_active_texture_lru().apply(gl).unwrap();
                        state
                            .set_bound_texture_2d(Some(&texture_object))
                            .apply(gl)
                            .unwrap();

                        let unit = state.active_texture();

                        *most_recent_unit = Some(unit);

                        unit
                    } else {
                        most_recent_unit.unwrap()
                    }
                })
            },
            SampledInternal::Texture2DArray(ref mut data) => unsafe {
                let data = unsafe { arc_get_mut_unchecked(data) };
                let most_recent_unit = &mut data.most_recent_unit;

                data.id.unwrap().with_value_unchecked(|texture_object| {
                    if most_recent_unit.is_none()
                        || !identical(
                        state.texture_units_textures()[most_recent_unit.unwrap() as usize]
                            .as_ref(),
                        Some(&texture_object),
                    ) {
                        state.set_active_texture_lru().apply(gl).unwrap();
                        state
                            .set_bound_texture_2d_array(Some(&texture_object))
                            .apply(gl)
                            .unwrap();

                        let unit = state.active_texture();

                        *most_recent_unit = Some(unit);

                        unit
                    } else {
                        most_recent_unit.unwrap()
                    }
                })
            },
            SampledInternal::Texture3D(ref mut data) => unsafe {
                let data = unsafe { arc_get_mut_unchecked(data) };
                let most_recent_unit = &mut data.most_recent_unit;

                data.id.unwrap().with_value_unchecked(|texture_object| {
                    if most_recent_unit.is_none()
                        || !identical(
                        state.texture_units_textures()[most_recent_unit.unwrap() as usize]
                            .as_ref(),
                        Some(&texture_object),
                    ) {
                        state.set_active_texture_lru().apply(gl).unwrap();
                        state
                            .set_bound_texture_3d(Some(&texture_object))
                            .apply(gl)
                            .unwrap();

                        let unit = state.active_texture();

                        *most_recent_unit = Some(unit);

                        unit
                    } else {
                        most_recent_unit.unwrap()
                    }
                })
            },
            SampledInternal::TextureCube(ref mut data) => unsafe {
                let data = unsafe { arc_get_mut_unchecked(data) };
                let most_recent_unit = &mut data.most_recent_unit;

                data.id.unwrap().with_value_unchecked(|texture_object| {
                    if most_recent_unit.is_none()
                        || !identical(
                        state.texture_units_textures()[most_recent_unit.unwrap() as usize]
                            .as_ref(),
                        Some(&texture_object),
                    ) {
                        state.set_active_texture_lru().apply(gl).unwrap();
                        state
                            .set_bound_texture_cube_map(Some(&texture_object))
                            .apply(gl)
                            .unwrap();

                        let unit = state.active_texture();

                        *most_recent_unit = Some(unit);

                        unit
                    } else {
                        most_recent_unit.unwrap()
                    }
                })
            },
        };

        unsafe {
            data.id.unwrap().with_value_unchecked(|sampler_object| {
                if !identical(state.bound_sampler(unit).as_ref(), Some(&sampler_object)) {
                    state
                        .set_bound_sampler(unit, Some(&sampler_object))
                        .apply(gl)
                        .unwrap();
                }
            });
        }

        unit
    }
}

pub(crate) struct SamplerData<T>
where
    T: AsSampled,
{
    pub(crate) id: Option<JsId>,
    dropper: RefCountedDropper,
    pub(crate) texture: T,
}

pub struct Sampled<'a> {
    pub(crate) internal: SampledInternal<'a>,
}

pub(crate) enum SampledInternal<'a> {
    Texture2D(&'a mut Arc<Texture2DData>),
    Texture2DArray(&'a mut Arc<Texture2DArrayData>),
    Texture3D(&'a mut Arc<Texture3DData>),
    TextureCube(&'a mut Arc<TextureCubeData>),
}

pub trait AsSampled {
    fn as_sampled(&mut self) -> Sampled;
}
