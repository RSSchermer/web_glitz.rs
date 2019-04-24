use std::marker;

use crate::render_pass::{DefaultRGBBuffer, Framebuffer, DefaultDepthStencilBuffer, DefaultDepthBuffer, DefaultStencilBuffer, DefaultRGBABuffer};
use crate::render_target::{RenderTargetDescription, EncodingContext, RenderTargetEncoding};
use crate::render_target::render_target_encoding::RenderTargetData;

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

    fn into_encoding(
        self,
        context: &mut EncodingContext,
    ) -> RenderTargetEncoding<Self::Framebuffer> {
        RenderTargetEncoding {
            framebuffer: Framebuffer {
                color: DefaultRGBBuffer::new(context.render_pass_id),
                depth_stencil: (),
                context_id: context.context_id,
                render_pass_id: context.render_pass_id,
                last_id: 0,
                dimensions: None,
            },
            context,
            data: RenderTargetData::Default,
        }
    }
}

impl RenderTargetDescription for DefaultRenderTarget<DefaultRGBBuffer, DefaultDepthStencilBuffer> {
    type Framebuffer = Framebuffer<DefaultRGBBuffer, DefaultDepthStencilBuffer>;

    fn into_encoding(
        self,
        context: &mut EncodingContext,
    ) -> RenderTargetEncoding<Self::Framebuffer> {
        RenderTargetEncoding {
            framebuffer: Framebuffer {
                color: DefaultRGBBuffer::new(context.render_pass_id),
                depth_stencil: DefaultDepthStencilBuffer::new(context.render_pass_id),
                context_id: context.context_id,
                render_pass_id: context.render_pass_id,
                last_id: 0,
                dimensions: None,
            },
            context,
            data: RenderTargetData::Default,
        }
    }
}

impl RenderTargetDescription for DefaultRenderTarget<DefaultRGBBuffer, DefaultDepthBuffer> {
    type Framebuffer = Framebuffer<DefaultRGBBuffer, DefaultDepthBuffer>;

    fn into_encoding(
        self,
        context: &mut EncodingContext,
    ) -> RenderTargetEncoding<Self::Framebuffer> {
        RenderTargetEncoding {
            framebuffer: Framebuffer {
                color: DefaultRGBBuffer::new(context.render_pass_id),
                depth_stencil: DefaultDepthBuffer::new(context.render_pass_id),
                context_id: context.context_id,
                render_pass_id: context.render_pass_id,
                last_id: 0,
                dimensions: None,
            },
            context,
            data: RenderTargetData::Default,
        }
    }
}

impl RenderTargetDescription for DefaultRenderTarget<DefaultRGBBuffer, DefaultStencilBuffer> {
    type Framebuffer = Framebuffer<DefaultRGBBuffer, DefaultStencilBuffer>;

    fn into_encoding(
        self,
        context: &mut EncodingContext,
    ) -> RenderTargetEncoding<Self::Framebuffer> {
        RenderTargetEncoding {
            framebuffer: Framebuffer {
                color: DefaultRGBBuffer::new(context.render_pass_id),
                depth_stencil: DefaultStencilBuffer::new(context.render_pass_id),
                context_id: context.context_id,
                render_pass_id: context.render_pass_id,
                last_id: 0,
                dimensions: None,
            },
            context,
            data: RenderTargetData::Default,
        }
    }
}

impl RenderTargetDescription for DefaultRenderTarget<DefaultRGBABuffer, ()> {
    type Framebuffer = Framebuffer<DefaultRGBABuffer, ()>;

    fn into_encoding(
        self,
        context: &mut EncodingContext,
    ) -> RenderTargetEncoding<Self::Framebuffer> {
        RenderTargetEncoding {
            framebuffer: Framebuffer {
                color: DefaultRGBABuffer::new(context.render_pass_id),
                depth_stencil: (),
                context_id: context.context_id,
                render_pass_id: context.render_pass_id,
                last_id: 0,
                dimensions: None,
            },
            context,
            data: RenderTargetData::Default,
        }
    }
}

impl RenderTargetDescription for DefaultRenderTarget<DefaultRGBABuffer, DefaultDepthStencilBuffer> {
    type Framebuffer = Framebuffer<DefaultRGBABuffer, DefaultDepthStencilBuffer>;

    fn into_encoding(
        self,
        context: &mut EncodingContext,
    ) -> RenderTargetEncoding<Self::Framebuffer> {
        RenderTargetEncoding {
            framebuffer: Framebuffer {
                color: DefaultRGBABuffer::new(context.render_pass_id),
                depth_stencil: DefaultDepthStencilBuffer::new(context.render_pass_id),
                context_id: context.context_id,
                render_pass_id: context.render_pass_id,
                last_id: 0,
                dimensions: None,
            },
            context,
            data: RenderTargetData::Default,
        }
    }
}

impl RenderTargetDescription for DefaultRenderTarget<DefaultRGBABuffer, DefaultDepthBuffer> {
    type Framebuffer = Framebuffer<DefaultRGBABuffer, DefaultDepthBuffer>;

    fn into_encoding(
        self,
        context: &mut EncodingContext,
    ) -> RenderTargetEncoding<Self::Framebuffer> {
        RenderTargetEncoding {
            framebuffer: Framebuffer {
                color: DefaultRGBABuffer::new(context.render_pass_id),
                depth_stencil: DefaultDepthBuffer::new(context.render_pass_id),
                context_id: context.context_id,
                render_pass_id: context.render_pass_id,
                last_id: 0,
                dimensions: None,
            },
            context,
            data: RenderTargetData::Default,
        }
    }
}

impl RenderTargetDescription for DefaultRenderTarget<DefaultRGBABuffer, DefaultStencilBuffer> {
    type Framebuffer = Framebuffer<DefaultRGBABuffer, DefaultStencilBuffer>;

    fn into_encoding(
        self,
        context: &mut EncodingContext,
    ) -> RenderTargetEncoding<Self::Framebuffer> {
        RenderTargetEncoding {
            framebuffer: Framebuffer {
                color: DefaultRGBABuffer::new(context.render_pass_id),
                depth_stencil: DefaultStencilBuffer::new(context.render_pass_id),
                context_id: context.context_id,
                render_pass_id: context.render_pass_id,
                last_id: 0,
                dimensions: None,
            },
            context,
            data: RenderTargetData::Default,
        }
    }
}
