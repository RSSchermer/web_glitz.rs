use crate::render_target::render_target_attachment::{LoadAction, StoreOp, FloatAttachment, IntegerAttachment, UnsignedIntegerAttachment, DepthStencilAttachment, DepthAttachment, StencilAttachment, AttachableImageRef, AttachableImageData};
use crate::runtime::state::{DepthStencilAttachmentDescriptor, DrawBuffer, AttachmentSet};
use crate::render_pass::{RenderBuffer, FloatBuffer, IntegerBuffer, UnsignedIntegerBuffer, DepthStencilBuffer, DepthBuffer, StencilBuffer, Framebuffer};
use crate::render_target::attachable_image_ref::AsAttachableImageRef;
use crate::image::format::{FloatRenderable, IntegerRenderable, UnsignedIntegerRenderable, DepthStencilRenderable, DepthRenderable, StencilRenderable};
use std::cmp;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, Copy, PartialEq, Hash)]
pub struct MaxColorAttachmentsExceeded;

pub struct EncodingContext {
    pub(crate) context_id: usize,
    pub(crate) render_pass_id: usize,
}

pub struct RenderTargetEncoder<'a, C, Ds> {
    context: &'a mut EncodingContext,
    color: C,
    depth_stencil: Ds,
    data: CustomRenderTargetData,
    dimensions: Option<(u32, u32)>,
}

impl<'a> RenderTargetEncoder<'a, (), ()> {
    pub fn new(context: &'a mut EncodingContext) -> Self {
        RenderTargetEncoder {
            context,
            color: (),
            depth_stencil: (),
            data: CustomRenderTargetData {
                load_ops: [LoadAction::Load; 17],
                store_ops: [StoreOp::Store; 17],
                color_count: 0,
                color_attachments: [
                    None, None, None, None, None, None, None, None, None, None, None, None, None,
                    None, None, None,
                ],
                depth_stencil_attachment: DepthStencilAttachmentDescriptor::None,
            },
            dimensions: None,
        }
    }
}

impl<'a, C, Ds> RenderTargetEncoder<'a, C, Ds> {
    pub fn add_color_float_buffer<'b, I>(
        mut self,
        attachment: FloatAttachment<I>,
    ) -> Result<RenderTargetEncoder<'a, (FloatBuffer<I::Format>, C), Ds>, MaxColorAttachmentsExceeded>
        where
            I: AsAttachableImageRef<'b>,
            I::Format: FloatRenderable,
    {
        let c = self.data.color_count;

        if c > 15 {
            Err(MaxColorAttachmentsExceeded)
        } else {
            let image_descriptor = attachment.image.as_attachable_image_ref().into_data();
            let width = image_descriptor.width();
            let height = image_descriptor.height();

            self.data.color_attachments[c] = Some(image_descriptor);
            self.data.load_ops[c] = attachment.load_op.as_action(c as i32);
            self.data.store_ops[c] = attachment.store_op;
            self.data.color_count += 1;

            let dimensions = if let Some((framebuffer_width, framebuffer_height)) = self.dimensions
            {
                Some((
                    cmp::min(framebuffer_width, width),
                    cmp::min(framebuffer_height, height),
                ))
            } else {
                Some((width, height))
            };

            Ok(RenderTargetEncoder {
                color: (
                    FloatBuffer::new(self.context.render_pass_id, c as i32, width, height),
                    self.color,
                ),
                context: self.context,
                depth_stencil: self.depth_stencil,
                data: self.data,
                dimensions,
            })
        }
    }

    pub fn add_color_integer_buffer<'b, I>(
        mut self,
        attachment: IntegerAttachment<I>,
    ) -> Result<
        RenderTargetEncoder<'a, (IntegerBuffer<I::Format>, C), Ds>,
        MaxColorAttachmentsExceeded,
    >
        where
            I: AsAttachableImageRef<'b>,
            I::Format: IntegerRenderable,
    {
        let c = self.data.color_count;

        if c > 15 {
            Err(MaxColorAttachmentsExceeded)
        } else {
            let image_descriptor = attachment.image.as_attachable_image_ref().into_data();
            let width = image_descriptor.width();
            let height = image_descriptor.height();

            self.data.color_attachments[c] = Some(image_descriptor);
            self.data.load_ops[c] = attachment.load_op.as_action(c as i32);
            self.data.store_ops[c] = attachment.store_op;
            self.data.color_count += 1;

            let dimensions = if let Some((framebuffer_width, framebuffer_height)) = self.dimensions
            {
                Some((
                    cmp::min(framebuffer_width, width),
                    cmp::min(framebuffer_height, height),
                ))
            } else {
                Some((width, height))
            };

            Ok(RenderTargetEncoder {
                color: (
                    IntegerBuffer::new(self.context.render_pass_id, c as i32, width, height),
                    self.color,
                ),
                context: self.context,
                depth_stencil: self.depth_stencil,
                data: self.data,
                dimensions,
            })
        }
    }

    pub fn add_color_unsigned_integer_buffer<'b, I>(
        mut self,
        attachment: UnsignedIntegerAttachment<I>,
    ) -> Result<
        RenderTargetEncoder<'a, (UnsignedIntegerBuffer<I::Format>, C), Ds>,
        MaxColorAttachmentsExceeded,
    >
        where
            I: AsAttachableImageRef<'b>,
            I::Format: UnsignedIntegerRenderable,
    {
        let c = self.data.color_count;

        if c > 15 {
            Err(MaxColorAttachmentsExceeded)
        } else {
            let image_descriptor = attachment.image.as_attachable_image_ref().into_data();
            let width = image_descriptor.width();
            let height = image_descriptor.height();

            self.data.color_attachments[c] = Some(image_descriptor);
            self.data.load_ops[c] = attachment.load_op.as_action(c as i32);
            self.data.store_ops[c] = attachment.store_op;
            self.data.color_count += 1;

            let dimensions = if let Some((framebuffer_width, framebuffer_height)) = self.dimensions
            {
                Some((
                    cmp::min(framebuffer_width, width),
                    cmp::min(framebuffer_height, height),
                ))
            } else {
                Some((width, height))
            };

            Ok(RenderTargetEncoder {
                color: (
                    UnsignedIntegerBuffer::new(
                        self.context.render_pass_id,
                        c as i32,
                        width,
                        height,
                    ),
                    self.color,
                ),
                context: self.context,
                depth_stencil: self.depth_stencil,
                data: self.data,
                dimensions,
            })
        }
    }

    pub fn set_depth_stencil_buffer<'b, I>(
        mut self,
        attachment: DepthStencilAttachment<I>,
    ) -> RenderTargetEncoder<'a, C, DepthStencilBuffer<I::Format>>
        where
            I: AsAttachableImageRef<'b>,
            I::Format: DepthStencilRenderable,
    {
        let image_descriptor = attachment.image.as_attachable_image_ref();
        let width = image_descriptor.width();
        let height = image_descriptor.height();

        self.data.load_ops[16] = attachment.load_op.as_action();
        self.data.store_ops[16] = attachment.store_op;
        self.data.depth_stencil_attachment =
            DepthStencilAttachmentDescriptor::DepthStencil(image_descriptor.into_data());

        let dimensions = if let Some((framebuffer_width, framebuffer_height)) = self.dimensions {
            Some((
                cmp::min(framebuffer_width, width),
                cmp::min(framebuffer_height, height),
            ))
        } else {
            Some((width, height))
        };

        RenderTargetEncoder {
            color: self.color,
            depth_stencil: DepthStencilBuffer::new(self.context.render_pass_id, width, height),
            data: self.data,
            context: self.context,
            dimensions,
        }
    }

    pub fn set_depth_stencil_depth_buffer<'b, I>(
        mut self,
        attachment: DepthAttachment<I>,
    ) -> RenderTargetEncoder<'a, C, DepthBuffer<I::Format>>
        where
            I: AsAttachableImageRef<'b>,
            I::Format: DepthRenderable,
    {
        let image_descriptor = attachment.image.as_attachable_image_ref();
        let width = image_descriptor.width();
        let height = image_descriptor.height();

        self.data.load_ops[16] = attachment.load_op.as_action();
        self.data.store_ops[16] = attachment.store_op;
        self.data.depth_stencil_attachment =
            DepthStencilAttachmentDescriptor::Depth(image_descriptor.into_data());

        let dimensions = if let Some((framebuffer_width, framebuffer_height)) = self.dimensions {
            Some((
                cmp::min(framebuffer_width, width),
                cmp::min(framebuffer_height, height),
            ))
        } else {
            Some((width, height))
        };

        RenderTargetEncoder {
            color: self.color,
            depth_stencil: DepthBuffer::new(self.context.render_pass_id, width, height),
            data: self.data,
            context: self.context,
            dimensions,
        }
    }

    pub fn set_depth_stencil_stencil_buffer<'b, I>(
        mut self,
        attachment: StencilAttachment<I>,
    ) -> RenderTargetEncoder<'a, C, StencilBuffer<I::Format>>
        where
            I: AsAttachableImageRef<'b>,
            I::Format: StencilRenderable,
    {
        let image_descriptor = attachment.image.as_attachable_image_ref();
        let width = image_descriptor.width();
        let height = image_descriptor.height();

        self.data.load_ops[16] = attachment.load_op.as_action();
        self.data.store_ops[16] = attachment.store_op;
        self.data.depth_stencil_attachment =
            DepthStencilAttachmentDescriptor::Stencil(image_descriptor.into_data());

        let dimensions = if let Some((framebuffer_width, framebuffer_height)) = self.dimensions {
            Some((
                cmp::min(framebuffer_width, width),
                cmp::min(framebuffer_height, height),
            ))
        } else {
            Some((width, height))
        };

        RenderTargetEncoder {
            color: self.color,
            depth_stencil: StencilBuffer::new(self.context.render_pass_id, width, height),
            data: self.data,
            context: self.context,
            dimensions,
        }
    }
}

macro_rules! nest_pairs {
    ($head:tt) => ($head);
    ($head:tt, $($tail:tt),*) => (($head, nest_pairs!($($tail),*)));
}

macro_rules! nest_pairs_reverse {
    ([$head:tt] $($reverse:tt)*) => (nest_pairs!($head, $($reverse),*));
    ([$head:tt, $($tail:tt),*] $($reverse:tt)*) => {
        nest_pairs_reverse!([$($tail),*] $head $($reverse)*)
    }
}

macro_rules! generate_encoder_finish {
    ($($C:ident),*) => {
        impl<'a, $($C),*> RenderTargetEncoder<'a, nest_pairs_reverse!([(), $($C),*]), ()>
            where $($C: RenderBuffer),*
        {
            pub fn finish(self) -> RenderTargetEncoding<'a, Framebuffer<($($C),*), ()>> {
                #[allow(non_snake_case)]
                let nest_pairs_reverse!([_, $($C),*]) = self.color;

                RenderTargetEncoding {
                    framebuffer: Framebuffer {
                        color: ($($C),*),
                        depth_stencil: (),
                        context_id: self.context.context_id,
                        render_pass_id: self.context.render_pass_id,
                        last_id: 0,
                        dimensions: self.dimensions
                    },
                    context: self.context,
                    data: RenderTargetEncodingData::FBO(self.data),
                }
            }
        }

        impl<'a, $($C),*, Ds> RenderTargetEncoder<'a, nest_pairs_reverse!([(), $($C),*]), Ds>
        where
            $($C: RenderBuffer),*,
            Ds: RenderBuffer
        {
            pub fn finish(self) -> RenderTargetEncoding<'a, Framebuffer<($($C),*), Ds>> {
                #[allow(non_snake_case)]
                let nest_pairs_reverse!([_, $($C),*]) = self.color;

                RenderTargetEncoding {
                    framebuffer: Framebuffer {
                        color: ($($C),*),
                        depth_stencil: self.depth_stencil,
                        context_id: self.context.context_id,
                        render_pass_id: self.context.render_pass_id,
                        last_id: 0,
                        dimensions: self.dimensions
                    },
                    context: self.context,
                    data: RenderTargetEncodingData::FBO(self.data),
                }
            }
        }
    }
}

generate_encoder_finish!(C0);
generate_encoder_finish!(C0, C1);
generate_encoder_finish!(C0, C1, C2);
generate_encoder_finish!(C0, C1, C2, C3);
generate_encoder_finish!(C0, C1, C2, C3, C4);
generate_encoder_finish!(C0, C1, C2, C3, C4, C5);
generate_encoder_finish!(C0, C1, C2, C3, C4, C5, C6);
generate_encoder_finish!(C0, C1, C2, C3, C4, C5, C6, C7);
generate_encoder_finish!(C0, C1, C2, C3, C4, C5, C6, C7, C8);
generate_encoder_finish!(C0, C1, C2, C3, C4, C5, C6, C7, C8, C9);
generate_encoder_finish!(C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10);
generate_encoder_finish!(C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11);
generate_encoder_finish!(C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12);
generate_encoder_finish!(C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12, C13);
generate_encoder_finish!(C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12, C13, C14);
generate_encoder_finish!(C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12, C13, C14, C15);

pub struct RenderTargetEncoding<'a, F> {
    #[allow(dead_code)]
    pub(crate) context: &'a mut EncodingContext,
    pub(crate) framebuffer: F,
    pub(crate) data: RenderTargetData,
}

impl<'a, F> RenderTargetEncoding<'a, Framebuffer<Vec<FloatBuffer<F>>, ()>>
    where
        F: FloatRenderable,
{
    pub fn from_float_colors<C, I>(context: &'a mut EncodingContext, colors: C) -> Self
        where
            C: IntoIterator<Item = FloatAttachment<I>>,
            for<'b> I: AsAttachableImageRef<'b, Format = F>,
    {
        let mut color_attachments = [
            None, None, None, None, None, None, None, None, None, None, None, None, None, None,
            None, None,
        ];
        let mut load_ops = [LoadAction::Load; 17];
        let mut store_ops = [StoreOp::Store; 17];
        let mut buffers = Vec::new();
        let mut framebuffer_dimensions = None;

        for (index, target) in colors.into_iter().enumerate() {
            let attachment = target.image.into_attachable_image_ref();
            let attachment_width = attachment.width();
            let attachment_height = attachment.height();

            buffers.push(FloatBuffer::new(
                context.render_pass_id,
                index as i32,
                attachment_width,
                attachment_height,
            ));

            if let Some((width, height)) = framebuffer_dimensions {
                framebuffer_dimensions = Some((
                    cmp::min(width, attachment_width),
                    cmp::min(height, attachment_height),
                ));
            } else {
                framebuffer_dimensions = Some((attachment_width, attachment_height));
            }

            color_attachments[index] = Some(attachment);
            load_ops[index] = target.load_op.as_action(index as i32);
            store_ops[index] = target.store_op;
        }

        let color_count = buffers.len();

        RenderTargetEncoding {
            framebuffer: Framebuffer {
                color: buffers,
                depth_stencil: (),
                context_id: context.context_id,
                render_pass_id: context.render_pass_id,
                last_id: 0,
                dimensions: framebuffer_dimensions,
            },
            data: RenderTargetData::Custom(CustomRenderTargetData {
                load_ops,
                store_ops,
                color_count,
                color_attachments,
                depth_stencil_attachment: DepthStencilAttachmentDescriptor::None,
            }),
            context,
        }
    }
}

impl<'a, F0, F1> RenderTargetEncoding<'a, Framebuffer<Vec<FloatBuffer<F0>>, DepthStencilBuffer<F1>>>
    where
        F0: FloatRenderable,
        F1: DepthStencilRenderable,
{
    pub fn from_float_colors_and_depth_stencil<C, I, Ds>(
        context: &'a mut EncodingContext,
        colors: C,
        depth_stencil: DepthStencilAttachment<Ds>,
    ) -> Self
        where
            C: IntoIterator<Item = FloatAttachment<I>>,
            for<'b> I: AsAttachableImageRef<'b, Format = F0>,
            for<'b> Ds: AsAttachableImageRef<'b, Format = F1>,
    {
        let mut color_attachments = [
            None, None, None, None, None, None, None, None, None, None, None, None, None, None,
            None, None,
        ];
        let mut load_ops = [LoadAction::Load; 17];
        let mut store_ops = [StoreOp::Store; 17];
        let mut buffers = Vec::new();
        let mut framebuffer_dimensions = None;

        for (index, target) in colors.into_iter().enumerate() {
            let attachment = target.image.into_attachable_image_ref();
            let attachment_width = attachment.width();
            let attachment_height = attachment.height();

            buffers.push(FloatBuffer::new(
                context.render_pass_id,
                index as i32,
                attachment_width,
                attachment_height,
            ));

            if let Some((width, height)) = framebuffer_dimensions {
                framebuffer_dimensions = Some((
                    cmp::min(width, attachment_width),
                    cmp::min(height, attachment_height),
                ));
            } else {
                framebuffer_dimensions = Some((attachment_width, attachment_height));
            }

            color_attachments[index] = Some(attachment);
            load_ops[index] = target.load_op.as_action(index as i32);
            store_ops[index] = target.store_op;
        }

        let color_count = buffers.len();

        load_ops[16] = depth_stencil.load_op.as_action();
        store_ops[16] = depth_stencil.store_op;

        let depth_stencil_attachment = depth_stencil.image.as_attachable_image_ref();
        let depth_stencil_width = depth_stencil_attachment.width();
        let depth_stencil_height = depth_stencil_attachment.height();

        if let Some((width, height)) = framebuffer_dimensions {
            framebuffer_dimensions = Some((
                cmp::min(width, depth_stencil_width),
                cmp::min(height, depth_stencil_height),
            ));
        } else {
            framebuffer_dimensions = Some((depth_stencil_width, depth_stencil_height));
        }

        RenderTargetEncoding {
            framebuffer: Framebuffer {
                color: buffers,
                depth_stencil: DepthStencilBuffer::new(
                    context.render_pass_id,
                    depth_stencil_width,
                    depth_stencil_height,
                ),
                context_id: context.context_id,
                render_pass_id: context.render_pass_id,
                last_id: 0,
                dimensions: framebuffer_dimensions,
            },
            data: RenderTargetData::Custom(CustomRenderTargetData {
                load_ops,
                store_ops,
                color_count,
                color_attachments,
                depth_stencil_attachment: DepthStencilAttachmentDescriptor::DepthStencil(
                    depth_stencil_attachment,
                ),
            }),
            context,
        }
    }
}

impl<'a, F0, F1> RenderTargetEncoding<'a, Framebuffer<Vec<FloatBuffer<F0>>, DepthBuffer<F1>>>
    where
        F0: FloatRenderable,
        F1: DepthRenderable,
{
    pub fn from_float_colors_and_depth<C, I, Ds>(
        context: &'a mut EncodingContext,
        colors: C,
        depth: DepthAttachment<Ds>,
    ) -> Self
        where
            C: IntoIterator<Item = FloatAttachment<I>>,
            for<'b> I: AsAttachableImageRef<'b, Format = F0>,
            for<'b> Ds: AsAttachableImageRef<'b, Format = F1>,
    {
        let mut color_attachments = [
            None, None, None, None, None, None, None, None, None, None, None, None, None, None,
            None, None,
        ];
        let mut load_ops = [LoadAction::Load; 17];
        let mut store_ops = [StoreOp::Store; 17];
        let mut buffers = Vec::new();
        let mut framebuffer_dimensions = None;

        for (index, target) in colors.into_iter().enumerate() {
            let attachment = target.image.into_attachable_image_ref();
            let attachment_width = attachment.width();
            let attachment_height = attachment.height();

            buffers.push(FloatBuffer::new(
                context.render_pass_id,
                index as i32,
                attachment_width,
                attachment_height,
            ));

            if let Some((width, height)) = framebuffer_dimensions {
                framebuffer_dimensions = Some((
                    cmp::min(width, attachment_width),
                    cmp::min(height, attachment_height),
                ));
            } else {
                framebuffer_dimensions = Some((attachment_width, attachment_height));
            }

            color_attachments[index] = Some(attachment);
            load_ops[index] = target.load_op.as_action(index as i32);
            store_ops[index] = target.store_op;
        }

        let color_count = buffers.len();

        load_ops[16] = depth.load_op.as_action();
        store_ops[16] = depth.store_op;

        let depth_attachment = depth.image.as_attachable_image_ref();
        let depth_width = depth_attachment.width();
        let depth_height = depth_attachment.height();

        if let Some((width, height)) = framebuffer_dimensions {
            framebuffer_dimensions =
                Some((cmp::min(width, depth_width), cmp::min(height, depth_height)));
        } else {
            framebuffer_dimensions = Some((depth_width, depth_height));
        }

        RenderTargetEncoding {
            framebuffer: Framebuffer {
                color: buffers,
                depth_stencil: DepthBuffer::new(context.render_pass_id, depth_width, depth_height),
                context_id: context.context_id,
                render_pass_id: context.render_pass_id,
                last_id: 0,
                dimensions: framebuffer_dimensions,
            },
            data: RenderTargetData::Custom(CustomRenderTargetData {
                load_ops,
                store_ops,
                color_count,
                color_attachments,
                depth_stencil_attachment: DepthStencilAttachmentDescriptor::Depth(depth_attachment),
            }),
            context,
        }
    }
}

impl<'a, F0, F1> RenderTargetEncoding<'a, Framebuffer<Vec<FloatBuffer<F0>>, StencilBuffer<F1>>>
    where
        F0: FloatRenderable,
        F1: StencilRenderable,
{
    pub fn from_float_colors_and_stencil<C, I, Ds>(
        context: &'a mut EncodingContext,
        colors: C,
        stencil: StencilAttachment<Ds>,
    ) -> Self
        where
            C: IntoIterator<Item = FloatAttachment<I>>,
            for<'b> I: AsAttachableImageRef<'b, Format = F0>,
            for<'b> Ds: AsAttachableImageRef<'b, Format = F1>,
    {
        let mut color_attachments = [
            None, None, None, None, None, None, None, None, None, None, None, None, None, None,
            None, None,
        ];
        let mut load_ops = [LoadAction::Load; 17];
        let mut store_ops = [StoreOp::Store; 17];
        let mut buffers = Vec::new();
        let mut framebuffer_dimensions = None;

        for (index, target) in colors.into_iter().enumerate() {
            let attachment = target.image.into_attachable_image_ref();
            let attachment_width = attachment.width();
            let attachment_height = attachment.height();

            buffers.push(FloatBuffer::new(
                context.render_pass_id,
                index as i32,
                attachment_width,
                attachment_height,
            ));

            if let Some((width, height)) = framebuffer_dimensions {
                framebuffer_dimensions = Some((
                    cmp::min(width, attachment_width),
                    cmp::min(height, attachment_height),
                ));
            } else {
                framebuffer_dimensions = Some((attachment_width, attachment_height));
            }

            color_attachments[index] = Some(attachment);
            load_ops[index] = target.load_op.as_action(index as i32);
            store_ops[index] = target.store_op;
        }

        let color_count = buffers.len();

        load_ops[16] = stencil.load_op.as_action();
        store_ops[16] = stencil.store_op;

        let stencil_attachment = stencil.image.as_attachable_image_ref();
        let stencil_width = stencil_attachment.width();
        let stencil_height = stencil_attachment.height();

        if let Some((width, height)) = framebuffer_dimensions {
            framebuffer_dimensions = Some((
                cmp::min(width, stencil_width),
                cmp::min(height, stencil_height),
            ));
        } else {
            framebuffer_dimensions = Some((stencil_width, stencil_height));
        }

        RenderTargetEncoding {
            framebuffer: Framebuffer {
                color: buffers,
                depth_stencil: StencilBuffer::new(
                    context.render_pass_id,
                    stencil_width,
                    stencil_height,
                ),
                context_id: context.context_id,
                render_pass_id: context.render_pass_id,
                last_id: 0,
                dimensions: framebuffer_dimensions,
            },
            data: RenderTargetData::Custom(CustomRenderTargetData {
                load_ops,
                store_ops,
                color_count,
                color_attachments,
                depth_stencil_attachment: DepthStencilAttachmentDescriptor::Stencil(
                    stencil_attachment,
                ),
            }),
            context,
        }
    }
}

impl<'a, F> RenderTargetEncoding<'a, Framebuffer<Vec<IntegerBuffer<F>>, ()>>
    where
        F: IntegerRenderable,
{
    pub fn from_integer_colors<C, I>(context: &'a mut EncodingContext, colors: C) -> Self
        where
            C: IntoIterator<Item = IntegerAttachment<I>>,
            for<'b> I: AsAttachableImageRef<'b, Format = F>,
    {
        let mut color_attachments = [
            None, None, None, None, None, None, None, None, None, None, None, None, None, None,
            None, None,
        ];
        let mut load_ops = [LoadAction::Load; 17];
        let mut store_ops = [StoreOp::Store; 17];
        let mut buffers = Vec::new();
        let mut framebuffer_dimensions = None;

        for (index, target) in colors.into_iter().enumerate() {
            let attachment = target.image.into_attachable_image_ref();
            let attachment_width = attachment.width();
            let attachment_height = attachment.height();

            buffers.push(IntegerBuffer::new(
                context.render_pass_id,
                index as i32,
                attachment_width,
                attachment_height,
            ));

            if let Some((width, height)) = framebuffer_dimensions {
                framebuffer_dimensions = Some((
                    cmp::min(width, attachment_width),
                    cmp::min(height, attachment_height),
                ));
            } else {
                framebuffer_dimensions = Some((attachment_width, attachment_height));
            }

            color_attachments[index] = Some(attachment);
            load_ops[index] = target.load_op.as_action(index as i32);
            store_ops[index] = target.store_op;
        }

        let color_count = buffers.len();

        RenderTargetEncoding {
            framebuffer: Framebuffer {
                color: buffers,
                depth_stencil: (),
                context_id: context.context_id,
                render_pass_id: context.render_pass_id,
                last_id: 0,
                dimensions: framebuffer_dimensions,
            },
            data: RenderTargetData::Custom(CustomRenderTargetData {
                load_ops,
                store_ops,
                color_count,
                color_attachments,
                depth_stencil_attachment: DepthStencilAttachmentDescriptor::None,
            }),
            context,
        }
    }
}

impl<'a, F0, F1>
RenderTargetEncoding<'a, Framebuffer<Vec<IntegerBuffer<F0>>, DepthStencilBuffer<F1>>>
    where
        F0: IntegerRenderable,
        F1: DepthStencilRenderable,
{
    pub fn from_integer_colors_and_depth_stencil<C, I, Ds>(
        context: &'a mut EncodingContext,
        colors: C,
        depth_stencil: DepthStencilAttachment<Ds>,
    ) -> Self
        where
            C: IntoIterator<Item = IntegerAttachment<I>>,
            for<'b> I: AsAttachableImageRef<'b, Format = F0>,
            for<'b> Ds: AsAttachableImageRef<'b, Format = F1>,
    {
        let mut color_attachments = [
            None, None, None, None, None, None, None, None, None, None, None, None, None, None,
            None, None,
        ];
        let mut load_ops = [LoadAction::Load; 17];
        let mut store_ops = [StoreOp::Store; 17];
        let mut buffers = Vec::new();
        let mut framebuffer_dimensions = None;

        for (index, target) in colors.into_iter().enumerate() {
            let attachment = target.image.into_attachable_image_ref();
            let attachment_width = attachment.width();
            let attachment_height = attachment.height();

            buffers.push(IntegerBuffer::new(
                context.render_pass_id,
                index as i32,
                attachment_width,
                attachment_height,
            ));

            if let Some((width, height)) = framebuffer_dimensions {
                framebuffer_dimensions = Some((
                    cmp::min(width, attachment_width),
                    cmp::min(height, attachment_height),
                ));
            } else {
                framebuffer_dimensions = Some((attachment_width, attachment_height));
            }

            color_attachments[index] = Some(attachment);
            load_ops[index] = target.load_op.as_action(index as i32);
            store_ops[index] = target.store_op;
        }

        let color_count = buffers.len();

        load_ops[16] = depth_stencil.load_op.as_action();
        store_ops[16] = depth_stencil.store_op;

        let depth_stencil_attachment = depth_stencil.image.as_attachable_image_ref();
        let depth_stencil_width = depth_stencil_attachment.width();
        let depth_stencil_height = depth_stencil_attachment.height();

        if let Some((width, height)) = framebuffer_dimensions {
            framebuffer_dimensions = Some((
                cmp::min(width, depth_stencil_width),
                cmp::min(height, depth_stencil_height),
            ));
        } else {
            framebuffer_dimensions = Some((depth_stencil_width, depth_stencil_height));
        }

        RenderTargetEncoding {
            framebuffer: Framebuffer {
                color: buffers,
                depth_stencil: DepthStencilBuffer::new(
                    context.render_pass_id,
                    depth_stencil_width,
                    depth_stencil_height,
                ),
                context_id: context.context_id,
                render_pass_id: context.render_pass_id,
                last_id: 0,
                dimensions: framebuffer_dimensions,
            },
            data: RenderTargetData::Custom(CustomRenderTargetData {
                load_ops,
                store_ops,
                color_count,
                color_attachments,
                depth_stencil_attachment: DepthStencilAttachmentDescriptor::DepthStencil(
                    depth_stencil_attachment,
                ),
            }),
            context,
        }
    }
}

impl<'a, F0, F1> RenderTargetEncoding<'a, Framebuffer<Vec<IntegerBuffer<F0>>, DepthBuffer<F1>>>
    where
        F0: IntegerRenderable,
        F1: DepthRenderable,
{
    pub fn from_integer_colors_and_depth<C, I, Ds>(
        context: &'a mut EncodingContext,
        colors: C,
        depth: DepthAttachment<Ds>,
    ) -> Self
        where
            C: IntoIterator<Item = IntegerAttachment<I>>,
            for<'b> I: AsAttachableImageRef<'b, Format = F0>,
            for<'b> Ds: AsAttachableImageRef<'b, Format = F1>,
    {
        let mut color_attachments = [
            None, None, None, None, None, None, None, None, None, None, None, None, None, None,
            None, None,
        ];
        let mut load_ops = [LoadAction::Load; 17];
        let mut store_ops = [StoreOp::Store; 17];
        let mut buffers = Vec::new();
        let mut framebuffer_dimensions = None;

        for (index, target) in colors.into_iter().enumerate() {
            let attachment = target.image.into_attachable_image_ref();
            let attachment_width = attachment.width();
            let attachment_height = attachment.height();

            buffers.push(IntegerBuffer::new(
                context.render_pass_id,
                index as i32,
                attachment_width,
                attachment_height,
            ));

            if let Some((width, height)) = framebuffer_dimensions {
                framebuffer_dimensions = Some((
                    cmp::min(width, attachment_width),
                    cmp::min(height, attachment_height),
                ));
            } else {
                framebuffer_dimensions = Some((attachment_width, attachment_height));
            }

            color_attachments[index] = Some(attachment);
            load_ops[index] = target.load_op.as_action(index as i32);
            store_ops[index] = target.store_op;
        }

        let color_count = buffers.len();

        load_ops[16] = depth.load_op.as_action();
        store_ops[16] = depth.store_op;

        let depth_attachment = depth.image.as_attachable_image_ref();
        let depth_width = depth_attachment.width();
        let depth_height = depth_attachment.height();

        if let Some((width, height)) = framebuffer_dimensions {
            framebuffer_dimensions =
                Some((cmp::min(width, depth_width), cmp::min(height, depth_height)));
        } else {
            framebuffer_dimensions = Some((depth_width, depth_height));
        }

        RenderTargetEncoding {
            framebuffer: Framebuffer {
                color: buffers,
                depth_stencil: DepthBuffer::new(context.render_pass_id, depth_width, depth_height),
                context_id: context.context_id,
                render_pass_id: context.render_pass_id,
                last_id: 0,
                dimensions: framebuffer_dimensions,
            },
            data: RenderTargetData::Custom(CustomRenderTargetData {
                load_ops,
                store_ops,
                color_count,
                color_attachments,
                depth_stencil_attachment: DepthStencilAttachmentDescriptor::Depth(depth_attachment),
            }),
            context,
        }
    }
}

impl<'a, F0, F1> RenderTargetEncoding<'a, Framebuffer<Vec<IntegerBuffer<F0>>, StencilBuffer<F1>>>
    where
        F0: IntegerRenderable,
        F1: StencilRenderable,
{
    pub fn from_integer_colors_and_stencil<C, I, Ds>(
        context: &'a mut EncodingContext,
        colors: C,
        stencil: StencilAttachment<Ds>,
    ) -> Self
        where
            C: IntoIterator<Item = IntegerAttachment<I>>,
            for<'b> I: AsAttachableImageRef<'b, Format = F0>,
            for<'b> Ds: AsAttachableImageRef<'b, Format = F1>,
    {
        let mut color_attachments = [
            None, None, None, None, None, None, None, None, None, None, None, None, None, None,
            None, None,
        ];
        let mut load_ops = [LoadAction::Load; 17];
        let mut store_ops = [StoreOp::Store; 17];
        let mut buffers = Vec::new();
        let mut framebuffer_dimensions = None;

        for (index, target) in colors.into_iter().enumerate() {
            let attachment = target.image.into_attachable_image_ref();
            let attachment_width = attachment.width();
            let attachment_height = attachment.height();

            buffers.push(IntegerBuffer::new(
                context.render_pass_id,
                index as i32,
                attachment_width,
                attachment_height,
            ));

            if let Some((width, height)) = framebuffer_dimensions {
                framebuffer_dimensions = Some((
                    cmp::min(width, attachment_width),
                    cmp::min(height, attachment_height),
                ));
            } else {
                framebuffer_dimensions = Some((attachment_width, attachment_height));
            }

            color_attachments[index] = Some(attachment);
            load_ops[index] = target.load_op.as_action(index as i32);
            store_ops[index] = target.store_op;
        }

        let color_count = buffers.len();

        load_ops[16] = stencil.load_op.as_action();
        store_ops[16] = stencil.store_op;

        let stencil_attachment = stencil.image.as_attachable_image_ref();
        let stencil_width = stencil_attachment.width();
        let stencil_height = stencil_attachment.height();

        if let Some((width, height)) = framebuffer_dimensions {
            framebuffer_dimensions = Some((
                cmp::min(width, stencil_width),
                cmp::min(height, stencil_height),
            ));
        } else {
            framebuffer_dimensions = Some((stencil_width, stencil_height));
        }

        RenderTargetEncoding {
            framebuffer: Framebuffer {
                color: buffers,
                depth_stencil: StencilBuffer::new(
                    context.render_pass_id,
                    stencil_width,
                    stencil_height,
                ),
                context_id: context.context_id,
                render_pass_id: context.render_pass_id,
                last_id: 0,
                dimensions: framebuffer_dimensions,
            },
            data: RenderTargetData::Custom(CustomRenderTargetData {
                load_ops,
                store_ops,
                color_count,
                color_attachments,
                depth_stencil_attachment: DepthStencilAttachmentDescriptor::Stencil(
                    stencil_attachment,
                ),
            }),
            context,
        }
    }
}

impl<'a, F> RenderTargetEncoding<'a, Framebuffer<Vec<UnsignedIntegerBuffer<F>>, ()>>
    where
        F: UnsignedIntegerRenderable,
{
    pub fn from_unsigned_integer_colors<C, I>(context: &'a mut EncodingContext, colors: C) -> Self
        where
            C: IntoIterator<Item = UnsignedIntegerAttachment<I>>,
            for<'b> I: AsAttachableImageRef<'b, Format = F>,
    {
        let mut color_attachments = [
            None, None, None, None, None, None, None, None, None, None, None, None, None, None,
            None, None,
        ];
        let mut load_ops = [LoadAction::Load; 17];
        let mut store_ops = [StoreOp::Store; 17];
        let mut buffers = Vec::new();
        let mut framebuffer_dimensions = None;

        for (index, target) in colors.into_iter().enumerate() {
            let attachment = target.image.into_attachable_image_ref();
            let attachment_width = attachment.width();
            let attachment_height = attachment.height();

            buffers.push(UnsignedIntegerBuffer::new(
                context.render_pass_id,
                index as i32,
                attachment_width,
                attachment_height,
            ));

            if let Some((width, height)) = framebuffer_dimensions {
                framebuffer_dimensions = Some((
                    cmp::min(width, attachment_width),
                    cmp::min(height, attachment_height),
                ));
            } else {
                framebuffer_dimensions = Some((attachment_width, attachment_height));
            }

            color_attachments[index] = Some(attachment);
            load_ops[index] = target.load_op.as_action(index as i32);
            store_ops[index] = target.store_op;
        }

        let color_count = buffers.len();

        RenderTargetEncoding {
            framebuffer: Framebuffer {
                color: buffers,
                depth_stencil: (),
                context_id: context.context_id,
                render_pass_id: context.render_pass_id,
                last_id: 0,
                dimensions: framebuffer_dimensions,
            },
            data: RenderTargetData::Custom(CustomRenderTargetData {
                load_ops,
                store_ops,
                color_count,
                color_attachments,
                depth_stencil_attachment: DepthStencilAttachmentDescriptor::None,
            }),
            context,
        }
    }
}

impl<'a, F0, F1>
RenderTargetEncoding<'a, Framebuffer<Vec<UnsignedIntegerBuffer<F0>>, DepthStencilBuffer<F1>>>
    where
        F0: UnsignedIntegerRenderable,
        F1: DepthStencilRenderable,
{
    pub fn from_unsigned_integer_colors_and_depth_stencil<C, I, Ds>(
        context: &'a mut EncodingContext,
        colors: C,
        depth_stencil: DepthStencilAttachment<Ds>,
    ) -> Self
        where
            C: IntoIterator<Item = UnsignedIntegerAttachment<I>>,
            for<'b> I: AsAttachableImageRef<'b, Format = F0>,
            for<'b> Ds: AsAttachableImageRef<'b, Format = F1>,
    {
        let mut color_attachments = [
            None, None, None, None, None, None, None, None, None, None, None, None, None, None,
            None, None,
        ];
        let mut load_ops = [LoadAction::Load; 17];
        let mut store_ops = [StoreOp::Store; 17];
        let mut buffers = Vec::new();
        let mut framebuffer_dimensions = None;

        for (index, target) in colors.into_iter().enumerate() {
            let attachment = target.image.into_attachable_image_ref();
            let attachment_width = attachment.width();
            let attachment_height = attachment.height();

            buffers.push(UnsignedIntegerBuffer::new(
                context.render_pass_id,
                index as i32,
                attachment_width,
                attachment_height,
            ));

            if let Some((width, height)) = framebuffer_dimensions {
                framebuffer_dimensions = Some((
                    cmp::min(width, attachment_width),
                    cmp::min(height, attachment_height),
                ));
            } else {
                framebuffer_dimensions = Some((attachment_width, attachment_height));
            }

            color_attachments[index] = Some(attachment);
            load_ops[index] = target.load_op.as_action(index as i32);
            store_ops[index] = target.store_op;
        }

        let color_count = buffers.len();

        load_ops[16] = depth_stencil.load_op.as_action();
        store_ops[16] = depth_stencil.store_op;

        let depth_stencil_attachment = depth_stencil.image.as_attachable_image_ref();
        let depth_stencil_width = depth_stencil_attachment.width();
        let depth_stencil_height = depth_stencil_attachment.height();

        if let Some((width, height)) = framebuffer_dimensions {
            framebuffer_dimensions = Some((
                cmp::min(width, depth_stencil_width),
                cmp::min(height, depth_stencil_height),
            ));
        } else {
            framebuffer_dimensions = Some((depth_stencil_width, depth_stencil_height));
        }

        RenderTargetEncoding {
            framebuffer: Framebuffer {
                color: buffers,
                depth_stencil: DepthStencilBuffer::new(
                    context.render_pass_id,
                    depth_stencil_width,
                    depth_stencil_height,
                ),
                context_id: context.context_id,
                render_pass_id: context.render_pass_id,
                last_id: 0,
                dimensions: framebuffer_dimensions,
            },
            data: RenderTargetData::Custom(CustomRenderTargetData {
                load_ops,
                store_ops,
                color_count,
                color_attachments,
                depth_stencil_attachment: DepthStencilAttachmentDescriptor::DepthStencil(
                    depth_stencil_attachment,
                ),
            }),
            context,
        }
    }
}

impl<'a, F0, F1>
RenderTargetEncoding<'a, Framebuffer<Vec<UnsignedIntegerBuffer<F0>>, DepthBuffer<F1>>>
    where
        F0: UnsignedIntegerRenderable,
        F1: DepthRenderable,
{
    pub fn from_unsigned_integer_colors_and_depth<C, I, Ds>(
        context: &'a mut EncodingContext,
        colors: C,
        depth: DepthAttachment<Ds>,
    ) -> Self
        where
            C: IntoIterator<Item = UnsignedIntegerAttachment<I>>,
            for<'b> I: AsAttachableImageRef<'b, Format = F0>,
            for<'b> Ds: AsAttachableImageRef<'b, Format = F1>,
    {
        let mut color_attachments = [
            None, None, None, None, None, None, None, None, None, None, None, None, None, None,
            None, None,
        ];
        let mut load_ops = [LoadAction::Load; 17];
        let mut store_ops = [StoreOp::Store; 17];
        let mut buffers = Vec::new();
        let mut framebuffer_dimensions = None;

        for (index, target) in colors.into_iter().enumerate() {
            let attachment = target.image.into_attachable_image_ref();
            let attachment_width = attachment.width();
            let attachment_height = attachment.height();

            buffers.push(UnsignedIntegerBuffer::new(
                context.render_pass_id,
                index as i32,
                attachment_width,
                attachment_height,
            ));

            if let Some((width, height)) = framebuffer_dimensions {
                framebuffer_dimensions = Some((
                    cmp::min(width, attachment_width),
                    cmp::min(height, attachment_height),
                ));
            } else {
                framebuffer_dimensions = Some((attachment_width, attachment_height));
            }

            color_attachments[index] = Some(attachment);
            load_ops[index] = target.load_op.as_action(index as i32);
            store_ops[index] = target.store_op;
        }

        let color_count = buffers.len();

        load_ops[16] = depth.load_op.as_action();
        store_ops[16] = depth.store_op;

        let depth_attachment = depth.image.as_attachable_image_ref();
        let depth_width = depth_attachment.width();
        let depth_height = depth_attachment.height();

        if let Some((width, height)) = framebuffer_dimensions {
            framebuffer_dimensions =
                Some((cmp::min(width, depth_width), cmp::min(height, depth_height)));
        } else {
            framebuffer_dimensions = Some((depth_width, depth_height));
        }

        RenderTargetEncoding {
            framebuffer: Framebuffer {
                color: buffers,
                depth_stencil: DepthBuffer::new(context.render_pass_id, depth_width, depth_height),
                context_id: context.context_id,
                render_pass_id: context.render_pass_id,
                last_id: 0,
                dimensions: framebuffer_dimensions,
            },
            data: RenderTargetData::Custom(CustomRenderTargetData {
                load_ops,
                store_ops,
                color_count,
                color_attachments,
                depth_stencil_attachment: DepthStencilAttachmentDescriptor::Depth(depth_attachment),
            }),
            context,
        }
    }
}

impl<'a, F0, F1>
RenderTargetEncoding<'a, Framebuffer<Vec<UnsignedIntegerBuffer<F0>>, StencilBuffer<F1>>>
    where
        F0: UnsignedIntegerRenderable,
        F1: StencilRenderable,
{
    pub fn from_unsigned_integer_colors_and_stencil<C, I, Ds>(
        context: &'a mut EncodingContext,
        colors: C,
        stencil: StencilAttachment<Ds>,
    ) -> Self
        where
            C: IntoIterator<Item = UnsignedIntegerAttachment<I>>,
            for<'b> I: AsAttachableImageRef<'b, Format = F0>,
            for<'b> Ds: AsAttachableImageRef<'b, Format = F1>,
    {
        let mut color_attachments = [
            None, None, None, None, None, None, None, None, None, None, None, None, None, None,
            None, None,
        ];
        let mut load_ops = [LoadAction::Load; 17];
        let mut store_ops = [StoreOp::Store; 17];
        let mut buffers = Vec::new();
        let mut framebuffer_dimensions = None;

        for (index, target) in colors.into_iter().enumerate() {
            let attachment = target.image.into_attachable_image_ref();
            let attachment_width = attachment.width();
            let attachment_height = attachment.height();

            buffers.push(UnsignedIntegerBuffer::new(
                context.render_pass_id,
                index as i32,
                attachment_width,
                attachment_height,
            ));

            if let Some((width, height)) = framebuffer_dimensions {
                framebuffer_dimensions = Some((
                    cmp::min(width, attachment_width),
                    cmp::min(height, attachment_height),
                ));
            } else {
                framebuffer_dimensions = Some((attachment_width, attachment_height));
            }

            color_attachments[index] = Some(attachment);
            load_ops[index] = target.load_op.as_action(index as i32);
            store_ops[index] = target.store_op;
        }

        let color_count = buffers.len();

        load_ops[16] = stencil.load_op.as_action();
        store_ops[16] = stencil.store_op;

        let stencil_attachment = stencil.image.as_attachable_image_ref();
        let stencil_width = stencil_attachment.width();
        let stencil_height = stencil_attachment.height();

        if let Some((width, height)) = framebuffer_dimensions {
            framebuffer_dimensions = Some((
                cmp::min(width, stencil_width),
                cmp::min(height, stencil_height),
            ));
        } else {
            framebuffer_dimensions = Some((stencil_width, stencil_height));
        }

        RenderTargetEncoding {
            framebuffer: Framebuffer {
                color: buffers,
                depth_stencil: StencilBuffer::new(
                    context.render_pass_id,
                    stencil_width,
                    stencil_height,
                ),
                context_id: context.context_id,
                render_pass_id: context.render_pass_id,
                last_id: 0,
                dimensions: framebuffer_dimensions,
            },
            data: RenderTargetData::Custom(CustomRenderTargetData {
                load_ops,
                store_ops,
                color_count,
                color_attachments,
                depth_stencil_attachment: DepthStencilAttachmentDescriptor::Stencil(
                    stencil_attachment,
                ),
            }),
            context,
        }
    }
}

