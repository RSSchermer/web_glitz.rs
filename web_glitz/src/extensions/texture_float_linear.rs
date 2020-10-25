//! Allows samplers that use linear interpolation to sample textures that use a 32-bit floating
//! point internal format.
//!
//! When this extension is available, a [Sampler] that uses a [Linear], [LinearMipmapNearest],
//! [LinearMipmapLinear] or [NearestMipMapLinear] as its [MinificationFilter], or that uses [Linear]
//! as its [MagnificationFilter], is allowed to sample textures that use one of the following
//! internal formats:
//!
//! - [R32F]
//! - [RG32F]
//! - [RGB32F]
//! - [RGBA32F]
//!
//! This extension uses an [Extended] wrapper type to act as a type proof for the availability of
//! this extension without requiring additional runtime checks when creating a sampled texture
//! resource.
//!
//! # Example
//!
//! ```
//! # use web_glitz::runtime::RenderingContext;
//! # fn wrapper<Rc>(context: &Rc) where Rc: RenderingContext {
//! use web_glitz::extensions::texture_float_linear::Extension as TextureFloatLinearExtension;
//! use web_glitz::image::MipmapLevels;
//! use web_glitz::image::format::RGBA32F;
//! use web_glitz::image::texture_2d::Texture2DDescriptor;
//! use web_glitz::image::sampler::{SamplerDescriptor, NearestMipmapLinear, Linear};
//! use web_glitz::rendering::{RenderTargetDescriptor, LoadOp, StoreOp};
//!
//! let texture = context.try_create_texture_2d(&Texture2DDescriptor {
//!         format: RGBA32F,
//!         width: 500,
//!         height: 500,
//!         levels: MipmapLevels::Partial(1)
//!     }).unwrap();
//!
//! let sampler = context.create_sampler(&SamplerDescriptor {
//!     minification_filter: NearestMipmapLinear,
//!     magnification_filter: Linear,
//!     ..SamplerDescriptor::default()
//! });
//!
//! let extension: Option<TextureFloatLinearExtension> = context.get_extension();
//!
//! if let Some(extension) = extension {
//!     let sampled_texture_resource = texture.float_sampled(extension.extend(&sampler));
//! }
//! # }
//! ```
//!
//! Here `context` is a [RenderingContext].
use std::ops::Deref;

use crate::image::format::{
    Filterable as FilterableBase, TextureFormat, R32F, RG32F, RGB32F, RGBA32F,
};
use crate::image::sampler::{
    CompatibleSampler, Linear, LinearMipmapLinear, LinearMipmapNearest, MagnificationFilter,
    MinificationFilter, Nearest, NearestMipmapLinear, NearestMipmapNearest, Sampler,
};
use crate::runtime::Connection;

/// Extension object for the [texture_float_linear] extension.
///
/// See the [texture_float_linear] module documentation for details.
#[derive(Clone, Debug)]
pub struct Extension {
    context_id: u64,
}

impl Extension {
    /// Wraps a sampler in a type that can be combined with a texture that uses a floating point
    /// internal format without causing a type error.
    ///
    /// # Panics
    ///
    /// Panics if the sampler belongs to a different context than the extension.
    pub fn extend<'a, Min, Mag>(&self, sampler: &'a Sampler<Min, Mag>) -> Extended<'a, Min, Mag> {
        if sampler.data().context_id() != self.context_id {
            panic!("Sampler belongs to a different context than this extension.");
        }

        Extended { sampler }
    }
}

impl super::Extension for Extension {
    fn try_init(connection: &mut Connection, context_id: u64) -> Option<Self> {
        let (gl, _) = unsafe { connection.unpack() };

        gl.get_extension("OES_texture_float_linear")
            .ok()
            .flatten()
            .map(|_| Extension { context_id })
    }
}

/// Wrapper type for samplers that acts as a type proof for the availability of this extension,
/// allowing the combination of samplers that use linear filtering with textures that use a floating
/// point internal format.
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

/// Marker trait for internal formats that become filterable when this extension is available.
pub unsafe trait Filterable {}

unsafe impl<F> Filterable for F where F: FilterableBase {}

unsafe impl Filterable for R32F {}
unsafe impl Filterable for RG32F {}
unsafe impl Filterable for RGB32F {}
unsafe impl Filterable for RGBA32F {}

/// Marker trait for filter-format combinations that become compatible when this extension is
/// available.
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
