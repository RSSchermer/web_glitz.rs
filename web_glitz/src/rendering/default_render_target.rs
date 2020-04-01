use std::cell::Cell;
use std::marker;

use crate::rendering::render_target::RenderTargetData;
use crate::rendering::{
    DefaultDepthBuffer, DefaultDepthStencilBuffer, DefaultRGBABuffer, DefaultRGBBuffer,
    DefaultStencilBuffer, Framebuffer, GraphicsPipelineTarget, RenderPass, RenderPassContext,
};
use crate::runtime::single_threaded::RenderPassIdGen;
use crate::task::{ContextId, GpuTask};

/// A handle to the default render target associated with a [RenderingContext].
#[derive(Clone)]
pub struct DefaultRenderTarget<C, Ds> {
    context_id: usize,
    render_pass_id_gen: RenderPassIdGen,
    color_buffer: marker::PhantomData<C>,
    depth_stencil_buffer: marker::PhantomData<Ds>,
}

impl<C, Ds> DefaultRenderTarget<C, Ds> {
    pub(crate) fn new(context_id: usize, render_pass_id_gen: RenderPassIdGen) -> Self {
        DefaultRenderTarget {
            context_id,
            render_pass_id_gen,
            color_buffer: marker::PhantomData,
            depth_stencil_buffer: marker::PhantomData,
        }
    }
}

impl DefaultRenderTarget<DefaultRGBBuffer, ()> {
    pub fn create_render_pass<F, T>(&mut self, f: F) -> RenderPass<T>
    where
        F: FnOnce(&Framebuffer<DefaultRGBBuffer, ()>) -> T,
        T: GpuTask<RenderPassContext>,
    {
        let id = self.render_pass_id_gen.next();

        let task = f(&Framebuffer {
            color: DefaultRGBBuffer::new(id),
            depth_stencil: (),
            data: GraphicsPipelineTarget {
                dimensions: None,
                context_id: self.context_id,
                render_pass_id: id,
                last_pipeline_task_id: Cell::new(0),
            },
        });

        if let ContextId::Id(render_pass_id) = task.context_id() {
            if render_pass_id != id {
                panic!("The render pass task belongs to a different render pass.")
            }
        }

        RenderPass {
            id,
            context_id: self.context_id,
            render_target: RenderTargetData::Default,
            task,
        }
    }
}

impl DefaultRenderTarget<DefaultRGBBuffer, DefaultDepthStencilBuffer> {
    pub fn create_render_pass<F, T>(&mut self, f: F) -> RenderPass<T>
    where
        F: FnOnce(&Framebuffer<DefaultRGBBuffer, DefaultDepthStencilBuffer>) -> T,
        T: GpuTask<RenderPassContext>,
    {
        let id = self.render_pass_id_gen.next();

        let task = f(&Framebuffer {
            color: DefaultRGBBuffer::new(id),
            depth_stencil: DefaultDepthStencilBuffer::new(id),
            data: GraphicsPipelineTarget {
                dimensions: None,
                context_id: self.context_id,
                render_pass_id: id,
                last_pipeline_task_id: Cell::new(0),
            },
        });

        if let ContextId::Id(render_pass_id) = task.context_id() {
            if render_pass_id != id {
                panic!("The render pass task belongs to a different render pass.")
            }
        }

        RenderPass {
            id,
            context_id: self.context_id,
            render_target: RenderTargetData::Default,
            task,
        }
    }
}

impl DefaultRenderTarget<DefaultRGBBuffer, DefaultDepthBuffer> {
    pub fn create_render_pass<F, T>(&mut self, f: F) -> RenderPass<T>
    where
        F: FnOnce(&Framebuffer<DefaultRGBBuffer, DefaultDepthBuffer>) -> T,
        T: GpuTask<RenderPassContext>,
    {
        let id = self.render_pass_id_gen.next();

        let task = f(&Framebuffer {
            color: DefaultRGBBuffer::new(id),
            depth_stencil: DefaultDepthBuffer::new(id),
            data: GraphicsPipelineTarget {
                dimensions: None,
                context_id: self.context_id,
                render_pass_id: id,
                last_pipeline_task_id: Cell::new(0),
            },
        });

        if let ContextId::Id(render_pass_id) = task.context_id() {
            if render_pass_id != id {
                panic!("The render pass task belongs to a different render pass.")
            }
        }

        RenderPass {
            id,
            context_id: self.context_id,
            render_target: RenderTargetData::Default,
            task,
        }
    }
}

impl DefaultRenderTarget<DefaultRGBBuffer, DefaultStencilBuffer> {
    pub fn create_render_pass<F, T>(&mut self, f: F) -> RenderPass<T>
    where
        F: FnOnce(&Framebuffer<DefaultRGBBuffer, DefaultStencilBuffer>) -> T,
        T: GpuTask<RenderPassContext>,
    {
        let id = self.render_pass_id_gen.next();

        let task = f(&Framebuffer {
            color: DefaultRGBBuffer::new(id),
            depth_stencil: DefaultStencilBuffer::new(id),
            data: GraphicsPipelineTarget {
                dimensions: None,
                context_id: self.context_id,
                render_pass_id: id,
                last_pipeline_task_id: Cell::new(0),
            },
        });

        if let ContextId::Id(render_pass_id) = task.context_id() {
            if render_pass_id != id {
                panic!("The render pass task belongs to a different render pass.")
            }
        }

        RenderPass {
            id,
            context_id: self.context_id,
            render_target: RenderTargetData::Default,
            task,
        }
    }
}

impl DefaultRenderTarget<DefaultRGBABuffer, ()> {
    pub fn create_render_pass<F, T>(&mut self, f: F) -> RenderPass<T>
    where
        F: FnOnce(&Framebuffer<DefaultRGBABuffer, ()>) -> T,
        T: GpuTask<RenderPassContext>,
    {
        let id = self.render_pass_id_gen.next();

        let task = f(&Framebuffer {
            color: DefaultRGBABuffer::new(id),
            depth_stencil: (),
            data: GraphicsPipelineTarget {
                dimensions: None,
                context_id: self.context_id,
                render_pass_id: id,
                last_pipeline_task_id: Cell::new(0),
            },
        });

        if let ContextId::Id(render_pass_id) = task.context_id() {
            if render_pass_id != id {
                panic!("The render pass task belongs to a different render pass.")
            }
        }

        RenderPass {
            id,
            context_id: self.context_id,
            render_target: RenderTargetData::Default,
            task,
        }
    }
}

impl DefaultRenderTarget<DefaultRGBABuffer, DefaultDepthStencilBuffer> {
    pub fn create_render_pass<F, T>(&mut self, f: F) -> RenderPass<T>
    where
        F: FnOnce(&Framebuffer<DefaultRGBABuffer, DefaultDepthStencilBuffer>) -> T,
        T: GpuTask<RenderPassContext>,
    {
        let id = self.render_pass_id_gen.next();

        let task = f(&Framebuffer {
            color: DefaultRGBABuffer::new(id),
            depth_stencil: DefaultDepthStencilBuffer::new(id),
            data: GraphicsPipelineTarget {
                dimensions: None,
                context_id: self.context_id,
                render_pass_id: id,
                last_pipeline_task_id: Cell::new(0),
            },
        });

        if let ContextId::Id(render_pass_id) = task.context_id() {
            if render_pass_id != id {
                panic!("The render pass task belongs to a different render pass.")
            }
        }

        RenderPass {
            id,
            context_id: self.context_id,
            render_target: RenderTargetData::Default,
            task,
        }
    }
}

impl DefaultRenderTarget<DefaultRGBABuffer, DefaultDepthBuffer> {
    pub fn create_render_pass<F, T>(&mut self, f: F) -> RenderPass<T>
    where
        F: FnOnce(&Framebuffer<DefaultRGBABuffer, DefaultDepthBuffer>) -> T,
        T: GpuTask<RenderPassContext>,
    {
        let id = self.render_pass_id_gen.next();

        let task = f(&Framebuffer {
            color: DefaultRGBABuffer::new(id),
            depth_stencil: DefaultDepthBuffer::new(id),
            data: GraphicsPipelineTarget {
                dimensions: None,
                context_id: self.context_id,
                render_pass_id: id,
                last_pipeline_task_id: Cell::new(0),
            },
        });

        if let ContextId::Id(render_pass_id) = task.context_id() {
            if render_pass_id != id {
                panic!("The render pass task belongs to a different render pass.")
            }
        }

        RenderPass {
            id,
            context_id: self.context_id,
            render_target: RenderTargetData::Default,
            task,
        }
    }
}

impl DefaultRenderTarget<DefaultRGBABuffer, DefaultStencilBuffer> {
    pub fn create_render_pass<F, T>(&mut self, f: F) -> RenderPass<T>
    where
        F: FnOnce(&Framebuffer<DefaultRGBABuffer, DefaultStencilBuffer>) -> T,
        T: GpuTask<RenderPassContext>,
    {
        let id = self.render_pass_id_gen.next();

        let task = f(&Framebuffer {
            color: DefaultRGBABuffer::new(id),
            depth_stencil: DefaultStencilBuffer::new(id),
            data: GraphicsPipelineTarget {
                dimensions: None,
                context_id: self.context_id,
                render_pass_id: id,
                last_pipeline_task_id: Cell::new(0),
            },
        });

        if let ContextId::Id(render_pass_id) = task.context_id() {
            if render_pass_id != id {
                panic!("The render pass task belongs to a different render pass.")
            }
        }

        RenderPass {
            id,
            context_id: self.context_id,
            render_target: RenderTargetData::Default,
            task,
        }
    }
}
