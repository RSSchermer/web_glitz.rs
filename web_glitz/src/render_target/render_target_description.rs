use crate::render_pass::{Framebuffer, RenderPass, RenderPassContext, RenderPassId};
use crate::render_target::attachable_image_ref::AttachableImageData;
use crate::render_target::render_target_attachment::{
    ColorAttachmentEncoding, ColorAttachmentEncodingContext, LoadAction,
};
use crate::render_target::{
    ColorAttachmentDescription, DepthStencilAttachmentDescription, DepthStencilAttachmentEncoding,
    DepthStencilAttachmentEncodingContext, RenderTarget, StoreOp,
};
use crate::runtime::state::{AttachmentSet, DepthStencilAttachmentDescriptor, DrawBuffer};
use crate::task::{ContextId, GpuTask};
use std::cmp;
use std::hash::{Hash, Hasher};


/// Describes a render target against which may be used with a render pass task.
///
/// See [RenderingContext::create_render_pass] for details.
pub trait RenderTargetDescription {
    /// The type of framebuffer the render pass task may operate on.
    type Framebuffer;

    /// Called by [RenderingContext::create_render_pass], which will supply the `id`; creates a
    /// render pass which may be.
    ///
    /// # Panics
    ///
    /// Panics if any of the attached images belong to a [RenderingContext] that is not the context
    /// that supplied the `id`.
    ///
    /// Panics if the render pass task returned from `f` is associated with a different render pass.
    fn create_render_pass<F, T>(&mut self, id: RenderPassId, f: F) -> RenderPass<T>
    where
        F: FnOnce(&mut Self::Framebuffer) -> T,
        for<'a> T: GpuTask<RenderPassContext<'a>>;
}

impl<'a, T> RenderTargetDescription for &'a mut T where T: RenderTargetDescription {
    type Framebuffer = T::Framebuffer;

    fn create_render_pass<F, Rt>(&mut self, id: RenderPassId, f: F) -> RenderPass<Rt>
        where
            F: FnOnce(&mut Self::Framebuffer) -> Rt,
            for<'b> Rt: GpuTask<RenderPassContext<'b>>
    {
        (*self).create_render_pass(id, f)
    }
}

macro_rules! impl_render_target_description {
    ($C0:ident $(,$C:ident)*) => {
        impl<$C0 $(,$C)*> RenderTargetDescription for RenderTarget<($C0 $(,$C)*), ()>
        where
            $C0: ColorAttachmentDescription $(,$C: ColorAttachmentDescription)*
        {
            type Framebuffer = Framebuffer<($C0::Buffer $(,$C::Buffer)*), ()>;

            #[allow(non_snake_case, unused_mut, unused_parens)]
            fn create_render_pass<F, T>(&mut self, id: RenderPassId, f: F) -> RenderPass<T>
                where
                    F: FnOnce(&mut Self::Framebuffer) -> T,
                    for<'a> T: GpuTask<RenderPassContext<'a>>
            {
                let RenderPassId { id, context_id } = id;

                let mut render_target = CustomRenderTargetData {
                    load_ops: [LoadAction::Load; 17],
                    store_ops: [StoreOp::Store; 17],
                    color_count: 0,
                    color_attachments: [
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                    ],
                    depth_stencil_attachment: DepthStencilAttachmentDescriptor::None,
                };

                let ($C0  $(,$C)*) = &mut self.color;

                let mut context = ColorAttachmentEncodingContext {
                    render_pass_id: id,
                    buffer_index: 0,
                };

                let $C0 = $C0.encode(&mut context);

                let mut width = $C0.image.width;
                let mut height = $C0.image.height;

                let $C0 = {
                    let ColorAttachmentEncoding {
                        load_action,
                        store_op,
                        image,
                        buffer,
                        ..
                    } = $C0;

                    if image.context_id != context_id {
                        panic!("The color attachment at position `0` does not belong to the same \
                            context as the render pass.");
                    }

                    render_target.load_ops[0] = load_action;
                    render_target.store_ops[0] = store_op;
                    render_target.color_attachments[0] = Some(image);

                    buffer
                };

                let mut color_count = 1;

                $(
                    let mut context = ColorAttachmentEncodingContext {
                        render_pass_id: id,
                        buffer_index: color_count as i32,
                    };

                    let $C = $C.encode(&mut context);

                    width = cmp::min(width, $C.image.width);
                    height = cmp::min(height, $C.image.height);

                    let $C = {
                        let ColorAttachmentEncoding {
                            load_action,
                            store_op,
                            image,
                            buffer,
                            ..
                        } = $C;

                        if image.context_id != context_id {
                            panic!("The color attachment at position `{}` does not belong to the \
                                same context as the render pass.", color_count);
                        }

                        render_target.load_ops[color_count] = load_action;
                        render_target.store_ops[color_count] = store_op;
                        render_target.color_attachments[color_count] = Some(image);

                        buffer
                    };

                    color_count += 1;
                )*

                render_target.color_count = color_count;

                let mut framebuffer = Framebuffer {
                    color: ($C0  $(,$C)*),
                    depth_stencil: (),
                    dimensions: Some((width, height)),
                    context_id,
                    render_pass_id: id,
                    last_id: 0,
                };

                let task = f(&mut framebuffer);

                if let ContextId::Id(render_pass_id) = task.context_id() {
                    if render_pass_id != id {
                        panic!("The render pass task belongs to a different render pass.")
                    }
                }

                RenderPass {
                    id,
                    context_id,
                    render_target: RenderTargetData::Custom(render_target),
                    task
                }
            }
        }
    }
}

impl_render_target_description!(C0);
impl_render_target_description!(C0, C1);
impl_render_target_description!(C0, C1, C2);
impl_render_target_description!(C0, C1, C2, C3);
impl_render_target_description!(C0, C1, C2, C3, C4);
impl_render_target_description!(C0, C1, C2, C3, C4, C5);
impl_render_target_description!(C0, C1, C2, C3, C4, C5, C6);
impl_render_target_description!(C0, C1, C2, C3, C4, C5, C6, C7);
impl_render_target_description!(C0, C1, C2, C3, C4, C5, C6, C7, C8);
impl_render_target_description!(C0, C1, C2, C3, C4, C5, C6, C7, C8, C9);
impl_render_target_description!(C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10);
impl_render_target_description!(C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11);
impl_render_target_description!(C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12);
impl_render_target_description!(C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12, C13);
impl_render_target_description!(C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12, C13, C14);
impl_render_target_description!(
    C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12, C13, C14, C15
);

macro_rules! impl_render_target_description_depth_stencil {
    ($($C:ident),*) => {
        impl<$($C,)* Ds> RenderTargetDescription for RenderTarget<($($C),*), Ds>
        where
            $($C: ColorAttachmentDescription,)*
            Ds: DepthStencilAttachmentDescription
        {
            type Framebuffer = Framebuffer<($($C::Buffer),* ), Ds::Buffer>;

            #[allow(non_snake_case, unused_parens)]
            fn create_render_pass<F, T>(&mut self, id: RenderPassId, f: F) -> RenderPass<T>
                where
                    F: FnOnce(&mut Self::Framebuffer) -> T,
                    for<'a> T: GpuTask<RenderPassContext<'a>>
            {
                let RenderPassId { id, context_id } = id;

                let mut context = DepthStencilAttachmentEncodingContext {
                    render_pass_id: id,
                };

                let DepthStencilAttachmentEncoding {
                    load_action,
                    store_op,
                    depth_stencil_type,
                    image,
                    buffer,
                    ..
                } = self.depth_stencil.encode(&mut context);

                if image.context_id != context_id {
                    panic!("The depth-stencil attachment does not belong to the same context as \
                        the render pass.");
                }

                let mut width = image.width;
                let mut height = image.height;

                let mut render_target = CustomRenderTargetData {
                    load_ops: [LoadAction::Load; 17],
                    store_ops: [StoreOp::Store; 17],
                    color_count: 0,
                    color_attachments: [
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                    ],
                    depth_stencil_attachment: depth_stencil_type.descriptor(image),
                };

                render_target.load_ops[16] = load_action;
                render_target.store_ops[16] = store_op;

                let ($($C),*) = &mut self.color;
                let mut color_count = 0;

                $(
                    let mut context = ColorAttachmentEncodingContext {
                        render_pass_id: id,
                        buffer_index: color_count as i32,
                    };

                    let $C = $C.encode(&mut context);

                    width = cmp::min(width, $C.image.width);
                    height = cmp::min(height, $C.image.height);

                    let $C = {
                        let ColorAttachmentEncoding {
                            load_action,
                            store_op,
                            image,
                            buffer,
                            ..
                        } = $C;

                        if image.context_id != context_id {
                            panic!("The color attachment at position `{}` does not belong to the \
                                same context as the render pass.", color_count);
                        }

                        render_target.load_ops[color_count] = load_action;
                        render_target.store_ops[color_count] = store_op;
                        render_target.color_attachments[color_count] = Some(image);

                        buffer
                    };

                    color_count += 1;
                )*

                render_target.color_count = color_count;

                let mut framebuffer = Framebuffer {
                    color: ($($C),*),
                    depth_stencil: buffer,
                    dimensions: Some((width, height)),
                    context_id,
                    render_pass_id: id,
                    last_id: 0,
                };

                let task = f(&mut framebuffer);

                if let ContextId::Id(render_pass_id) = task.context_id() {
                    if render_pass_id != id {
                        panic!("The render pass task belongs to a different render pass.")
                    }
                }

                RenderPass {
                    id,
                    context_id,
                    render_target: RenderTargetData::Custom(render_target),
                    task
                }
            }
        }
    }
}

impl_render_target_description_depth_stencil!(C0);
impl_render_target_description_depth_stencil!(C0, C1);
impl_render_target_description_depth_stencil!(C0, C1, C2);
impl_render_target_description_depth_stencil!(C0, C1, C2, C3);
impl_render_target_description_depth_stencil!(C0, C1, C2, C3, C4);
impl_render_target_description_depth_stencil!(C0, C1, C2, C3, C4, C5);
impl_render_target_description_depth_stencil!(C0, C1, C2, C3, C4, C5, C6);
impl_render_target_description_depth_stencil!(C0, C1, C2, C3, C4, C5, C6, C7);
impl_render_target_description_depth_stencil!(C0, C1, C2, C3, C4, C5, C6, C7, C8);
impl_render_target_description_depth_stencil!(C0, C1, C2, C3, C4, C5, C6, C7, C8, C9);
impl_render_target_description_depth_stencil!(C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10);
impl_render_target_description_depth_stencil!(C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11);
impl_render_target_description_depth_stencil!(
    C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12
);
impl_render_target_description_depth_stencil!(
    C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12, C13
);
impl_render_target_description_depth_stencil!(
    C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12, C13, C14
);
impl_render_target_description_depth_stencil!(
    C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12, C13, C14, C15
);

pub(crate) enum RenderTargetData {
    Default,
    Custom(CustomRenderTargetData),
}

pub(crate) struct CustomRenderTargetData {
    pub(crate) load_ops: [LoadAction; 17],
    pub(crate) store_ops: [StoreOp; 17],
    pub(crate) color_count: usize,
    pub(crate) color_attachments: [Option<AttachableImageData>; 16],
    pub(crate) depth_stencil_attachment: DepthStencilAttachmentDescriptor,
}

impl CustomRenderTargetData {
    pub(crate) fn draw_buffers(&self) -> &[DrawBuffer] {
        const DRAW_BUFFERS_SEQUENTIAL: [DrawBuffer; 16] = [
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

        &DRAW_BUFFERS_SEQUENTIAL[0..self.color_count]
    }
}

impl Hash for CustomRenderTargetData {
    fn hash<H>(&self, hasher: &mut H)
    where
        H: Hasher,
    {
        self.color_attachments().hash(hasher);
        self.depth_stencil_attachment().hash(hasher);
    }
}

impl AttachmentSet for CustomRenderTargetData {
    fn color_attachments(&self) -> &[Option<AttachableImageData>] {
        &self.color_attachments[0..self.color_count]
    }

    fn depth_stencil_attachment(&self) -> &DepthStencilAttachmentDescriptor {
        &self.depth_stencil_attachment
    }
}
