use super::base_rendering_context::BaseRenderingContext;
use super::gpu_command::{GpuCommand, CommandObject};

pub mod draw_command;

pub struct RenderPassCommand<Rc, C, F> {
    framebuffer: F,
    command: Option<CommandObject<Rc::RenderPass, C>>
}

impl<Rc, C, F> RenderPassCommand<Rc, C, Rc::FramebufferHandle> where Rc: BaseRenderingContext, C: GpuCommand<Rc::RenderPass, Output=ContextError> {
    pub fn new<T>(framebuffer: F, command: T) -> Self where T: Into<CommandObject<Rc::RenderPass, C>> {
        RenderPassCommand {
            framebuffer,
            command: Some(command.into())
        }
    }

    fn execute_internal(&mut self, rendering_context: &mut Rc) -> Result<(), ContextError> {
        let render_pass = rendering_context.begin_render_pass(self.framebuffer)?;
        let command = self.command.take().expect("Cannot execute render pass twice.");

        command.execute(render_pass)?;

        Ok(())
    }
}

impl<Rc, C, F> GpuCommand<Rc> for RenderPassCommand<Rc, C, Rc::FramebufferHandle> where Rc: BaseRenderingContext, C: GpuCommand<Rc::RenderPass, Output=ContextError> {
    type Output = ();

    type Error = ContextError;

    fn execute_static(self, rendering_context: &mut Rc) -> Result<Self::Output, Self::Error> {
        self.execute_internal(rendering_context)
    }

    fn execute_dynamic(self: Box<Self>, rendering_context: &mut Rc) -> Result<Self::Output, Self::Error> {
        self.execute_internal(rendering_context)
    }
}
