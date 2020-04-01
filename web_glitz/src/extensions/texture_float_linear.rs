use crate::image::format::{TextureFormat, R32F, RG32F, RGB32F, RGBA32F};
use crate::runtime::Connection;
use crate::sampler::{
    CompatibleSampler, Linear, LinearMipmapLinear, LinearMipmapNearest, MagnificationFilter,
    MinificationFilter, Nearest, NearestMipmapLinear, NearestMipmapNearest, Sampler,
};
use std::ops::Deref;

#[derive(Clone, Debug)]
pub struct Extension {
    context_id: usize,
}

impl Extension {
    pub fn extend<'a, Min, Mag>(&self, sampler: &'a Sampler<Min, Mag>) -> Extended<'a, Min, Mag> {
        if sampler.data().context_id() != self.context_id {
            panic!("Sampler belongs to a different context than this extension.");
        }

        Extended { sampler }
    }
}

impl super::Extension for Extension {
    fn try_init(connection: &mut Connection, context_id: usize) -> Option<Self> {
        let (gl, _) = unsafe { connection.unpack() };

        gl.get_extension("OES_texture_float_linear")
            .ok()
            .flatten()
            .map(|_| Extension { context_id })
    }
}

#[derive(Clone, Copy)]
pub struct Extended<'a, Min, Mag> {
    sampler: &'a Sampler<Min, Mag>,
}

impl<Min, Mag> Deref for Extended<'_, Min, Mag> {
    type Target = Sampler<Min, Mag>;

    fn deref(&self) -> &Self::Target {
        &self.sampler
    }
}

pub unsafe trait Filterable {}

unsafe impl Filterable for R32F {}
unsafe impl Filterable for RG32F {}
unsafe impl Filterable for RGB32F {}
unsafe impl Filterable for RGBA32F {}

pub unsafe trait CompatibleFilter<F>
where
    F: Filterable,
{
}

unsafe impl<F> CompatibleFilter<F> for Nearest where F: Filterable {}
unsafe impl<F> CompatibleFilter<F> for Linear where F: Filterable {}
unsafe impl<F> CompatibleFilter<F> for LinearMipmapNearest where F: Filterable {}
unsafe impl<F> CompatibleFilter<F> for LinearMipmapLinear where F: Filterable {}
unsafe impl<F> CompatibleFilter<F> for NearestMipmapLinear where F: Filterable {}
unsafe impl<F> CompatibleFilter<F> for NearestMipmapNearest where F: Filterable {}

unsafe impl<'a, F, Min, Mag> CompatibleSampler<F> for Extended<'a, Min, Mag>
where
    F: TextureFormat + Filterable,
    Min: CompatibleFilter<F> + MinificationFilter,
    Mag: CompatibleFilter<F> + MagnificationFilter,
{
    type Min = Min;
    type Mag = Mag;

    fn get_ref(&self) -> &Sampler<Self::Min, Self::Mag> {
        &self.sampler
    }
}
