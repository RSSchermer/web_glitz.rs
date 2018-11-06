use super::super::base_rendering_context::BaseRenderingContext;
use super::super::gpu_command::GpuCommand;

pub struct DrawCommand<Rc> {
    graphics_pipeline: Rc::GraphicsPipeline,
    vertex_stream_description: VertexStreamDescription<Rc>,
    uniforms: Rc::Uniforms
}

impl<Rc> DrawCommand<Rc> where Rc: BasicRenderingContext {
    pub fn new<P, U>(graphics_pipeline: P, vertex_stream_description: VertexStreamDescription<Rc>, uniforms: U) -> Self where P: Into<Rc::GraphicsPipeline>, U: Into<Rc::Uniforms> {
        DrawCommand {
            graphics_pipeline: graphics_pipeline.into(),
            vertex_stream_description,
            uniforms: uniforms.into()
        }
    }

    fn execute_internal(&self, render_pass: &mut Rc::RenderPass) -> Result<(), ContextError> {
        render_pass.draw(self.graphics_pipeline, self.vertex_stream_description, self.uniforms)
    }
}

impl<Rc> GpuCommand<Rc::RenderPass> for DrawCommand<Rc> where Rc: BaseRenderingContext {
    type Output = ();

    type Error = ContextError;

    fn execute_static(self, render_pass: &mut Rc::RenderPass) -> Result<Self::Output, Self::Error> {
        self.execute_internal(render_pass)
    }

    fn execute_dynamic(self: Box<Self>, render_pass: &mut Rc::RenderPass) -> Result<Self::Output, Self::Error> {
        self.execute_internal(render_pass)
    }
}
