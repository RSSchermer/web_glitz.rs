use task::GpuTask;
use runtime::Connection;
use task::Progress;

pub struct RenderPass<D, F> {
    render_pass_descriptor: D,
    f: F
}

pub struct RenderPassContext {

}

impl<D, F> GpuTask<Connection> for RenderPass<D, F> where D: RenderPassDescriptor, F: FnOnce(&mut D::Framebuffer) -> GpuTask<RenderPassContext> {
    type Output = ();

    fn progress(&mut self, connection: &mut Connection) -> Progress<Self::Output> {

    }
}

pub trait RenderPassDescriptor {
    type Framebuffer: Framebuffer;


}

pub trait Framebuffer {
    type Color0;

    type Color1;

    type Color2;

    type Color3;

    type Color4;

    type Color5;

    type Color6;

    type Color7;

    type Color8;

    type Color9;

    type Color10;

    type Color11;

    type Color12;

    type Color13;

    type Color14;

    type Color15;

    type DepthStencil;

    fn blit_color<I>(&mut self, from: I) where I: BlitColorCompatible<Self>;
}

pub struct FloatColorAttachment<F> where F: FloatColorRenderable;

pub struct IntegerColorAttachment<F> where F: IntegerColorRenderable;

pub struct UnsignedIntegerColorAttachment<F> where F: UnsignedIntegerColorRenderable;

pub unsafe trait BlitColorCompatible<F> {}

pub unsafe trait BlitDepthStencilCompatible<F> {}
