use std::cell::Cell;
use std::marker;

use crate::rendering::render_target::RenderTargetData;
use crate::rendering::{
    DefaultDepthBuffer, DefaultDepthStencilBuffer, DefaultRGBABuffer, DefaultRGBBuffer,
    DefaultStencilBuffer, GraphicsPipelineTarget, MultisampleFramebuffer, RenderPass,
    RenderPassContext,
};
use crate::runtime::single_threaded::ObjectIdGen;
use crate::task::{ContextId, GpuTask};

/// A handle to the default render target associated with a [RenderingContext].
#[derive(Clone)]
pub struct DefaultMultisampleRenderTarget<C, Ds> {
    context_id: u64,
    samples: u8,
    render_pass_id_gen: ObjectIdGen,
    color_buffer: marker::PhantomData<C>,
    depth_stencil_buffer: marker::PhantomData<Ds>,
}

impl<C, Ds> DefaultMultisampleRenderTarget<C, Ds> {
    pub(crate) fn new(
        context_id: u64,
        samples: u8,
        render_pass_id_gen: ObjectIdGen,
    ) -> Self {
        DefaultMultisampleRenderTarget {
            context_id,
            samples,
            render_pass_id_gen,
            color_buffer: marker::PhantomData,
            depth_stencil_buffer: marker::PhantomData,
        }
    }

    pub fn samples(&self) -> u8 {
        self.samples
    }
}

impl DefaultMultisampleRenderTarget<DefaultRGBBuffer, ()> {
    pub fn create_render_pass<F, T>(&mut self, f: F) -> RenderPass<T>
    where
        F: FnOnce(&MultisampleFramebuffer<DefaultRGBBuffer, ()>) -> T,
        T: GpuTask<RenderPassContext>,
    {
        let id = self.render_pass_id_gen.next();

        let task = f(&MultisampleFramebuffer {
            color: DefaultRGBBuffer::new(id),
            depth_stencil: (),
            data: GraphicsPipelineTarget {
                dimensions: None,
                context_id: self.context_id,
                render_pass_id: id,
                last_pipeline_task_id: Cell::new(0),
            },
            samples: self.samples,
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

impl DefaultMultisampleRenderTarget<DefaultRGBBuffer, DefaultDepthStencilBuffer> {
    pub fn create_render_pass<F, T>(&mut self, f: F) -> RenderPass<T>
    where
        F: FnOnce(&MultisampleFramebuffer<DefaultRGBBuffer, DefaultDepthStencilBuffer>) -> T,
        T: GpuTask<RenderPassContext>,
    {
        let id = self.render_pass_id_gen.next();

        let task = f(&MultisampleFramebuffer {
            color: DefaultRGBBuffer::new(id),
            depth_stencil: DefaultDepthStencilBuffer::new(id),
            data: GraphicsPipelineTarget {
                dimensions: None,
                context_id: self.context_id,
                render_pass_id: id,
                last_pipeline_task_id: Cell::new(0),
            },
            samples: self.samples,
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

impl DefaultMultisampleRenderTarget<DefaultRGBBuffer, DefaultDepthBuffer> {
    pub fn create_render_pass<F, T>(&mut self, f: F) -> RenderPass<T>
    where
        F: FnOnce(&MultisampleFramebuffer<DefaultRGBBuffer, DefaultDepthBuffer>) -> T,
        T: GpuTask<RenderPassContext>,
    {
        let id = self.render_pass_id_gen.next();

        let task = f(&MultisampleFramebuffer {
            color: DefaultRGBBuffer::new(id),
            depth_stencil: DefaultDepthBuffer::new(id),
            data: GraphicsPipelineTarget {
                dimensions: None,
                context_id: self.context_id,
                render_pass_id: id,
                last_pipeline_task_id: Cell::new(0),
            },
            samples: self.samples,
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

impl DefaultMultisampleRenderTarget<DefaultRGBBuffer, DefaultStencilBuffer> {
    pub fn create_render_pass<F, T>(&mut self, f: F) -> RenderPass<T>
    where
        F: FnOnce(&MultisampleFramebuffer<DefaultRGBBuffer, DefaultStencilBuffer>) -> T,
        T: GpuTask<RenderPassContext>,
    {
        let id = self.render_pass_id_gen.next();

        let task = f(&MultisampleFramebuffer {
            color: DefaultRGBBuffer::new(id),
            depth_stencil: DefaultStencilBuffer::new(id),
            data: GraphicsPipelineTarget {
                dimensions: None,
                context_id: self.context_id,
                render_pass_id: id,
                last_pipeline_task_id: Cell::new(0),
            },
            samples: self.samples,
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

impl DefaultMultisampleRenderTarget<DefaultRGBABuffer, ()> {
    pub fn create_render_pass<F, T>(&mut self, f: F) -> RenderPass<T>
    where
        F: FnOnce(&MultisampleFramebuffer<DefaultRGBABuffer, ()>) -> T,
        T: GpuTask<RenderPassContext>,
    {
        let id = self.render_pass_id_gen.next();

        let task = f(&MultisampleFramebuffer {
            color: DefaultRGBABuffer::new(id),
            depth_stencil: (),
            data: GraphicsPipelineTarget {
                dimensions: None,
                context_id: self.context_id,
                render_pass_id: id,
                last_pipeline_task_id: Cell::new(0),
            },
            samples: self.samples,
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

impl DefaultMultisampleRenderTarget<DefaultRGBABuffer, DefaultDepthStencilBuffer> {
    pub fn create_render_pass<F, T>(&mut self, f: F) -> RenderPass<T>
    where
        F: FnOnce(&MultisampleFramebuffer<DefaultRGBABuffer, DefaultDepthStencilBuffer>) -> T,
        T: GpuTask<RenderPassContext>,
    {
        let id = self.render_pass_id_gen.next();

        let task = f(&MultisampleFramebuffer {
            color: DefaultRGBABuffer::new(id),
            depth_stencil: DefaultDepthStencilBuffer::new(id),
            data: GraphicsPipelineTarget {
                dimensions: None,
                context_id: self.context_id,
                render_pass_id: id,
                last_pipeline_task_id: Cell::new(0),
            },
            samples: self.samples,
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

impl DefaultMultisampleRenderTarget<DefaultRGBABuffer, DefaultDepthBuffer> {
    pub fn create_render_pass<F, T>(&mut self, f: F) -> RenderPass<T>
    where
        F: FnOnce(&MultisampleFramebuffer<DefaultRGBABuffer, DefaultDepthBuffer>) -> T,
        T: GpuTask<RenderPassContext>,
    {
        let id = self.render_pass_id_gen.next();

        let task = f(&MultisampleFramebuffer {
            color: DefaultRGBABuffer::new(id),
            depth_stencil: DefaultDepthBuffer::new(id),
            data: GraphicsPipelineTarget {
                dimensions: None,
                context_id: self.context_id,
                render_pass_id: id,
                last_pipeline_task_id: Cell::new(0),
            },
            samples: self.samples,
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

impl DefaultMultisampleRenderTarget<DefaultRGBABuffer, DefaultStencilBuffer> {
    pub fn create_render_pass<F, T>(&mut self, f: F) -> RenderPass<T>
    where
        F: FnOnce(&MultisampleFramebuffer<DefaultRGBABuffer, DefaultStencilBuffer>) -> T,
        T: GpuTask<RenderPassContext>,
    {
        let id = self.render_pass_id_gen.next();

        let task = f(&MultisampleFramebuffer {
            color: DefaultRGBABuffer::new(id),
            depth_stencil: DefaultStencilBuffer::new(id),
            data: GraphicsPipelineTarget {
                dimensions: None,
                context_id: self.context_id,
                render_pass_id: id,
                last_pipeline_task_id: Cell::new(0),
            },
            samples: self.samples,
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
