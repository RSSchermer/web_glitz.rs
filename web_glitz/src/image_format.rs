use web_sys::WebGl2RenderingContext as Gl;

pub unsafe trait InternalFormat {
    fn id() -> u32;
}

pub unsafe trait ClientFormat<T> where T: InternalFormat {
    fn id() -> u32;
}

pub unsafe trait ColorRenderable: InternalFormat {}

unsafe impl ColorRenderable for R8 {}
unsafe impl ColorRenderable for R8UI {}
unsafe impl ColorRenderable for R8I {}
unsafe impl ColorRenderable for R16UI {}
unsafe impl ColorRenderable for R16I {}
unsafe impl ColorRenderable for R32UI {}
unsafe impl ColorRenderable for R32I {}
unsafe impl ColorRenderable for RG8 {}
unsafe impl ColorRenderable for RG8UI {}
unsafe impl ColorRenderable for RG8I {}
unsafe impl ColorRenderable for RG16UI {}
unsafe impl ColorRenderable for RG16I {}
unsafe impl ColorRenderable for RG32UI {}
unsafe impl ColorRenderable for RG32I {}
unsafe impl ColorRenderable for RGB8 {}
unsafe impl ColorRenderable for RGBA8 {}
unsafe impl ColorRenderable for SRGB8_ALPHA8 {}
unsafe impl ColorRenderable for RGBA4 {}
unsafe impl ColorRenderable for RGB565 {}
unsafe impl ColorRenderable for RGB5_A1 {}
unsafe impl ColorRenderable for RGB10_A2 {}
unsafe impl ColorRenderable for RGBA8UI {}
unsafe impl ColorRenderable for RGBA8I {}
unsafe impl ColorRenderable for RGB10_A2UI {}
unsafe impl ColorRenderable for RGBA16UI {}
unsafe impl ColorRenderable for RGBA16I {}
unsafe impl ColorRenderable for RGBA32I {}
unsafe impl ColorRenderable for RGBA32UI {}

pub unsafe trait DepthRenderable: InternalFormat {}

unsafe impl DepthRenderable for DepthComponent16 {}
unsafe impl DepthRenderable for DepthComponent24 {}
unsafe impl DepthRenderable for DepthComponent32F {}
unsafe impl DepthRenderable for Depth24Stencil8 {}
unsafe impl DepthRenderable for Depth32FStencil8 {}

pub unsafe trait StencilRenderable: InternalFormat {}

unsafe impl StencilRenderable for StencilIndex8 {}
unsafe impl StencilRenderable for Depth24Stencil8 {}
unsafe impl StencilRenderable for Depth32FStencil8 {}

pub unsafe trait CopyCompatible<F> where F: InternalFormat {}

// TODO implement CopyCompatible for formats: copyTexSubImage requires the target format to contain
// a subset of the information

pub struct R8;

unsafe impl InternalFormat for R8 {
    fn id() -> u32 {
        Gl::R8
    }
}

unsafe impl ClientFormat<R8> for u8 {
    fn id() -> u32 {
        Gl::UNSIGNED_BYTE
    }
}

pub struct R16F;

unsafe impl InternalFormat for R16F {
    fn id() -> u32 {
        Gl::R16F
    }
}

unsafe impl ClientFormat<R16F> for f32 {
    fn id() -> u32 {
        Gl::FLOAT
    }
}

pub struct R32F;

unsafe impl InternalFormat for R32F {
    fn id() -> u32 {
        Gl::R32F
    }
}

unsafe impl ClientFormat<R32F> for f32 {
    fn id() -> u32 {
        Gl::FLOAT
    }
}

pub struct R8UI;

unsafe impl InternalFormat for R8UI {
    fn id() -> u32 {
        Gl::R8UI
    }
}

unsafe impl ClientFormat<R8UI> for u8 {
    fn id() -> u32 {
        Gl::UNSIGNED_BYTE
    }
}

pub struct R8I;

unsafe impl InternalFormat for R8I {
    fn id() -> u32 {
        Gl::R8I
    }
}

unsafe impl ClientFormat<R8I> for u8 {
    fn id() -> u32 {
        Gl::BYTE
    }
}

pub struct R16UI;

unsafe impl InternalFormat for R16UI {
    fn id() -> u32 {
        Gl::R16UI
    }
}

unsafe impl ClientFormat<R16UI> for u16 {
    fn id() -> u32 {
        Gl::UNSIGNED_SHORT
    }
}

pub struct R16I;

unsafe impl InternalFormat for R16I {
    fn id() -> u32 {
        Gl::R16I
    }
}

unsafe impl ClientFormat<R16I> for i16 {
    fn id() -> u32 {
        Gl::SHORT
    }
}

pub struct R32UI;

unsafe impl InternalFormat for R32UI {
    fn id() -> u32 {
        Gl::R32UI
    }
}

unsafe impl ClientFormat<R32UI> for u32 {
    fn id() -> u32 {
        Gl::UNSIGNED_INT
    }
}

pub struct R32I;

unsafe impl InternalFormat for R32I {
    fn id() -> u32 {
        Gl::R32I
    }
}

unsafe impl ClientFormat<R32I> for i32 {
    fn id() -> u32 {
        Gl::INT
    }
}

pub struct RG8;

unsafe impl InternalFormat for RG8 {
    fn id() -> u32 {
        Gl::RG8
    }
}

unsafe impl ClientFormat<RG8> for (u8, u8) {
    fn id() -> u32 {
        Gl::UNSIGNED_BYTE
    }
}

pub struct RG16F;

unsafe impl InternalFormat for RG16F {
    fn id() -> u32 {
        Gl::RG16F
    }
}

unsafe impl ClientFormat<RG16F> for (f32, f32) {
    fn id() -> u32 {
        Gl::FLOAT
    }
}

pub struct RG32F;

unsafe impl InternalFormat for RG32F {
    fn id() -> u32 {
        Gl::RG32F
    }
}

unsafe impl ClientFormat<RG32F> for (f32, f32) {
    fn id() -> u32 {
        Gl::FLOAT
    }
}

pub struct RG8UI;

unsafe impl InternalFormat for RG8UI {
    fn id() -> u32 {
        Gl::RG8UI
    }
}

unsafe impl ClientFormat<RG8UI> for (u8, u8) {
    fn id() -> u32 {
        Gl::UNSIGNED_BYTE
    }
}

pub struct RG8I;

unsafe impl InternalFormat for RG8I {
    fn id() -> u32 {
        Gl::RG8I
    }
}

unsafe impl ClientFormat<RG8I> for (i8, i8) {
    fn id() -> u32 {
        Gl::BYTE
    }
}

pub struct RG16UI;

unsafe impl InternalFormat for RG16UI {
    fn id() -> u32 {
        Gl::RG16UI
    }
}

unsafe impl ClientFormat<RG16UI> for (u16, u16) {
    fn id() -> u32 {
        Gl::UNSIGNED_SHORT
    }
}

pub struct RG16I;

unsafe impl InternalFormat for RG16I {
    fn id() -> u32 {
        Gl::RG16I
    }
}

unsafe impl ClientFormat<RG16I> for (i16, i16) {
    fn id() -> u32 {
        Gl::SHORT
    }
}

pub struct RG32UI;

unsafe impl InternalFormat for RG32UI {
    fn id() -> u32 {
        Gl::RG32UI
    }
}

unsafe impl ClientFormat<RG32UI> for (u32, u32) {
    fn id() -> u32 {
        Gl::UNSIGNED_INT
    }
}

pub struct RG32I;

unsafe impl InternalFormat for RG32I {
    fn id() -> u32 {
        Gl::RG32I
    }
}

unsafe impl ClientFormat<RG32I> for (i32, i32) {
    fn id() -> u32 {
        Gl::INT
    }
}

pub struct RGB8;

unsafe impl InternalFormat for RGB8 {
    fn id() -> u32 {
        Gl::RGB8
    }
}

unsafe impl ClientFormat<RGB8> for (u8, u8, u8) {
    fn id() -> u32 {
        Gl::UNSIGNED_BYTE
    }
}

pub struct SRGB8;

unsafe impl InternalFormat for SRGB8 {
    fn id() -> u32 {
        Gl::SRGB8
    }
}

unsafe impl ClientFormat<SRGB8> for (u8, u8, u8) {
    fn id() -> u32 {
        Gl::UNSIGNED_BYTE
    }
}

pub struct RGB565;

unsafe impl InternalFormat for RGB565 {
    fn id() -> u32 {
        Gl::RGB565
    }
}

unsafe impl ClientFormat<RGB565> for (u8, u8, u8) {
    fn id() -> u32 {
        Gl::UNSIGNED_BYTE
    }
}

unsafe impl ClientFormat<RGB565> for u16 {
    fn id() -> u32 {
        Gl::UNSIGNED_SHORT_5_6_5
    }
}

#[allow(non_camel_case_types)]
pub struct R11F_G11F_B10F;

unsafe impl InternalFormat for R11F_G11F_B10F {
    fn id() -> u32 {
        Gl::R11F_G11F_B10F
    }
}

unsafe impl ClientFormat<R11F_G11F_B10F> for u32 {
    fn id() -> u32 {
        Gl::UNSIGNED_INT_10F_11F_11F_REV
    }
}

unsafe impl ClientFormat<R11F_G11F_B10F> for (f32, f32, f32) {
    fn id() -> u32 {
        Gl::FLOAT
    }
}

#[allow(non_camel_case_types)]
pub struct RGB9_E5;

unsafe impl InternalFormat for RGB9_E5 {
    fn id() -> u32 {
        Gl::RGB9_E5
    }
}

unsafe impl ClientFormat<RGB9_E5> for u32 {
    fn id() -> u32 {
        Gl::UNSIGNED_INT_5_9_9_9_REV
    }
}

unsafe impl ClientFormat<RGB9_E5> for f32 {
    fn id() -> u32 {
        Gl::FLOAT
    }
}

pub struct RGB16F;

unsafe impl InternalFormat for RGB16F {
    fn id() -> u32 {
        Gl::RGB16F
    }
}

unsafe impl ClientFormat<RGB16F> for (f32, f32, f32) {
    fn id() -> u32 {
        Gl::FLOAT
    }
}

pub struct RGB32F;

unsafe impl InternalFormat for RGB32F {
    fn id() -> u32 {
        Gl::RGB32F
    }
}

unsafe impl ClientFormat<RGB32F> for (f32, f32, f32) {
    fn id() -> u32 {
        Gl::FLOAT
    }
}

pub struct RGB8UI;

unsafe impl InternalFormat for RGB8UI {
    fn id() -> u32 {
        Gl::RGB8UI
    }
}

unsafe impl ClientFormat<RGB8UI> for (u8, u8, u8) {
    fn id() -> u32 {
        Gl::FLOAT
    }
}

pub struct RGBA8;

unsafe impl InternalFormat for RGBA8 {
    fn id() -> u32 {
        Gl::RGBA8
    }
}

unsafe impl ClientFormat<RGBA8> for (u8, u8, u8, u8) {
    fn id() -> u32 {
        Gl::UNSIGNED_BYTE
    }
}

#[allow(non_camel_case_types)]
pub struct SRGB8_ALPHA8;

unsafe impl InternalFormat for SRGB8_ALPHA8 {
    fn id() -> u32 {
        Gl::SRGB8_ALPHA8
    }
}

unsafe impl ClientFormat<SRGB8_ALPHA8> for (u8, u8, u8, u8) {
    fn id() -> u32 {
        Gl::UNSIGNED_BYTE
    }
}

#[allow(non_camel_case_types)]
pub struct RGB5_A1;

unsafe impl InternalFormat for RGB5_A1 {
    fn id() -> u32 {
        Gl::RGB5_A1
    }
}

unsafe impl ClientFormat<RGB5_A1> for (u8, u8, u8, u8) {
    fn id() -> u32 {
        Gl::UNSIGNED_BYTE
    }
}

unsafe impl ClientFormat<RGB5_A1> for u16 {
    fn id() -> u32 {
        Gl::UNSIGNED_SHORT_5_5_5_1
    }
}

unsafe impl ClientFormat<RGB5_A1> for u32 {
    fn id() -> u32 {
        Gl::UNSIGNED_INT_2_10_10_10_REV
    }
}

pub struct RGBA4;

unsafe impl InternalFormat for RGBA4 {
    fn id() -> u32 {
        Gl::RGBA4
    }
}

unsafe impl ClientFormat<RGBA4> for (u8, u8, u8, u8) {
    fn id() -> u32 {
        Gl::UNSIGNED_BYTE
    }
}

unsafe impl ClientFormat<RGBA4> for u16 {
    fn id() -> u32 {
        Gl::UNSIGNED_SHORT_4_4_4_4
    }
}

#[allow(non_camel_case_types)]
pub struct RGB10_A2;

unsafe impl InternalFormat for RGB10_A2 {
    fn id() -> u32 {
        Gl::RGB10_A2
    }
}

unsafe impl ClientFormat<RGB10_A2> for u32 {
    fn id() -> u32 {
        Gl::UNSIGNED_INT_2_10_10_10_REV
    }
}

#[allow(non_camel_case_types)]
pub struct RGB10_A2UI;

unsafe impl InternalFormat for RGB10_A2UI {
    fn id() -> u32 {
        Gl::RGB10_A2UI
    }
}

unsafe impl ClientFormat<RGB10_A2UI> for u32 {
    fn id() -> u32 {
        Gl::UNSIGNED_INT_2_10_10_10_REV
    }
}

pub struct RGBA16F;

unsafe impl InternalFormat for RGBA16F {
    fn id() -> u32 {
        Gl::RGBA16F
    }
}

unsafe impl ClientFormat<RGBA16F> for (f32, f32, f32, f32) {
    fn id() -> u32 {
        Gl::FLOAT
    }
}

pub struct RGBA32F;

unsafe impl InternalFormat for RGBA32F {
    fn id() -> u32 {
        Gl::RGBA32F
    }
}

unsafe impl ClientFormat<RGBA32F> for (f32, f32, f32, f32) {
    fn id() -> u32 {
        Gl::FLOAT
    }
}

pub struct RGBA8UI;

unsafe impl InternalFormat for RGBA8UI {
    fn id() -> u32 {
        Gl::RGBA8UI
    }
}

unsafe impl ClientFormat<RGBA8UI> for (u8, u8, u8, u8) {
    fn id() -> u32 {
        Gl::UNSIGNED_BYTE
    }
}

pub struct RGBA8I;

unsafe impl InternalFormat for RGBA8I {
    fn id() -> u32 {
        Gl::RGBA8I
    }
}

unsafe impl ClientFormat<RGBA8I> for (i8, i8, i8, i8) {
    fn id() -> u32 {
        Gl::BYTE
    }
}

pub struct RGBA16UI;

unsafe impl InternalFormat for RGBA16UI {
    fn id() -> u32 {
        Gl::RGBA16UI
    }
}

unsafe impl ClientFormat<RGBA16UI> for (u16, u16, u16, u16) {
    fn id() -> u32 {
        Gl::UNSIGNED_SHORT
    }
}

pub struct RGBA16I;

unsafe impl InternalFormat for RGBA16I {
    fn id() -> u32 {
        Gl::RGBA16I
    }
}

unsafe impl ClientFormat<RGBA16I> for (i16, i16, i16, i16) {
    fn id() -> u32 {
        Gl::SHORT
    }
}

pub struct RGBA32UI;

unsafe impl InternalFormat for RGBA32UI {
    fn id() -> u32 {
        Gl::RGBA32UI
    }
}

unsafe impl ClientFormat<RGBA32UI> for (u32, u32, u32, u32) {
    fn id() -> u32 {
        Gl::UNSIGNED_INT
    }
}

pub struct RGBA32I;

unsafe impl InternalFormat for RGBA32I {
    fn id() -> u32 {
        Gl::RGBA32I
    }
}

unsafe impl ClientFormat<RGBA32I> for (i32, i32, i32, i32) {
    fn id() -> u32 {
        Gl::INT
    }
}

pub struct DepthComponent16;

unsafe impl InternalFormat for DepthComponent16 {
    fn id() -> u32 {
        Gl::DEPTH_COMPONENT16
    }
}

unsafe impl ClientFormat<DepthComponent16> for u16 {
    fn id() -> u32 {
        Gl::UNSIGNED_SHORT
    }
}

unsafe impl ClientFormat<DepthComponent16> for u32 {
    fn id() -> u32 {
        Gl::UNSIGNED_INT
    }
}

pub struct DepthComponent24;

unsafe impl InternalFormat for DepthComponent24 {
    fn id() -> u32 {
        Gl::DEPTH_COMPONENT24
    }
}

unsafe impl ClientFormat<DepthComponent24> for u32 {
    fn id() -> u32 {
        Gl::UNSIGNED_INT
    }
}

pub struct DepthComponent32F;

unsafe impl InternalFormat for DepthComponent32F {
    fn id() -> u32 {
        Gl::DEPTH_COMPONENT32F
    }
}

unsafe impl ClientFormat<DepthComponent32F> for f32 {
    fn id() -> u32 {
        Gl::FLOAT
    }
}

pub struct StencilIndex8;

unsafe impl InternalFormat for StencilIndex8 {
    fn id() -> u32 {
        Gl::STENCIL_INDEX8
    }
}

pub struct Depth24Stencil8;

unsafe impl InternalFormat for Depth24Stencil8 {
    fn id() -> u32 {
        Gl::DEPTH24_STENCIL8
    }
}

unsafe impl ClientFormat<Depth24Stencil8> for u32 {
    fn id() -> u32 {
        Gl::UNSIGNED_INT_24_8
    }
}

pub struct Depth32FStencil8;

unsafe impl InternalFormat for Depth32FStencil8 {
    fn id() -> u32 {
        Gl::DEPTH32F_STENCIL8
    }
}

pub struct Luminance;

unsafe impl InternalFormat for Luminance {
    fn id() -> u32 {
        Gl::LUMINANCE
    }
}

unsafe impl ClientFormat<Luminance> for u8 {
    fn id() -> u32 {
        Gl::UNSIGNED_BYTE
    }
}

pub struct LuminanceAlpha;

unsafe impl InternalFormat for LuminanceAlpha {
    fn id() -> u32 {
        Gl::LUMINANCE_ALPHA
    }
}

unsafe impl ClientFormat<LuminanceAlpha> for (u8, u8) {
    fn id() -> u32 {
        Gl::UNSIGNED_BYTE
    }
}

pub struct Alpha;

unsafe impl InternalFormat for Alpha {
    fn id() -> u32 {
        Gl::ALPHA
    }
}

unsafe impl ClientFormat<Alpha> for u8 {
    fn id() -> u32 {
        Gl::UNSIGNED_BYTE
    }
}
