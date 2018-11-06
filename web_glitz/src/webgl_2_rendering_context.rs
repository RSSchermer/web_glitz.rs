use webgl_bindings as gl;

use core::gpu_command::GpuCommand;
use core::texture::*;

struct Webgl2RenderingContext {
    raw: gl::WebGL2RenderingContext,
    state: BackendState,

}

struct Webgl2Texture2D {
    width: u32,
    height: u32,
    internal_format: TextureFormat,

}

impl Texture for Webgl2Texture2D {

}

impl Texture2D<Webgl2RenderingContext> for Webgl2Texture2D {

}

impl AllocateTexture2DCommand<Webgl2RenderingContext> {
    fn execute_internal(&self, rendering_context: &mut Webgl2RenderingContext) -> Result<impl Texture2D<Webgl2RenderingContext>, AllocateTexture2DError> {
        let texture_object = rendering_context.raw.create_texture().unwrap();

        rendering_context.state.bind_texture(texture_object);

        rendering_context.raw.tex_storage2_d(gl::TEXTURE_2D, self.max_level(), self.base_level_width(), self.base_level_height())
    }
}

impl GpuCommand<Webgl2RenderingContext> for AllocateTexture2DCommand<Webgl2RenderingContext> {
    type Output = impl Texture2D<Webgl2RenderingContext>;

    type Error = AllocateTexture2DError;

    fn execute_static(self, rendering_context: &mut Webgl2RenderingContext) -> Result<Self::Output, Self::Error> {
        self.execute_internal(rendering_context)
    }

    fn execute_dynamic(self: Box<Self>, rendering_context: &mut Webgl2RenderingContext) -> Result<Self::Output, Self::Error> {
        self.execute_internal(rendering_context)
    }
}

struct BackendState {

}
