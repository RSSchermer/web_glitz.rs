//! This module provides data types and traits for the allocation and manipulation of GPU-accessible
//! image data.
//!
//! The [format] module defines a number of types that implement [InternalFormat]; these types
//! represent the available data formats for image storage.
//!
//! A distinction is made between a texture ([Texture2D], [Texture3D], [Texture2DArray],
//! [TextureCube]) and a renderbuffer ([Renderbuffer]): a texture may be sampled (see
//! [web_glitz::sampler]), but is not necessarily optimized for use with a
//! [RenderTarget](web_glitz::render_pass::RenderTarget), whereas a renderbuffer may not be sampled,
//! but is optimized for use with a [RenderTarget](web_glitz::render_pass::RenderTarget).
//!
//! # 2-Dimensional image storage
//!
//! 2-Dimensional image storage has a `width` and a `height`. It may be thought of as a `width` by
//! `height` 2-dimensional grid of cells, where each cell stores one value; all values share a type
//! determined by the storage's [InternalFormat].
//!
//! The [Texture2D] and [Renderbuffer] types provide 2-dimensional image storage. An [Image2DSource]
//! may hold data that is to be uploaded to 2-dimensional image storage.
//!
//! # Layered image storage
//!
//! Layered image storage (also 3-dimensional image storage) has a `width`, a `height` and `depth`.
//! It may be thought of as a `width` by `height` by `depth` 3-dimensional grid of cells, where each
//! cell stores one value; all values share a type determined by the storage's [InternalFormat]. It
//! may alternatively be thought of as layered 2-dimensional image storage, where there are `depth`
//! layers of 2-dimensional images of size `width` by `height`.
//!
//! The [Texture2DArray] and [Texture3D] types provide layered image storage. A [LayeredImageSource]
//! may hold data that is to be uploaded to layered image storage. Alternatively, an [Image2DSource]
//! may hold data that is to be uploaded to an individual layer of a layered image.
//!
//! # Cube map storage
//!
//! Cube map storage stores 6 2-dimensional images (one for each face of a cube) of the same size
//! (all six images share the same `width`, and all 6 images share the same `height`) and the same
//! [InternalFormat]. The six images in cube map storage are also referred to as its "faces",
//! specifically they are:
//!
//! - The "positive x" face.
//! - The "negative x" face.
//! - The "positive y" face.
//! - The "negative y" face.
//! - The "positive z" face.
//! - The "negative z" face.
//!
//! The [TextureCube] type provides cube map storage. An [Image2DSource] may hold data that is to
//! be uploaded to an individual face.
//!
//! # Mipmapping
//!
//! Texture storage ([Texture2D], [Texture3D], [Texture2DArray], [TextureCube]) does not store just
//! base images, it stores partial or complete mipmap chains. A mipmap chain consists of a series
//! of (2-dimensional or layered) images, starting with a base image with a predetermined width and
//! height. Each subsequent image in the chain is half the width and half the height of the previous
//! image in the chain, rounded down to the nearest integer. The chain is complete when either the
//! width or the height reaches `1`. The images in the mipmap chain are also called the "levels" of
//! the mipmap and they are numbered incrementally, starting at `0`. For example, a `256` by `128`
//! 2 dimensional image has the following complete mipmap chain:
//!
//! - Level `0`: width `256`, height `128`.
//! - Level `1`: width `128`, height `64`.
//! - Level `2`: width `64`, height `32`.
//! - Level `3`: width `32`, height `16`.
//! - Level `4`: width `16`, height `8`.
//! - Level `5`: width `8`, height `4`.
//! - Level `6`: width `4`, height `2`.
//! - Level `7`: width `2`, height `1`.
//!
//! The chain stops at level `7`, when the height reaches `1`, for a total of 8 levels. Level 0 is
//! also called the "base level".
//!
//! Texture storage does not necessarily have to store a complete mipmap chain, it may only store
//! the first `N` levels, where `N` is smaller than the number of levels in the complete mipmap
//! chain for the image. `N` must be at least `1`; a texture must have at least a base level.
//!
//! The main application for mipmapping is minification filtering, in which case each level in the
//! chain is typically obtained by linear minification filtering of the preceding level (see
//! [MinificationFilter] for details). If the texture format implements [Filterable], then the image
//! data for such a chain can be generated from the base level by the driver (see
//! [Texture2D::generate_mipmap], [Texture3D::generate_mipmap], [Texture2DArray::generate_mipmap],
//! [TextureCube::generate_mipmap]).

pub(crate) mod image_source;
pub use self::image_source::{FromPixelsError, Image2DSource, LayeredImageSource};

pub mod format;
pub mod renderbuffer;
pub mod texture_2d;
pub mod texture_2d_array;
pub mod texture_3d;
pub mod texture_cube;

mod texture_object_dropper;
mod util;

/// Represents a region of a 2-dimensional image.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Region2D {
    /// Variant that represents the entire image.
    Fill,

    /// Variant that represents an explicit rectangular area of the image.
    ///
    /// The first value represent the offset of the origin of the area, relative to the origin of
    /// the image; the second argument represents the width of the area; the third argument
    /// represents the height of the area.
    Area((u32, u32), u32, u32),
}

/// Represents a region of a 3-dimensional (layered) image.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Region3D {
    /// Variant that represents the entire image.
    Fill,

    /// Variant that represents an explicit rectangular area of the image.
    ///
    /// The first value represent the offset of the origin of the area, relative to the origin of
    /// the image; the second argument represents the width of the area; the third argument
    /// represents the height of the area; the fourth argument represents the depth of the area.
    Area((u32, u32, u32), u32, u32, u32),
}

impl Into<Region2D> for Region3D {
    fn into(self) -> Region2D {
        match self {
            Region3D::Fill => Region2D::Fill,
            Region3D::Area((offset_x, offset_y, _), width, height, _) => {
                Region2D::Area((offset_x, offset_y), width, height)
            }
        }
    }
}

/// Describes the number of mipmap levels that are to be allocated for a texture.
///
/// See the module documentation for [web_glitz::image] for details on mipmap storage.
pub enum MipmapLevels {
    /// Variant that will allocate storage for all mipmap levels in the complete mipmap chain for
    /// an image of the relevant width and height.
    Complete,

    /// Variant that specifies a partial mipmap chain with an explicit number of levels.
    Partial(usize),
}

/// Error returned when creating a texture using [MipmapLevels::Partial] to specify a specific
/// number of mipmap levels, but the number of levels specified is greater than the number of levels
/// in the complete mipmap chain for an image of the relevant dimensions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MaxMipmapLevelsExceeded {
    /// The number of levels specified with [MipmapLevels::Partial].
    pub given: usize,

    /// The maximum number of levels possible in a mipmap chain for an image of the relevant
    /// dimensions.
    pub max: usize,
}
