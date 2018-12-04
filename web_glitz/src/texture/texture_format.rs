use crate::image_format::*;

pub unsafe trait TextureFormat: InternalFormat {}

unsafe impl TextureFormat for R8 {}
unsafe impl TextureFormat for R16F {}
unsafe impl TextureFormat for R32F {}
unsafe impl TextureFormat for R8UI {}
unsafe impl TextureFormat for RG8 {}
unsafe impl TextureFormat for RG16F {}
unsafe impl TextureFormat for RG32F {}
unsafe impl TextureFormat for RG8UI {}
unsafe impl TextureFormat for RGB8 {}
unsafe impl TextureFormat for SRGB8 {}
unsafe impl TextureFormat for RGB565 {}
unsafe impl TextureFormat for R11F_G11F_B10F {}
unsafe impl TextureFormat for RGB9_E5 {}
unsafe impl TextureFormat for RGB16F {}
unsafe impl TextureFormat for RGB32F {}
unsafe impl TextureFormat for RGB8UI {}
unsafe impl TextureFormat for RGBA8 {}
unsafe impl TextureFormat for SRGB8_ALPHA8 {}
unsafe impl TextureFormat for RGB5_A1 {}
unsafe impl TextureFormat for RGBA4 {}
unsafe impl TextureFormat for RGB10_A2 {}
unsafe impl TextureFormat for RGBA16F {}
unsafe impl TextureFormat for RGBA32F {}
unsafe impl TextureFormat for RGBA8UI {}
unsafe impl TextureFormat for DepthComponent16 {}
unsafe impl TextureFormat for DepthComponent24 {}
unsafe impl TextureFormat for DepthComponent32F {}
unsafe impl TextureFormat for Depth24Stencil8 {}
unsafe impl TextureFormat for Depth32FStencil8 {}
unsafe impl TextureFormat for Luminance {}
unsafe impl TextureFormat for LuminanceAlpha {}
