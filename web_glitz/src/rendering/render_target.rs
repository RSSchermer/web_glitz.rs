use std::cell::Cell;
use std::cmp;
use std::hash::{Hash, Hasher};

use crate::image::format::{
    DepthRenderable, DepthStencilRenderable, FloatRenderable, IntegerRenderable, Multisamplable,
    Multisample, RenderbufferFormat, StencilRenderable, TextureFormat, UnsignedIntegerRenderable,
};
use crate::image::renderbuffer::Renderbuffer;
use crate::image::texture_2d::LevelMut as Texture2DLevelMut;
use crate::image::texture_2d_array::LevelLayerMut as Texture2DArrayLevelLayerMut;
use crate::image::texture_3d::LevelLayerMut as Texture3DLevelLayerMut;
use crate::image::texture_cube::LevelFaceMut as TextureCubeLevelFaceMut;
use crate::rendering::attachment::AttachmentData;
use crate::rendering::load_op::LoadAction;
use crate::rendering::{
    AsAttachment, AsMultisampleAttachment, ColorBufferEncoding, ColorBufferEncodingContext,
    DepthAttachment, DepthStencilAttachment, DepthStencilBufferEncoding,
    DepthStencilBufferEncodingContext, EncodeColorBuffer, EncodeDepthStencilBuffer,
    EncodeMultisampleColorBuffer, EncodeMultisampleDepthStencilBuffer, FloatAttachment,
    Framebuffer, GraphicsPipelineTarget, IntegerAttachment, LoadOp, MultisampleFramebuffer,
    RenderPass, RenderPassContext, StencilAttachment, StoreOp, UnsignedIntegerAttachment,
};
use crate::runtime::single_threaded::RenderPassIdGen;
use crate::runtime::state::{AttachmentSet, DepthStencilAttachmentDescriptor, DrawBuffer};
use crate::task::{ContextId, GpuTask};

/// Marker trait for image reference types that may be attached to a [RenderTargetDescriptor] as a
/// floating point color attachment.
///
/// See [RenderTargetDescriptor::attach_color_float] for details.
pub unsafe trait AttachColorFloat: AsAttachment {}

unsafe impl<'a, T> AttachColorFloat for &'a mut T where T: AttachColorFloat {}

unsafe impl<'a, F> AttachColorFloat for Texture2DLevelMut<'a, F> where
    F: TextureFormat + FloatRenderable
{
}

unsafe impl<'a, F> AttachColorFloat for Texture2DArrayLevelLayerMut<'a, F> where
    F: TextureFormat + FloatRenderable
{
}

unsafe impl<'a, F> AttachColorFloat for Texture3DLevelLayerMut<'a, F> where
    F: TextureFormat + FloatRenderable
{
}

unsafe impl<'a, F> AttachColorFloat for TextureCubeLevelFaceMut<'a, F> where
    F: TextureFormat + FloatRenderable
{
}

unsafe impl<F> AttachColorFloat for Renderbuffer<F> where
    F: RenderbufferFormat + FloatRenderable + 'static
{
}

/// Marker trait for image reference types that may be attached to a [RenderTargetDescriptor] as an
/// integer color attachment.
///
/// See [RenderTargetDescriptor::attach_color_integer] for details.
pub unsafe trait AttachColorInteger: AsAttachment {}

unsafe impl<'a, T> AttachColorInteger for &'a mut T where T: AttachColorInteger {}

unsafe impl<'a, F> AttachColorInteger for Texture2DLevelMut<'a, F> where
    F: TextureFormat + IntegerRenderable
{
}

unsafe impl<'a, F> AttachColorInteger for Texture2DArrayLevelLayerMut<'a, F> where
    F: TextureFormat + IntegerRenderable
{
}

unsafe impl<'a, F> AttachColorInteger for Texture3DLevelLayerMut<'a, F> where
    F: TextureFormat + IntegerRenderable
{
}

unsafe impl<'a, F> AttachColorInteger for TextureCubeLevelFaceMut<'a, F> where
    F: TextureFormat + IntegerRenderable
{
}

unsafe impl<F> AttachColorInteger for Renderbuffer<F> where
    F: RenderbufferFormat + IntegerRenderable + 'static
{
}

/// Marker trait for image reference types that may be attached to a [RenderTargetDescriptor] as an
/// unsigned integer color attachment.
///
/// See [RenderTargetDescriptor::attach_color_unsigned_integer] for details.
pub unsafe trait AttachColorUnsignedInteger: AsAttachment {}

unsafe impl<'a, T> AttachColorUnsignedInteger for &'a mut T where T: AttachColorUnsignedInteger {}

unsafe impl<'a, F> AttachColorUnsignedInteger for Texture2DLevelMut<'a, F> where
    F: TextureFormat + UnsignedIntegerRenderable
{
}

unsafe impl<'a, F> AttachColorUnsignedInteger for Texture2DArrayLevelLayerMut<'a, F> where
    F: TextureFormat + UnsignedIntegerRenderable
{
}

unsafe impl<'a, F> AttachColorUnsignedInteger for Texture3DLevelLayerMut<'a, F> where
    F: TextureFormat + UnsignedIntegerRenderable
{
}

unsafe impl<'a, F> AttachColorUnsignedInteger for TextureCubeLevelFaceMut<'a, F> where
    F: TextureFormat + UnsignedIntegerRenderable
{
}

unsafe impl<F> AttachColorUnsignedInteger for Renderbuffer<F> where
    F: RenderbufferFormat + UnsignedIntegerRenderable + 'static
{
}

/// Marker trait for image reference types that may be attached to a [RenderTargetDescriptor] as a
/// depth-stencil attachment.
///
/// See [RenderTargetDescriptor::attach_depth_stencil] for details.
pub unsafe trait AttachDepthStencil: AsAttachment {}

unsafe impl<'a, T> AttachDepthStencil for &'a mut T where T: AttachDepthStencil {}

unsafe impl<'a, F> AttachDepthStencil for Texture2DLevelMut<'a, F> where
    F: TextureFormat + DepthStencilRenderable
{
}

unsafe impl<'a, F> AttachDepthStencil for Texture2DArrayLevelLayerMut<'a, F> where
    F: TextureFormat + DepthStencilRenderable
{
}

unsafe impl<'a, F> AttachDepthStencil for Texture3DLevelLayerMut<'a, F> where
    F: TextureFormat + DepthStencilRenderable
{
}

unsafe impl<'a, F> AttachDepthStencil for TextureCubeLevelFaceMut<'a, F> where
    F: TextureFormat + DepthStencilRenderable
{
}

unsafe impl<F> AttachDepthStencil for Renderbuffer<F> where
    F: RenderbufferFormat + DepthStencilRenderable + 'static
{
}

/// Marker trait for image reference types that may be attached to a [RenderTargetDescriptor] as a
/// depth attachment.
///
/// See [RenderTargetDescriptor::attach_depth] for details.
pub unsafe trait AttachDepth: AsAttachment {}

unsafe impl<'a, T> AttachDepth for &'a mut T where T: AttachDepth {}

unsafe impl<'a, F> AttachDepth for Texture2DLevelMut<'a, F> where F: TextureFormat + DepthRenderable {}

unsafe impl<'a, F> AttachDepth for Texture2DArrayLevelLayerMut<'a, F> where
    F: TextureFormat + DepthRenderable
{
}

unsafe impl<'a, F> AttachDepth for Texture3DLevelLayerMut<'a, F> where
    F: TextureFormat + DepthRenderable
{
}

unsafe impl<'a, F> AttachDepth for TextureCubeLevelFaceMut<'a, F> where
    F: TextureFormat + DepthRenderable
{
}

unsafe impl<F> AttachDepth for Renderbuffer<F> where
    F: RenderbufferFormat + DepthRenderable + 'static
{
}

/// Marker trait for image reference types that may be attached to a [RenderTargetDescriptor] as a
/// stencil attachment.
///
/// See [RenderTargetDescriptor::attach_stencil] for details.
pub unsafe trait AttachStencil: AsAttachment {}

unsafe impl<'a, T> AttachStencil for &'a mut T where T: AttachStencil {}

unsafe impl<'a, F> AttachStencil for Texture2DLevelMut<'a, F> where
    F: TextureFormat + StencilRenderable
{
}

unsafe impl<'a, F> AttachStencil for Texture2DArrayLevelLayerMut<'a, F> where
    F: TextureFormat + StencilRenderable
{
}

unsafe impl<'a, F> AttachStencil for Texture3DLevelLayerMut<'a, F> where
    F: TextureFormat + StencilRenderable
{
}

unsafe impl<'a, F> AttachStencil for TextureCubeLevelFaceMut<'a, F> where
    F: TextureFormat + StencilRenderable
{
}

unsafe impl<F> AttachStencil for Renderbuffer<F> where
    F: RenderbufferFormat + StencilRenderable + 'static
{
}

/// Describes a [RenderTarget] for a [RenderPass].
///
/// Zero or more color images and zero or 1 depth-stencil image(s) may be attached to the render
/// target. Together these images define a [Framebuffer] which commands in the render pass task may
/// modify: a [RenderBuffer] will be allocated in the framebuffer for each of the attached images.
/// For each attachment, the render target must also describe how image data from the image is
/// loaded into its associated render buffer before the render pass, and how image data from the
/// render buffer is stored back into the image after the render pass.
///
/// See also [RenderingContext::create_render_target] and
/// [RenderingContext::try_create_render_target] for details on how a [RenderTargetDescriptor] may
/// be used to initialize a [RenderTarget].
///
/// # Examples
///
/// The following example constructs a [RenderTargetDescriptor] with one color attachment:
///
/// ```
/// # use web_glitz::runtime::RenderingContext;
/// # fn wrapper<Rc>(context: &Rc) where Rc: RenderingContext {
/// use web_glitz::image::format::RGBA8;
/// use web_glitz::image::renderbuffer::RenderbufferDescriptor;
/// use web_glitz::rendering::{RenderTargetDescriptor, LoadOp, StoreOp};
///
/// let mut color_image = context.create_renderbuffer(&RenderbufferDescriptor{
///     format: RGBA8,
///     width: 500,
///     height: 500
/// });
///
/// let render_target_descriptor = RenderTargetDescriptor::new()
///     .attach_color_float(&mut color_image, LoadOp::Load, StoreOp::Store);
/// # }
/// ```
///
/// We attach a [Renderbuffer] as the first (and only) color attachment that stores floating point
/// pixel data. We declare a "load operation" of `LoadOp::Load` to indicate that at the start of
/// any [RenderPass] that uses this render target, the image data currently stored in the
/// renderbuffer needs to be loaded into framebuffer. A declare a "store operation" of
/// `StoreOp::Store` to indicate that at the end of the render pass, the data in the framebuffer (as
/// modified by the render pass task) needs to be stored back into the renderbuffer. See [LoadOp]
/// and [StoreOp] for details on alternative load and store operations.
///
/// The image we attach must implement [AsAttachment], as well as the [AttachColorFloat] marker
/// trait to indicate that the image's [InternalFormat] is compatible with floating point data. You
/// may attach other image types than [Renderbuffer]s; for example, we might also use a [Texture2D]
/// level:
///
/// ```
/// # use web_glitz::runtime::RenderingContext;
///
/// # fn wrapper<Rc>(context: &Rc) where Rc: RenderingContext {
/// use web_glitz::image::format::RGBA8;
/// use web_glitz::image::texture_2d::Texture2DDescriptor;
/// use web_glitz::image::MipmapLevels;
/// use web_glitz::rendering::{RenderTargetDescriptor, LoadOp, StoreOp};
///
/// let mut color_image = context.try_create_texture_2d(&Texture2DDescriptor {
///     format: RGBA8,
///     width: 500,
///     height: 500,
///     levels: MipmapLevels::Complete
/// }).unwrap();
///
/// let render_target_descriptor = RenderTargetDescriptor::new()
///     .attach_color_float(&mut color_image, LoadOp::Load, StoreOp::Store);
/// # }
/// ```
///
/// As we use [attach_color_float] to attach the image as a floating point attachment, the
/// [RenderBuffer] that is allocated for this attachment in framebuffer will store the image data as
/// float values. Any graphics pipeline that draws to the framebuffer during the render pass must
/// output float values (rather than integer or unsigned integer values).
///
/// You may also attach color images that store integer values with [attach_color_integer] if the
/// image reference implements the [AttachColorInteger] marker trait:
///
/// ```
/// # use web_glitz::runtime::RenderingContext;
/// # fn wrapper<Rc>(context: &Rc) where Rc: RenderingContext {
/// use web_glitz::image::format::RGBA8I;
/// use web_glitz::image::renderbuffer::RenderbufferDescriptor;
/// use web_glitz::rendering::{RenderTargetDescriptor, LoadOp, StoreOp};
///
/// let mut color_image = context.create_renderbuffer(&RenderbufferDescriptor{
///     format: RGBA8I,
///     width: 500,
///     height: 500
/// });
///
/// let render_target_descriptor = RenderTargetDescriptor::new()
///     .attach_color_integer(&mut color_image, LoadOp::Load, StoreOp::Store);
/// # }
/// ```
///
/// Or color images that store unsigned integer values with [attach_color_unsigned_integer] if the
/// image reference implements the [AttachColorUnsignedInteger] marker trait:
///
/// ```
/// # use web_glitz::runtime::RenderingContext;
/// # fn wrapper<Rc>(context: &Rc) where Rc: RenderingContext {
/// use web_glitz::image::format::RGBA8UI;
/// use web_glitz::image::renderbuffer::RenderbufferDescriptor;
/// use web_glitz::rendering::{RenderTargetDescriptor, LoadOp, StoreOp};
///
/// let mut color_image = context.create_renderbuffer(&RenderbufferDescriptor{
///     format: RGBA8UI,
///     width: 500,
///     height: 500
/// });
///
/// let render_target_descriptor = RenderTargetDescriptor::new()
///     .attach_color_unsigned_integer(&mut color_image, LoadOp::Load, StoreOp::Store);
/// # }
/// ```
///
/// It is also possible to attach multiple color images (up to 16):
///
/// ```
/// # use web_glitz::runtime::RenderingContext;
/// # fn wrapper<Rc>(context: &Rc) where Rc: RenderingContext {
/// use web_glitz::image::format::{RGBA8, RGBA8I};
/// use web_glitz::image::renderbuffer::RenderbufferDescriptor;
/// use web_glitz::rendering::{RenderTargetDescriptor, LoadOp, StoreOp};
///
/// let mut color_image_0 = context.create_renderbuffer(&RenderbufferDescriptor{
///     format: RGBA8,
///     width: 500,
///     height: 500
/// });
///
/// let mut color_image_1 = context.create_renderbuffer(&RenderbufferDescriptor{
///     format: RGBA8,
///     width: 500,
///     height: 500
/// });
///
/// let render_target_descriptor = RenderTargetDescriptor::new()
///     .attach_color_float(&mut color_image_0, LoadOp::Load, StoreOp::Store)
///     .attach_color_integer(&mut color_image_1, LoadOp::Load, StoreOp::Store);
/// # }
/// ```
///
/// The first image that is attached will be attached to the framebuffer in color slot `0`, the
/// second image will be attached in color slot `1`, etc.
///
/// Note that a rendering context does not always support up to 16 color attachments (see
/// [RenderingContext::max_color_attachments]), but always support at least 1 color attachment.
/// Therefor, if you declare a [RenderTargetDescriptor] with only a single color attachment, you may
/// use [RenderingContext::create_render_target] to obtain a [RenderTarget]; this always succeeds.
/// However, if you declare a [RenderTargetDescriptor] with more than 1 color attachment, you must
/// use [RenderingContext::try_create_render_target] instead, which may fail with an error if the
/// number of attachments you specified exceeds [RenderingContext::max_color_attachments].
///
/// Finally, you may attach a single depth-stencil image. This may be an image that stores combined
/// depth and stencil values attached with [attach_depth_stencil] if the image reference implements
/// the [AttachDepthStencil] marker trait:
///
/// ```
/// # use web_glitz::runtime::RenderingContext;
/// # fn wrapper<Rc>(context: &Rc) where Rc: RenderingContext {
/// use web_glitz::image::format::Depth24Stencil8;
/// use web_glitz::image::renderbuffer::RenderbufferDescriptor;
/// use web_glitz::rendering::{RenderTargetDescriptor, LoadOp, StoreOp};
///
/// let mut depth_stencil_image = context.create_renderbuffer(&RenderbufferDescriptor{
///     format: Depth24Stencil8,
///     width: 500,
///     height: 500
/// });
///
/// let render_target_descriptor = RenderTargetDescriptor::new()
///     .attach_depth_stencil(&mut depth_stencil_image, LoadOp::Load, StoreOp::Store);
/// # }
/// ```
///
/// Or an image that stores only depth values attached with [attach_depth] if the image reference
/// implements the [AttachDepth] marker trait:
///
/// ```
/// # use web_glitz::runtime::RenderingContext;
/// # fn wrapper<Rc>(context: &Rc) where Rc: RenderingContext {
/// use web_glitz::image::format::DepthComponent24;
/// use web_glitz::image::renderbuffer::RenderbufferDescriptor;
/// use web_glitz::rendering::{RenderTargetDescriptor, LoadOp, StoreOp};
///
/// let mut depth_image = context.create_renderbuffer(&RenderbufferDescriptor{
///     format: DepthComponent24,
///     width: 500,
///     height: 500
/// });
///
/// let render_target_descriptor = RenderTargetDescriptor::new()
///     .attach_depth(&mut depth_image, LoadOp::Load, StoreOp::Store);
/// # }
/// ```
///
/// Or an image that stores only stencil values attached with [attach_stencil] if the image
/// reference implements the [AttachStencil] marker trait:
///
/// ```
/// # use web_glitz::runtime::RenderingContext;
/// # fn wrapper<Rc>(context: &Rc) where Rc: RenderingContext {
/// use web_glitz::image::format::StencilIndex8;
/// use web_glitz::image::renderbuffer::RenderbufferDescriptor;
/// use web_glitz::rendering::{RenderTargetDescriptor, LoadOp, StoreOp};
///
/// let mut stencil_image = context.create_renderbuffer(&RenderbufferDescriptor{
///     format: StencilIndex8,
///     width: 500,
///     height: 500
/// });
///
/// let render_target_descriptor = RenderTargetDescriptor::new()
///     .attach_stencil(&mut stencil_image, LoadOp::Load, StoreOp::Store);
/// # }
/// ```
pub struct RenderTargetDescriptor<C, Ds> {
    pub(crate) color_attachments: C,
    pub(crate) depth_stencil_attachment: Ds,
    pub(crate) color_attachment_count: usize,
}

impl RenderTargetDescriptor<(), ()> {
    /// Creates a new [RenderTargetDescriptor] without any attachments.
    pub fn new() -> Self {
        RenderTargetDescriptor {
            color_attachments: (),
            depth_stencil_attachment: (),
            color_attachment_count: 0,
        }
    }
}

impl<C> RenderTargetDescriptor<C, ()> {
    /// Attaches an image to the depth-stencil slot that stores combined depth and stencil values.
    ///
    /// # Example
    ///
    /// ```
    /// # use web_glitz::runtime::RenderingContext;
    /// # fn wrapper<Rc>(context: &Rc) where Rc: RenderingContext {
    /// use web_glitz::image::format::Depth24Stencil8;
    /// use web_glitz::image::renderbuffer::RenderbufferDescriptor;
    /// use web_glitz::rendering::{RenderTargetDescriptor, LoadOp, StoreOp};
    ///
    /// let mut depth_stencil_image = context.create_renderbuffer(&RenderbufferDescriptor{
    ///     format: Depth24Stencil8,
    ///     width: 500,
    ///     height: 500
    /// });
    ///
    /// let render_target_descriptor = RenderTargetDescriptor::new()
    ///     .attach_depth_stencil(&mut depth_stencil_image, LoadOp::Load, StoreOp::Store);
    /// # }
    /// ```
    pub fn attach_depth_stencil<Ds>(
        self,
        image: Ds,
        load_op: LoadOp<(f32, i32)>,
        store_op: StoreOp,
    ) -> RenderTargetDescriptor<C, DepthStencilAttachment<Ds>>
    where
        Ds: AttachDepthStencil,
    {
        RenderTargetDescriptor {
            color_attachments: self.color_attachments,
            depth_stencil_attachment: DepthStencilAttachment {
                image,
                load_op,
                store_op,
            },
            color_attachment_count: self.color_attachment_count,
        }
    }

    /// Attaches an image to the depth-stencil slot that stores depth values.
    ///
    /// # Example
    ///
    /// ```
    /// # use web_glitz::runtime::RenderingContext;
    /// # fn wrapper<Rc>(context: &Rc) where Rc: RenderingContext {
    /// use web_glitz::image::format::DepthComponent24;
    /// use web_glitz::image::renderbuffer::RenderbufferDescriptor;
    /// use web_glitz::rendering::{RenderTargetDescriptor, LoadOp, StoreOp};
    ///
    /// let mut depth_image = context.create_renderbuffer(&RenderbufferDescriptor{
    ///     format: DepthComponent24,
    ///     width: 500,
    ///     height: 500
    /// });
    ///
    /// let render_target_descriptor = RenderTargetDescriptor::new()
    ///     .attach_depth(&mut depth_image, LoadOp::Load, StoreOp::Store);
    /// # }
    /// ```
    pub fn attach_depth<Ds>(
        self,
        image: Ds,
        load_op: LoadOp<f32>,
        store_op: StoreOp,
    ) -> RenderTargetDescriptor<C, DepthAttachment<Ds>>
    where
        Ds: AttachDepth,
    {
        RenderTargetDescriptor {
            color_attachments: self.color_attachments,
            depth_stencil_attachment: DepthAttachment {
                image,
                load_op,
                store_op,
            },
            color_attachment_count: self.color_attachment_count,
        }
    }

    /// Attaches an image to the depth-stencil slot that stores stencil values.
    ///
    /// # Example
    ///
    /// ```
    /// # use web_glitz::runtime::RenderingContext;
    /// # fn wrapper<Rc>(context: &Rc) where Rc: RenderingContext {
    /// use web_glitz::image::format::StencilIndex8;
    /// use web_glitz::image::renderbuffer::RenderbufferDescriptor;
    /// use web_glitz::rendering::{RenderTargetDescriptor, LoadOp, StoreOp};
    ///
    /// let mut stencil_image = context.create_renderbuffer(&RenderbufferDescriptor{
    ///     format: StencilIndex8,
    ///     width: 500,
    ///     height: 500
    /// });
    ///
    /// let render_target_descriptor = RenderTargetDescriptor::new()
    ///     .attach_stencil(&mut stencil_image, LoadOp::Load, StoreOp::Store);
    /// # }
    /// ```
    pub fn attach_stencil<Ds>(
        self,
        image: Ds,
        load_op: LoadOp<i32>,
        store_op: StoreOp,
    ) -> RenderTargetDescriptor<C, StencilAttachment<Ds>>
    where
        Ds: AttachStencil,
    {
        RenderTargetDescriptor {
            color_attachments: self.color_attachments,
            depth_stencil_attachment: StencilAttachment {
                image,
                load_op,
                store_op,
            },
            color_attachment_count: self.color_attachment_count,
        }
    }
}

macro_rules! impl_attach_render_target_color {
    ($count:tt, $($C:ident),*) => {
        impl<$($C,)* Ds> RenderTargetDescriptor<($($C,)*), Ds> {
            /// Attaches an image that stores floating point values to the next color slot.
            ///
            /// # Example
            ///
            /// ```
            /// # use web_glitz::runtime::RenderingContext;
            /// # fn wrapper<Rc>(context: &Rc) where Rc: RenderingContext {
            /// use web_glitz::image::format::RGBA8;
            /// use web_glitz::image::renderbuffer::RenderbufferDescriptor;
            /// use web_glitz::rendering::{RenderTargetDescriptor, LoadOp, StoreOp};
            ///
            /// let mut color_image = context.create_renderbuffer(&RenderbufferDescriptor{
            ///     format: RGBA8,
            ///     width: 500,
            ///     height: 500
            /// });
            ///
            /// let render_target_descriptor = RenderTargetDescriptor::new()
            ///     .attach_color_float(&mut color_image, LoadOp::Load, StoreOp::Store);
            /// # }
            /// ```
            pub fn attach_color_float<C>(self, image: C, load_op: LoadOp<[f32; 4]>, store_op: StoreOp) -> RenderTargetDescriptor<($($C,)* FloatAttachment<C>,), Ds> where C: AttachColorFloat {
                #[allow(non_snake_case)]
                let ($($C,)*) = self.color_attachments;

                RenderTargetDescriptor {
                    color_attachments: ($($C,)* FloatAttachment {
                        image,
                        load_op,
                        store_op
                    },),
                    depth_stencil_attachment: self.depth_stencil_attachment,
                    color_attachment_count: $count
                }
            }

            /// Attaches an image that stores integer values to the next color slot.
            ///
            /// # Example
            ///
            /// ```
            /// # use web_glitz::runtime::RenderingContext;
            /// # fn wrapper<Rc>(context: &Rc) where Rc: RenderingContext {
            /// use web_glitz::image::format::RGBA8I;
            /// use web_glitz::image::renderbuffer::RenderbufferDescriptor;
            /// use web_glitz::rendering::{RenderTargetDescriptor, LoadOp, StoreOp};
            ///
            /// let mut color_image = context.create_renderbuffer(&RenderbufferDescriptor{
            ///     format: RGBA8I,
            ///     width: 500,
            ///     height: 500
            /// });
            ///
            /// let render_target_descriptor = RenderTargetDescriptor::new()
            ///     .attach_color_integer(&mut color_image, LoadOp::Load, StoreOp::Store);
            /// # }
            /// ```
            pub fn attach_color_integer<C>(self, image: C, load_op: LoadOp<[i32; 4]>, store_op: StoreOp) -> RenderTargetDescriptor<($($C,)* IntegerAttachment<C>,), Ds> where C: AttachColorInteger {
                #[allow(non_snake_case)]
                let ($($C,)*) = self.color_attachments;

                RenderTargetDescriptor {
                    color_attachments: ($($C,)* IntegerAttachment {
                        image,
                        load_op,
                        store_op
                    },),
                    depth_stencil_attachment: self.depth_stencil_attachment,
                    color_attachment_count: $count
                }
            }

            /// Attaches an image that stores integer values to the next color slot.
            ///
            /// # Example
            ///
            /// ```
            /// # use web_glitz::runtime::RenderingContext;
            /// # fn wrapper<Rc>(context: &Rc) where Rc: RenderingContext {
            /// use web_glitz::image::format::RGBA8UI;
            /// use web_glitz::image::renderbuffer::RenderbufferDescriptor;
            /// use web_glitz::rendering::{RenderTargetDescriptor, LoadOp, StoreOp};
            ///
            /// let mut color_image = context.create_renderbuffer(&RenderbufferDescriptor{
            ///     format: RGBA8UI,
            ///     width: 500,
            ///     height: 500
            /// });
            ///
            /// let render_target_descriptor = RenderTargetDescriptor::new()
            ///     .attach_color_unsigned_integer(&mut color_image, LoadOp::Load, StoreOp::Store);
            /// # }
            /// ```
            pub fn attach_color_unsigned_integer<C>(self, image: C, load_op: LoadOp<[u32; 4]>, store_op: StoreOp) -> RenderTargetDescriptor<($($C,)* UnsignedIntegerAttachment<C>,), Ds> where C: AttachColorUnsignedInteger {
                #[allow(non_snake_case)]
                let ($($C,)*) = self.color_attachments;

                RenderTargetDescriptor {
                    color_attachments: ($($C,)* UnsignedIntegerAttachment {
                        image,
                        load_op,
                        store_op
                    },),
                    depth_stencil_attachment: self.depth_stencil_attachment,
                    color_attachment_count: $count
                }
            }
        }
    }
}

impl_attach_render_target_color!(1,);
impl_attach_render_target_color!(2, C0);
impl_attach_render_target_color!(3, C0, C1);
impl_attach_render_target_color!(4, C0, C1, C2);
impl_attach_render_target_color!(5, C0, C1, C2, C3);
impl_attach_render_target_color!(6, C0, C1, C2, C3, C4);
impl_attach_render_target_color!(7, C0, C1, C2, C3, C4, C5);
impl_attach_render_target_color!(8, C0, C1, C2, C3, C4, C5, C6);
impl_attach_render_target_color!(9, C0, C1, C2, C3, C4, C5, C6, C7);
impl_attach_render_target_color!(10, C0, C1, C2, C3, C4, C5, C6, C7, C8);
impl_attach_render_target_color!(11, C0, C1, C2, C3, C4, C5, C6, C7, C8, C9);
impl_attach_render_target_color!(12, C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10);
impl_attach_render_target_color!(13, C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11);
impl_attach_render_target_color!(14, C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12);
impl_attach_render_target_color!(15, C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12, C13);
impl_attach_render_target_color!(
    16, C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12, C13, C14
);

/// Marker trait for multisample image reference types that may be attached to a
/// [MultisampleRenderTargetDescriptor] as a floating point color attachment.
///
/// See [MultisampleRenderTargetDescriptor::attach_color_float] for details.
pub unsafe trait AttachMultisampleColorFloat: AsMultisampleAttachment {}

unsafe impl<'a, T> AttachMultisampleColorFloat for &'a mut T where T: AttachMultisampleColorFloat {}

unsafe impl<F> AttachMultisampleColorFloat for Renderbuffer<Multisample<F>> where
    F: RenderbufferFormat + Multisamplable + FloatRenderable + 'static
{
}

/// Marker trait for multisample image reference types that may be attached to a
/// [MultisampleRenderTargetDescriptor] as a depth-stencil attachment.
///
/// See [MultisampleRenderTargetDescriptor::attach_depth_stencil] for details.
pub unsafe trait AttachMultisampleDepthStencil: AsMultisampleAttachment {}

unsafe impl<'a, T> AttachMultisampleDepthStencil for &'a mut T where T: AttachMultisampleDepthStencil
{}

unsafe impl<F> AttachMultisampleDepthStencil for Renderbuffer<Multisample<F>> where
    F: RenderbufferFormat + Multisamplable + DepthStencilRenderable + 'static
{
}

/// Marker trait for multisample image reference types that may be attached to a
/// [MultisampleRenderTargetDescriptor] as a depth attachment.
///
/// See [MultisampleRenderTargetDescriptor::attach_depth] for details.
pub unsafe trait AttachMultisampleDepth: AsMultisampleAttachment {}

unsafe impl<'a, T> AttachMultisampleDepth for &'a mut T where T: AttachMultisampleDepth {}

unsafe impl<F> AttachMultisampleDepth for Renderbuffer<Multisample<F>> where
    F: RenderbufferFormat + Multisamplable + DepthRenderable + 'static
{
}

/// Describes a [MultisampleRenderTarget] for a [RenderPass].
///
/// Similar to a [RenderTargetDescriptor], except in that when you initially create the descriptor,
/// you must specify the number of samples the render target will use:
///
/// ```
/// # use web_glitz::runtime::RenderingContext;
/// # fn wrapper<Rc>(context: &Rc) where Rc: RenderingContext {
/// use web_glitz::rendering::MultisampleRenderTargetDescriptor;
///
/// let render_target_descriptor = MultisampleRenderTargetDescriptor::new(4);
/// # }
/// ```
///
/// Attaching an image that uses a different number of samples will cause a panic:
///
/// ```
/// # use web_glitz::runtime::RenderingContext;
/// # fn wrapper<Rc>(context: &Rc) where Rc: RenderingContext {
/// use web_glitz::image::format::{Multisample, RGBA8};
/// use web_glitz::image::renderbuffer::MultisampleRenderbufferDescriptor;
/// use web_glitz::rendering::{RenderTargetDescriptor, LoadOp, StoreOp};
///
/// let mut color_image = context.create_multisample_renderbuffer(&MultisampleRenderbufferDescriptor {
///     format: Multisample(RGBA8, 16),
///     width: 500,
///     height: 500
/// });
///
/// render_target_descriptor.attach_color_float(&mut color_image, LoadOp::Load, StoreOp::Store);
///
/// // Panic! The descriptor excepts images that use 4 samples, but we're trying to attach an image
/// // that uses 16 samples!
/// # }
/// ```
///
/// In all other ways, constructing a [MultisampleRenderTargetDescriptor] is the same as
/// constructing a [RenderTargetDescriptor]; please refer to the documentation for
/// [RenderTargetDescriptor] for more examples and details on the attachment types.
pub struct MultisampleRenderTargetDescriptor<C, Ds> {
    pub(crate) color_attachments: C,
    pub(crate) depth_stencil_attachment: Ds,
    pub(crate) samples: usize,
    pub(crate) color_attachment_count: usize,
}

impl MultisampleRenderTargetDescriptor<(), ()> {
    /// Creates a new [MultisampleRenderTargetDescriptor] that excepts it attachments to use a
    /// sample count of `samples`.
    pub fn new(samples: usize) -> Self {
        MultisampleRenderTargetDescriptor {
            color_attachments: (),
            depth_stencil_attachment: (),
            samples,
            color_attachment_count: 0,
        }
    }
}

impl<C> MultisampleRenderTargetDescriptor<C, ()> {
    /// Attaches an image to the depth-stencil slot that stores combined depth and stencil values.
    ///
    /// # Example
    ///
    /// ```
    /// # use web_glitz::runtime::RenderingContext;
    /// # fn wrapper<Rc>(context: &Rc) where Rc: RenderingContext {
    /// use web_glitz::image::format::{Depth24Stencil8, Multisample};
    /// use web_glitz::image::renderbuffer::MultisampleRenderbufferDescriptor;
    /// use web_glitz::rendering::{RenderTargetDescriptor, LoadOp, StoreOp};
    ///
    /// let mut depth_stencil_image = context.create_multisample_renderbuffer(&MultisampleRenderbufferDescriptor {
    ///     format: Multisample(Depth24Stencil8, 4),
    ///     width: 500,
    ///     height: 500
    /// });
    ///
    /// let render_target_descriptor = MultisampleRenderTargetDescriptor::new(4)
    ///     .attach_depth_stencil(&mut depth_stencil_image, LoadOp::Load, StoreOp::Store);
    /// # }
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if the number of samples used by the `image` does not match the nubmer of samples
    /// specified for this [MultisampleRenderTargetDescriptor] (see
    /// [MultisampleRenderTargetDescriptor::new]).
    pub fn attach_depth_stencil<Ds>(
        self,
        mut image: Ds,
        load_op: LoadOp<(f32, i32)>,
        store_op: StoreOp,
    ) -> MultisampleRenderTargetDescriptor<C, DepthStencilAttachment<Ds>>
    where
        Ds: AttachMultisampleDepthStencil,
    {
        let image_samples = image.as_multisample_attachment().samples();

        if image_samples != self.samples {
            panic!(
                "Descriptor expects {} samples, but image uses {} samples",
                self.samples, image_samples
            );
        }

        MultisampleRenderTargetDescriptor {
            color_attachments: self.color_attachments,
            depth_stencil_attachment: DepthStencilAttachment {
                image,
                load_op,
                store_op,
            },
            samples: self.samples,
            color_attachment_count: self.color_attachment_count,
        }
    }

    /// Attaches an image to the depth-stencil slot that stores combined depth and stencil values.
    ///
    /// # Example
    ///
    /// ```
    /// # use web_glitz::runtime::RenderingContext;
    /// # fn wrapper<Rc>(context: &Rc) where Rc: RenderingContext {
    /// use web_glitz::image::format::{DepthComponent24, Multisample};
    /// use web_glitz::image::renderbuffer::MultisampleRenderbufferDescriptor;
    /// use web_glitz::rendering::{RenderTargetDescriptor, LoadOp, StoreOp};
    ///
    /// let mut depth_image = context.create_multisample_renderbuffer(&MultisampleRenderbufferDescriptor {
    ///     format: Multisample(DepthComponent24, 4),
    ///     width: 500,
    ///     height: 500
    /// });
    ///
    /// let render_target_descriptor = MultisampleRenderTargetDescriptor::new(4)
    ///     .attach_depth(&mut depth_image, LoadOp::Load, StoreOp::Store);
    /// # }
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if the number of samples used by the `image` does not match the nubmer of samples
    /// specified for this [MultisampleRenderTargetDescriptor] (see
    /// [MultisampleRenderTargetDescriptor::new]).
    pub fn attach_depth<Ds>(
        self,
        mut image: Ds,
        load_op: LoadOp<f32>,
        store_op: StoreOp,
    ) -> MultisampleRenderTargetDescriptor<C, DepthAttachment<Ds>>
    where
        Ds: AttachMultisampleDepth,
    {
        let image_samples = image.as_multisample_attachment().samples();

        if image_samples != self.samples {
            panic!(
                "Descriptor expects {} samples, but image uses {} samples",
                self.samples, image_samples
            );
        }

        MultisampleRenderTargetDescriptor {
            color_attachments: self.color_attachments,
            depth_stencil_attachment: DepthAttachment {
                image,
                load_op,
                store_op,
            },
            samples: self.samples,
            color_attachment_count: self.color_attachment_count,
        }
    }
}

macro_rules! impl_attach_multisample_render_target_color {
    ($count:tt, $($C:ident),*) => {
        impl<$($C,)* Ds> MultisampleRenderTargetDescriptor<($($C,)*), Ds> {
            /// Attaches an image that stores floating point values to the next color slot.
            ///
            /// # Example
            ///
            /// ```
            /// # use web_glitz::runtime::RenderingContext;
            /// # fn wrapper<Rc>(context: &Rc) where Rc: RenderingContext {
            /// use web_glitz::image::format::{Multisample, RGBA8};
            /// use web_glitz::image::renderbuffer::MultisampleRenderbufferDescriptor;
            /// use web_glitz::rendering::{RenderTargetDescriptor, LoadOp, StoreOp};
            ///
            /// let mut color_image = context.create_multisample_renderbuffer(&MultisampleRenderbufferDescriptor{
            ///     format: Multisample(RGBA8, 4),
            ///     width: 500,
            ///     height: 500
            /// });
            ///
            /// let render_target_descriptor = MultisampleRenderTargetDescriptor::new(4)
            ///     .attach_color_float(&mut color_image, LoadOp::Load, StoreOp::Store);
            /// # }
            /// ```
            ///
            /// # Panics
            ///
            /// Panics if the number of samples used by the `image` does not match the nubmer of
            /// samples specified for this [MultisampleRenderTargetDescriptor] (see
            /// [MultisampleRenderTargetDescriptor::new]).
            pub fn attach_color_float<C>(
                self,
                mut image: C,
                load_op: LoadOp<[f32; 4]>,
                store_op: StoreOp
            ) -> MultisampleRenderTargetDescriptor<($($C,)* FloatAttachment<C>,), Ds>
                where C: AttachMultisampleColorFloat
            {
                let image_samples = image.as_multisample_attachment().samples();

                if image_samples != self.samples {
                    panic!(
                        "Descriptor expects {} samples, but image uses {} samples",
                        self.samples,
                        image_samples
                    );
                }

                #[allow(non_snake_case)]
                let ($($C,)*) = self.color_attachments;

                MultisampleRenderTargetDescriptor {
                    color_attachments: ($($C,)* FloatAttachment {
                        image,
                        load_op,
                        store_op
                    },),
                    depth_stencil_attachment: self.depth_stencil_attachment,
                    samples: self.samples,
                    color_attachment_count: $count
                }
            }
        }
    }
}

impl_attach_multisample_render_target_color!(1,);
impl_attach_multisample_render_target_color!(2, C0);
impl_attach_multisample_render_target_color!(3, C0, C1);
impl_attach_multisample_render_target_color!(4, C0, C1, C2);
impl_attach_multisample_render_target_color!(5, C0, C1, C2, C3);
impl_attach_multisample_render_target_color!(6, C0, C1, C2, C3, C4);
impl_attach_multisample_render_target_color!(7, C0, C1, C2, C3, C4, C5);
impl_attach_multisample_render_target_color!(8, C0, C1, C2, C3, C4, C5, C6);
impl_attach_multisample_render_target_color!(9, C0, C1, C2, C3, C4, C5, C6, C7);
impl_attach_multisample_render_target_color!(10, C0, C1, C2, C3, C4, C5, C6, C7, C8);
impl_attach_multisample_render_target_color!(11, C0, C1, C2, C3, C4, C5, C6, C7, C8, C9);
impl_attach_multisample_render_target_color!(12, C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10);
impl_attach_multisample_render_target_color!(13, C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11);
impl_attach_multisample_render_target_color!(
    14, C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12
);
impl_attach_multisample_render_target_color!(
    15, C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12, C13
);
impl_attach_multisample_render_target_color!(
    16, C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12, C13, C14
);

/// Describes a target for rendering operations.
///
/// A [RenderTarget] is used to create [RenderPass] tasks, see below.
///
/// A render target consists of a collection of references to renderable images, from which image
/// data may be loaded into [Framebuffer] memory, then modified by rendering commands as part of a
/// [RenderPass], and then the modified image data may be stored back into the images. These images
/// are said to be "attached" to the render target and are referred to as its "attachments".
///
/// A [RenderTarget] may be initialized from a [RenderTargetDescriptor] with
/// [RenderingContext::create_render_target] if the descriptor declares only a single color
/// attachment, or with [RenderingContext::try_create_render_target] if the descriptor declares
/// multiple color attachments, which may fail with an error if the number of color attachments
/// exceeds [RenderingContext::max_color_attachments].
///
/// Note that the [RenderTarget] holds on to exclusive references to the images attached to it; this
/// prevents you from accidentally also binding these images as pipeline resources within the
/// [RenderPass] (attempting to read from an image while it is loaded in the framebuffer is
/// undefined behavior).
///
/// [RenderTarget]s are cheap to create (their creation does not involve any interaction with the
/// GPU backend). You are encouraged to create ephemeral [RenderTarget]s every time you want to
/// define a [RenderPass], rather than attempting to keep a [RenderTarget] alive (which is
/// cumbersome as this will keep exclusive borrows to each of the attached images alive for the
/// lifetime of the [RenderTarget]).
///
/// # Creating a render pass
///
/// A new render pass can be created by calling [create_render_pass], passing it a function that
/// takes a [Framebuffer] as its input and returns a render pass task:
///
/// ```
/// # use web_glitz::rendering::{LoadOp, StoreOp};
/// # use web_glitz::runtime::RenderingContext;
/// # use web_glitz::image::renderbuffer::RenderbufferDescriptor;
/// # use web_glitz::image::format::RGBA8;
/// # fn wrapper<Rc>(
/// #     context: &Rc,
/// # )
/// # where
/// #     Rc: RenderingContext,
/// # {
/// let mut color_image = context.create_renderbuffer(&RenderbufferDescriptor{
///     format: RGBA8,
///     width: 500,
///     height: 500
/// });
///
/// let render_target_descriptor = RenderTargetDescriptor::new()
///     .attach_color_float(&mut color_image, LoadOp::Load, StoreOp::Store);
///
/// //  Mark our render target as `mut`, as `create_render_pass` requires a `&mut` reference.
/// let mut render_target = context.create_render_target(render_target_descriptor);
///
/// let render_pass = render_target.create_render_pass(|framebuffer| {
///     // Return a task that modifies the framebuffer, see the documentation for the
///     // `Framebuffer` type for details. For this example, we'll clear the first (and only) color
///     // attachment to "opaque red".
///     framebuffer.color.0.clear_command([1.0, 0.0, 0.0, 1.0])
/// });
/// # }
/// ```
///
/// Rendering output buffers will be allocated in the framebuffer for each of the color images
/// attached to the render target (if any) and for the depth-stencil image attached to the render
/// target (if any).
///
/// The function must return a [GpuTask] that can be executed in a [RenderPassContext]. This
/// function will receive a reference to the framebuffer representation associated with the
/// render target. It may use this reference to construct commands that make up that task, see
/// [Framebuffer] for details.
///
/// At the start of the render pass, the [LoadOp]s associated with each of the images attached to
/// the render target will be performed to initialize the framebuffer. Then the task returned from
/// the function is executed, which may modify the contents of the framebuffer. Finally, the
/// [StoreOp]s associated with each of the images attached to the render target will be performed to
/// store the (modified) contents of the framebuffer back to these images.
pub struct RenderTarget<C, Ds> {
    pub(crate) color_attachments: C,
    pub(crate) depth_stencil_attachment: Ds,
    pub(crate) context_id: usize,
    pub(crate) render_pass_id_gen: RenderPassIdGen,
}

pub struct MultisampleRenderTarget<C, Ds> {
    pub(crate) color_attachments: C,
    pub(crate) depth_stencil_attachment: Ds,
    pub(crate) samples: usize,
    pub(crate) context_id: usize,
    pub(crate) render_pass_id_gen: RenderPassIdGen,
}

macro_rules! impl_create_render_pass {
    ($C0:ident $(,$C:ident)*) => {
        #[allow(unused_parens)]
        impl<$C0 $(,$C)*> RenderTarget<($C0, $($C,)*), ()>
        where
            $C0: EncodeColorBuffer, $($C: EncodeColorBuffer,)*
        {
            /// Creates a new [RenderPass] that will output to this [RenderTarget].
            ///
            /// The `f` function will receive a reference to a [Framebuffer] with a buffer layout
            /// that matches the [RenderTarget]'s attachment layout.
            ///
            /// For details and examples on defining render passes, please refer to the struct
            /// documentation for [RenderTarget].
            ///
            /// # Panics
            ///
            /// Panics if the render pass context ID associated with the task returned from `f` does
            /// not match the ID generated for this render pass (the task returned from `f` must not
            /// contain commands that were created for a different render pass).
            #[allow(non_snake_case, unused_mut, unused_parens)]
            pub fn create_render_pass<F, T>(&mut self, f: F) -> RenderPass<T>
                where
                    F: FnOnce(&Framebuffer<($C0::Buffer, $($C::Buffer,)*), ()>) -> T,
                    T: GpuTask<RenderPassContext>
            {
                let id = self.render_pass_id_gen.next();

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

                let ($C0, $($C,)*) = &mut self.color_attachments;

                let mut context = ColorBufferEncodingContext {
                    render_pass_id: id,
                    buffer_index: 0,
                };

                let $C0 = $C0.encode_color_buffer(&mut context);

                let mut width = $C0.image.width;
                let mut height = $C0.image.height;

                let $C0 = {
                    let ColorBufferEncoding {
                        load_action,
                        store_op,
                        image,
                        buffer,
                        ..
                    } = $C0;

                    render_target.load_ops[0] = load_action;
                    render_target.store_ops[0] = store_op;
                    render_target.color_attachments[0] = Some(image);

                    buffer
                };

                let mut color_count = 1;

                $(
                    let mut context = ColorBufferEncodingContext {
                        render_pass_id: id,
                        buffer_index: color_count as i32,
                    };

                    let $C = $C.encode_color_buffer(&mut context);

                    width = cmp::min(width, $C.image.width);
                    height = cmp::min(height, $C.image.height);

                    let $C = {
                        let ColorBufferEncoding {
                            load_action,
                            store_op,
                            image,
                            buffer,
                            ..
                        } = $C;

                        render_target.load_ops[color_count] = load_action;
                        render_target.store_ops[color_count] = store_op;
                        render_target.color_attachments[color_count] = Some(image);

                        buffer
                    };

                    color_count += 1;
                )*

                render_target.color_count = color_count;

                let task = f(&Framebuffer {
                    color: ($C0, $($C,)*),
                    depth_stencil: (),
                    data: GraphicsPipelineTarget {
                        dimensions: Some((width, height)),
                        context_id: self.context_id,
                        render_pass_id: id,
                        last_pipeline_task_id: Cell::new(0),
                    }
                });

                if let ContextId::Id(render_pass_id) = task.context_id() {
                    if render_pass_id != id {
                        panic!("The render pass task belongs to a different render pass.")
                    }
                }

                RenderPass {
                    id,
                    context_id: self.context_id,
                    render_target: RenderTargetData::Custom(render_target),
                    task
                }
            }
        }

        #[allow(unused_parens)]
        impl<$C0 $(,$C)*> MultisampleRenderTarget<($C0, $($C,)*), ()>
        where
            $C0: EncodeMultisampleColorBuffer $(,$C: EncodeMultisampleColorBuffer)*
        {
            /// Creates a new [RenderPass] that will output to this [MultisampleRenderTarget].
            ///
            /// The `f` function will receive a reference to a [MultisampleFramebuffer] with a
            /// buffer layout that matches the [MultisampleRenderTarget]'s attachment layout.
            ///
            /// For details and examples on defining render passes, please refer to the struct
            /// documentation for [RenderTarget].
            ///
            /// # Panics
            ///
            /// Panics if the render pass context ID associated with the task returned from `f` does
            /// not match the ID generated for this render pass (the task returned from `f` must not
            /// contain commands that were created for a different render pass).
            #[allow(non_snake_case, unused_mut, unused_parens)]
            pub fn create_render_pass<F, T>(&mut self, f: F) -> RenderPass<T>
                where
                    F: FnOnce(&MultisampleFramebuffer<($C0::Buffer, $($C::Buffer,)*), ()>) -> T,
                    T: GpuTask<RenderPassContext>
            {
                let id = self.render_pass_id_gen.next();

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

                let ($C0, $($C,)*) = &mut self.color_attachments;

                let mut context = ColorBufferEncodingContext {
                    render_pass_id: id,
                    buffer_index: 0,
                };

                let $C0 = $C0.encode_multisample_color_buffer(&mut context);

                let mut width = $C0.image.width;
                let mut height = $C0.image.height;

                let $C0 = {
                    let ColorBufferEncoding {
                        load_action,
                        store_op,
                        image,
                        buffer,
                        ..
                    } = $C0;

                    render_target.load_ops[0] = load_action;
                    render_target.store_ops[0] = store_op;
                    render_target.color_attachments[0] = Some(image);

                    buffer
                };

                let mut color_count = 1;

                $(
                    let mut context = ColorBufferEncodingContext {
                        render_pass_id: id,
                        buffer_index: color_count as i32,
                    };

                    let $C = $C.encode_multisample_color_buffer(&mut context);

                    width = cmp::min(width, $C.image.width);
                    height = cmp::min(height, $C.image.height);

                    let $C = {
                        let ColorBufferEncoding {
                            load_action,
                            store_op,
                            image,
                            buffer,
                            ..
                        } = $C;

                        render_target.load_ops[color_count] = load_action;
                        render_target.store_ops[color_count] = store_op;
                        render_target.color_attachments[color_count] = Some(image);

                        buffer
                    };

                    color_count += 1;
                )*

                render_target.color_count = color_count;

                let task = f(&MultisampleFramebuffer {
                    color: ($C0, $($C,)*),
                    depth_stencil: (),
                    samples: self.samples,
                    data: GraphicsPipelineTarget {
                        dimensions: Some((width, height)),
                        context_id: self.context_id,
                        render_pass_id: id,
                        last_pipeline_task_id: Cell::new(0),
                    }
                });

                if let ContextId::Id(render_pass_id) = task.context_id() {
                    if render_pass_id != id {
                        panic!("The render pass task belongs to a different render pass.")
                    }
                }

                RenderPass {
                    id,
                    context_id: self.context_id,
                    render_target: RenderTargetData::Custom(render_target),
                    task
                }
            }
        }

        #[allow(unused_parens)]
        impl<$($C,)* Ds> RenderTarget<($($C,)*), Ds>
        where
            $($C: EncodeColorBuffer,)*
            Ds: EncodeDepthStencilBuffer
        {
            /// Creates a new [RenderPass] that will output to this [RenderTarget].
            ///
            /// The `f` function will receive a reference to a [Framebuffer] with a buffer layout
            /// that matches the [RenderTarget]'s attachment layout.
            ///
            /// For details and examples on defining render passes, please refer to the struct
            /// documentation for [RenderTarget].
            ///
            /// # Panics
            ///
            /// Panics if the render pass context ID associated with the task returned from `f` does
            /// not match the ID generated for this render pass (the task returned from `f` must not
            /// contain commands that were created for a different render pass).
            #[allow(non_snake_case, unused_parens)]
            pub fn create_render_pass<F, T>(&mut self, f: F) -> RenderPass<T>
                where
                    F: FnOnce(&Framebuffer<($($C::Buffer,)*), Ds::Buffer>) -> T,
                    T: GpuTask<RenderPassContext>
            {
                let id = self.render_pass_id_gen.next();

                let mut context = DepthStencilBufferEncodingContext {
                    render_pass_id: id,
                };

                let DepthStencilBufferEncoding {
                    load_action,
                    store_op,
                    depth_stencil_type,
                    image,
                    buffer,
                    ..
                } = self.depth_stencil_attachment.encode_depth_stencil_buffer(&mut context);

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

                let ($($C,)*) = &mut self.color_attachments;
                let mut color_count = 0;

                $(
                    let mut context = ColorBufferEncodingContext {
                        render_pass_id: id,
                        buffer_index: color_count as i32,
                    };

                    let $C = $C.encode_color_buffer(&mut context);

                    width = cmp::min(width, $C.image.width);
                    height = cmp::min(height, $C.image.height);

                    let $C = {
                        let ColorBufferEncoding {
                            load_action,
                            store_op,
                            image,
                            buffer,
                            ..
                        } = $C;

                        render_target.load_ops[color_count] = load_action;
                        render_target.store_ops[color_count] = store_op;
                        render_target.color_attachments[color_count] = Some(image);

                        buffer
                    };

                    color_count += 1;
                )*

                render_target.color_count = color_count;

                let task = f(&Framebuffer {
                    color: ($($C,)*),
                    depth_stencil: buffer,
                    data: GraphicsPipelineTarget {
                        dimensions: Some((width, height)),
                        context_id: self.context_id,
                        render_pass_id: id,
                        last_pipeline_task_id: Cell::new(0),
                    }
                });

                if let ContextId::Id(render_pass_id) = task.context_id() {
                    if render_pass_id != id {
                        panic!("The render pass task belongs to a different render pass.")
                    }
                }

                RenderPass {
                    id,
                    context_id: self.context_id,
                    render_target: RenderTargetData::Custom(render_target),
                    task
                }
            }
        }

        #[allow(unused_parens)]
        impl<$($C,)* Ds> MultisampleRenderTarget<($($C,)*), Ds>
        where
            $($C: EncodeMultisampleColorBuffer,)*
            Ds: EncodeMultisampleDepthStencilBuffer
        {
            /// Creates a new [RenderPass] that will output to this [MultisampleRenderTarget].
            ///
            /// The `f` function will receive a reference to a [MultisampleFramebuffer] with a
            /// buffer layout that matches the [MultisampleRenderTarget]'s attachment layout.
            ///
            /// For details and examples on defining render passes, please refer to the struct
            /// documentation for [RenderTarget].
            ///
            /// # Panics
            ///
            /// Panics if the render pass context ID associated with the task returned from `f` does
            /// not match the ID generated for this render pass (the task returned from `f` must not
            /// contain commands that were created for a different render pass).
            #[allow(non_snake_case, unused_parens)]
            pub fn create_render_pass<F, T>(&mut self, f: F) -> RenderPass<T>
                where
                    F: FnOnce(&MultisampleFramebuffer<($($C::Buffer,)*), Ds::Buffer>) -> T,
                    T: GpuTask<RenderPassContext>
            {
                let id = self.render_pass_id_gen.next();

                let mut context = DepthStencilBufferEncodingContext {
                    render_pass_id: id,
                };

                let DepthStencilBufferEncoding {
                    load_action,
                    store_op,
                    depth_stencil_type,
                    image,
                    buffer,
                    ..
                } = self.depth_stencil_attachment.encode_multisample_depth_stencil_buffer(&mut context);

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

                let ($($C,)*) = &mut self.color_attachments;
                let mut color_count = 0;

                $(
                    let mut context = ColorBufferEncodingContext {
                        render_pass_id: id,
                        buffer_index: color_count as i32,
                    };

                    let $C = $C.encode_multisample_color_buffer(&mut context);

                    width = cmp::min(width, $C.image.width);
                    height = cmp::min(height, $C.image.height);

                    let $C = {
                        let ColorBufferEncoding {
                            load_action,
                            store_op,
                            image,
                            buffer,
                            ..
                        } = $C;

                        render_target.load_ops[color_count] = load_action;
                        render_target.store_ops[color_count] = store_op;
                        render_target.color_attachments[color_count] = Some(image);

                        buffer
                    };

                    color_count += 1;
                )*

                render_target.color_count = color_count;

                let task = f(&MultisampleFramebuffer {
                    color: ($($C,)*),
                    depth_stencil: buffer,
                    samples: self.samples,
                    data: GraphicsPipelineTarget {
                        dimensions: Some((width, height)),
                        context_id: self.context_id,
                        render_pass_id: id,
                        last_pipeline_task_id: Cell::new(0),
                    }
                });

                if let ContextId::Id(render_pass_id) = task.context_id() {
                    if render_pass_id != id {
                        panic!("The render pass task belongs to a different render pass.")
                    }
                }

                RenderPass {
                    id,
                    context_id: self.context_id,
                    render_target: RenderTargetData::Custom(render_target),
                    task
                }
            }
        }
    }
}

impl_create_render_pass!(C0);
impl_create_render_pass!(C0, C1);
impl_create_render_pass!(C0, C1, C2);
impl_create_render_pass!(C0, C1, C2, C3);
impl_create_render_pass!(C0, C1, C2, C3, C4);
impl_create_render_pass!(C0, C1, C2, C3, C4, C5);
impl_create_render_pass!(C0, C1, C2, C3, C4, C5, C6);
impl_create_render_pass!(C0, C1, C2, C3, C4, C5, C6, C7);
impl_create_render_pass!(C0, C1, C2, C3, C4, C5, C6, C7, C8);
impl_create_render_pass!(C0, C1, C2, C3, C4, C5, C6, C7, C8, C9);
impl_create_render_pass!(C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10);
impl_create_render_pass!(C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11);
impl_create_render_pass!(C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12);
impl_create_render_pass!(C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12, C13);
impl_create_render_pass!(C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12, C13, C14);
impl_create_render_pass!(C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12, C13, C14, C15);

#[derive(Clone)]
pub(crate) enum RenderTargetData {
    Default,
    Custom(CustomRenderTargetData),
}

#[derive(Clone)]
pub(crate) struct CustomRenderTargetData {
    pub(crate) load_ops: [LoadAction; 17],
    pub(crate) store_ops: [StoreOp; 17],
    pub(crate) color_count: usize,
    pub(crate) color_attachments: [Option<AttachmentData>; 16],
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
    fn color_attachments(&self) -> &[Option<AttachmentData>] {
        &self.color_attachments[0..self.color_count]
    }

    fn depth_stencil_attachment(&self) -> &DepthStencilAttachmentDescriptor {
        &self.depth_stencil_attachment
    }
}
