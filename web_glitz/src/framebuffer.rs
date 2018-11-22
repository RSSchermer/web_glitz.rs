use std::iter::IntoIterator;
use std::marker;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

use wasm_bindgen::JsCast;

use crate::image_format::{ColorRenderable, DepthRenderable, StencilRenderable};
use crate::renderbuffer::{Renderbuffer, RenderbufferData, RenderbufferHandle};
use crate::rendering_context::{Connection, RenderingContext};
use crate::task::{GpuTask, Progress};
use crate::texture::{Texture2DImageRef, TextureImage, TextureImageData};
use crate::util::JsId;

trait FramebufferDescriptor<Rc>
where
    Rc: RenderingContext,
{
    type ColorAttachment0: ColorAttachable + AsFramebufferAttachment<Rc>;

    type ColorAttachment1: ColorAttachable + AsFramebufferAttachment<Rc>;

    type ColorAttachment2: ColorAttachable + AsFramebufferAttachment<Rc>;

    type ColorAttachment3: ColorAttachable + AsFramebufferAttachment<Rc>;

    type ColorAttachment4: ColorAttachable + AsFramebufferAttachment<Rc>;

    type ColorAttachment5: ColorAttachable + AsFramebufferAttachment<Rc>;

    type ColorAttachment6: ColorAttachable + AsFramebufferAttachment<Rc>;

    type ColorAttachment7: ColorAttachable + AsFramebufferAttachment<Rc>;

    type ColorAttachment8: ColorAttachable + AsFramebufferAttachment<Rc>;

    type ColorAttachment9: ColorAttachable + AsFramebufferAttachment<Rc>;

    type ColorAttachment10: ColorAttachable + AsFramebufferAttachment<Rc>;

    type ColorAttachment11: ColorAttachable + AsFramebufferAttachment<Rc>;

    type ColorAttachment12: ColorAttachable + AsFramebufferAttachment<Rc>;

    type ColorAttachment13: ColorAttachable + AsFramebufferAttachment<Rc>;

    type ColorAttachment14: ColorAttachable + AsFramebufferAttachment<Rc>;

    type ColorAttachment15: ColorAttachable + AsFramebufferAttachment<Rc>;

    type DepthAttachment: DepthAttachable + AsFramebufferAttachment<Rc>;

    type StencilAttachment: StencilAttachable + AsFramebufferAttachment<Rc>;

    fn color_attachment_0(&self) -> &Self::ColorAttachment0;

    fn color_attachment_1(&self) -> &Self::ColorAttachment1;

    fn color_attachment_2(&self) -> &Self::ColorAttachment2;

    fn color_attachment_3(&self) -> &Self::ColorAttachment3;

    fn color_attachment_4(&self) -> &Self::ColorAttachment4;

    fn color_attachment_5(&self) -> &Self::ColorAttachment5;

    fn color_attachment_6(&self) -> &Self::ColorAttachment6;

    fn color_attachment_7(&self) -> &Self::ColorAttachment7;

    fn color_attachment_8(&self) -> &Self::ColorAttachment8;

    fn color_attachment_9(&self) -> &Self::ColorAttachment9;

    fn color_attachment_10(&self) -> &Self::ColorAttachment10;

    fn color_attachment_11(&self) -> &Self::ColorAttachment11;

    fn color_attachment_12(&self) -> &Self::ColorAttachment12;

    fn color_attachment_13(&self) -> &Self::ColorAttachment13;

    fn color_attachment_14(&self) -> &Self::ColorAttachment14;

    fn color_attachment_15(&self) -> &Self::ColorAttachment15;

    fn depth_attachment(&self) -> &Self::DepthAttachment;

    fn stencil_attachment(&self) -> &Self::StencilAttachment;
}

pub trait AsFramebufferAttachment<Rc>
where
    Rc: RenderingContext,
{
    fn as_framebuffer_attachment(&self) -> FramebufferAttachment<Rc>;
}

pub struct FramebufferDescriptorBuilder<
    Rc,
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
    _marker: marker::PhantomData<Rc>,
}

impl<Rc>
    FramebufferDescriptorBuilder<
        Rc,
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
where
    Rc: RenderingContext,
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
            _marker: marker::PhantomData,
        }
    }
}

impl<Rc, C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12, C13, C14, C15, D, S>
    FramebufferDescriptorBuilder<
        Rc,
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
    Rc: RenderingContext,
    C0: ColorAttachable + AsFramebufferAttachment<Rc>,
    C1: ColorAttachable + AsFramebufferAttachment<Rc>,
    C2: ColorAttachable + AsFramebufferAttachment<Rc>,
    C3: ColorAttachable + AsFramebufferAttachment<Rc>,
    C4: ColorAttachable + AsFramebufferAttachment<Rc>,
    C5: ColorAttachable + AsFramebufferAttachment<Rc>,
    C6: ColorAttachable + AsFramebufferAttachment<Rc>,
    C7: ColorAttachable + AsFramebufferAttachment<Rc>,
    C8: ColorAttachable + AsFramebufferAttachment<Rc>,
    C9: ColorAttachable + AsFramebufferAttachment<Rc>,
    C10: ColorAttachable + AsFramebufferAttachment<Rc>,
    C11: ColorAttachable + AsFramebufferAttachment<Rc>,
    C12: ColorAttachable + AsFramebufferAttachment<Rc>,
    C13: ColorAttachable + AsFramebufferAttachment<Rc>,
    C14: ColorAttachable + AsFramebufferAttachment<Rc>,
    C15: ColorAttachable + AsFramebufferAttachment<Rc>,
    D: DepthAttachable + AsFramebufferAttachment<Rc>,
    S: StencilAttachable + AsFramebufferAttachment<Rc>,
{
    fn color_attachment_0<A>(
        self,
        attachable: A,
    ) -> FramebufferDescriptorBuilder<
        Rc,
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
        A: ColorAttachable + AsFramebufferAttachment<Rc>,
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
            _marker: marker::PhantomData,
        }
    }

    fn color_attachment_1<A>(
        self,
        attachable: A,
    ) -> FramebufferDescriptorBuilder<
        Rc,
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
        A: ColorAttachable + AsFramebufferAttachment<Rc>,
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
            _marker: marker::PhantomData,
        }
    }

    fn color_attachment_2<A>(
        self,
        attachable: A,
    ) -> FramebufferDescriptorBuilder<
        Rc,
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
        A: ColorAttachable + AsFramebufferAttachment<Rc>,
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
            _marker: marker::PhantomData,
        }
    }

    fn color_attachment_3<A>(
        self,
        attachable: A,
    ) -> FramebufferDescriptorBuilder<
        Rc,
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
        A: ColorAttachable + AsFramebufferAttachment<Rc>,
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
            _marker: marker::PhantomData,
        }
    }

    fn color_attachment_4<A>(
        self,
        attachable: A,
    ) -> FramebufferDescriptorBuilder<
        Rc,
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
        A: ColorAttachable + AsFramebufferAttachment<Rc>,
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
            _marker: marker::PhantomData,
        }
    }

    fn color_attachment_5<A>(
        self,
        attachable: A,
    ) -> FramebufferDescriptorBuilder<
        Rc,
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
        A: ColorAttachable + AsFramebufferAttachment<Rc>,
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
            _marker: marker::PhantomData,
        }
    }

    fn color_attachment_6<A>(
        self,
        attachable: A,
    ) -> FramebufferDescriptorBuilder<
        Rc,
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
        A: ColorAttachable + AsFramebufferAttachment<Rc>,
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
            _marker: marker::PhantomData,
        }
    }

    fn color_attachment_7<A>(
        self,
        attachable: A,
    ) -> FramebufferDescriptorBuilder<
        Rc,
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
        A: ColorAttachable + AsFramebufferAttachment<Rc>,
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
            _marker: marker::PhantomData,
        }
    }

    fn color_attachment_8<A>(
        self,
        attachable: A,
    ) -> FramebufferDescriptorBuilder<
        Rc,
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
        A: ColorAttachable + AsFramebufferAttachment<Rc>,
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
            _marker: marker::PhantomData,
        }
    }

    fn color_attachment_9<A>(
        self,
        attachable: A,
    ) -> FramebufferDescriptorBuilder<
        Rc,
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
        A: ColorAttachable + AsFramebufferAttachment<Rc>,
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
            _marker: marker::PhantomData,
        }
    }

    fn color_attachment_10<A>(
        self,
        attachable: A,
    ) -> FramebufferDescriptorBuilder<
        Rc,
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
        A: ColorAttachable + AsFramebufferAttachment<Rc>,
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
            _marker: marker::PhantomData,
        }
    }

    fn color_attachment_11<A>(
        self,
        attachable: A,
    ) -> FramebufferDescriptorBuilder<
        Rc,
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
        A: ColorAttachable + AsFramebufferAttachment<Rc>,
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
            _marker: marker::PhantomData,
        }
    }

    fn color_attachment_12<A>(
        self,
        attachable: A,
    ) -> FramebufferDescriptorBuilder<
        Rc,
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
        A: ColorAttachable + AsFramebufferAttachment<Rc>,
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
            _marker: marker::PhantomData,
        }
    }

    fn color_attachment_13<A>(
        self,
        attachable: A,
    ) -> FramebufferDescriptorBuilder<
        Rc,
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
        A: ColorAttachable + AsFramebufferAttachment<Rc>,
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
            _marker: marker::PhantomData,
        }
    }

    fn color_attachment_14<A>(
        self,
        attachable: A,
    ) -> FramebufferDescriptorBuilder<
        Rc,
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
        A: ColorAttachable + AsFramebufferAttachment<Rc>,
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
            _marker: marker::PhantomData,
        }
    }

    fn color_attachment_15<A>(
        self,
        attachable: A,
    ) -> FramebufferDescriptorBuilder<
        Rc,
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
        A: ColorAttachable + AsFramebufferAttachment<Rc>,
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
            _marker: marker::PhantomData,
        }
    }

    fn depth_attachment<A>(
        self,
        attachable: A,
    ) -> FramebufferDescriptorBuilder<
        Rc,
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
        A: DepthAttachable + AsFramebufferAttachment<Rc>,
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
            _marker: marker::PhantomData,
        }
    }

    fn stencil_attachment<A>(
        self,
        attachable: A,
    ) -> FramebufferDescriptorBuilder<
        Rc,
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
        A: StencilAttachable + AsFramebufferAttachment<Rc>,
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
            _marker: marker::PhantomData,
        }
    }

    fn depth_stencil_attachment<A>(
        self,
        attachable: A,
    ) -> FramebufferDescriptorBuilder<
        Rc,
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
        A: DepthAttachable + StencilAttachable + AsFramebufferAttachment<Rc> + Clone,
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
            _marker: marker::PhantomData,
        }
    }

    fn finish(
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

impl<Rc, C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12, C13, C14, C15, D, S>
    FramebufferDescriptor<Rc>
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
    Rc: RenderingContext,
    C0: ColorAttachable + AsFramebufferAttachment<Rc>,
    C1: ColorAttachable + AsFramebufferAttachment<Rc>,
    C2: ColorAttachable + AsFramebufferAttachment<Rc>,
    C3: ColorAttachable + AsFramebufferAttachment<Rc>,
    C4: ColorAttachable + AsFramebufferAttachment<Rc>,
    C5: ColorAttachable + AsFramebufferAttachment<Rc>,
    C6: ColorAttachable + AsFramebufferAttachment<Rc>,
    C7: ColorAttachable + AsFramebufferAttachment<Rc>,
    C8: ColorAttachable + AsFramebufferAttachment<Rc>,
    C9: ColorAttachable + AsFramebufferAttachment<Rc>,
    C10: ColorAttachable + AsFramebufferAttachment<Rc>,
    C11: ColorAttachable + AsFramebufferAttachment<Rc>,
    C12: ColorAttachable + AsFramebufferAttachment<Rc>,
    C13: ColorAttachable + AsFramebufferAttachment<Rc>,
    C14: ColorAttachable + AsFramebufferAttachment<Rc>,
    C15: ColorAttachable + AsFramebufferAttachment<Rc>,
    D: DepthAttachable + AsFramebufferAttachment<Rc>,
    S: StencilAttachable + AsFramebufferAttachment<Rc>,
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

pub struct FramebufferAttachment<Rc>
where
    Rc: RenderingContext,
{
    internal: FramebufferAttachmentInternal<Rc>,
}

enum FramebufferAttachmentInternal<Rc>
where
    Rc: RenderingContext,
{
    TextureImage(TextureImageData<Rc>),
    Renderbuffer(Arc<RenderbufferData<Rc>>),
    Empty,
}

impl<Rc> AsFramebufferAttachment<Rc> for ()
where
    Rc: RenderingContext,
{
    fn as_framebuffer_attachment(&self) -> FramebufferAttachment<Rc> {
        FramebufferAttachment {
            internal: FramebufferAttachmentInternal::Empty,
        }
    }
}

impl<Rc, T> AsFramebufferAttachment<Rc> for Option<T>
    where
        T: AsFramebufferAttachment<Rc>,
        Rc: RenderingContext,
{
    fn as_framebuffer_attachment(&self) -> FramebufferAttachment<Rc> {
        match self {
            Some(a) => a.as_framebuffer_attachment(),
            None => FramebufferAttachment {
                internal: FramebufferAttachmentInternal::Empty
            }
        }
    }
}

impl<F, Rc> AsFramebufferAttachment<Rc> for RenderbufferHandle<F, Rc> where Rc: RenderingContext {
    fn as_framebuffer_attachment(&self) -> FramebufferAttachment<Rc> {
        FramebufferAttachment {
            internal: FramebufferAttachmentInternal::Renderbuffer(self.data.clone())
        }
    }
}

impl<F, Rc> AsFramebufferAttachment<Rc> for Texture2DImageRef<F, Rc> where Rc: RenderingContext {
    fn as_framebuffer_attachment(&self) -> FramebufferAttachment<Rc> {
        FramebufferAttachment {
            internal: FramebufferAttachmentInternal::TextureImage(self.data.clone())
        }
    }
}

pub unsafe trait ColorAttachable {}

unsafe impl<F, Rc> ColorAttachable for Texture2DImageRef<F, Rc>
where
    F: ColorRenderable,
    Rc: RenderingContext,
{
}

unsafe impl<F, Rc> ColorAttachable for RenderbufferHandle<F, Rc>
where
    F: ColorRenderable,
    Rc: RenderingContext,
{
}

pub unsafe trait DepthAttachable {}

unsafe impl<F, Rc> DepthAttachable for Texture2DImageRef<F, Rc>
where
    F: DepthRenderable,
    Rc: RenderingContext,
{
}

unsafe impl<F, Rc> DepthAttachable for RenderbufferHandle<F, Rc>
where
    F: DepthRenderable,
    Rc: RenderingContext,
{
}

pub unsafe trait StencilAttachable {}

unsafe impl<F, Rc> StencilAttachable for Texture2DImageRef<F, Rc>
where
    F: StencilRenderable,
    Rc: RenderingContext,
{
}

unsafe impl<F, Rc> StencilAttachable for RenderbufferHandle<F, Rc>
where
    F: StencilRenderable,
    Rc: RenderingContext,
{
}

pub struct FramebufferHandle<Rc>
where
    Rc: RenderingContext,
{
    data: Arc<FramebufferData<Rc>>,
}

struct FramebufferData<Rc>
where
    Rc: RenderingContext,
{
    context: Rc,
    gl_object_id: Option<JsId>,
    color_attachments: [FramebufferAttachment<Rc>; 16],
    depth_attachment: FramebufferAttachment<Rc>,
    stencil_attachment: FramebufferAttachment<Rc>,
}

impl<Rc> Drop for FramebufferData<Rc>
where
    Rc: RenderingContext,
{
    fn drop(&mut self) {
        if let Some(id) = self.gl_object_id {
            self.context.submit(FramebufferDropTask { id });
        }
    }
}

struct FramebufferDropTask {
    id: JsId,
}

impl GpuTask<Connection> for FramebufferDropTask {
    type Output = ();

    type Error = ();

    fn progress(&mut self, connection: &mut Connection) -> Progress<Self::Output, Self::Error> {
        let Connection(gl, _) = connection;

        unsafe {
            gl.delete_framebuffer(Some(&JsId::into_value(self.id).unchecked_into()));
        }

        Progress::Finished(Ok(()))
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum DrawBuffer {
    None,
    ColorAttachment0,
    ColorAttachment1,
    ColorAttachment2,
    ColorAttachment3,
    ColorAttachment4,
    ColorAttachment5,
    ColorAttachment6,
    ColorAttachment7,
    ColorAttachment8,
    ColorAttachment9,
    ColorAttachment10,
    ColorAttachment11,
    ColorAttachment12,
    ColorAttachment13,
    ColorAttachment14,
    ColorAttachment15,
}

pub struct RenderPass<T> {
    task: T,
}

impl<T> GpuTask<Connection> for RenderPass<T>
where
    T: GpuTask<RenderPassContext>,
{
    type Output = T::Output;

    type Error = T::Error;

    fn progress(&mut self, connection: &mut Connection) -> Progress<Self::Output, Self::Error> {
        // TODO: bind framebuffer

        self.task.progress(&mut RenderPassContext {
            connection: connection as *mut _,
        })
    }
}

pub struct RenderPassContext {
    connection: *mut Connection,
}

impl Deref for RenderPassContext {
    type Target = Connection;

    fn deref(&self) -> &Connection {
        unsafe { &*self.connection }
    }
}

impl DerefMut for RenderPassContext {
    fn deref_mut(&mut self) -> &mut Connection {
        unsafe { &mut *self.connection }
    }
}

pub struct SubPass<T> {
    draw_buffers: [DrawBuffer; 16],
    task: T,
}

pub fn sub_pass<B, T>(draw_buffers: B, task: T) -> SubPass<T>
where
    B: IntoIterator<Item = DrawBuffer>,
    T: GpuTask<SubPassContext>,
{
    let mut draw_buffer_array = [DrawBuffer::None; 16];

    for (i, buffer) in draw_buffers.into_iter().enumerate() {
        draw_buffer_array[i] = buffer;
    }

    SubPass {
        draw_buffers: draw_buffer_array,
        task,
    }
}

impl<T> GpuTask<RenderPassContext> for SubPass<T>
where
    T: GpuTask<SubPassContext>,
{
    type Output = T::Output;

    type Error = T::Error;

    fn progress(&mut self, context: &mut RenderPassContext) -> Progress<Self::Output, Self::Error> {
        // TODO: set draw_buffers

        self.task.progress(&mut SubPassContext {
            connection: context.connection,
        })
    }
}

pub struct SubPassContext {
    connection: *mut Connection,
}

impl Deref for SubPassContext {
    type Target = Connection;

    fn deref(&self) -> &Connection {
        unsafe { &*self.connection }
    }
}

impl DerefMut for SubPassContext {
    fn deref_mut(&mut self) -> &mut Connection {
        unsafe { &mut *self.connection }
    }
}
