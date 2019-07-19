use std::cell::Cell;
use std::cmp;
use std::hash::{Hash, Hasher};

use crate::render_pass::{Framebuffer, RenderPass, RenderPassContext, RenderPassId};
use crate::render_target::attachable_image_ref::AttachableImageData;
use crate::render_target::render_target_attachment::{
    ColorAttachmentEncoding, ColorAttachmentEncodingContext, LoadAction,
};
use crate::render_target::{
    ColorAttachmentDescription, DepthStencilAttachmentDescription, DepthStencilAttachmentEncoding,
    DepthStencilAttachmentEncodingContext, StoreOp,
};
use crate::runtime::state::{AttachmentSet, DepthStencilAttachmentDescriptor, DrawBuffer};
use crate::task::{ContextId, GpuTask};

/// Describes a render target that may be used with a [RenderPass] task.
///
/// See [RenderTarget] for details on how to declare a valid [RenderTargetDescription].
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
        F: FnOnce(&Self::Framebuffer) -> T,
        T: GpuTask<RenderPassContext>;
}

impl<'a, T> RenderTargetDescription for &'a mut T
where
    T: RenderTargetDescription,
{
    type Framebuffer = T::Framebuffer;

    fn create_render_pass<F, Rt>(&mut self, id: RenderPassId, f: F) -> RenderPass<Rt>
    where
        F: FnOnce(&Self::Framebuffer) -> Rt,
        Rt: GpuTask<RenderPassContext>,
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
                    F: FnOnce(&Self::Framebuffer) -> T,
                    T: GpuTask<RenderPassContext>
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

                let task = f(&Framebuffer {
                    color: ($C0  $(,$C)*),
                    depth_stencil: (),
                    dimensions: Some((width, height)),
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
                    F: FnOnce(&Self::Framebuffer) -> T,
                    T: GpuTask<RenderPassContext>
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

                let task = f(&Framebuffer {
                    color: ($($C),*),
                    depth_stencil: buffer,
                    dimensions: Some((width, height)),
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

/// May be used to define a [RenderTargetDescription] for a [RenderPass].
///
/// Zero or more color images and zero or 1 depth-stencil image(s) may be attached to the render
/// target. Together these images define a [Framebuffer] that commands in the render pass task may
/// modify: a [RenderBuffer] will be allocated in the framebuffer for each of the attached images.
/// For each attachment, the render target must also describe how image data from the image is
/// loaded into its associated render buffer before the render pass, and how image data from the
/// render buffer is stored back into the image after the render pass.
///
/// # Examples
///
/// The following example defines a [RenderTarget] with one color attachment:
///
/// ```
/// # use web_glitz::runtime::RenderingContext;
/// # fn wrapper<Rc>(context: &Rc) where Rc: RenderingContext {
/// use web_glitz::image::format::RGBA8;
/// use web_glitz::image::renderbuffer::RenderbufferDescriptor;
/// use web_glitz::render_target::{RenderTarget, FloatAttachment, LoadOp, StoreOp};
///
/// let mut color_image = context.create_renderbuffer(&RenderbufferDescriptor{
///     format: RGBA8,
///     width: 500,
///     height: 500
/// });
///
/// let render_target = RenderTarget {
///     color: FloatAttachment {
///         image: &mut color_image,
///         load_op: LoadOp::Load,
///         store_op: StoreOp::Store,
///     },
///     depth_stencil: ()
/// };
/// # }
/// ```
///
/// The color attachment is declared to be a [FloatAttachment] that associates a [Renderbuffer] with
/// the render target as the first (and only) color attachment. A `load_op` value of `LoadOp::Load`
/// indicates that at the start of any [RenderPass] that uses this render target, the image data
/// currently stored in the renderbuffer will be loaded into framebuffer. A `store_op` value of
/// `StoreOp::Store` indicates that at the end of the render pass, the data in the framebuffer (as
/// modified by the render pass task) will be stored back into the renderbuffer. The value used for
/// `image` must implement [AsAttachableImageRef]. For example, we might also use a [Texture2D]
/// level:
///
/// ```
/// # use web_glitz::runtime::RenderingContext;
///
/// # fn wrapper<Rc>(context: &Rc) where Rc: RenderingContext {
/// use web_glitz::image::format::RGBA8;
/// use web_glitz::image::texture_2d::Texture2DDescriptor;
/// use web_glitz::image::MipmapLevels;
/// use web_glitz::render_target::{RenderTarget, FloatAttachment, LoadOp, StoreOp};
///
/// let mut color_image = context.create_texture_2d(&Texture2DDescriptor {
///     format: RGBA8,
///     width: 500,
///     height: 500,
///     levels: MipmapLevels::Complete
/// }).unwrap();
///
/// let render_target = RenderTarget {
///     color: FloatAttachment {
///         image: color_image.base_level_mut(),
///         load_op: LoadOp::Load,
///         store_op: StoreOp::Store,
///     },
///     depth_stencil: ()
/// };
/// # }
/// ```
///
/// As we use a [FloatAttachment], the [RenderBuffer] that is allocated for this attachment in
/// framebuffer will store the image data as float values. Any graphics pipeline that draws to the
/// framebuffer during the render pass must output float values (rather than integer or unsigned
/// integer values). The image storage format used by the attached image (`RGBA8` in the example),
/// must implement [FloatRenderable]. You may alternatively assign an [IntegerAttachment] value to
/// `color` to allocate a [RenderBuffer] that stores integer values; the image storage formats must
/// then implement [IntegerRenderable] and any graphics pipeline that draws to the framebuffer must
/// output integer values. You may also assign an [UnsignedIntegerAttachment] value to `color` to
/// allocate a [RenderBuffer] that store unsigned integer values; the image storage formats must
/// then [UnsignedIntegerRenderable], and any graphics pipeline that draws to the framebuffer must
/// output unsigned integer values.
///
/// The empty tuple `()` value for the `depth_stencil` field indicates that this render target does
/// not define a depth-stencil attachment. The next example defines a render target that does have
/// a depth-stencil attachment:
///
/// ```
/// # use web_glitz::runtime::RenderingContext;
/// # fn wrapper<Rc>(context: &Rc) where Rc: RenderingContext {
/// use web_glitz::image::format::{RGBA8, DepthComponent16};
/// use web_glitz::image::renderbuffer::RenderbufferDescriptor;
/// use web_glitz::render_target::{RenderTarget, FloatAttachment, DepthAttachment, LoadOp, StoreOp};
///
/// let mut color_image = context.create_renderbuffer(&RenderbufferDescriptor{
///     format: RGBA8,
///     width: 500,
///     height: 500
/// });
///
/// let mut depth_image = context.create_renderbuffer(&RenderbufferDescriptor{
///     format: DepthComponent16,
///     width: 500,
///     height: 500
/// });
///
/// let render_target = RenderTarget {
///     color: FloatAttachment {
///         image: &mut color_image,
///         load_op: LoadOp::Load,
///         store_op: StoreOp::Store,
///     },
///     depth_stencil: DepthAttachment {
///         image: &mut depth_image,
///         load_op: LoadOp::Clear(1.0),
///         store_op: StoreOp::DontCare
///     }
/// };
/// # }
/// ```
///
/// This render target uses a [DepthAttachment] value for `depth_stencil`: a buffer that stores
/// depth values will be allocated in the framebuffer and any graphics pipeline that draws to the
/// framebuffer during the render pass may do depth testing. A `load_op` value of
/// `LoadOp::Clear(1.0)` indicates that rather than loading the image data currently stored in
/// `depth_image` into the framebuffer at the start of the render pass, instead the depth buffer
/// will start out with every value set to `1.0`; note that starting a render pass with a cleared
/// buffer may be faster than loading image data into the buffer. A `store_op` value of
/// `StoreOp::DontCare` indicates that we do not care whether or not the data is stored back into
/// the `depth_image` after the render pass completes; this may be faster than `StoreOp::Store`.
/// Alternatively, a [StencilAttachment] value can be assigned to `depth_stencil`: a buffer that
/// stores stencil values will be allocated in the framebuffer and any graphics pipeline that draws
/// to the framebuffer during the render pass may do stencil testing, but may not do depth testing.
/// Lastly, a [DepthStencilAttachment] value can be assigned to `depth_stencil`: a buffer that
/// stores both depth values and stencil values will be allocated in the framebuffer and any
/// graphics pipeline that draws to the framebuffer during the render pass may do both depth testing
/// and stencil testing.
///
/// It is also possible to attach more than 1 color image to a render target by assigning a tuple of
/// attachments to `color`:
///
/// ```
/// # use web_glitz::runtime::RenderingContext;
/// # fn wrapper<Rc>(context: &Rc) where Rc: RenderingContext {
/// use web_glitz::image::format::{RGBA8, R8UI, DepthComponent16};
/// use web_glitz::image::renderbuffer::RenderbufferDescriptor;
/// use web_glitz::render_target::{
///     RenderTarget, FloatAttachment, UnsignedIntegerAttachment, DepthAttachment, LoadOp, StoreOp
/// };
///
/// let mut color_0_image = context.create_renderbuffer(&RenderbufferDescriptor{
///     format: RGBA8,
///     width: 500,
///     height: 500
/// });
///
/// let mut color_1_image = context.create_renderbuffer(&RenderbufferDescriptor{
///     format: R8UI,
///     width: 500,
///     height: 500
/// });
///
/// let mut depth_image = context.create_renderbuffer(&RenderbufferDescriptor{
///     format: DepthComponent16,
///     width: 500,
///     height: 500
/// });
///
/// let render_target = RenderTarget {
///     color: (
///         FloatAttachment {
///             image: &mut color_0_image,
///             load_op: LoadOp::Load,
///             store_op: StoreOp::Store,
///         },
///         UnsignedIntegerAttachment {
///             image: &mut color_1_image,
///             load_op: LoadOp::Clear([0, 0, 0, 0]),
///             store_op: StoreOp::Store,
///         },
///     ),
///     depth_stencil: DepthAttachment {
///         image: &mut depth_image,
///         load_op: LoadOp::Clear(1.0),
///         store_op: StoreOp::DontCare
///     }
/// };
/// # }
/// ```
///
/// The render target now provides a [FloatAttachment] for color output `0` and an
/// [UnsignedIntegerAttachment] for color output `1`; any graphics pipeline that draws to the
/// framebuffer may now output float values to output `0` and unsigned integer values to output `1`.
///
/// Note that even though `color_1_image` specifies a storage format that stores only 1 component
/// (`R8IU`), the render buffer allocated in the framebuffer can store 4 component values; the image
/// storage format is applied when storing values back into the image. In this case, only the first
/// component (the "red" component) will be stored, the latter 3 will be discarded.
///
/// For details on how a [RenderTarget] may be used to create a [RenderPass], see
/// [RenderingContext::create_render_pass].
pub struct RenderTarget<C, Ds> {
    /// Zero or more color images attached to this [RenderTarget].
    pub color: C,

    /// Zero or one depth-stencil image(s) attached to this [RenderTarget].
    pub depth_stencil: Ds,
}

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
