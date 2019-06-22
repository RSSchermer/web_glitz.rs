use std::cell::Cell;
use std::marker;

use crate::render_pass::{
    DefaultDepthBuffer, DefaultDepthStencilBuffer, DefaultRGBABuffer, DefaultRGBBuffer,
    DefaultStencilBuffer, Framebuffer, RenderPass, RenderPassContext, RenderPassId,
};
use crate::render_target::render_target_description::RenderTargetData;
use crate::render_target::RenderTargetDescription;
use crate::task::{ContextId, GpuTask};

/// A handle to the default render target associated with a [RenderingContext].
#[derive(Clone, Copy, PartialEq)]
pub struct DefaultRenderTarget<C, Ds> {
    context_id: usize,
    color_buffer: marker::PhantomData<C>,
    depth_stencil_buffer: marker::PhantomData<Ds>,
}

impl<C, Ds> DefaultRenderTarget<C, Ds> {
    pub(crate) fn new(context_id: usize) -> Self {
        DefaultRenderTarget {
            context_id,
            color_buffer: marker::PhantomData,
            depth_stencil_buffer: marker::PhantomData,
        }
    }
}

impl RenderTargetDescription for DefaultRenderTarget<DefaultRGBBuffer, ()> {
    type Framebuffer = Framebuffer<DefaultRGBBuffer, ()>;

    fn create_render_pass<F, T>(&mut self, id: RenderPassId, f: F) -> RenderPass<T>
    where
        F: FnOnce(&Self::Framebuffer) -> T,
        for<'a> T: GpuTask<RenderPassContext<'a>>,
    {
        let RenderPassId { id, context_id } = id;

        let task = f(&Framebuffer {
            color: DefaultRGBBuffer::new(id),
            depth_stencil: (),
            dimensions: None,
            context_id,
            render_pass_id: id,
            last_pipeline_task_id: Cell::new(0),
        });

        if let ContextId::Id(render_pass_id) = task.context_id() {
            if render_pass_id != id {
                panic!("The render pass task belongs to a different render pass.")
            }
        }

        RenderPass {
            id,
            context_id,
            render_target: RenderTargetData::Default,
            task,
        }
    }
}

impl RenderTargetDescription for DefaultRenderTarget<DefaultRGBBuffer, DefaultDepthStencilBuffer> {
    type Framebuffer = Framebuffer<DefaultRGBBuffer, DefaultDepthStencilBuffer>;

    fn create_render_pass<F, T>(&mut self, id: RenderPassId, f: F) -> RenderPass<T>
    where
        F: FnOnce(&Self::Framebuffer) -> T,
        for<'a> T: GpuTask<RenderPassContext<'a>>,
    {
        let RenderPassId { id, context_id } = id;

        let task = f(&Framebuffer {
            color: DefaultRGBBuffer::new(id),
            depth_stencil: DefaultDepthStencilBuffer::new(id),
            dimensions: None,
            context_id,
            render_pass_id: id,
            last_pipeline_task_id: Cell::new(0),
        });

        if let ContextId::Id(render_pass_id) = task.context_id() {
            if render_pass_id != id {
                panic!("The render pass task belongs to a different render pass.")
            }
        }

        RenderPass {
            id,
            context_id,
            render_target: RenderTargetData::Default,
            task,
        }
    }
}

impl RenderTargetDescription for DefaultRenderTarget<DefaultRGBBuffer, DefaultDepthBuffer> {
    type Framebuffer = Framebuffer<DefaultRGBBuffer, DefaultDepthBuffer>;

    fn create_render_pass<F, T>(&mut self, id: RenderPassId, f: F) -> RenderPass<T>
    where
        F: FnOnce(&Self::Framebuffer) -> T,
        for<'a> T: GpuTask<RenderPassContext<'a>>,
    {
        let RenderPassId { id, context_id } = id;

        let task = f(&Framebuffer {
            color: DefaultRGBBuffer::new(id),
            depth_stencil: DefaultDepthBuffer::new(id),
            dimensions: None,
            context_id,
            render_pass_id: id,
            last_pipeline_task_id: Cell::new(0),
        });

        if let ContextId::Id(render_pass_id) = task.context_id() {
            if render_pass_id != id {
                panic!("The render pass task belongs to a different render pass.")
            }
        }

        RenderPass {
            id,
            context_id,
            render_target: RenderTargetData::Default,
            task,
        }
    }
}

impl RenderTargetDescription for DefaultRenderTarget<DefaultRGBBuffer, DefaultStencilBuffer> {
    type Framebuffer = Framebuffer<DefaultRGBBuffer, DefaultStencilBuffer>;

    fn create_render_pass<F, T>(&mut self, id: RenderPassId, f: F) -> RenderPass<T>
    where
        F: FnOnce(&Self::Framebuffer) -> T,
        for<'a> T: GpuTask<RenderPassContext<'a>>,
    {
        let RenderPassId { id, context_id } = id;

        let task = f(&Framebuffer {
            color: DefaultRGBBuffer::new(id),
            depth_stencil: DefaultStencilBuffer::new(id),
            dimensions: None,
            context_id,
            render_pass_id: id,
            last_pipeline_task_id: Cell::new(0),
        });

        if let ContextId::Id(render_pass_id) = task.context_id() {
            if render_pass_id != id {
                panic!("The render pass task belongs to a different render pass.")
            }
        }

        RenderPass {
            id,
            context_id,
            render_target: RenderTargetData::Default,
            task,
        }
    }
}

impl RenderTargetDescription for DefaultRenderTarget<DefaultRGBABuffer, ()> {
    type Framebuffer = Framebuffer<DefaultRGBABuffer, ()>;

    fn create_render_pass<F, T>(&mut self, id: RenderPassId, f: F) -> RenderPass<T>
    where
        F: FnOnce(&Self::Framebuffer) -> T,
        for<'a> T: GpuTask<RenderPassContext<'a>>,
    {
        let RenderPassId { id, context_id } = id;

        let task = f(&Framebuffer {
            color: DefaultRGBABuffer::new(id),
            depth_stencil: (),
            dimensions: None,
            context_id,
            render_pass_id: id,
            last_pipeline_task_id: Cell::new(0),
        });

        if let ContextId::Id(render_pass_id) = task.context_id() {
            if render_pass_id != id {
                panic!("The render pass task belongs to a different render pass.")
            }
        }

        RenderPass {
            id,
            context_id,
            render_target: RenderTargetData::Default,
            task,
        }
    }
}

impl RenderTargetDescription for DefaultRenderTarget<DefaultRGBABuffer, DefaultDepthStencilBuffer> {
    type Framebuffer = Framebuffer<DefaultRGBABuffer, DefaultDepthStencilBuffer>;

    fn create_render_pass<F, T>(&mut self, id: RenderPassId, f: F) -> RenderPass<T>
    where
        F: FnOnce(&Self::Framebuffer) -> T,
        for<'a> T: GpuTask<RenderPassContext<'a>>,
    {
        let RenderPassId { id, context_id } = id;

        let task = f(&Framebuffer {
            color: DefaultRGBABuffer::new(id),
            depth_stencil: DefaultDepthStencilBuffer::new(id),
            dimensions: None,
            context_id,
            render_pass_id: id,
            last_pipeline_task_id: Cell::new(0),
        });

        if let ContextId::Id(render_pass_id) = task.context_id() {
            if render_pass_id != id {
                panic!("The render pass task belongs to a different render pass.")
            }
        }

        RenderPass {
            id,
            context_id,
            render_target: RenderTargetData::Default,
            task,
        }
    }
}

impl RenderTargetDescription for DefaultRenderTarget<DefaultRGBABuffer, DefaultDepthBuffer> {
    type Framebuffer = Framebuffer<DefaultRGBABuffer, DefaultDepthBuffer>;

    fn create_render_pass<F, T>(&mut self, id: RenderPassId, f: F) -> RenderPass<T>
    where
        F: FnOnce(&Self::Framebuffer) -> T,
        for<'a> T: GpuTask<RenderPassContext<'a>>,
    {
        let RenderPassId { id, context_id } = id;

        let task = f(&Framebuffer {
            color: DefaultRGBABuffer::new(id),
            depth_stencil: DefaultDepthBuffer::new(id),
            dimensions: None,
            context_id,
            render_pass_id: id,
            last_pipeline_task_id: Cell::new(0),
        });

        if let ContextId::Id(render_pass_id) = task.context_id() {
            if render_pass_id != id {
                panic!("The render pass task belongs to a different render pass.")
            }
        }

        RenderPass {
            id,
            context_id,
            render_target: RenderTargetData::Default,
            task,
        }
    }
}

impl RenderTargetDescription for DefaultRenderTarget<DefaultRGBABuffer, DefaultStencilBuffer> {
    type Framebuffer = Framebuffer<DefaultRGBABuffer, DefaultStencilBuffer>;

    fn create_render_pass<F, T>(&mut self, id: RenderPassId, f: F) -> RenderPass<T>
    where
        F: FnOnce(&Self::Framebuffer) -> T,
        for<'a> T: GpuTask<RenderPassContext<'a>>,
    {
        let RenderPassId { id, context_id } = id;

        let task = f(&Framebuffer {
            color: DefaultRGBABuffer::new(id),
            depth_stencil: DefaultStencilBuffer::new(id),
            dimensions: None,
            context_id,
            render_pass_id: id,
            last_pipeline_task_id: Cell::new(0),
        });

        if let ContextId::Id(render_pass_id) = task.context_id() {
            if render_pass_id != id {
                panic!("The render pass task belongs to a different render pass.")
            }
        }

        RenderPass {
            id,
            context_id,
            render_target: RenderTargetData::Default,
            task,
        }
    }
}
