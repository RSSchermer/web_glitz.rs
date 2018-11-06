use std::cmp;

fn max_mipmap_level(width: u32, height: u32) -> u8 {
    1 + cmp::max(width, height).log2()
}

pub trait Texture {
    fn width(&self) -> u32;

    fn height(&self) -> u32;

    fn depth(&self) -> u32;

    fn internal_format(&self) -> PixelFormat;
}

pub trait Texture2D<Rc>: Texture {
    fn allocate_no_mipmap(width: u32, height: u32, internal_format: TextureFormat) -> AllocateTexture2DCommand<Rc> {
        AllocateTexture2DCommand::no_mipmap(width, height, internal_format)
    }

    fn mipmap_all(base_level_width: u32, base_level_height: u32, internal_format: TextureFormat) -> AllocateTexture2DCommand<Rc> {
        AllocateTexture2DCommand::mipmap_all(base_level_width, base_level_height, internal_format)
    }

    fn mipmap_levels(mipmap_levels: u8, base_level_width: u32, base_level_height: u32, internal_format: TextureFormat) -> Result<AllocateTexture2DCommand<Rc>, MipmapLevelsError> {
        AllocateTexture2DCommand::mipmap_levels(mipmap_levels, base_level_width, base_level_height, internal_format)
    }
}

pub struct AllocateTexture2DCommand<Rc> {
    base_level_width: u32,
    base_level_height: u32,
    internal_format: TextureFormat,
    max_level: u8
}

impl<Rc> AllocateTexture2DCommand<Rc> {
    pub fn no_mipmap(width: u32, height: u32, internal_format: TextureFormat) -> Self {
        AllocateTexture2DCommand {
            base_level_width: width,
            base_level_height: height,
            internal_format,
            max_level: 0
        }
    }

    pub fn mipmap_all(base_level_width: u32, base_level_height: u32, internal_format: TextureFormat) -> Self {
        AllocateTexture2DCommand {
            base_level_width,
            base_level_height,
            internal_format,
            max_level: max_mipmap_level(base_level_width, base_level_height)
        }
    }

    pub fn mipmap_levels(mipmap_levels: u8, base_level_width: u32, base_level_height: u32, internal_format: TextureFormat) -> Result<Self, MipmapLevelsError> {
        let max_level = max_mipmap_level(base_level_width, base_level_height);

        if mipmap_levels > max_level {
            Err(MipmapLevelsError {
                level: mipmap_levels,
                max_level
            })
        } else {
            Ok(AllocateTexture2DCommand {
                base_level_width,
                base_level_height,
                internal_format,
                max_level: mipmap_levels
            })
        }
    }
}

struct MipmapLevelsError {
    level: u8,
    max_level: u8
}

pub struct GenerateMipmapCommand<T, Rc> {
    texture: T
}

impl<T, Rc> GenerateMipmapCommand<T, Rc> where Rc: BaseRenderingContext, T: Into<Rc::SupportTextures> {
    pub fn new(texture: T) -> Self {
        GenerateMipmapCommand {
            texture
        }
    }

    fn execute_internal(&self, rendering_context: Rc) -> Result<(), ContextMismatchError> {
        rendering_context.generate_mipmap(self.texture)
    }
}

impl<T, Rc> GpuCommand<Rc> for GenerateMipmapCommand<T, Rc> where Rc: StandardRenderingContext, T: Into<Rc::SupportTextures> {

}

struct Texture2DBuilder {

}

impl Texture2DBuilder {
    fn generate_mipmap<I>(base_image_source: I, internal_format: TextureFormat) -> Self where I: Into<ImageSource> {

    }

    fn no_mipmap<I>(image_source: I, internal_format: TextureFormat) -> Self where I: Into<ImageSource> {

    }

    fn manual_mipmap(internal_format: TextureFormat) -> Self {

    }
}

struct Sampler<T> {

}

struct CreateTexture2DCommand {
    image_source: ImageSource,
    internal_format: PixelFormat,

}

struct Texture2DArray {
    layers: Vec<TextureLayer>
}

struct TextureCube {
    positive_x_layer: TextureLayer,
    negative_x_layer: TextureLayer,
    positive_y_layer: TextureLayer,
    negative_y_layer: TextureLayer,
    positive_z_layer: TextureLayer,
    negative_z_layer: TextureLayer,
}

struct TextureLayer {

}

struct MipMap {
    levels: Vec<MipMapLevel>
}

pub struct TextureImage<T> where T: Texture {
    texture: T,
    level: u8,
    layer: u32,
    width: u32,
    height: u32
}

impl<T> TextureImage<T> where T: Texture {
    pub fn texture(&self) -> &T {
        self.texture
    }

    pub fn level(&self) -> u8 {
        self.level
    }

    pub fn layer(&self) -> u32 {
        self.layer
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }
}

struct TextureLevel {
    image: TextureImage
}

impl TextureLevel {
    fn width(&self) -> u32 {
        self.image.width
    }

    fn height(&self) -> u32 {
        self.image.height
    }

    fn image(&self) -> &TextureImage {
        &self.image
    }
}

struct LayeredTextureLevel {
    width: u32,
    height: u32,
    layers: Vec<TextureImage>
}

impl LayeredTextureLevel {
    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }

    fn depth(&self) -> u32 {
        self.layers.len() as u32
    }

    fn layers(&self) -> &[TextureImage] {
        self.layers
    }
}

enum ImageSource {
    Blank(u32, u32),
    Bytes(Box<[u8]>, u32, u32),
    ImageElement(ImageElement),
    ImageElementRegion(),
    CanvasElement(CanvasElement),
    CanvasElementRegion()
}

pub enum TextureFormat {
    R8,
    R16F,
    R32F,
    R8UI,
    RG8,
    RG16F,
    RG32F,
    RGUI,
    RGB8,
    SRGB8,
    RGB565,
    R11F_G11F_B10F,
    RGB9_E5,
    RGB16F,
    RGB32F,
    RGB8UI,
    RGBA8,
    SRGB_APLHA8,
    RGB5_A1,
    RGBA4444,
    RGBA16F,
    RGBA32F,
    RGBA8UI,
}
