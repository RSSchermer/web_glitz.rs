use web_sys::WebGl2RenderingContext as Gl;

use crate::runtime::Extensions;
use crate::sampler::{MagnificationFilter, MinificationFilter};
use std::ops::Deref;

/// Trait implemented for types that represent image formats for storing image data in
/// GPU-accessible memory.
pub unsafe trait InternalFormat {
    /// Identifier for associated OpenGl value.
    const ID: u32;
}

/// Trait implemented for types that represent data for a single pixel that can be unpacked into an
/// image with a certain [InternalFormat].
///
/// # Unsafe
///
/// Must only be implemented for types that are binary compatible with the specified [TYPE_ID].
/// Additionally, the [InternalFormat::ID] of `T`, the [FORMAT_ID], and the [TYPE_ID] must be one of
/// the valid combinations enumerated in [table 2](https://www.khronos.org/registry/OpenGL-Refpages/es3.0/html/glTexImage2D.xhtml).
pub unsafe trait PixelUnpack<T>
where
    T: InternalFormat,
{
    /// Identifier for associated OpenGl format value.
    const FORMAT_ID: u32;

    /// Identifier for associated OpenGl type value.
    const TYPE_ID: u32;
}

pub unsafe trait PackFormat<T> where T: InternalFormat {
    const FORMAT_ID: u32;
}

pub struct PackRed;

pub struct PackRedInteger;

pub struct PackRG;

pub struct PackRGInteger;

pub struct PackRGB;

pub struct PackRGBInteger;

pub struct PackRGBA;

pub struct PackRGBAInteger;

pub struct PackAlpha;

pub unsafe trait PixelPack<F, T> where F: PackFormat<T>, T: InternalFormat {
    const TYPE_ID: u32;
}

/// Marker trait for formats from which a [Sampler] can sample floating point values.
pub unsafe trait FloatSamplable: InternalFormat {}

unsafe impl FloatSamplable for R8 {}
unsafe impl FloatSamplable for R16F {}
unsafe impl FloatSamplable for R32F {}
unsafe impl FloatSamplable for RG8 {}
unsafe impl FloatSamplable for RG16F {}
unsafe impl FloatSamplable for RG32F {}
unsafe impl FloatSamplable for RGB8 {}
unsafe impl FloatSamplable for SRGB8 {}
unsafe impl FloatSamplable for RGB565 {}
unsafe impl FloatSamplable for RGB16F {}
unsafe impl FloatSamplable for RGB32F {}
unsafe impl FloatSamplable for R11F_G11F_B10F {}
unsafe impl FloatSamplable for RGB9_E5 {}
unsafe impl FloatSamplable for RGBA8 {}
unsafe impl FloatSamplable for SRGB8_ALPHA8 {}
unsafe impl FloatSamplable for RGBA4 {}
unsafe impl FloatSamplable for RGB5_A1 {}
unsafe impl FloatSamplable for RGB10_A2 {}
unsafe impl FloatSamplable for RGBA16F {}
unsafe impl FloatSamplable for RGBA32F {}
unsafe impl FloatSamplable for DepthComponent16 {}
unsafe impl FloatSamplable for DepthComponent24 {}
unsafe impl FloatSamplable for DepthComponent32F {}
unsafe impl FloatSamplable for Depth24Stencil8 {}
unsafe impl FloatSamplable for Depth32FStencil8 {}
unsafe impl FloatSamplable for Luminance {}
unsafe impl FloatSamplable for LuminanceAlpha {}

/// Marker trait for formats from which a [Sampler] can sample integer values.
pub unsafe trait IntegerSamplable: InternalFormat {}

unsafe impl UnsignedIntegerSamplable for R8I {}
unsafe impl UnsignedIntegerSamplable for R16I {}
unsafe impl UnsignedIntegerSamplable for R32I {}
unsafe impl UnsignedIntegerSamplable for RG8I {}
unsafe impl UnsignedIntegerSamplable for RG16I {}
unsafe impl UnsignedIntegerSamplable for RG32I {}
unsafe impl UnsignedIntegerSamplable for RGBA8I {}
unsafe impl UnsignedIntegerSamplable for RGBA16I {}
unsafe impl UnsignedIntegerSamplable for RGBA32I {}

/// Marker trait for formats from which a [Sampler] can sample unsigned integer values.
pub unsafe trait UnsignedIntegerSamplable: InternalFormat {}

unsafe impl UnsignedIntegerSamplable for R8UI {}
unsafe impl UnsignedIntegerSamplable for R16UI {}
unsafe impl UnsignedIntegerSamplable for R32UI {}
unsafe impl UnsignedIntegerSamplable for RG8UI {}
unsafe impl UnsignedIntegerSamplable for RG16UI {}
unsafe impl UnsignedIntegerSamplable for RG32UI {}
unsafe impl UnsignedIntegerSamplable for RGB8UI {}
unsafe impl UnsignedIntegerSamplable for RGBA8UI {}
unsafe impl UnsignedIntegerSamplable for RGB10_A2UI {}
unsafe impl UnsignedIntegerSamplable for RGBA16UI {}
unsafe impl UnsignedIntegerSamplable for RGBA32UI {}
unsafe impl UnsignedIntegerSamplable for StencilIndex8 {}

/// Marker trait for formats that can be used with a [ShadowSampler].
pub unsafe trait ShadowSamplable: InternalFormat {}

unsafe impl ShadowSamplable for DepthComponent16 {}
unsafe impl ShadowSamplable for DepthComponent24 {}
unsafe impl ShadowSamplable for DepthComponent32F {}
unsafe impl ShadowSamplable for Depth24Stencil8 {}
unsafe impl ShadowSamplable for Depth32FStencil8 {}

/// Marker trait for formats that can be used as a color attachment for a [RenderTarget] for a
/// color out that outputs floating point values.
pub unsafe trait FloatRenderable: InternalFormat {}

unsafe impl FloatRenderable for R8 {}
unsafe impl FloatRenderable for RG8 {}
unsafe impl FloatRenderable for RGB8 {}
unsafe impl FloatRenderable for RGBA8 {}
unsafe impl FloatRenderable for SRGB8_ALPHA8 {}
unsafe impl FloatRenderable for RGBA4 {}
unsafe impl FloatRenderable for RGB565 {}
unsafe impl FloatRenderable for RGB5_A1 {}
unsafe impl FloatRenderable for RGB10_A2 {}

// TODO: these require EXT_color_buffer_float, needs a check, probably at render pass creation.
unsafe impl FloatRenderable for R16F {}
unsafe impl FloatRenderable for R32F {}
unsafe impl FloatRenderable for RG16F {}
unsafe impl FloatRenderable for RG32F {}
unsafe impl FloatRenderable for RGBA16F {}
unsafe impl FloatRenderable for RGBA32F {}
unsafe impl FloatRenderable for R11F_G11F_B10F {}

/// Marker trait for formats that can be used as a color attachment for a [RenderTarget] for a
/// color out that outputs integer values.
pub unsafe trait IntegerRenderable: InternalFormat {}

unsafe impl IntegerRenderable for R8I {}
unsafe impl IntegerRenderable for R16I {}
unsafe impl IntegerRenderable for R32I {}
unsafe impl IntegerRenderable for RG8I {}
unsafe impl IntegerRenderable for RG16I {}
unsafe impl IntegerRenderable for RG32I {}
unsafe impl IntegerRenderable for RGBA8I {}
unsafe impl IntegerRenderable for RGBA16I {}
unsafe impl IntegerRenderable for RGBA32I {}

/// Marker trait for formats that can be used as a color attachment for a [RenderTarget] for a
/// color out that outputs unsigned integer values.
pub unsafe trait UnsignedIntegerRenderable: InternalFormat {}

unsafe impl UnsignedIntegerRenderable for R8UI {}
unsafe impl UnsignedIntegerRenderable for R16UI {}
unsafe impl UnsignedIntegerRenderable for R32UI {}
unsafe impl UnsignedIntegerRenderable for RG8UI {}
unsafe impl UnsignedIntegerRenderable for RG16UI {}
unsafe impl UnsignedIntegerRenderable for RG32UI {}
unsafe impl UnsignedIntegerRenderable for RGBA8UI {}
unsafe impl UnsignedIntegerRenderable for RGB10_A2UI {}
unsafe impl UnsignedIntegerRenderable for RGBA16UI {}
unsafe impl UnsignedIntegerRenderable for RGBA32UI {}

/// Marker trait for formats that can be used as a depth-stencil attachment for a [RenderTarget].
pub unsafe trait DepthStencilRenderable: InternalFormat {}

unsafe impl DepthStencilRenderable for Depth24Stencil8 {}
unsafe impl DepthStencilRenderable for Depth32FStencil8 {}

/// Marker trait for formats that can be used as a depth attachment for a [RenderTarget].
pub unsafe trait DepthRenderable: InternalFormat {}

unsafe impl DepthRenderable for DepthComponent16 {}
unsafe impl DepthRenderable for DepthComponent24 {}
unsafe impl DepthRenderable for DepthComponent32F {}

/// Marker trait for formats that can be used as a stencil attachment for a [RenderTarget].
pub unsafe trait StencilRenderable: InternalFormat {}

unsafe impl StencilRenderable for StencilIndex8 {}

/// Marker trait for formats that support linear filtering.
pub unsafe trait Filterable {}

unsafe impl Filterable for R8 {}
unsafe impl Filterable for R16F {}
unsafe impl Filterable for RG8 {}
unsafe impl Filterable for RG16F {}
unsafe impl Filterable for RGB8 {}
unsafe impl Filterable for SRGB8 {}
unsafe impl Filterable for RGB565 {}
unsafe impl Filterable for R11F_G11F_B10F {}
unsafe impl Filterable for RGB9_E5 {}
unsafe impl Filterable for RGB16F {}
unsafe impl Filterable for RGBA8 {}
unsafe impl Filterable for SRGB8_ALPHA8 {}
unsafe impl Filterable for RGB5_A1 {}
unsafe impl Filterable for RGBA4 {}
unsafe impl Filterable for RGB10_A2 {}
unsafe impl Filterable for RGBA16F {}

//pub unsafe trait CopyCompatible<F>
//    where
//        F: InternalFormat,
//{
//}
//
// TODO implement CopyCompatible for formats: copyTexSubImage requires the target format to contain
// a subset of the information contained in the source format

/// Trait implemented by image formats that can be used with texture images.
pub unsafe trait TextureFormat: InternalFormat {
    /// Returns an error if the `magnification_filter` cannot be used with this texture format given
    /// the enabled `extensions`.
    fn validate_magnification_filter(
        extensions: &Extensions,
        magnification_filter: MagnificationFilter,
    ) -> Result<(), InvalidMagnificationFilter>;

    /// Returns an error if the `minification_filter` cannot be used with this texture format given
    /// the enabled `extensions`.
    fn validate_minification_filter(
        extensions: &Extensions,
        minification_filter: MinificationFilter,
    ) -> Result<(), InvalidMinificationFilter>;
}

/// Error returned by [TextureFormat::validate_minification_filter].
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InvalidMinificationFilter {
    /// Returned if the [MinificationFilter] is always invalid in combination with the
    /// [TextureFormat].
    Always(MinificationFilter),

    /// Returned if the [MinificationFilter] can be valid in combination with the [TextureFormat],
    /// but requires an extension that is not enabled.
    ExtensionRequired(MinificationFilter),
}

/// Error returned by [TextureFormat::validate_magnification_filter].
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InvalidMagnificationFilter {
    /// Returned if the [MagnificationFilter] is always invalid in combination with the
    /// [TextureFormat].
    Always(MagnificationFilter),

    /// Returned if the [MagnificationFilter] can be valid in combination with the [TextureFormat],
    /// but requires an extension that is not enabled.
    ExtensionRequired(MagnificationFilter),
}

unsafe impl TextureFormat for R8 {
    fn validate_magnification_filter(
        _extensions: &Extensions,
        _filter: MagnificationFilter,
    ) -> Result<(), InvalidMagnificationFilter> {
        Ok(())
    }

    fn validate_minification_filter(
        _extensions: &Extensions,
        _filter: MinificationFilter,
    ) -> Result<(), InvalidMinificationFilter> {
        Ok(())
    }
}

unsafe impl TextureFormat for R16F {
    fn validate_magnification_filter(
        _extensions: &Extensions,
        _filter: MagnificationFilter,
    ) -> Result<(), InvalidMagnificationFilter> {
        Ok(())
    }

    fn validate_minification_filter(
        _extensions: &Extensions,
        _filter: MinificationFilter,
    ) -> Result<(), InvalidMinificationFilter> {
        Ok(())
    }
}

unsafe impl TextureFormat for R32F {
    fn validate_magnification_filter(
        extensions: &Extensions,
        filter: MagnificationFilter,
    ) -> Result<(), InvalidMagnificationFilter> {
        match filter {
            MagnificationFilter::Nearest => Ok(()),
            MagnificationFilter::Linear => {
                if extensions.texture_float_linear().is_enabled() {
                    Ok(())
                } else {
                    Err(InvalidMagnificationFilter::ExtensionRequired(filter))
                }
            }
        }
    }

    fn validate_minification_filter(
        extensions: &Extensions,
        filter: MinificationFilter,
    ) -> Result<(), InvalidMinificationFilter> {
        match filter {
            MinificationFilter::Nearest => Ok(()),
            MinificationFilter::NearestMipmapNearest => Ok(()),
            _ => {
                if extensions.texture_float_linear().is_enabled() {
                    Ok(())
                } else {
                    Err(InvalidMinificationFilter::ExtensionRequired(filter))
                }
            }
        }
    }
}

unsafe impl TextureFormat for R8UI {
    fn validate_magnification_filter(
        _extensions: &Extensions,
        filter: MagnificationFilter,
    ) -> Result<(), InvalidMagnificationFilter> {
        match filter {
            MagnificationFilter::Nearest => Ok(()),
            MagnificationFilter::Linear => Err(InvalidMagnificationFilter::Always(filter)),
        }
    }

    fn validate_minification_filter(
        _extensions: &Extensions,
        filter: MinificationFilter,
    ) -> Result<(), InvalidMinificationFilter> {
        match filter {
            MinificationFilter::Nearest => Ok(()),
            MinificationFilter::NearestMipmapNearest => Ok(()),
            _ => Err(InvalidMinificationFilter::Always(filter)),
        }
    }
}

unsafe impl TextureFormat for R8I {
    fn validate_magnification_filter(
        _extensions: &Extensions,
        filter: MagnificationFilter,
    ) -> Result<(), InvalidMagnificationFilter> {
        match filter {
            MagnificationFilter::Nearest => Ok(()),
            MagnificationFilter::Linear => Err(InvalidMagnificationFilter::Always(filter)),
        }
    }

    fn validate_minification_filter(
        _extensions: &Extensions,
        filter: MinificationFilter,
    ) -> Result<(), InvalidMinificationFilter> {
        match filter {
            MinificationFilter::Nearest => Ok(()),
            MinificationFilter::NearestMipmapNearest => Ok(()),
            _ => Err(InvalidMinificationFilter::Always(filter)),
        }
    }
}

unsafe impl TextureFormat for R16UI {
    fn validate_magnification_filter(
        _extensions: &Extensions,
        filter: MagnificationFilter,
    ) -> Result<(), InvalidMagnificationFilter> {
        match filter {
            MagnificationFilter::Nearest => Ok(()),
            MagnificationFilter::Linear => Err(InvalidMagnificationFilter::Always(filter)),
        }
    }

    fn validate_minification_filter(
        _extensions: &Extensions,
        filter: MinificationFilter,
    ) -> Result<(), InvalidMinificationFilter> {
        match filter {
            MinificationFilter::Nearest => Ok(()),
            MinificationFilter::NearestMipmapNearest => Ok(()),
            _ => Err(InvalidMinificationFilter::Always(filter)),
        }
    }
}

unsafe impl TextureFormat for R16I {
    fn validate_magnification_filter(
        _extensions: &Extensions,
        filter: MagnificationFilter,
    ) -> Result<(), InvalidMagnificationFilter> {
        match filter {
            MagnificationFilter::Nearest => Ok(()),
            MagnificationFilter::Linear => Err(InvalidMagnificationFilter::Always(filter)),
        }
    }

    fn validate_minification_filter(
        _extensions: &Extensions,
        filter: MinificationFilter,
    ) -> Result<(), InvalidMinificationFilter> {
        match filter {
            MinificationFilter::Nearest => Ok(()),
            MinificationFilter::NearestMipmapNearest => Ok(()),
            _ => Err(InvalidMinificationFilter::Always(filter)),
        }
    }
}

unsafe impl TextureFormat for R32UI {
    fn validate_magnification_filter(
        _extensions: &Extensions,
        filter: MagnificationFilter,
    ) -> Result<(), InvalidMagnificationFilter> {
        match filter {
            MagnificationFilter::Nearest => Ok(()),
            MagnificationFilter::Linear => Err(InvalidMagnificationFilter::Always(filter)),
        }
    }

    fn validate_minification_filter(
        _extensions: &Extensions,
        filter: MinificationFilter,
    ) -> Result<(), InvalidMinificationFilter> {
        match filter {
            MinificationFilter::Nearest => Ok(()),
            MinificationFilter::NearestMipmapNearest => Ok(()),
            _ => Err(InvalidMinificationFilter::Always(filter)),
        }
    }
}

unsafe impl TextureFormat for R32I {
    fn validate_magnification_filter(
        _extensions: &Extensions,
        filter: MagnificationFilter,
    ) -> Result<(), InvalidMagnificationFilter> {
        match filter {
            MagnificationFilter::Nearest => Ok(()),
            MagnificationFilter::Linear => Err(InvalidMagnificationFilter::Always(filter)),
        }
    }

    fn validate_minification_filter(
        _extensions: &Extensions,
        filter: MinificationFilter,
    ) -> Result<(), InvalidMinificationFilter> {
        match filter {
            MinificationFilter::Nearest => Ok(()),
            MinificationFilter::NearestMipmapNearest => Ok(()),
            _ => Err(InvalidMinificationFilter::Always(filter)),
        }
    }
}

unsafe impl TextureFormat for RG8 {
    fn validate_magnification_filter(
        _extensions: &Extensions,
        _filter: MagnificationFilter,
    ) -> Result<(), InvalidMagnificationFilter> {
        Ok(())
    }

    fn validate_minification_filter(
        _extensions: &Extensions,
        _filter: MinificationFilter,
    ) -> Result<(), InvalidMinificationFilter> {
        Ok(())
    }
}

unsafe impl TextureFormat for RG16F {
    fn validate_magnification_filter(
        _extensions: &Extensions,
        _filter: MagnificationFilter,
    ) -> Result<(), InvalidMagnificationFilter> {
        Ok(())
    }

    fn validate_minification_filter(
        _extensions: &Extensions,
        _filter: MinificationFilter,
    ) -> Result<(), InvalidMinificationFilter> {
        Ok(())
    }
}

unsafe impl TextureFormat for RG32F {
    fn validate_magnification_filter(
        extensions: &Extensions,
        filter: MagnificationFilter,
    ) -> Result<(), InvalidMagnificationFilter> {
        match filter {
            MagnificationFilter::Nearest => Ok(()),
            MagnificationFilter::Linear => {
                if extensions.texture_float_linear().is_enabled() {
                    Ok(())
                } else {
                    Err(InvalidMagnificationFilter::ExtensionRequired(filter))
                }
            }
        }
    }

    fn validate_minification_filter(
        extensions: &Extensions,
        filter: MinificationFilter,
    ) -> Result<(), InvalidMinificationFilter> {
        match filter {
            MinificationFilter::Nearest => Ok(()),
            MinificationFilter::NearestMipmapNearest => Ok(()),
            _ => {
                if extensions.texture_float_linear().is_enabled() {
                    Ok(())
                } else {
                    Err(InvalidMinificationFilter::ExtensionRequired(filter))
                }
            }
        }
    }
}

unsafe impl TextureFormat for RG8UI {
    fn validate_magnification_filter(
        _extensions: &Extensions,
        filter: MagnificationFilter,
    ) -> Result<(), InvalidMagnificationFilter> {
        match filter {
            MagnificationFilter::Nearest => Ok(()),
            MagnificationFilter::Linear => Err(InvalidMagnificationFilter::Always(filter)),
        }
    }

    fn validate_minification_filter(
        _extensions: &Extensions,
        filter: MinificationFilter,
    ) -> Result<(), InvalidMinificationFilter> {
        match filter {
            MinificationFilter::Nearest => Ok(()),
            MinificationFilter::NearestMipmapNearest => Ok(()),
            _ => Err(InvalidMinificationFilter::Always(filter)),
        }
    }
}

unsafe impl TextureFormat for RG8I {
    fn validate_magnification_filter(
        _extensions: &Extensions,
        filter: MagnificationFilter,
    ) -> Result<(), InvalidMagnificationFilter> {
        match filter {
            MagnificationFilter::Nearest => Ok(()),
            MagnificationFilter::Linear => Err(InvalidMagnificationFilter::Always(filter)),
        }
    }

    fn validate_minification_filter(
        _extensions: &Extensions,
        filter: MinificationFilter,
    ) -> Result<(), InvalidMinificationFilter> {
        match filter {
            MinificationFilter::Nearest => Ok(()),
            MinificationFilter::NearestMipmapNearest => Ok(()),
            _ => Err(InvalidMinificationFilter::Always(filter)),
        }
    }
}

unsafe impl TextureFormat for RG16UI {
    fn validate_magnification_filter(
        _extensions: &Extensions,
        filter: MagnificationFilter,
    ) -> Result<(), InvalidMagnificationFilter> {
        match filter {
            MagnificationFilter::Nearest => Ok(()),
            MagnificationFilter::Linear => Err(InvalidMagnificationFilter::Always(filter)),
        }
    }

    fn validate_minification_filter(
        _extensions: &Extensions,
        filter: MinificationFilter,
    ) -> Result<(), InvalidMinificationFilter> {
        match filter {
            MinificationFilter::Nearest => Ok(()),
            MinificationFilter::NearestMipmapNearest => Ok(()),
            _ => Err(InvalidMinificationFilter::Always(filter)),
        }
    }
}

unsafe impl TextureFormat for RG16I {
    fn validate_magnification_filter(
        _extensions: &Extensions,
        filter: MagnificationFilter,
    ) -> Result<(), InvalidMagnificationFilter> {
        match filter {
            MagnificationFilter::Nearest => Ok(()),
            MagnificationFilter::Linear => Err(InvalidMagnificationFilter::Always(filter)),
        }
    }

    fn validate_minification_filter(
        _extensions: &Extensions,
        filter: MinificationFilter,
    ) -> Result<(), InvalidMinificationFilter> {
        match filter {
            MinificationFilter::Nearest => Ok(()),
            MinificationFilter::NearestMipmapNearest => Ok(()),
            _ => Err(InvalidMinificationFilter::Always(filter)),
        }
    }
}

unsafe impl TextureFormat for RG32UI {
    fn validate_magnification_filter(
        _extensions: &Extensions,
        filter: MagnificationFilter,
    ) -> Result<(), InvalidMagnificationFilter> {
        match filter {
            MagnificationFilter::Nearest => Ok(()),
            MagnificationFilter::Linear => Err(InvalidMagnificationFilter::Always(filter)),
        }
    }

    fn validate_minification_filter(
        _extensions: &Extensions,
        filter: MinificationFilter,
    ) -> Result<(), InvalidMinificationFilter> {
        match filter {
            MinificationFilter::Nearest => Ok(()),
            MinificationFilter::NearestMipmapNearest => Ok(()),
            _ => Err(InvalidMinificationFilter::Always(filter)),
        }
    }
}

unsafe impl TextureFormat for RG32I {
    fn validate_magnification_filter(
        _extensions: &Extensions,
        filter: MagnificationFilter,
    ) -> Result<(), InvalidMagnificationFilter> {
        match filter {
            MagnificationFilter::Nearest => Ok(()),
            MagnificationFilter::Linear => Err(InvalidMagnificationFilter::Always(filter)),
        }
    }

    fn validate_minification_filter(
        _extensions: &Extensions,
        filter: MinificationFilter,
    ) -> Result<(), InvalidMinificationFilter> {
        match filter {
            MinificationFilter::Nearest => Ok(()),
            MinificationFilter::NearestMipmapNearest => Ok(()),
            _ => Err(InvalidMinificationFilter::Always(filter)),
        }
    }
}

unsafe impl TextureFormat for RGB8 {
    fn validate_magnification_filter(
        _extensions: &Extensions,
        _filter: MagnificationFilter,
    ) -> Result<(), InvalidMagnificationFilter> {
        Ok(())
    }

    fn validate_minification_filter(
        _extensions: &Extensions,
        _filter: MinificationFilter,
    ) -> Result<(), InvalidMinificationFilter> {
        Ok(())
    }
}

unsafe impl TextureFormat for SRGB8 {
    fn validate_magnification_filter(
        _extensions: &Extensions,
        _filter: MagnificationFilter,
    ) -> Result<(), InvalidMagnificationFilter> {
        Ok(())
    }

    fn validate_minification_filter(
        _extensions: &Extensions,
        _filter: MinificationFilter,
    ) -> Result<(), InvalidMinificationFilter> {
        Ok(())
    }
}

unsafe impl TextureFormat for RGB565 {
    fn validate_magnification_filter(
        _extensions: &Extensions,
        _filter: MagnificationFilter,
    ) -> Result<(), InvalidMagnificationFilter> {
        Ok(())
    }

    fn validate_minification_filter(
        _extensions: &Extensions,
        _filter: MinificationFilter,
    ) -> Result<(), InvalidMinificationFilter> {
        Ok(())
    }
}

unsafe impl TextureFormat for R11F_G11F_B10F {
    fn validate_magnification_filter(
        _extensions: &Extensions,
        _filter: MagnificationFilter,
    ) -> Result<(), InvalidMagnificationFilter> {
        Ok(())
    }

    fn validate_minification_filter(
        _extensions: &Extensions,
        _filter: MinificationFilter,
    ) -> Result<(), InvalidMinificationFilter> {
        Ok(())
    }
}

unsafe impl TextureFormat for RGB9_E5 {
    fn validate_magnification_filter(
        _extensions: &Extensions,
        _filter: MagnificationFilter,
    ) -> Result<(), InvalidMagnificationFilter> {
        Ok(())
    }

    fn validate_minification_filter(
        _extensions: &Extensions,
        _filter: MinificationFilter,
    ) -> Result<(), InvalidMinificationFilter> {
        Ok(())
    }
}

unsafe impl TextureFormat for RGB16F {
    fn validate_magnification_filter(
        _extensions: &Extensions,
        _filter: MagnificationFilter,
    ) -> Result<(), InvalidMagnificationFilter> {
        Ok(())
    }

    fn validate_minification_filter(
        _extensions: &Extensions,
        _filter: MinificationFilter,
    ) -> Result<(), InvalidMinificationFilter> {
        Ok(())
    }
}

unsafe impl TextureFormat for RGB32F {
    fn validate_magnification_filter(
        extensions: &Extensions,
        filter: MagnificationFilter,
    ) -> Result<(), InvalidMagnificationFilter> {
        match filter {
            MagnificationFilter::Nearest => Ok(()),
            MagnificationFilter::Linear => {
                if extensions.texture_float_linear().is_enabled() {
                    Ok(())
                } else {
                    Err(InvalidMagnificationFilter::ExtensionRequired(filter))
                }
            }
        }
    }

    fn validate_minification_filter(
        extensions: &Extensions,
        filter: MinificationFilter,
    ) -> Result<(), InvalidMinificationFilter> {
        match filter {
            MinificationFilter::Nearest => Ok(()),
            MinificationFilter::NearestMipmapNearest => Ok(()),
            _ => {
                if extensions.texture_float_linear().is_enabled() {
                    Ok(())
                } else {
                    Err(InvalidMinificationFilter::ExtensionRequired(filter))
                }
            }
        }
    }
}

unsafe impl TextureFormat for RGB8UI {
    fn validate_magnification_filter(
        _extensions: &Extensions,
        filter: MagnificationFilter,
    ) -> Result<(), InvalidMagnificationFilter> {
        match filter {
            MagnificationFilter::Nearest => Ok(()),
            MagnificationFilter::Linear => Err(InvalidMagnificationFilter::Always(filter)),
        }
    }

    fn validate_minification_filter(
        _extensions: &Extensions,
        filter: MinificationFilter,
    ) -> Result<(), InvalidMinificationFilter> {
        match filter {
            MinificationFilter::Nearest => Ok(()),
            MinificationFilter::NearestMipmapNearest => Ok(()),
            _ => Err(InvalidMinificationFilter::Always(filter)),
        }
    }
}

unsafe impl TextureFormat for RGB8I {
    fn validate_magnification_filter(
        _extensions: &Extensions,
        filter: MagnificationFilter,
    ) -> Result<(), InvalidMagnificationFilter> {
        match filter {
            MagnificationFilter::Nearest => Ok(()),
            MagnificationFilter::Linear => Err(InvalidMagnificationFilter::Always(filter)),
        }
    }

    fn validate_minification_filter(
        _extensions: &Extensions,
        filter: MinificationFilter,
    ) -> Result<(), InvalidMinificationFilter> {
        match filter {
            MinificationFilter::Nearest => Ok(()),
            MinificationFilter::NearestMipmapNearest => Ok(()),
            _ => Err(InvalidMinificationFilter::Always(filter)),
        }
    }
}

unsafe impl TextureFormat for RGB16UI {
    fn validate_magnification_filter(
        _extensions: &Extensions,
        filter: MagnificationFilter,
    ) -> Result<(), InvalidMagnificationFilter> {
        match filter {
            MagnificationFilter::Nearest => Ok(()),
            MagnificationFilter::Linear => Err(InvalidMagnificationFilter::Always(filter)),
        }
    }

    fn validate_minification_filter(
        _extensions: &Extensions,
        filter: MinificationFilter,
    ) -> Result<(), InvalidMinificationFilter> {
        match filter {
            MinificationFilter::Nearest => Ok(()),
            MinificationFilter::NearestMipmapNearest => Ok(()),
            _ => Err(InvalidMinificationFilter::Always(filter)),
        }
    }
}

unsafe impl TextureFormat for RGB16I {
    fn validate_magnification_filter(
        _extensions: &Extensions,
        filter: MagnificationFilter,
    ) -> Result<(), InvalidMagnificationFilter> {
        match filter {
            MagnificationFilter::Nearest => Ok(()),
            MagnificationFilter::Linear => Err(InvalidMagnificationFilter::Always(filter)),
        }
    }

    fn validate_minification_filter(
        _extensions: &Extensions,
        filter: MinificationFilter,
    ) -> Result<(), InvalidMinificationFilter> {
        match filter {
            MinificationFilter::Nearest => Ok(()),
            MinificationFilter::NearestMipmapNearest => Ok(()),
            _ => Err(InvalidMinificationFilter::Always(filter)),
        }
    }
}

unsafe impl TextureFormat for RGB32UI {
    fn validate_magnification_filter(
        _extensions: &Extensions,
        filter: MagnificationFilter,
    ) -> Result<(), InvalidMagnificationFilter> {
        match filter {
            MagnificationFilter::Nearest => Ok(()),
            MagnificationFilter::Linear => Err(InvalidMagnificationFilter::Always(filter)),
        }
    }

    fn validate_minification_filter(
        _extensions: &Extensions,
        filter: MinificationFilter,
    ) -> Result<(), InvalidMinificationFilter> {
        match filter {
            MinificationFilter::Nearest => Ok(()),
            MinificationFilter::NearestMipmapNearest => Ok(()),
            _ => Err(InvalidMinificationFilter::Always(filter)),
        }
    }
}

unsafe impl TextureFormat for RGB32I {
    fn validate_magnification_filter(
        _extensions: &Extensions,
        filter: MagnificationFilter,
    ) -> Result<(), InvalidMagnificationFilter> {
        match filter {
            MagnificationFilter::Nearest => Ok(()),
            MagnificationFilter::Linear => Err(InvalidMagnificationFilter::Always(filter)),
        }
    }

    fn validate_minification_filter(
        _extensions: &Extensions,
        filter: MinificationFilter,
    ) -> Result<(), InvalidMinificationFilter> {
        match filter {
            MinificationFilter::Nearest => Ok(()),
            MinificationFilter::NearestMipmapNearest => Ok(()),
            _ => Err(InvalidMinificationFilter::Always(filter)),
        }
    }
}

unsafe impl TextureFormat for RGBA8 {
    fn validate_magnification_filter(
        _extensions: &Extensions,
        _filter: MagnificationFilter,
    ) -> Result<(), InvalidMagnificationFilter> {
        Ok(())
    }

    fn validate_minification_filter(
        _extensions: &Extensions,
        _filter: MinificationFilter,
    ) -> Result<(), InvalidMinificationFilter> {
        Ok(())
    }
}

unsafe impl TextureFormat for SRGB8_ALPHA8 {
    fn validate_magnification_filter(
        _extensions: &Extensions,
        _filter: MagnificationFilter,
    ) -> Result<(), InvalidMagnificationFilter> {
        Ok(())
    }

    fn validate_minification_filter(
        _extensions: &Extensions,
        _filter: MinificationFilter,
    ) -> Result<(), InvalidMinificationFilter> {
        Ok(())
    }
}

unsafe impl TextureFormat for RGB5_A1 {
    fn validate_magnification_filter(
        _extensions: &Extensions,
        _filter: MagnificationFilter,
    ) -> Result<(), InvalidMagnificationFilter> {
        Ok(())
    }

    fn validate_minification_filter(
        _extensions: &Extensions,
        _filter: MinificationFilter,
    ) -> Result<(), InvalidMinificationFilter> {
        Ok(())
    }
}

unsafe impl TextureFormat for RGBA4 {
    fn validate_magnification_filter(
        _extensions: &Extensions,
        _filter: MagnificationFilter,
    ) -> Result<(), InvalidMagnificationFilter> {
        Ok(())
    }

    fn validate_minification_filter(
        _extensions: &Extensions,
        _filter: MinificationFilter,
    ) -> Result<(), InvalidMinificationFilter> {
        Ok(())
    }
}

unsafe impl TextureFormat for RGB10_A2 {
    fn validate_magnification_filter(
        _extensions: &Extensions,
        _filter: MagnificationFilter,
    ) -> Result<(), InvalidMagnificationFilter> {
        Ok(())
    }

    fn validate_minification_filter(
        _extensions: &Extensions,
        _filter: MinificationFilter,
    ) -> Result<(), InvalidMinificationFilter> {
        Ok(())
    }
}

unsafe impl TextureFormat for RGBA16F {
    fn validate_magnification_filter(
        _extensions: &Extensions,
        _filter: MagnificationFilter,
    ) -> Result<(), InvalidMagnificationFilter> {
        Ok(())
    }

    fn validate_minification_filter(
        _extensions: &Extensions,
        _filter: MinificationFilter,
    ) -> Result<(), InvalidMinificationFilter> {
        Ok(())
    }
}

unsafe impl TextureFormat for RGBA32F {
    fn validate_magnification_filter(
        extensions: &Extensions,
        filter: MagnificationFilter,
    ) -> Result<(), InvalidMagnificationFilter> {
        match filter {
            MagnificationFilter::Nearest => Ok(()),
            MagnificationFilter::Linear => {
                if extensions.texture_float_linear().is_enabled() {
                    Ok(())
                } else {
                    Err(InvalidMagnificationFilter::ExtensionRequired(filter))
                }
            }
        }
    }

    fn validate_minification_filter(
        extensions: &Extensions,
        filter: MinificationFilter,
    ) -> Result<(), InvalidMinificationFilter> {
        match filter {
            MinificationFilter::Nearest => Ok(()),
            MinificationFilter::NearestMipmapNearest => Ok(()),
            _ => {
                if extensions.texture_float_linear().is_enabled() {
                    Ok(())
                } else {
                    Err(InvalidMinificationFilter::ExtensionRequired(filter))
                }
            }
        }
    }
}

unsafe impl TextureFormat for RGBA8UI {
    fn validate_magnification_filter(
        _extensions: &Extensions,
        filter: MagnificationFilter,
    ) -> Result<(), InvalidMagnificationFilter> {
        match filter {
            MagnificationFilter::Nearest => Ok(()),
            MagnificationFilter::Linear => Err(InvalidMagnificationFilter::Always(filter)),
        }
    }

    fn validate_minification_filter(
        _extensions: &Extensions,
        filter: MinificationFilter,
    ) -> Result<(), InvalidMinificationFilter> {
        match filter {
            MinificationFilter::Nearest => Ok(()),
            MinificationFilter::NearestMipmapNearest => Ok(()),
            _ => Err(InvalidMinificationFilter::Always(filter)),
        }
    }
}

unsafe impl TextureFormat for RGBA8I {
    fn validate_magnification_filter(
        _extensions: &Extensions,
        filter: MagnificationFilter,
    ) -> Result<(), InvalidMagnificationFilter> {
        match filter {
            MagnificationFilter::Nearest => Ok(()),
            MagnificationFilter::Linear => Err(InvalidMagnificationFilter::Always(filter)),
        }
    }

    fn validate_minification_filter(
        _extensions: &Extensions,
        filter: MinificationFilter,
    ) -> Result<(), InvalidMinificationFilter> {
        match filter {
            MinificationFilter::Nearest => Ok(()),
            MinificationFilter::NearestMipmapNearest => Ok(()),
            _ => Err(InvalidMinificationFilter::Always(filter)),
        }
    }
}

unsafe impl TextureFormat for RGBA16UI {
    fn validate_magnification_filter(
        _extensions: &Extensions,
        filter: MagnificationFilter,
    ) -> Result<(), InvalidMagnificationFilter> {
        match filter {
            MagnificationFilter::Nearest => Ok(()),
            MagnificationFilter::Linear => Err(InvalidMagnificationFilter::Always(filter)),
        }
    }

    fn validate_minification_filter(
        _extensions: &Extensions,
        filter: MinificationFilter,
    ) -> Result<(), InvalidMinificationFilter> {
        match filter {
            MinificationFilter::Nearest => Ok(()),
            MinificationFilter::NearestMipmapNearest => Ok(()),
            _ => Err(InvalidMinificationFilter::Always(filter)),
        }
    }
}

unsafe impl TextureFormat for RGBA16I {
    fn validate_magnification_filter(
        _extensions: &Extensions,
        filter: MagnificationFilter,
    ) -> Result<(), InvalidMagnificationFilter> {
        match filter {
            MagnificationFilter::Nearest => Ok(()),
            MagnificationFilter::Linear => Err(InvalidMagnificationFilter::Always(filter)),
        }
    }

    fn validate_minification_filter(
        _extensions: &Extensions,
        filter: MinificationFilter,
    ) -> Result<(), InvalidMinificationFilter> {
        match filter {
            MinificationFilter::Nearest => Ok(()),
            MinificationFilter::NearestMipmapNearest => Ok(()),
            _ => Err(InvalidMinificationFilter::Always(filter)),
        }
    }
}

unsafe impl TextureFormat for RGBA32UI {
    fn validate_magnification_filter(
        _extensions: &Extensions,
        filter: MagnificationFilter,
    ) -> Result<(), InvalidMagnificationFilter> {
        match filter {
            MagnificationFilter::Nearest => Ok(()),
            MagnificationFilter::Linear => Err(InvalidMagnificationFilter::Always(filter)),
        }
    }

    fn validate_minification_filter(
        _extensions: &Extensions,
        filter: MinificationFilter,
    ) -> Result<(), InvalidMinificationFilter> {
        match filter {
            MinificationFilter::Nearest => Ok(()),
            MinificationFilter::NearestMipmapNearest => Ok(()),
            _ => Err(InvalidMinificationFilter::Always(filter)),
        }
    }
}

unsafe impl TextureFormat for RGBA32I {
    fn validate_magnification_filter(
        _extensions: &Extensions,
        filter: MagnificationFilter,
    ) -> Result<(), InvalidMagnificationFilter> {
        match filter {
            MagnificationFilter::Nearest => Ok(()),
            MagnificationFilter::Linear => Err(InvalidMagnificationFilter::Always(filter)),
        }
    }

    fn validate_minification_filter(
        _extensions: &Extensions,
        filter: MinificationFilter,
    ) -> Result<(), InvalidMinificationFilter> {
        match filter {
            MinificationFilter::Nearest => Ok(()),
            MinificationFilter::NearestMipmapNearest => Ok(()),
            _ => Err(InvalidMinificationFilter::Always(filter)),
        }
    }
}

unsafe impl TextureFormat for DepthComponent16 {
    fn validate_magnification_filter(
        _extensions: &Extensions,
        filter: MagnificationFilter,
    ) -> Result<(), InvalidMagnificationFilter> {
        match filter {
            MagnificationFilter::Nearest => Ok(()),
            MagnificationFilter::Linear => Err(InvalidMagnificationFilter::Always(filter)),
        }
    }

    fn validate_minification_filter(
        _extensions: &Extensions,
        filter: MinificationFilter,
    ) -> Result<(), InvalidMinificationFilter> {
        match filter {
            MinificationFilter::Nearest => Ok(()),
            MinificationFilter::NearestMipmapNearest => Ok(()),
            _ => Err(InvalidMinificationFilter::Always(filter)),
        }
    }
}

unsafe impl TextureFormat for DepthComponent24 {
    fn validate_magnification_filter(
        _extensions: &Extensions,
        filter: MagnificationFilter,
    ) -> Result<(), InvalidMagnificationFilter> {
        match filter {
            MagnificationFilter::Nearest => Ok(()),
            MagnificationFilter::Linear => Err(InvalidMagnificationFilter::Always(filter)),
        }
    }

    fn validate_minification_filter(
        _extensions: &Extensions,
        filter: MinificationFilter,
    ) -> Result<(), InvalidMinificationFilter> {
        match filter {
            MinificationFilter::Nearest => Ok(()),
            MinificationFilter::NearestMipmapNearest => Ok(()),
            _ => Err(InvalidMinificationFilter::Always(filter)),
        }
    }
}

unsafe impl TextureFormat for DepthComponent32F {
    fn validate_magnification_filter(
        _extensions: &Extensions,
        filter: MagnificationFilter,
    ) -> Result<(), InvalidMagnificationFilter> {
        match filter {
            MagnificationFilter::Nearest => Ok(()),
            MagnificationFilter::Linear => Err(InvalidMagnificationFilter::Always(filter)),
        }
    }

    fn validate_minification_filter(
        _extensions: &Extensions,
        filter: MinificationFilter,
    ) -> Result<(), InvalidMinificationFilter> {
        match filter {
            MinificationFilter::Nearest => Ok(()),
            MinificationFilter::NearestMipmapNearest => Ok(()),
            _ => Err(InvalidMinificationFilter::Always(filter)),
        }
    }
}

unsafe impl TextureFormat for Depth24Stencil8 {
    fn validate_magnification_filter(
        _extensions: &Extensions,
        filter: MagnificationFilter,
    ) -> Result<(), InvalidMagnificationFilter> {
        match filter {
            MagnificationFilter::Nearest => Ok(()),
            MagnificationFilter::Linear => Err(InvalidMagnificationFilter::Always(filter)),
        }
    }

    fn validate_minification_filter(
        _extensions: &Extensions,
        filter: MinificationFilter,
    ) -> Result<(), InvalidMinificationFilter> {
        match filter {
            MinificationFilter::Nearest => Ok(()),
            MinificationFilter::NearestMipmapNearest => Ok(()),
            _ => Err(InvalidMinificationFilter::Always(filter)),
        }
    }
}

unsafe impl TextureFormat for Depth32FStencil8 {
    fn validate_magnification_filter(
        _extensions: &Extensions,
        filter: MagnificationFilter,
    ) -> Result<(), InvalidMagnificationFilter> {
        match filter {
            MagnificationFilter::Nearest => Ok(()),
            MagnificationFilter::Linear => Err(InvalidMagnificationFilter::Always(filter)),
        }
    }

    fn validate_minification_filter(
        _extensions: &Extensions,
        filter: MinificationFilter,
    ) -> Result<(), InvalidMinificationFilter> {
        match filter {
            MinificationFilter::Nearest => Ok(()),
            MinificationFilter::NearestMipmapNearest => Ok(()),
            _ => Err(InvalidMinificationFilter::Always(filter)),
        }
    }
}

unsafe impl TextureFormat for Luminance {
    fn validate_magnification_filter(
        _extensions: &Extensions,
        _filter: MagnificationFilter,
    ) -> Result<(), InvalidMagnificationFilter> {
        Ok(())
    }

    fn validate_minification_filter(
        _extensions: &Extensions,
        _filter: MinificationFilter,
    ) -> Result<(), InvalidMinificationFilter> {
        Ok(())
    }
}

unsafe impl TextureFormat for LuminanceAlpha {
    fn validate_magnification_filter(
        _extensions: &Extensions,
        _filter: MagnificationFilter,
    ) -> Result<(), InvalidMagnificationFilter> {
        Ok(())
    }

    fn validate_minification_filter(
        _extensions: &Extensions,
        _filter: MinificationFilter,
    ) -> Result<(), InvalidMinificationFilter> {
        Ok(())
    }
}

/// Marker trait for formats that can be used as the format for a [Renderbuffer] image.
pub unsafe trait RenderbufferFormat: InternalFormat {}

unsafe impl RenderbufferFormat for R8 {}
unsafe impl RenderbufferFormat for R8UI {}
unsafe impl RenderbufferFormat for R8I {}
unsafe impl RenderbufferFormat for R16UI {}
unsafe impl RenderbufferFormat for R16I {}
unsafe impl RenderbufferFormat for R32UI {}
unsafe impl RenderbufferFormat for R32I {}
unsafe impl RenderbufferFormat for RG8 {}
unsafe impl RenderbufferFormat for RG8UI {}
unsafe impl RenderbufferFormat for RG8I {}
unsafe impl RenderbufferFormat for RG16UI {}
unsafe impl RenderbufferFormat for RG16I {}
unsafe impl RenderbufferFormat for RG32UI {}
unsafe impl RenderbufferFormat for RG32I {}
unsafe impl RenderbufferFormat for RGB8 {}
unsafe impl RenderbufferFormat for RGBA8 {}
unsafe impl RenderbufferFormat for SRGB8_ALPHA8 {}
unsafe impl RenderbufferFormat for RGB10_A2 {}
unsafe impl RenderbufferFormat for RGBA8UI {}
unsafe impl RenderbufferFormat for RGBA8I {}
unsafe impl RenderbufferFormat for RGB10_A2UI {}
unsafe impl RenderbufferFormat for RGBA16UI {}
unsafe impl RenderbufferFormat for RGBA16I {}
unsafe impl RenderbufferFormat for RGBA32UI {}
unsafe impl RenderbufferFormat for RGBA32I {}
unsafe impl RenderbufferFormat for DepthComponent16 {}
unsafe impl RenderbufferFormat for DepthComponent24 {}
unsafe impl RenderbufferFormat for DepthComponent32F {}
unsafe impl RenderbufferFormat for Depth24Stencil8 {}
unsafe impl RenderbufferFormat for Depth32FStencil8 {}
unsafe impl RenderbufferFormat for StencilIndex8 {}

pub struct R8;

unsafe impl InternalFormat for R8 {
    const ID: u32 = Gl::R8;
}

unsafe impl PixelUnpack<R8> for u8 {
    const FORMAT_ID: u32 = Gl::RED;

    const TYPE_ID: u32 = Gl::UNSIGNED_BYTE;
}

pub struct R16F;

unsafe impl InternalFormat for R16F {
    const ID: u32 = Gl::R16F;
}

unsafe impl PixelUnpack<R16F> for f32 {
    const FORMAT_ID: u32 = Gl::RED;

    const TYPE_ID: u32 = Gl::FLOAT;
}

pub struct R32F;

unsafe impl InternalFormat for R32F {
    const ID: u32 = Gl::R32F;
}

unsafe impl PixelUnpack<R32F> for f32 {
    const FORMAT_ID: u32 = Gl::RED;

    const TYPE_ID: u32 = Gl::FLOAT;
}

pub struct R8UI;

unsafe impl InternalFormat for R8UI {
    const ID: u32 = Gl::R8UI;
}

unsafe impl PixelUnpack<R8UI> for u8 {
    const FORMAT_ID: u32 = Gl::RED_INTEGER;

    const TYPE_ID: u32 = Gl::UNSIGNED_BYTE;
}

pub struct R8I;

unsafe impl InternalFormat for R8I {
    const ID: u32 = Gl::R8I;
}

unsafe impl PixelUnpack<R8I> for u8 {
    const FORMAT_ID: u32 = Gl::RED_INTEGER;

    const TYPE_ID: u32 = Gl::BYTE;
}

pub struct R16UI;

unsafe impl InternalFormat for R16UI {
    const ID: u32 = Gl::R16UI;
}

unsafe impl PixelUnpack<R16UI> for u16 {
    const FORMAT_ID: u32 = Gl::RED_INTEGER;

    const TYPE_ID: u32 = Gl::UNSIGNED_SHORT;
}

pub struct R16I;

unsafe impl InternalFormat for R16I {
    const ID: u32 = Gl::R16I;
}

unsafe impl PixelUnpack<R16I> for i16 {
    const FORMAT_ID: u32 = Gl::RED_INTEGER;

    const TYPE_ID: u32 = Gl::SHORT;
}

pub struct R32UI;

unsafe impl InternalFormat for R32UI {
    const ID: u32 = Gl::R32UI;
}

unsafe impl PixelUnpack<R32UI> for u32 {
    const FORMAT_ID: u32 = Gl::RED_INTEGER;

    const TYPE_ID: u32 = Gl::UNSIGNED_INT;
}

pub struct R32I;

unsafe impl InternalFormat for R32I {
    const ID: u32 = Gl::R32I;
}

unsafe impl PixelUnpack<R32I> for i32 {
    const FORMAT_ID: u32 = Gl::RED_INTEGER;

    const TYPE_ID: u32 = Gl::INT;
}

pub struct RG8;

unsafe impl InternalFormat for RG8 {
    const ID: u32 = Gl::RG8;
}

unsafe impl PixelUnpack<RG8> for [u8; 2] {
    const FORMAT_ID: u32 = Gl::RG;

    const TYPE_ID: u32 = Gl::UNSIGNED_BYTE;
}

unsafe impl PixelUnpack<RG8> for (u8, u8) {
    const FORMAT_ID: u32 = Gl::RG;

    const TYPE_ID: u32 = Gl::UNSIGNED_BYTE;
}

pub struct RG16F;

unsafe impl InternalFormat for RG16F {
    const ID: u32 = Gl::RG16F;
}

unsafe impl PixelUnpack<RG16F> for [f32; 2] {
    const FORMAT_ID: u32 = Gl::RG;

    const TYPE_ID: u32 = Gl::FLOAT;
}

unsafe impl PixelUnpack<RG16F> for (f32, f32) {
    const FORMAT_ID: u32 = Gl::RG;

    const TYPE_ID: u32 = Gl::FLOAT;
}

pub struct RG32F;

unsafe impl InternalFormat for RG32F {
    const ID: u32 = Gl::RG32F;
}

unsafe impl PixelUnpack<RG32F> for [f32; 2] {
    const FORMAT_ID: u32 = Gl::RG;

    const TYPE_ID: u32 = Gl::FLOAT;
}

unsafe impl PixelUnpack<RG32F> for (f32, f32) {
    const FORMAT_ID: u32 = Gl::RG;

    const TYPE_ID: u32 = Gl::FLOAT;
}

pub struct RG8UI;

unsafe impl InternalFormat for RG8UI {
    const ID: u32 = Gl::RG8UI;
}

unsafe impl PixelUnpack<RG8UI> for [u8; 2] {
    const FORMAT_ID: u32 = Gl::RG_INTEGER;

    const TYPE_ID: u32 = Gl::UNSIGNED_BYTE;
}

unsafe impl PixelUnpack<RG8UI> for (u8, u8) {
    const FORMAT_ID: u32 = Gl::RG_INTEGER;

    const TYPE_ID: u32 = Gl::UNSIGNED_BYTE;
}

pub struct RG8I;

unsafe impl InternalFormat for RG8I {
    const ID: u32 = Gl::RG8I;
}

unsafe impl PixelUnpack<RG8I> for [i8; 2] {
    const FORMAT_ID: u32 = Gl::RG_INTEGER;

    const TYPE_ID: u32 = Gl::BYTE;
}

unsafe impl PixelUnpack<RG8I> for (i8, i8) {
    const FORMAT_ID: u32 = Gl::RG_INTEGER;

    const TYPE_ID: u32 = Gl::BYTE;
}

pub struct RG16UI;

unsafe impl InternalFormat for RG16UI {
    const ID: u32 = Gl::RG16UI;
}

unsafe impl PixelUnpack<RG16UI> for [u16; 2] {
    const FORMAT_ID: u32 = Gl::RG_INTEGER;

    const TYPE_ID: u32 = Gl::UNSIGNED_SHORT;
}

unsafe impl PixelUnpack<RG16UI> for (u16, u16) {
    const FORMAT_ID: u32 = Gl::RG_INTEGER;

    const TYPE_ID: u32 = Gl::UNSIGNED_SHORT;
}

pub struct RG16I;

unsafe impl InternalFormat for RG16I {
    const ID: u32 = Gl::RG16I;
}

unsafe impl PixelUnpack<RG16I> for [i16; 2] {
    const FORMAT_ID: u32 = Gl::RG_INTEGER;

    const TYPE_ID: u32 = Gl::SHORT;
}

unsafe impl PixelUnpack<RG16I> for (i16, i16) {
    const FORMAT_ID: u32 = Gl::RG_INTEGER;

    const TYPE_ID: u32 = Gl::SHORT;
}

pub struct RG32UI;

unsafe impl InternalFormat for RG32UI {
    const ID: u32 = Gl::RG32UI;
}

unsafe impl PixelUnpack<RG32UI> for [u32; 2] {
    const FORMAT_ID: u32 = Gl::RG_INTEGER;

    const TYPE_ID: u32 = Gl::UNSIGNED_INT;
}

unsafe impl PixelUnpack<RG32UI> for (u32, u32) {
    const FORMAT_ID: u32 = Gl::RG_INTEGER;

    const TYPE_ID: u32 = Gl::UNSIGNED_INT;
}

pub struct RG32I;

unsafe impl InternalFormat for RG32I {
    const ID: u32 = Gl::RG32I;
}

unsafe impl PixelUnpack<RG32I> for [i32; 2] {
    const FORMAT_ID: u32 = Gl::RG_INTEGER;

    const TYPE_ID: u32 = Gl::INT;
}

unsafe impl PixelUnpack<RG32I> for (i32, i32) {
    const FORMAT_ID: u32 = Gl::RG_INTEGER;

    const TYPE_ID: u32 = Gl::INT;
}

pub struct RGB8;

unsafe impl InternalFormat for RGB8 {
    const ID: u32 = Gl::RGB8;
}

unsafe impl PixelUnpack<RGB8> for [u8; 3] {
    const FORMAT_ID: u32 = Gl::RGB;

    const TYPE_ID: u32 = Gl::UNSIGNED_BYTE;
}

unsafe impl PixelUnpack<RGB8> for (u8, u8, u8) {
    const FORMAT_ID: u32 = Gl::RGB;

    const TYPE_ID: u32 = Gl::UNSIGNED_BYTE;
}

pub struct SRGB8;

unsafe impl InternalFormat for SRGB8 {
    const ID: u32 = Gl::SRGB8;
}

unsafe impl PixelUnpack<SRGB8> for [u8; 3] {
    const FORMAT_ID: u32 = Gl::RGB;

    const TYPE_ID: u32 = Gl::UNSIGNED_BYTE;
}

unsafe impl PixelUnpack<SRGB8> for (u8, u8, u8) {
    const FORMAT_ID: u32 = Gl::RGB;

    const TYPE_ID: u32 = Gl::UNSIGNED_BYTE;
}

pub struct RGB565;

unsafe impl InternalFormat for RGB565 {
    const ID: u32 = Gl::RGB565;
}

unsafe impl PixelUnpack<RGB565> for [u8; 3] {
    const FORMAT_ID: u32 = Gl::RGB;

    const TYPE_ID: u32 = Gl::UNSIGNED_BYTE;
}

unsafe impl PixelUnpack<RGB565> for (u8, u8, u8) {
    const FORMAT_ID: u32 = Gl::RGB;

    const TYPE_ID: u32 = Gl::UNSIGNED_BYTE;
}

unsafe impl PixelUnpack<RGB565> for u16 {
    const FORMAT_ID: u32 = Gl::RGB;

    const TYPE_ID: u32 = Gl::UNSIGNED_SHORT_5_6_5;
}

#[allow(non_camel_case_types)]
pub struct R11F_G11F_B10F;

unsafe impl InternalFormat for R11F_G11F_B10F {
    const ID: u32 = Gl::R11F_G11F_B10F;
}

unsafe impl PixelUnpack<R11F_G11F_B10F> for u32 {
    const FORMAT_ID: u32 = Gl::RGB;

    const TYPE_ID: u32 = Gl::UNSIGNED_INT_10F_11F_11F_REV;
}

unsafe impl PixelUnpack<R11F_G11F_B10F> for [f32; 3] {
    const FORMAT_ID: u32 = Gl::RGB;

    const TYPE_ID: u32 = Gl::FLOAT;
}

unsafe impl PixelUnpack<R11F_G11F_B10F> for (f32, f32, f32) {
    const FORMAT_ID: u32 = Gl::RGB;

    const TYPE_ID: u32 = Gl::FLOAT;
}

#[allow(non_camel_case_types)]
pub struct RGB9_E5;

unsafe impl InternalFormat for RGB9_E5 {
    const ID: u32 = Gl::RGB9_E5;
}

unsafe impl PixelUnpack<RGB9_E5> for u32 {
    const FORMAT_ID: u32 = Gl::RGB;

    const TYPE_ID: u32 = Gl::UNSIGNED_INT_5_9_9_9_REV;
}

unsafe impl PixelUnpack<RGB9_E5> for f32 {
    const FORMAT_ID: u32 = Gl::RGB;

    const TYPE_ID: u32 = Gl::FLOAT;
}

pub struct RGB16F;

unsafe impl InternalFormat for RGB16F {
    const ID: u32 = Gl::RGB16F;
}

unsafe impl PixelUnpack<RGB16F> for [f32; 3] {
    const FORMAT_ID: u32 = Gl::RGB;

    const TYPE_ID: u32 = Gl::FLOAT;
}

unsafe impl PixelUnpack<RGB16F> for (f32, f32, f32) {
    const FORMAT_ID: u32 = Gl::RGB;

    const TYPE_ID: u32 = Gl::FLOAT;
}

pub struct RGB32F;

unsafe impl InternalFormat for RGB32F {
    const ID: u32 = Gl::RGB32F;
}

unsafe impl PixelUnpack<RGB32F> for [f32; 3] {
    const FORMAT_ID: u32 = Gl::RGB;

    const TYPE_ID: u32 = Gl::FLOAT;
}

unsafe impl PixelUnpack<RGB32F> for (f32, f32, f32) {
    const FORMAT_ID: u32 = Gl::RGB;

    const TYPE_ID: u32 = Gl::FLOAT;
}

pub struct RGB8UI;

unsafe impl InternalFormat for RGB8UI {
    const ID: u32 = Gl::RGB8UI;
}

unsafe impl PixelUnpack<RGB8UI> for [u8; 3] {
    const FORMAT_ID: u32 = Gl::RGB_INTEGER;

    const TYPE_ID: u32 = Gl::UNSIGNED_BYTE;
}

unsafe impl PixelUnpack<RGB8UI> for (u8, u8, u8) {
    const FORMAT_ID: u32 = Gl::RGB_INTEGER;

    const TYPE_ID: u32 = Gl::UNSIGNED_BYTE;
}

pub struct RGB8I;

unsafe impl InternalFormat for RGB8I {
    const ID: u32 = Gl::RGB8I;
}

unsafe impl PixelUnpack<RGB8I> for [i8; 3] {
    const FORMAT_ID: u32 = Gl::RGB_INTEGER;

    const TYPE_ID: u32 = Gl::BYTE;
}

unsafe impl PixelUnpack<RGB8I> for (i8, i8, i8) {
    const FORMAT_ID: u32 = Gl::RGB_INTEGER;

    const TYPE_ID: u32 = Gl::BYTE;
}

pub struct RGB16UI;

unsafe impl InternalFormat for RGB16UI {
    const ID: u32 = Gl::RGB16UI;
}

unsafe impl PixelUnpack<RGB16UI> for [u16; 3] {
    const FORMAT_ID: u32 = Gl::RGB_INTEGER;

    const TYPE_ID: u32 = Gl::UNSIGNED_SHORT;
}

unsafe impl PixelUnpack<RGB16UI> for (u16, u16, u16) {
    const FORMAT_ID: u32 = Gl::RGB_INTEGER;

    const TYPE_ID: u32 = Gl::UNSIGNED_SHORT;
}

pub struct RGB16I;

unsafe impl InternalFormat for RGB16I {
    const ID: u32 = Gl::RGB16I;
}

unsafe impl PixelUnpack<RGB16I> for [i16; 3] {
    const FORMAT_ID: u32 = Gl::RGB_INTEGER;

    const TYPE_ID: u32 = Gl::SHORT;
}

unsafe impl PixelUnpack<RGB16I> for (i16, i16, i16) {
    const FORMAT_ID: u32 = Gl::RGB_INTEGER;

    const TYPE_ID: u32 = Gl::SHORT;
}

pub struct RGB32UI;

unsafe impl InternalFormat for RGB32UI {
    const ID: u32 = Gl::RGB32UI;
}

unsafe impl PixelUnpack<RGB32UI> for [u32; 3] {
    const FORMAT_ID: u32 = Gl::RGB_INTEGER;

    const TYPE_ID: u32 = Gl::UNSIGNED_INT;
}

unsafe impl PixelUnpack<RGB32UI> for (u32, u32, u32) {
    const FORMAT_ID: u32 = Gl::RGB_INTEGER;

    const TYPE_ID: u32 = Gl::UNSIGNED_INT;
}

pub struct RGB32I;

unsafe impl InternalFormat for RGB32I {
    const ID: u32 = Gl::RGB32I;
}

unsafe impl PixelUnpack<RGB32I> for [i32; 3] {
    const FORMAT_ID: u32 = Gl::RGB_INTEGER;

    const TYPE_ID: u32 = Gl::INT;
}

unsafe impl PixelUnpack<RGB32I> for (i32, i32, i32) {
    const FORMAT_ID: u32 = Gl::RGB_INTEGER;

    const TYPE_ID: u32 = Gl::INT;
}

pub struct RGBA8;

unsafe impl InternalFormat for RGBA8 {
    const ID: u32 = Gl::RGBA8;
}

unsafe impl PixelUnpack<RGBA8> for [u8; 4] {
    const FORMAT_ID: u32 = Gl::RGBA;

    const TYPE_ID: u32 = Gl::UNSIGNED_BYTE;
}

unsafe impl PixelUnpack<RGBA8> for (u8, u8, u8, u8) {
    const FORMAT_ID: u32 = Gl::RGBA;

    const TYPE_ID: u32 = Gl::UNSIGNED_BYTE;
}

#[allow(non_camel_case_types)]
pub struct SRGB8_ALPHA8;

unsafe impl InternalFormat for SRGB8_ALPHA8 {
    const ID: u32 = Gl::SRGB8_ALPHA8;
}

unsafe impl PixelUnpack<SRGB8_ALPHA8> for [u8; 4] {
    const FORMAT_ID: u32 = Gl::RGBA;

    const TYPE_ID: u32 = Gl::UNSIGNED_BYTE;
}

unsafe impl PixelUnpack<SRGB8_ALPHA8> for (u8, u8, u8, u8) {
    const FORMAT_ID: u32 = Gl::RGBA;

    const TYPE_ID: u32 = Gl::UNSIGNED_BYTE;
}

#[allow(non_camel_case_types)]
pub struct RGB5_A1;

unsafe impl InternalFormat for RGB5_A1 {
    const ID: u32 = Gl::RGB5_A1;
}

unsafe impl PixelUnpack<RGB5_A1> for [u8; 4] {
    const FORMAT_ID: u32 = Gl::RGBA;

    const TYPE_ID: u32 = Gl::UNSIGNED_BYTE;
}

unsafe impl PixelUnpack<RGB5_A1> for (u8, u8, u8, u8) {
    const FORMAT_ID: u32 = Gl::RGBA;

    const TYPE_ID: u32 = Gl::UNSIGNED_BYTE;
}

unsafe impl PixelUnpack<RGB5_A1> for u16 {
    const FORMAT_ID: u32 = Gl::RGBA;

    const TYPE_ID: u32 = Gl::UNSIGNED_SHORT_5_5_5_1;
}

unsafe impl PixelUnpack<RGB5_A1> for u32 {
    const FORMAT_ID: u32 = Gl::RGBA;

    const TYPE_ID: u32 = Gl::UNSIGNED_INT_2_10_10_10_REV;
}

pub struct RGBA4;

unsafe impl InternalFormat for RGBA4 {
    const ID: u32 = Gl::RGBA4;
}

unsafe impl PixelUnpack<RGBA4> for [u8; 4] {
    const FORMAT_ID: u32 = Gl::RGBA;

    const TYPE_ID: u32 = Gl::UNSIGNED_BYTE;
}

unsafe impl PixelUnpack<RGBA4> for (u8, u8, u8, u8) {
    const FORMAT_ID: u32 = Gl::RGBA;

    const TYPE_ID: u32 = Gl::UNSIGNED_BYTE;
}

unsafe impl PixelUnpack<RGBA4> for u16 {
    const FORMAT_ID: u32 = Gl::RGBA;

    const TYPE_ID: u32 = Gl::UNSIGNED_SHORT_4_4_4_4;
}

#[allow(non_camel_case_types)]
pub struct RGB10_A2;

unsafe impl InternalFormat for RGB10_A2 {
    const ID: u32 = Gl::RGB10_A2;
}

unsafe impl PixelUnpack<RGB10_A2> for u32 {
    const FORMAT_ID: u32 = Gl::RGBA;

    const TYPE_ID: u32 = Gl::UNSIGNED_INT_2_10_10_10_REV;
}

#[allow(non_camel_case_types)]
pub struct RGB10_A2UI;

unsafe impl InternalFormat for RGB10_A2UI {
    const ID: u32 = Gl::RGB10_A2UI;
}

unsafe impl PixelUnpack<RGB10_A2UI> for u32 {
    const FORMAT_ID: u32 = Gl::RGBA;

    const TYPE_ID: u32 = Gl::UNSIGNED_INT_2_10_10_10_REV;
}

pub struct RGBA16F;

unsafe impl InternalFormat for RGBA16F {
    const ID: u32 = Gl::RGBA16F;
}

unsafe impl PixelUnpack<RGBA16F> for [f32; 4] {
    const FORMAT_ID: u32 = Gl::RGBA;

    const TYPE_ID: u32 = Gl::FLOAT;
}

unsafe impl PixelUnpack<RGBA16F> for (f32, f32, f32, f32) {
    const FORMAT_ID: u32 = Gl::RGBA;

    const TYPE_ID: u32 = Gl::FLOAT;
}

pub struct RGBA32F;

unsafe impl InternalFormat for RGBA32F {
    const ID: u32 = Gl::RGBA32F;
}

unsafe impl PixelUnpack<RGBA32F> for [f32; 4] {
    const FORMAT_ID: u32 = Gl::RGBA;

    const TYPE_ID: u32 = Gl::FLOAT;
}

unsafe impl PixelUnpack<RGBA32F> for (f32, f32, f32, f32) {
    const FORMAT_ID: u32 = Gl::RGBA;

    const TYPE_ID: u32 = Gl::FLOAT;
}

pub struct RGBA8UI;

unsafe impl InternalFormat for RGBA8UI {
    const ID: u32 = Gl::RGBA8UI;
}

unsafe impl PixelUnpack<RGBA8UI> for [u8; 4] {
    const FORMAT_ID: u32 = Gl::RGBA_INTEGER;

    const TYPE_ID: u32 = Gl::UNSIGNED_BYTE;
}

unsafe impl PixelUnpack<RGBA8UI> for (u8, u8, u8, u8) {
    const FORMAT_ID: u32 = Gl::RGBA_INTEGER;

    const TYPE_ID: u32 = Gl::UNSIGNED_BYTE;
}

pub struct RGBA8I;

unsafe impl InternalFormat for RGBA8I {
    const ID: u32 = Gl::RGBA8I;
}

unsafe impl PixelUnpack<RGBA8I> for [i8; 4] {
    const FORMAT_ID: u32 = Gl::RGBA_INTEGER;

    const TYPE_ID: u32 = Gl::BYTE;
}

unsafe impl PixelUnpack<RGBA8I> for (i8, i8, i8, i8) {
    const FORMAT_ID: u32 = Gl::RGBA_INTEGER;

    const TYPE_ID: u32 = Gl::BYTE;
}

pub struct RGBA16UI;

unsafe impl InternalFormat for RGBA16UI {
    const ID: u32 = Gl::RGBA16UI;
}

unsafe impl PixelUnpack<RGBA16UI> for [u16; 4] {
    const FORMAT_ID: u32 = Gl::RGBA_INTEGER;

    const TYPE_ID: u32 = Gl::UNSIGNED_SHORT;
}

unsafe impl PixelUnpack<RGBA16UI> for (u16, u16, u16, u16) {
    const FORMAT_ID: u32 = Gl::RGBA_INTEGER;

    const TYPE_ID: u32 = Gl::UNSIGNED_SHORT;
}

pub struct RGBA16I;

unsafe impl InternalFormat for RGBA16I {
    const ID: u32 = Gl::RGBA16I;
}

unsafe impl PixelUnpack<RGBA16I> for [i16; 4] {
    const FORMAT_ID: u32 = Gl::RGBA_INTEGER;

    const TYPE_ID: u32 = Gl::SHORT;
}

unsafe impl PixelUnpack<RGBA16I> for (i16, i16, i16, i16) {
    const FORMAT_ID: u32 = Gl::RGBA_INTEGER;

    const TYPE_ID: u32 = Gl::SHORT;
}

pub struct RGBA32UI;

unsafe impl InternalFormat for RGBA32UI {
    const ID: u32 = Gl::RGBA32UI;
}

unsafe impl PixelUnpack<RGBA32UI> for [u32; 4] {
    const FORMAT_ID: u32 = Gl::RGBA_INTEGER;

    const TYPE_ID: u32 = Gl::UNSIGNED_INT;
}

unsafe impl PixelUnpack<RGBA32UI> for (u32, u32, u32, u32) {
    const FORMAT_ID: u32 = Gl::RGBA_INTEGER;

    const TYPE_ID: u32 = Gl::UNSIGNED_INT;
}

pub struct RGBA32I;

unsafe impl InternalFormat for RGBA32I {
    const ID: u32 = Gl::RGBA32I;
}

unsafe impl PixelUnpack<RGBA32I> for [i32; 4] {
    const FORMAT_ID: u32 = Gl::RGBA_INTEGER;

    const TYPE_ID: u32 = Gl::INT;
}

unsafe impl PixelUnpack<RGBA32I> for (i32, i32, i32, i32) {
    const FORMAT_ID: u32 = Gl::RGBA_INTEGER;

    const TYPE_ID: u32 = Gl::INT;
}

pub struct DepthComponent16;

unsafe impl InternalFormat for DepthComponent16 {
    const ID: u32 = Gl::DEPTH_COMPONENT16;
}

unsafe impl PixelUnpack<DepthComponent16> for u16 {
    const FORMAT_ID: u32 = Gl::DEPTH_COMPONENT;

    const TYPE_ID: u32 = Gl::UNSIGNED_SHORT;
}

unsafe impl PixelUnpack<DepthComponent16> for u32 {
    const FORMAT_ID: u32 = Gl::DEPTH_COMPONENT;

    const TYPE_ID: u32 = Gl::UNSIGNED_INT;
}

pub struct DepthComponent24;

unsafe impl InternalFormat for DepthComponent24 {
    const ID: u32 = Gl::DEPTH_COMPONENT24;
}

unsafe impl PixelUnpack<DepthComponent24> for u32 {
    const FORMAT_ID: u32 = Gl::DEPTH_COMPONENT;

    const TYPE_ID: u32 = Gl::UNSIGNED_INT;
}

pub struct DepthComponent32F;

unsafe impl InternalFormat for DepthComponent32F {
    const ID: u32 = Gl::DEPTH_COMPONENT32F;
}

unsafe impl PixelUnpack<DepthComponent32F> for f32 {
    const FORMAT_ID: u32 = Gl::DEPTH_COMPONENT;

    const TYPE_ID: u32 = Gl::FLOAT;
}

pub struct StencilIndex8;

unsafe impl InternalFormat for StencilIndex8 {
    const ID: u32 = Gl::STENCIL_INDEX8;
}

pub struct Depth24Stencil8;

unsafe impl InternalFormat for Depth24Stencil8 {
    const ID: u32 = Gl::DEPTH24_STENCIL8;
}

unsafe impl PixelUnpack<Depth24Stencil8> for u32 {
    const FORMAT_ID: u32 = Gl::DEPTH_STENCIL;

    const TYPE_ID: u32 = Gl::UNSIGNED_INT_24_8;
}

pub struct Depth32FStencil8;

unsafe impl InternalFormat for Depth32FStencil8 {
    const ID: u32 = Gl::DEPTH32F_STENCIL8;
}

pub struct Luminance;

unsafe impl InternalFormat for Luminance {
    const ID: u32 = Gl::LUMINANCE;
}

unsafe impl PixelUnpack<Luminance> for u8 {
    const FORMAT_ID: u32 = Gl::LUMINANCE;

    const TYPE_ID: u32 = Gl::UNSIGNED_BYTE;
}

pub struct LuminanceAlpha;

unsafe impl InternalFormat for LuminanceAlpha {
    const ID: u32 = Gl::LUMINANCE_ALPHA;
}

unsafe impl PixelUnpack<LuminanceAlpha> for [u8; 2] {
    const FORMAT_ID: u32 = Gl::LUMINANCE_ALPHA;

    const TYPE_ID: u32 = Gl::UNSIGNED_BYTE;
}

unsafe impl PixelUnpack<LuminanceAlpha> for (u8, u8) {
    const FORMAT_ID: u32 = Gl::LUMINANCE_ALPHA;

    const TYPE_ID: u32 = Gl::UNSIGNED_BYTE;
}

pub struct Alpha;

unsafe impl InternalFormat for Alpha {
    const ID: u32 = Gl::ALPHA;
}

unsafe impl PixelUnpack<Alpha> for u8 {
    const FORMAT_ID: u32 = Gl::ALPHA;

    const TYPE_ID: u32 = Gl::UNSIGNED_BYTE;
}
