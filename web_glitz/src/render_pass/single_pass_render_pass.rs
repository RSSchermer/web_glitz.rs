use crate::runtime::Connection;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::Hash;
use task::GpuTask;
use task::Progress;
use texture::CubeFace;
use util::JsId;
use runtime::framebuffer_cache::DrawBuffer;
use runtime::framebuffer_cache::AttachmentSet;
use image_format::InternalFormat;
use image_format::RGBA8;
use std::marker;
use runtime::framebuffer_cache::DepthStencilAttachmentDescriptor;
use runtime::framebuffer_cache::AttachmentImage;

const COLOR_BUFFERS: [DrawBuffer; 16] = [
    DrawBuffer::Color0,
    DrawBuffer::Color1,
    DrawBuffer::Color2,
    DrawBuffer::Color3,
    DrawBuffer::Color4,
    DrawBuffer::Color5,
    DrawBuffer::Color6,
    DrawBuffer::Color7,
    DrawBuffer::Color8,
    DrawBuffer::Color9,
    DrawBuffer::Color10,
    DrawBuffer::Color11,
    DrawBuffer::Color12,
    DrawBuffer::Color13,
    DrawBuffer::Color14,
    DrawBuffer::Color15,
];

//trait SinglePassAttachmentSet: AttachmentSet {
//    fn draw_buffers(&self) -> &[DrawBuffer];
//
//    fn load_all(&self, connection: &mut Connection);
//
//    fn store_all(&self, connection: &mut Connection);
//}

pub struct SinglePassAttachments<C, D> {
    pub color: C,
    pub depth_stencil: D,
}

pub enum ClearOp {
    ColorFloat(ClearColorFloatOp)
}

pub struct ClearColorFloatOp {

}

pub struct SinglePassAttacher {
    color_attachment_count: usize,
    color_attachment_images: [AttachmentImage;16],
    depth_stencil_attachment: DepthStencilAttachmentDescriptor,
    load_ops: [LoadOpInstance;17],
    store_ops: [StoreOp;17]
}

impl SinglePassAttacher {
    fn new() -> Self {
        SinglePassAttacher {
            color_attachment_count: 0,
            color_attachment_images: [AttachmentImage::None;16],
            depth_stencil_attachment: DepthStencilAttachmentDescriptor::None,
            load_ops: [LoadOpInstance::Load;17],
            store_ops: [StoreOp::Store;17]
        }
    }

    pub fn add_color_attachment<I, T>(&mut self, attachment: SinglePassAttachment<I, T>) where I: ColorAttachable, T: ClearValue<I> {

    }

    pub fn set_depth_stencil_attachment<I, T>(&mut self, attachment: SinglePassAttachment<I, T>) where I: DepthStencilAttachable, T: ClearValue<I> {

    }

    fn attachment_set(&self) -> SinglePassAttachmentSet {
        SinglePassAttachmentSet {
            color_attachment_images: &self.color_attachment_images[0..self.color_attachment_count],
            depth_stencil_attachment: &self.depth_stencil_attachment
        }
    }
}

#[derive(Hash)]
struct SinglePassAttachmentSet<'a> {
    color_attachment_images: &'a [AttachmentImage],
    depth_stencil_attachment: &'a DepthStencilAttachmentDescriptor
}

impl AttachmentSet for SinglePassAttachmentSet {
    fn color_attachments(&self) -> &[AttachmentImage] {
        &self.color_attachment_images
    }

    fn depth_stencil_attachment(&self) -> &DepthStencilAttachmentDescriptor {
        &self.depth_stencil_attachment
    }
}

impl<C, D> AttachmentSet for SinglePassAttachments<C, D> where C: ColorAttachable, D: DepthStencilAttachable {

}

impl<C, D> SinglePassAttachmentSet for SinglePassAttachments<C, D> where C: ColorAttachable, D: DepthStencilAttachable {
    fn draw_buffers(&self) -> &[DrawBuffer] {
        &COLOR_BUFFERS[0..self.color.len()]
    }

    fn load_all(&self, connection: &mut Connection) {
        self.color.load_all(connection);
        self.depth_stencil.load(connection);
    }

    fn store_all(&self, connection: &mut Connection) {
        self.color.store_all(connection);
        self.depth_stencil.store(connection);
    }
}

trait ColorAttachable {
    fn len(&self) -> usize;

    fn load_all(&self, connection: &mut Connection);

    fn store_all(&self, connection: &mut Connection);
}

impl<'a, I, T> ColorAttachable for SinglePassAttachment<'a, I, T> where I: AttachableImage, T: ClearValue<I> {
    fn len(&self) -> usize {
        1
    }

    fn load_all(&self, connection: &mut Connection) {

    }

    fn store_all(&self, connection: &mut Connection) {
        match self.store_op {
            StoreOp::DontCare => {
                let Connection(gl, _) = connection;

                // TODO: actually implement when web-sys supports invalidate_framebuffer
                // gl.invalidate_framebuffer(Gl::DRAW_FRAMEBUFFER, &[Gl::COLOR_ATTACHMENT0]);
                unimplemented!()
            },
            StoreOp::Store => ()
        }
    }
}

trait DepthStencilAttachable {
    fn load(&self, connection: &mut Connection);

    fn store(&self, connection: &mut Connection);

    fn attachment_descriptor(&self) -> Option<DepthStencilAttachmentDescriptor>;
}

pub struct SinglePassRenderTarget<C, D> {
    pub color: C,
    pub depth_stencil: D,
}

pub struct SinglePassAttachment<'a, I, T> where I: AttachableImage, T: ClearValue<I> {
    pub image: &'a mut I,
    pub load_op: LoadOp<T>,
    pub store_op: StoreOp,
}

pub enum AttachedImage {
    FloatColor(AttachedFloatColorImage),
}

pub enum LoadOp<T> {
    Load,
    Clear(T),
}

impl<T> LoadOp<T> {
    fn apply<I>(&self, attached_image: AttachedImage<I>) where I: AttachableImage, T: ClearValue<I> {
        match &self.load_op {
            LoadOp::Clear(value) => {
                value.clear(attached_image)
            },
            LoadOp::Load => ()
        }
    }
}

trait ClearValue<I> where I: AttachableImage {
    fn clear(&self, attachment: AttachedImage<I>);
}

trait AttachableImage {
    type
}

enum AttachedImage<I> {
    FloatColor {
        slot: u8,
        _marker: marker::PhantomData<I>
    },
    IntegerColor {
        slot: u8,
        _marker: marker::PhantomData<I>
    },
    UnsignedIntegerColor {

    },
    Depth {
        _marker: marker::PhantomData<I>
    },
    Stencil {
        _marker: marker::PhantomData<I>
    },
    DepthStencil {
        _marker: marker::PhantomData<I>
    }
}

impl ClearValue<RGBA8> for [f32;4] {
    fn clear(connection: &mut Connection) {
        let Connection(gl, _) = connection;

        gl.clear_buffer
    }
}

pub enum StoreOp {
    Store,
    DontCare,
}

pub struct SinglePassRenderPass<A, F> {
    attachments: A,
    f: F,
}

pub struct SinglePassRenderPassContext {
    connection: *mut Connection,
}

impl<A, F> GpuTask<Connection> for SinglePassRenderPass<A, F>
where
    A: SinglePassRenderPassDescriptor,
    F: FnOnce(&mut A::Attachments) -> GpuTask<SinglePassRenderPassContext>,
{
    type Output = ();

    fn progress(&mut self, connection: &mut Connection) -> Progress<Self::Output> {
        let Connection(gl, state) = connection;

        let framebuffer = state.framebuffer_cache_mut().get_or_create_mut(self.attachments.as_ref(), connection);

        framebuffer.bind(connection).set_draw_buffers(self.attachments.draw_buffers(), connection);

        self.attachments.load_all(connection);

        (self.f)().progress(&mut SinglePassRenderPassContext {
            connection: connection as *mut _
        });

        self.attachments.store_all(connection);

        Progress::Finished(())
    }
}
