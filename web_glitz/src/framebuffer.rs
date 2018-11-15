pub struct FramebufferHandle<C> {
    color_attachment_slots: [AttachmentSlot;16],
    depth_attachment_slot: AttachmentSlot,
    stencil_attachment_slot: AttachmentSlot,
    depth_stencil_attachment_slot: AttachmentSlot,
}

impl<C> FramebufferHandle<C> where C: RenderingContext {
    pub fn color_attachments(&self) -> &[AttachmentSlot] {

    }

    pub fn depth_attachment(&self) -> &AttachmentSlot {

    }

    pub fn stencil_attachment(&self) -> &AttachmentSlot {

    }

    pub fn depth_stencil_attachment(&self) -> &AttachmentSlot {

    }

    pub fn task<F, T>(&mut self, f: F) -> FramebufferTask<F, C> where F: FnOnce(BoundFramebuffer<C>) -> T, T: GpuTask<BoundFramebuffer<C>> {

    }
}

pub struct BoundFramebuffer<C> {

}

#[derive(Clone)]
enum AttachmentSlot {
    Image2D(Image2DRef),
    Renderbuffer,
    Empty
}

pub struct FramebufferTask<F, C> {

}
