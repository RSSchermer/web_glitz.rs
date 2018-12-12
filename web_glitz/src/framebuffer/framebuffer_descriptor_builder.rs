use std::marker;

use crate::framebuffer::{
    AsFramebufferAttachment, ColorAttachable, DepthAttachable, FramebufferDescriptor,
    StencilAttachable,
};
use crate::rendering_context::RenderingContext;

pub struct FramebufferDescriptorBuilder<
    C0,
    C1,
    C2,
    C3,
    C4,
    C5,
    C6,
    C7,
    C8,
    C9,
    C10,
    C11,
    C12,
    C13,
    C14,
    C15,
    D,
    S,
> {
    color_attachment_0: C0,
    color_attachment_1: C1,
    color_attachment_2: C2,
    color_attachment_3: C3,
    color_attachment_4: C4,
    color_attachment_5: C5,
    color_attachment_6: C6,
    color_attachment_7: C7,
    color_attachment_8: C8,
    color_attachment_9: C9,
    color_attachment_10: C10,
    color_attachment_11: C11,
    color_attachment_12: C12,
    color_attachment_13: C13,
    color_attachment_14: C14,
    color_attachment_15: C15,
    depth_attachment: D,
    stencil_attachment: S,
}

impl
    FramebufferDescriptorBuilder<
        (),
        (),
        (),
        (),
        (),
        (),
        (),
        (),
        (),
        (),
        (),
        (),
        (),
        (),
        (),
        (),
        (),
        (),
    >
{
    pub fn new() -> Self {
        FramebufferDescriptorBuilder {
            color_attachment_0: (),
            color_attachment_1: (),
            color_attachment_2: (),
            color_attachment_3: (),
            color_attachment_4: (),
            color_attachment_5: (),
            color_attachment_6: (),
            color_attachment_7: (),
            color_attachment_8: (),
            color_attachment_9: (),
            color_attachment_10: (),
            color_attachment_11: (),
            color_attachment_12: (),
            color_attachment_13: (),
            color_attachment_14: (),
            color_attachment_15: (),
            depth_attachment: (),
            stencil_attachment: (),
        }
    }
}

impl<C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12, C13, C14, C15, D, S>
    FramebufferDescriptorBuilder<
        C0,
        C1,
        C2,
        C3,
        C4,
        C5,
        C6,
        C7,
        C8,
        C9,
        C10,
        C11,
        C12,
        C13,
        C14,
        C15,
        D,
        S,
    >
where
    C0: ColorAttachable + AsFramebufferAttachment,
    C1: ColorAttachable + AsFramebufferAttachment,
    C2: ColorAttachable + AsFramebufferAttachment,
    C3: ColorAttachable + AsFramebufferAttachment,
    C4: ColorAttachable + AsFramebufferAttachment,
    C5: ColorAttachable + AsFramebufferAttachment,
    C6: ColorAttachable + AsFramebufferAttachment,
    C7: ColorAttachable + AsFramebufferAttachment,
    C8: ColorAttachable + AsFramebufferAttachment,
    C9: ColorAttachable + AsFramebufferAttachment,
    C10: ColorAttachable + AsFramebufferAttachment,
    C11: ColorAttachable + AsFramebufferAttachment,
    C12: ColorAttachable + AsFramebufferAttachment,
    C13: ColorAttachable + AsFramebufferAttachment,
    C14: ColorAttachable + AsFramebufferAttachment,
    C15: ColorAttachable + AsFramebufferAttachment,
    D: DepthAttachable + AsFramebufferAttachment,
    S: StencilAttachable + AsFramebufferAttachment,
{
    pub fn color_attachment_0<A>(
        self,
        attachable: A,
    ) -> FramebufferDescriptorBuilder<
        A,
        C1,
        C2,
        C3,
        C4,
        C5,
        C6,
        C7,
        C8,
        C9,
        C10,
        C11,
        C12,
        C13,
        C14,
        C15,
        D,
        S,
    >
    where
        A: ColorAttachable + AsFramebufferAttachment,
    {
        FramebufferDescriptorBuilder {
            color_attachment_0: attachable,
            color_attachment_1: self.color_attachment_1,
            color_attachment_2: self.color_attachment_2,
            color_attachment_3: self.color_attachment_3,
            color_attachment_4: self.color_attachment_4,
            color_attachment_5: self.color_attachment_5,
            color_attachment_6: self.color_attachment_6,
            color_attachment_7: self.color_attachment_7,
            color_attachment_8: self.color_attachment_8,
            color_attachment_9: self.color_attachment_9,
            color_attachment_10: self.color_attachment_10,
            color_attachment_11: self.color_attachment_11,
            color_attachment_12: self.color_attachment_12,
            color_attachment_13: self.color_attachment_13,
            color_attachment_14: self.color_attachment_14,
            color_attachment_15: self.color_attachment_15,
            depth_attachment: self.depth_attachment,
            stencil_attachment: self.stencil_attachment,
        }
    }

    pub fn color_attachment_1<A>(
        self,
        attachable: A,
    ) -> FramebufferDescriptorBuilder<
        C0,
        A,
        C2,
        C3,
        C4,
        C5,
        C6,
        C7,
        C8,
        C9,
        C10,
        C11,
        C12,
        C13,
        C14,
        C15,
        D,
        S,
    >
    where
        A: ColorAttachable + AsFramebufferAttachment,
    {
        FramebufferDescriptorBuilder {
            color_attachment_0: self.color_attachment_0,
            color_attachment_1: attachable,
            color_attachment_2: self.color_attachment_2,
            color_attachment_3: self.color_attachment_3,
            color_attachment_4: self.color_attachment_4,
            color_attachment_5: self.color_attachment_5,
            color_attachment_6: self.color_attachment_6,
            color_attachment_7: self.color_attachment_7,
            color_attachment_8: self.color_attachment_8,
            color_attachment_9: self.color_attachment_9,
            color_attachment_10: self.color_attachment_10,
            color_attachment_11: self.color_attachment_11,
            color_attachment_12: self.color_attachment_12,
            color_attachment_13: self.color_attachment_13,
            color_attachment_14: self.color_attachment_14,
            color_attachment_15: self.color_attachment_15,
            depth_attachment: self.depth_attachment,
            stencil_attachment: self.stencil_attachment,
        }
    }

    pub fn color_attachment_2<A>(
        self,
        attachable: A,
    ) -> FramebufferDescriptorBuilder<
        C0,
        C1,
        A,
        C3,
        C4,
        C5,
        C6,
        C7,
        C8,
        C9,
        C10,
        C11,
        C12,
        C13,
        C14,
        C15,
        D,
        S,
    >
    where
        A: ColorAttachable + AsFramebufferAttachment,
    {
        FramebufferDescriptorBuilder {
            color_attachment_0: self.color_attachment_0,
            color_attachment_1: self.color_attachment_1,
            color_attachment_2: attachable,
            color_attachment_3: self.color_attachment_3,
            color_attachment_4: self.color_attachment_4,
            color_attachment_5: self.color_attachment_5,
            color_attachment_6: self.color_attachment_6,
            color_attachment_7: self.color_attachment_7,
            color_attachment_8: self.color_attachment_8,
            color_attachment_9: self.color_attachment_9,
            color_attachment_10: self.color_attachment_10,
            color_attachment_11: self.color_attachment_11,
            color_attachment_12: self.color_attachment_12,
            color_attachment_13: self.color_attachment_13,
            color_attachment_14: self.color_attachment_14,
            color_attachment_15: self.color_attachment_15,
            depth_attachment: self.depth_attachment,
            stencil_attachment: self.stencil_attachment,
        }
    }

    pub fn color_attachment_3<A>(
        self,
        attachable: A,
    ) -> FramebufferDescriptorBuilder<
        C0,
        C1,
        C2,
        A,
        C4,
        C5,
        C6,
        C7,
        C8,
        C9,
        C10,
        C11,
        C12,
        C13,
        C14,
        C15,
        D,
        S,
    >
    where
        A: ColorAttachable + AsFramebufferAttachment,
    {
        FramebufferDescriptorBuilder {
            color_attachment_0: self.color_attachment_0,
            color_attachment_1: self.color_attachment_1,
            color_attachment_2: self.color_attachment_2,
            color_attachment_3: attachable,
            color_attachment_4: self.color_attachment_4,
            color_attachment_5: self.color_attachment_5,
            color_attachment_6: self.color_attachment_6,
            color_attachment_7: self.color_attachment_7,
            color_attachment_8: self.color_attachment_8,
            color_attachment_9: self.color_attachment_9,
            color_attachment_10: self.color_attachment_10,
            color_attachment_11: self.color_attachment_11,
            color_attachment_12: self.color_attachment_12,
            color_attachment_13: self.color_attachment_13,
            color_attachment_14: self.color_attachment_14,
            color_attachment_15: self.color_attachment_15,
            depth_attachment: self.depth_attachment,
            stencil_attachment: self.stencil_attachment,
        }
    }

    pub fn color_attachment_4<A>(
        self,
        attachable: A,
    ) -> FramebufferDescriptorBuilder<
        C0,
        C1,
        C2,
        C3,
        A,
        C5,
        C6,
        C7,
        C8,
        C9,
        C10,
        C11,
        C12,
        C13,
        C14,
        C15,
        D,
        S,
    >
    where
        A: ColorAttachable + AsFramebufferAttachment,
    {
        FramebufferDescriptorBuilder {
            color_attachment_0: self.color_attachment_0,
            color_attachment_1: self.color_attachment_1,
            color_attachment_2: self.color_attachment_2,
            color_attachment_3: self.color_attachment_3,
            color_attachment_4: attachable,
            color_attachment_5: self.color_attachment_5,
            color_attachment_6: self.color_attachment_6,
            color_attachment_7: self.color_attachment_7,
            color_attachment_8: self.color_attachment_8,
            color_attachment_9: self.color_attachment_9,
            color_attachment_10: self.color_attachment_10,
            color_attachment_11: self.color_attachment_11,
            color_attachment_12: self.color_attachment_12,
            color_attachment_13: self.color_attachment_13,
            color_attachment_14: self.color_attachment_14,
            color_attachment_15: self.color_attachment_15,
            depth_attachment: self.depth_attachment,
            stencil_attachment: self.stencil_attachment,
        }
    }

    pub fn color_attachment_5<A>(
        self,
        attachable: A,
    ) -> FramebufferDescriptorBuilder<
        C0,
        C1,
        C2,
        C3,
        C4,
        A,
        C6,
        C7,
        C8,
        C9,
        C10,
        C11,
        C12,
        C13,
        C14,
        C15,
        D,
        S,
    >
    where
        A: ColorAttachable + AsFramebufferAttachment,
    {
        FramebufferDescriptorBuilder {
            color_attachment_0: self.color_attachment_0,
            color_attachment_1: self.color_attachment_1,
            color_attachment_2: self.color_attachment_2,
            color_attachment_3: self.color_attachment_3,
            color_attachment_4: self.color_attachment_4,
            color_attachment_5: attachable,
            color_attachment_6: self.color_attachment_6,
            color_attachment_7: self.color_attachment_7,
            color_attachment_8: self.color_attachment_8,
            color_attachment_9: self.color_attachment_9,
            color_attachment_10: self.color_attachment_10,
            color_attachment_11: self.color_attachment_11,
            color_attachment_12: self.color_attachment_12,
            color_attachment_13: self.color_attachment_13,
            color_attachment_14: self.color_attachment_14,
            color_attachment_15: self.color_attachment_15,
            depth_attachment: self.depth_attachment,
            stencil_attachment: self.stencil_attachment,
        }
    }

    pub fn color_attachment_6<A>(
        self,
        attachable: A,
    ) -> FramebufferDescriptorBuilder<
        C0,
        C1,
        C2,
        C3,
        C4,
        C5,
        A,
        C7,
        C8,
        C9,
        C10,
        C11,
        C12,
        C13,
        C14,
        C15,
        D,
        S,
    >
    where
        A: ColorAttachable + AsFramebufferAttachment,
    {
        FramebufferDescriptorBuilder {
            color_attachment_0: self.color_attachment_0,
            color_attachment_1: self.color_attachment_1,
            color_attachment_2: self.color_attachment_2,
            color_attachment_3: self.color_attachment_3,
            color_attachment_4: self.color_attachment_4,
            color_attachment_5: self.color_attachment_5,
            color_attachment_6: attachable,
            color_attachment_7: self.color_attachment_7,
            color_attachment_8: self.color_attachment_8,
            color_attachment_9: self.color_attachment_9,
            color_attachment_10: self.color_attachment_10,
            color_attachment_11: self.color_attachment_11,
            color_attachment_12: self.color_attachment_12,
            color_attachment_13: self.color_attachment_13,
            color_attachment_14: self.color_attachment_14,
            color_attachment_15: self.color_attachment_15,
            depth_attachment: self.depth_attachment,
            stencil_attachment: self.stencil_attachment,
        }
    }

    pub fn color_attachment_7<A>(
        self,
        attachable: A,
    ) -> FramebufferDescriptorBuilder<
        C0,
        C1,
        C2,
        C3,
        C4,
        C5,
        C6,
        A,
        C8,
        C9,
        C10,
        C11,
        C12,
        C13,
        C14,
        C15,
        D,
        S,
    >
    where
        A: ColorAttachable + AsFramebufferAttachment,
    {
        FramebufferDescriptorBuilder {
            color_attachment_0: self.color_attachment_0,
            color_attachment_1: self.color_attachment_1,
            color_attachment_2: self.color_attachment_2,
            color_attachment_3: self.color_attachment_3,
            color_attachment_4: self.color_attachment_4,
            color_attachment_5: self.color_attachment_5,
            color_attachment_6: self.color_attachment_6,
            color_attachment_7: attachable,
            color_attachment_8: self.color_attachment_8,
            color_attachment_9: self.color_attachment_9,
            color_attachment_10: self.color_attachment_10,
            color_attachment_11: self.color_attachment_11,
            color_attachment_12: self.color_attachment_12,
            color_attachment_13: self.color_attachment_13,
            color_attachment_14: self.color_attachment_14,
            color_attachment_15: self.color_attachment_15,
            depth_attachment: self.depth_attachment,
            stencil_attachment: self.stencil_attachment,
        }
    }

    pub fn color_attachment_8<A>(
        self,
        attachable: A,
    ) -> FramebufferDescriptorBuilder<
        C0,
        C1,
        C2,
        C3,
        C4,
        C5,
        C6,
        C7,
        A,
        C9,
        C10,
        C11,
        C12,
        C13,
        C14,
        C15,
        D,
        S,
    >
    where
        A: ColorAttachable + AsFramebufferAttachment,
    {
        FramebufferDescriptorBuilder {
            color_attachment_0: self.color_attachment_0,
            color_attachment_1: self.color_attachment_1,
            color_attachment_2: self.color_attachment_2,
            color_attachment_3: self.color_attachment_3,
            color_attachment_4: self.color_attachment_4,
            color_attachment_5: self.color_attachment_5,
            color_attachment_6: self.color_attachment_6,
            color_attachment_7: self.color_attachment_7,
            color_attachment_8: attachable,
            color_attachment_9: self.color_attachment_9,
            color_attachment_10: self.color_attachment_10,
            color_attachment_11: self.color_attachment_11,
            color_attachment_12: self.color_attachment_12,
            color_attachment_13: self.color_attachment_13,
            color_attachment_14: self.color_attachment_14,
            color_attachment_15: self.color_attachment_15,
            depth_attachment: self.depth_attachment,
            stencil_attachment: self.stencil_attachment,
        }
    }

    pub fn color_attachment_9<A>(
        self,
        attachable: A,
    ) -> FramebufferDescriptorBuilder<
        C0,
        C1,
        C2,
        C3,
        C4,
        C5,
        C6,
        C7,
        C8,
        A,
        C10,
        C11,
        C12,
        C13,
        C14,
        C15,
        D,
        S,
    >
    where
        A: ColorAttachable + AsFramebufferAttachment,
    {
        FramebufferDescriptorBuilder {
            color_attachment_0: self.color_attachment_0,
            color_attachment_1: self.color_attachment_1,
            color_attachment_2: self.color_attachment_2,
            color_attachment_3: self.color_attachment_3,
            color_attachment_4: self.color_attachment_4,
            color_attachment_5: self.color_attachment_5,
            color_attachment_6: self.color_attachment_6,
            color_attachment_7: self.color_attachment_7,
            color_attachment_8: self.color_attachment_8,
            color_attachment_9: attachable,
            color_attachment_10: self.color_attachment_10,
            color_attachment_11: self.color_attachment_11,
            color_attachment_12: self.color_attachment_12,
            color_attachment_13: self.color_attachment_13,
            color_attachment_14: self.color_attachment_14,
            color_attachment_15: self.color_attachment_15,
            depth_attachment: self.depth_attachment,
            stencil_attachment: self.stencil_attachment,
        }
    }

    pub fn color_attachment_10<A>(
        self,
        attachable: A,
    ) -> FramebufferDescriptorBuilder<
        C0,
        C1,
        C2,
        C3,
        C4,
        C5,
        C6,
        C7,
        C8,
        C9,
        A,
        C11,
        C12,
        C13,
        C14,
        C15,
        D,
        S,
    >
    where
        A: ColorAttachable + AsFramebufferAttachment,
    {
        FramebufferDescriptorBuilder {
            color_attachment_0: self.color_attachment_0,
            color_attachment_1: self.color_attachment_1,
            color_attachment_2: self.color_attachment_2,
            color_attachment_3: self.color_attachment_3,
            color_attachment_4: self.color_attachment_4,
            color_attachment_5: self.color_attachment_5,
            color_attachment_6: self.color_attachment_6,
            color_attachment_7: self.color_attachment_7,
            color_attachment_8: self.color_attachment_8,
            color_attachment_9: self.color_attachment_9,
            color_attachment_10: attachable,
            color_attachment_11: self.color_attachment_11,
            color_attachment_12: self.color_attachment_12,
            color_attachment_13: self.color_attachment_13,
            color_attachment_14: self.color_attachment_14,
            color_attachment_15: self.color_attachment_15,
            depth_attachment: self.depth_attachment,
            stencil_attachment: self.stencil_attachment,
        }
    }

    pub fn color_attachment_11<A>(
        self,
        attachable: A,
    ) -> FramebufferDescriptorBuilder<
        C0,
        C1,
        C2,
        C3,
        C4,
        C5,
        C6,
        C7,
        C8,
        C9,
        C10,
        A,
        C12,
        C13,
        C14,
        C15,
        D,
        S,
    >
    where
        A: ColorAttachable + AsFramebufferAttachment,
    {
        FramebufferDescriptorBuilder {
            color_attachment_0: self.color_attachment_0,
            color_attachment_1: self.color_attachment_1,
            color_attachment_2: self.color_attachment_2,
            color_attachment_3: self.color_attachment_3,
            color_attachment_4: self.color_attachment_4,
            color_attachment_5: self.color_attachment_5,
            color_attachment_6: self.color_attachment_6,
            color_attachment_7: self.color_attachment_7,
            color_attachment_8: self.color_attachment_8,
            color_attachment_9: self.color_attachment_9,
            color_attachment_10: self.color_attachment_10,
            color_attachment_11: attachable,
            color_attachment_12: self.color_attachment_12,
            color_attachment_13: self.color_attachment_13,
            color_attachment_14: self.color_attachment_14,
            color_attachment_15: self.color_attachment_15,
            depth_attachment: self.depth_attachment,
            stencil_attachment: self.stencil_attachment,
        }
    }

    pub fn color_attachment_12<A>(
        self,
        attachable: A,
    ) -> FramebufferDescriptorBuilder<
        C0,
        C1,
        C2,
        C3,
        C4,
        C5,
        C6,
        C7,
        C8,
        C9,
        C10,
        C11,
        A,
        C13,
        C14,
        C15,
        D,
        S,
    >
    where
        A: ColorAttachable + AsFramebufferAttachment,
    {
        FramebufferDescriptorBuilder {
            color_attachment_0: self.color_attachment_0,
            color_attachment_1: self.color_attachment_1,
            color_attachment_2: self.color_attachment_2,
            color_attachment_3: self.color_attachment_3,
            color_attachment_4: self.color_attachment_4,
            color_attachment_5: self.color_attachment_5,
            color_attachment_6: self.color_attachment_6,
            color_attachment_7: self.color_attachment_7,
            color_attachment_8: self.color_attachment_8,
            color_attachment_9: self.color_attachment_9,
            color_attachment_10: self.color_attachment_10,
            color_attachment_11: self.color_attachment_11,
            color_attachment_12: attachable,
            color_attachment_13: self.color_attachment_13,
            color_attachment_14: self.color_attachment_14,
            color_attachment_15: self.color_attachment_15,
            depth_attachment: self.depth_attachment,
            stencil_attachment: self.stencil_attachment,
        }
    }

    pub fn color_attachment_13<A>(
        self,
        attachable: A,
    ) -> FramebufferDescriptorBuilder<
        C0,
        C1,
        C2,
        C3,
        C4,
        C5,
        C6,
        C7,
        C8,
        C9,
        C10,
        C11,
        C12,
        A,
        C14,
        C15,
        D,
        S,
    >
    where
        A: ColorAttachable + AsFramebufferAttachment,
    {
        FramebufferDescriptorBuilder {
            color_attachment_0: self.color_attachment_0,
            color_attachment_1: self.color_attachment_1,
            color_attachment_2: self.color_attachment_2,
            color_attachment_3: self.color_attachment_3,
            color_attachment_4: self.color_attachment_4,
            color_attachment_5: self.color_attachment_5,
            color_attachment_6: self.color_attachment_6,
            color_attachment_7: self.color_attachment_7,
            color_attachment_8: self.color_attachment_8,
            color_attachment_9: self.color_attachment_9,
            color_attachment_10: self.color_attachment_10,
            color_attachment_11: self.color_attachment_11,
            color_attachment_12: self.color_attachment_12,
            color_attachment_13: attachable,
            color_attachment_14: self.color_attachment_14,
            color_attachment_15: self.color_attachment_15,
            depth_attachment: self.depth_attachment,
            stencil_attachment: self.stencil_attachment,
        }
    }

    pub fn color_attachment_14<A>(
        self,
        attachable: A,
    ) -> FramebufferDescriptorBuilder<
        C0,
        C1,
        C2,
        C3,
        C4,
        C5,
        C6,
        C7,
        C8,
        C9,
        C10,
        C11,
        C12,
        C13,
        A,
        C15,
        D,
        S,
    >
    where
        A: ColorAttachable + AsFramebufferAttachment,
    {
        FramebufferDescriptorBuilder {
            color_attachment_0: self.color_attachment_0,
            color_attachment_1: self.color_attachment_1,
            color_attachment_2: self.color_attachment_2,
            color_attachment_3: self.color_attachment_3,
            color_attachment_4: self.color_attachment_4,
            color_attachment_5: self.color_attachment_5,
            color_attachment_6: self.color_attachment_6,
            color_attachment_7: self.color_attachment_7,
            color_attachment_8: self.color_attachment_8,
            color_attachment_9: self.color_attachment_9,
            color_attachment_10: self.color_attachment_10,
            color_attachment_11: self.color_attachment_11,
            color_attachment_12: self.color_attachment_12,
            color_attachment_13: self.color_attachment_13,
            color_attachment_14: attachable,
            color_attachment_15: self.color_attachment_15,
            depth_attachment: self.depth_attachment,
            stencil_attachment: self.stencil_attachment,
        }
    }

    pub fn color_attachment_15<A>(
        self,
        attachable: A,
    ) -> FramebufferDescriptorBuilder<
        C0,
        C1,
        C2,
        C3,
        C4,
        C5,
        C6,
        C7,
        C8,
        C9,
        C10,
        C11,
        C12,
        C13,
        C14,
        A,
        D,
        S,
    >
    where
        A: ColorAttachable + AsFramebufferAttachment,
    {
        FramebufferDescriptorBuilder {
            color_attachment_0: self.color_attachment_0,
            color_attachment_1: self.color_attachment_1,
            color_attachment_2: self.color_attachment_2,
            color_attachment_3: self.color_attachment_3,
            color_attachment_4: self.color_attachment_4,
            color_attachment_5: self.color_attachment_5,
            color_attachment_6: self.color_attachment_6,
            color_attachment_7: self.color_attachment_7,
            color_attachment_8: self.color_attachment_8,
            color_attachment_9: self.color_attachment_9,
            color_attachment_10: self.color_attachment_10,
            color_attachment_11: self.color_attachment_11,
            color_attachment_12: self.color_attachment_12,
            color_attachment_13: self.color_attachment_13,
            color_attachment_14: self.color_attachment_14,
            color_attachment_15: attachable,
            depth_attachment: self.depth_attachment,
            stencil_attachment: self.stencil_attachment,
        }
    }

    pub fn depth_attachment<A>(
        self,
        attachable: A,
    ) -> FramebufferDescriptorBuilder<
        C0,
        C1,
        C2,
        C3,
        C4,
        C5,
        C6,
        C7,
        C8,
        C9,
        C10,
        C11,
        C12,
        C13,
        C14,
        C15,
        A,
        S,
    >
    where
        A: DepthAttachable + AsFramebufferAttachment,
    {
        FramebufferDescriptorBuilder {
            color_attachment_0: self.color_attachment_0,
            color_attachment_1: self.color_attachment_1,
            color_attachment_2: self.color_attachment_2,
            color_attachment_3: self.color_attachment_3,
            color_attachment_4: self.color_attachment_4,
            color_attachment_5: self.color_attachment_5,
            color_attachment_6: self.color_attachment_6,
            color_attachment_7: self.color_attachment_7,
            color_attachment_8: self.color_attachment_8,
            color_attachment_9: self.color_attachment_9,
            color_attachment_10: self.color_attachment_10,
            color_attachment_11: self.color_attachment_11,
            color_attachment_12: self.color_attachment_12,
            color_attachment_13: self.color_attachment_13,
            color_attachment_14: self.color_attachment_14,
            color_attachment_15: self.color_attachment_15,
            depth_attachment: attachable,
            stencil_attachment: self.stencil_attachment,
        }
    }

    pub fn stencil_attachment<A>(
        self,
        attachable: A,
    ) -> FramebufferDescriptorBuilder<
        C0,
        C1,
        C2,
        C3,
        C4,
        C5,
        C6,
        C7,
        C8,
        C9,
        C10,
        C11,
        C12,
        C13,
        C14,
        C15,
        D,
        A,
    >
    where
        A: StencilAttachable + AsFramebufferAttachment,
    {
        FramebufferDescriptorBuilder {
            color_attachment_0: self.color_attachment_0,
            color_attachment_1: self.color_attachment_1,
            color_attachment_2: self.color_attachment_2,
            color_attachment_3: self.color_attachment_3,
            color_attachment_4: self.color_attachment_4,
            color_attachment_5: self.color_attachment_5,
            color_attachment_6: self.color_attachment_6,
            color_attachment_7: self.color_attachment_7,
            color_attachment_8: self.color_attachment_8,
            color_attachment_9: self.color_attachment_9,
            color_attachment_10: self.color_attachment_10,
            color_attachment_11: self.color_attachment_11,
            color_attachment_12: self.color_attachment_12,
            color_attachment_13: self.color_attachment_13,
            color_attachment_14: self.color_attachment_14,
            color_attachment_15: self.color_attachment_15,
            depth_attachment: self.depth_attachment,
            stencil_attachment: attachable,
        }
    }

    pub fn depth_stencil_attachment<A>(
        self,
        attachable: A,
    ) -> FramebufferDescriptorBuilder<
        C0,
        C1,
        C2,
        C3,
        C4,
        C5,
        C6,
        C7,
        C8,
        C9,
        C10,
        C11,
        C12,
        C13,
        C14,
        C15,
        A,
        A,
    >
    where
        A: DepthAttachable + StencilAttachable + AsFramebufferAttachment + Clone,
    {
        FramebufferDescriptorBuilder {
            color_attachment_0: self.color_attachment_0,
            color_attachment_1: self.color_attachment_1,
            color_attachment_2: self.color_attachment_2,
            color_attachment_3: self.color_attachment_3,
            color_attachment_4: self.color_attachment_4,
            color_attachment_5: self.color_attachment_5,
            color_attachment_6: self.color_attachment_6,
            color_attachment_7: self.color_attachment_7,
            color_attachment_8: self.color_attachment_8,
            color_attachment_9: self.color_attachment_9,
            color_attachment_10: self.color_attachment_10,
            color_attachment_11: self.color_attachment_11,
            color_attachment_12: self.color_attachment_12,
            color_attachment_13: self.color_attachment_13,
            color_attachment_14: self.color_attachment_14,
            color_attachment_15: self.color_attachment_15,
            depth_attachment: attachable.clone(),
            stencil_attachment: attachable,
        }
    }

    pub fn finish(
        self,
    ) -> BuildFramebufferDescriptor<
        C0,
        C1,
        C2,
        C3,
        C4,
        C5,
        C6,
        C7,
        C8,
        C9,
        C10,
        C11,
        C12,
        C13,
        C14,
        C15,
        D,
        S,
    > {
        BuildFramebufferDescriptor {
            color_attachment_0: self.color_attachment_0,
            color_attachment_1: self.color_attachment_1,
            color_attachment_2: self.color_attachment_2,
            color_attachment_3: self.color_attachment_3,
            color_attachment_4: self.color_attachment_4,
            color_attachment_5: self.color_attachment_5,
            color_attachment_6: self.color_attachment_6,
            color_attachment_7: self.color_attachment_7,
            color_attachment_8: self.color_attachment_8,
            color_attachment_9: self.color_attachment_9,
            color_attachment_10: self.color_attachment_10,
            color_attachment_11: self.color_attachment_11,
            color_attachment_12: self.color_attachment_12,
            color_attachment_13: self.color_attachment_13,
            color_attachment_14: self.color_attachment_14,
            color_attachment_15: self.color_attachment_15,
            depth_attachment: self.depth_attachment,
            stencil_attachment: self.stencil_attachment,
        }
    }
}

pub struct BuildFramebufferDescriptor<
    C0,
    C1,
    C2,
    C3,
    C4,
    C5,
    C6,
    C7,
    C8,
    C9,
    C10,
    C11,
    C12,
    C13,
    C14,
    C15,
    D,
    S,
> {
    color_attachment_0: C0,
    color_attachment_1: C1,
    color_attachment_2: C2,
    color_attachment_3: C3,
    color_attachment_4: C4,
    color_attachment_5: C5,
    color_attachment_6: C6,
    color_attachment_7: C7,
    color_attachment_8: C8,
    color_attachment_9: C9,
    color_attachment_10: C10,
    color_attachment_11: C11,
    color_attachment_12: C12,
    color_attachment_13: C13,
    color_attachment_14: C14,
    color_attachment_15: C15,
    depth_attachment: D,
    stencil_attachment: S,
}

impl<C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12, C13, C14, C15, D, S>
    FramebufferDescriptor
    for BuildFramebufferDescriptor<
        C0,
        C1,
        C2,
        C3,
        C4,
        C5,
        C6,
        C7,
        C8,
        C9,
        C10,
        C11,
        C12,
        C13,
        C14,
        C15,
        D,
        S,
    >
where
    C0: ColorAttachable + AsFramebufferAttachment,
    C1: ColorAttachable + AsFramebufferAttachment,
    C2: ColorAttachable + AsFramebufferAttachment,
    C3: ColorAttachable + AsFramebufferAttachment,
    C4: ColorAttachable + AsFramebufferAttachment,
    C5: ColorAttachable + AsFramebufferAttachment,
    C6: ColorAttachable + AsFramebufferAttachment,
    C7: ColorAttachable + AsFramebufferAttachment,
    C8: ColorAttachable + AsFramebufferAttachment,
    C9: ColorAttachable + AsFramebufferAttachment,
    C10: ColorAttachable + AsFramebufferAttachment,
    C11: ColorAttachable + AsFramebufferAttachment,
    C12: ColorAttachable + AsFramebufferAttachment,
    C13: ColorAttachable + AsFramebufferAttachment,
    C14: ColorAttachable + AsFramebufferAttachment,
    C15: ColorAttachable + AsFramebufferAttachment,
    D: DepthAttachable + AsFramebufferAttachment,
    S: StencilAttachable + AsFramebufferAttachment,
{
    type ColorAttachment0 = C0;

    type ColorAttachment1 = C1;

    type ColorAttachment2 = C2;

    type ColorAttachment3 = C3;

    type ColorAttachment4 = C4;

    type ColorAttachment5 = C5;

    type ColorAttachment6 = C6;

    type ColorAttachment7 = C7;

    type ColorAttachment8 = C8;

    type ColorAttachment9 = C9;

    type ColorAttachment10 = C10;

    type ColorAttachment11 = C11;

    type ColorAttachment12 = C12;

    type ColorAttachment13 = C13;

    type ColorAttachment14 = C14;

    type ColorAttachment15 = C15;

    type DepthAttachment = D;

    type StencilAttachment = S;

    fn color_attachment_0(&self) -> &Self::ColorAttachment0 {
        &self.color_attachment_0
    }

    fn color_attachment_1(&self) -> &Self::ColorAttachment1 {
        &self.color_attachment_1
    }

    fn color_attachment_2(&self) -> &Self::ColorAttachment2 {
        &self.color_attachment_2
    }

    fn color_attachment_3(&self) -> &Self::ColorAttachment3 {
        &self.color_attachment_3
    }

    fn color_attachment_4(&self) -> &Self::ColorAttachment4 {
        &self.color_attachment_4
    }

    fn color_attachment_5(&self) -> &Self::ColorAttachment5 {
        &self.color_attachment_5
    }

    fn color_attachment_6(&self) -> &Self::ColorAttachment6 {
        &self.color_attachment_6
    }

    fn color_attachment_7(&self) -> &Self::ColorAttachment7 {
        &self.color_attachment_7
    }

    fn color_attachment_8(&self) -> &Self::ColorAttachment8 {
        &self.color_attachment_8
    }

    fn color_attachment_9(&self) -> &Self::ColorAttachment9 {
        &self.color_attachment_9
    }

    fn color_attachment_10(&self) -> &Self::ColorAttachment10 {
        &self.color_attachment_10
    }

    fn color_attachment_11(&self) -> &Self::ColorAttachment11 {
        &self.color_attachment_11
    }

    fn color_attachment_12(&self) -> &Self::ColorAttachment12 {
        &self.color_attachment_12
    }

    fn color_attachment_13(&self) -> &Self::ColorAttachment13 {
        &self.color_attachment_13
    }

    fn color_attachment_14(&self) -> &Self::ColorAttachment14 {
        &self.color_attachment_14
    }

    fn color_attachment_15(&self) -> &Self::ColorAttachment15 {
        &self.color_attachment_15
    }

    fn depth_attachment(&self) -> &Self::DepthAttachment {
        &self.depth_attachment
    }

    fn stencil_attachment(&self) -> &Self::StencilAttachment {
        &self.stencil_attachment
    }
}
