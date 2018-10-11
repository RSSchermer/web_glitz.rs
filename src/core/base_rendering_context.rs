use core::buffer::*;
use core::texture::*;

pub trait BaseRenderingContext {
    type BufferHandle: GpuBufferView<Self>;

    type FramebufferHandle: Framebuffer<Self>;

    type ShaderHandle: Shader<Self>;

    type Texture2DHandle: Texture2D<Self>;

    type Texture2DArrayHandle: Texture2DArray<Self>;

    type Texture3D: Texture3D<Self>;

    type TextureCube: TextureCube<Self>;

    type TextureCubeArray: TextureCubeArray<Self>;

    type SupportedTexture = BaseRenderingContextTexture<Self>;

    type RenderPass: RenderPass<Self>;

    fn allocate_buffer(&mut self) -> Self::BufferHandle;

    fn buffer_upload(&mut self, buffer: &Self::BufferHandle, data: &[u8]);

    fn allocate_texture_2d(&mut self, base_level_width: u32, base_level_height: u32, internal_format: TextureFormat, levels: u8) -> Result<Self::Texture2DHandle, AllocateTexture2DError>;

    fn upload_texture_image<T>(&mut self, image: TextureImage<T>, data: &[u8]) where T: Into<Self::SupportedTexture>;
}

enum BaseRenderingContextTexture<Rc> where Rc: BaseRenderingContext {
    Texture2D(Rc::Texture2DHandle),
    Texture2DArray(Rc::Texture2DArrayHandle),
    Texture3D(Rc::Texture3D),
    TextureCube(Rc::TextureCube),
    TextureCubeArray(Rc::TextureCubeArray)
}

trait RenderPass<Rc> where Rc: BaseRenderingContext {
    fn draw(&self, pipeline: &Rc::GraphicsPipeline, vertex_stream_description: &VertexStreamDescription<Rc>, uniforms: &Rc::Uniforms);
}
