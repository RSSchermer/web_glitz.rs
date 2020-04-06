use std::borrow::Borrow;
use std::cell::{Cell, UnsafeCell};
use std::hash::{Hash, Hasher};
use std::marker;
use std::sync::Arc;

use fnv::FnvHasher;
use web_sys::WebGl2RenderingContext as Gl;

use crate::image::format::{
    DepthRenderable, DepthStencilRenderable, Filterable, FloatRenderable, IntegerRenderable,
    InternalFormat, Multisamplable, Multisample, RenderbufferFormat, StencilRenderable,
    TextureFormat, UnsignedIntegerRenderable, RGB8, RGBA8,
};
use crate::image::renderbuffer::Renderbuffer;
use crate::image::texture_2d::{Level as Texture2DLevel, LevelSubImage as Texture2DLevelSubImage};
use crate::image::texture_2d_array::{
    LevelLayer as Texture2DArrayLevelLayer, LevelLayerSubImage as Texture2DArrayLevelLayerSubImage,
};
use crate::image::texture_3d::{
    LevelLayer as Texture3DLevelLayer, LevelLayerSubImage as Texture3DLevelLayerSubImage,
};
use crate::image::texture_cube::{
    LevelFace as TextureCubeLevelFace, LevelFaceSubImage as TextureCubeLevelFaceSubImage,
};
use crate::image::Region2D;
use crate::pipeline::graphics::graphics_pipeline::{
    RecordTransformFeedback, TransformFeedbackData, TransformFeedbackState,
};
use crate::pipeline::graphics::primitive_assembly::Topology;
use crate::pipeline::graphics::shader::{FragmentShaderData, VertexShaderData};
use crate::pipeline::graphics::util::BufferDescriptors;
use crate::pipeline::graphics::{
    Blending, DepthTest, GraphicsPipeline, IndexData, IndexDataDescriptor, PrimitiveAssembly,
    StencilTest, TypedVertexBuffers, TypedVertexInputLayout, VertexBuffers,
    VertexBuffersEncodingContext, VertexInputLayoutDescriptor, Viewport,
};
use crate::pipeline::resources::{
    BindGroupDescriptor, ResourceBindings, ResourceBindingsEncodingContext, TypedResourceBindings,
    TypedResourceBindingsLayout,
};
use crate::rendering::attachment::{Attachment, AttachmentData};
use crate::rendering::RenderPassContext;
use crate::runtime::state::{BufferRange, ContextUpdate, DynamicState};
use crate::runtime::Connection;
use crate::task::{sequence, ContextId, Empty, GpuTask, Progress, Sequence};
use crate::util::JsId;
use crate::Unspecified;

/// Helper trait for implementing [Framebuffer::pipeline_task] for both a plain graphics pipeline
/// and a graphics pipeline that will record transform feedback.
pub trait GraphicsPipelineState<V, R, Tf> {
    /// Creates a new pipeline task.
    ///
    /// See [Framebuffer::pipeline_task] for details.
    fn pipeline_task<F, T>(&self, target: &GraphicsPipelineTarget, f: F) -> PipelineTask<T>
    where
        F: Fn(ActiveGraphicsPipeline<V, R, Tf>) -> T,
        T: GpuTask<PipelineTaskContext>;
}

impl<V, R, Tf> GraphicsPipelineState<V, R, Tf> for GraphicsPipeline<V, R, Tf> {
    fn pipeline_task<F, T>(&self, target: &GraphicsPipelineTarget, f: F) -> PipelineTask<T>
    where
        F: Fn(ActiveGraphicsPipeline<V, R, Tf>) -> T,
        T: GpuTask<PipelineTaskContext>,
    {
        PipelineTask::new(target, self, None, f)
    }
}

impl<'a, V, R, Tf, Fb> GraphicsPipelineState<V, R, Tf>
    for RecordTransformFeedback<'a, V, R, Tf, Fb>
{
    fn pipeline_task<F, T>(&self, target: &GraphicsPipelineTarget, f: F) -> PipelineTask<T>
    where
        F: Fn(ActiveGraphicsPipeline<V, R, Tf>) -> T,
        T: GpuTask<PipelineTaskContext>,
    {
        PipelineTask::new(target, &self.pipeline, Some(self.buffers.clone()), f)
    }
}

pub struct GraphicsPipelineTarget {
    pub(crate) dimensions: Option<(u32, u32)>,
    pub(crate) context_id: usize,
    pub(crate) render_pass_id: usize,
    pub(crate) last_pipeline_task_id: Cell<usize>,
}

/// Represents a set of image memory buffers that serve as the rendering destination for a
/// [RenderPass].
///
/// The image buffers allocated in the framebuffer correspond to to the images attached to the
/// [RenderTargetDescription] that was used to define the [RenderPass] (see also [RenderTarget]);
/// specifically, [color] provides handles to the color buffers (if any), and [depth_stencil]
/// provides a handle to the depth-stencil buffer (if any).
pub struct Framebuffer<C, Ds> {
    pub color: C,
    pub depth_stencil: Ds,
    pub(crate) data: GraphicsPipelineTarget,
}

impl<C, Ds> Framebuffer<C, Ds> {
    /// Creates a pipeline task using the given `graphics_pipeline`.
    ///
    /// The second parameter `f` must be a function that returns the task that is to be executed
    /// while the `graphics_pipeline` is bound as the active graphics pipeline. This function
    /// will receive a reference to this [ActiveGraphicsPipeline] which may be used to encode
    /// draw commands (see [ActiveGraphicsPipeline::draw_command]). The task returned by the
    /// function typically consists of 1 ore more draw commands that were created in this way. The
    /// current framebuffer serves as the output target for the `graphics_pipeline` (your draw
    /// commands may modify the current framebuffer).
    ///
    /// # Example
    ///
    /// ```
    /// # use web_glitz::rendering::DefaultRGBBuffer;
    /// # use web_glitz::rendering::DefaultRenderTarget;
    /// # use web_glitz::buffer::{Buffer, UsageHint};
    /// # use web_glitz::pipeline::graphics::{GraphicsPipeline, Vertex};
    /// # use web_glitz::pipeline::resources::BindGroup;
    /// # fn wrapper<V>(
    /// #     mut render_target: DefaultRenderTarget<DefaultRGBBuffer, ()>,
    /// #     vertex_buffer: Buffer<[V]>,
    /// #     graphics_pipeline: GraphicsPipeline<V, (), ()>
    /// # )
    /// # where
    /// #     V: Vertex,
    /// # {
    /// # let resources = BindGroup::empty();
    /// let render_pass = render_target.create_render_pass(|framebuffer| {
    ///     framebuffer.pipeline_task(&graphics_pipeline, |active_pipeline| {
    ///         active_pipeline.task_builder()
    ///             .bind_vertex_buffers(&vertex_buffer)
    ///             .bind_resources(&resources)
    ///             .draw(16, 1)
    ///             .finish()
    ///     })
    /// });
    /// # }
    /// ```
    ///
    /// In this example, `context` is a [RenderingContext]; `render_target` is a [RenderTarget], see
    /// also [DefaultRenderTarget] and [RenderTarget]; `graphics_pipeline` is a [GraphicsPipeline],
    /// see [GraphicsPipeline] for details; `vertex_buffer` is a [Buffer] holding a [Vertex] type,
    /// see [Buffer], [Vertex] for details; `resources` is a resource [BindGroup], see [BindGroup]
    /// for details.
    ///
    /// # Panics
    ///
    /// Panics if the `graphics_pipeline` belongs to a different context than the framebuffer for
    /// which this pipeline task is being created.
    ///
    /// Panics if the task returned by `f` contains commands that were constructed for a different
    /// pipeline task context.
    pub fn pipeline_task<P, V, R, Tf, F, T>(&self, pipeline: &P, f: F) -> PipelineTask<T>
    where
        P: GraphicsPipelineState<V, R, Tf>,
        F: Fn(ActiveGraphicsPipeline<V, R, Tf>) -> T,
        T: GpuTask<PipelineTaskContext>,
    {
        pipeline.pipeline_task(&self.data, f)
    }
}

impl<C, Ds> Framebuffer<C, Ds>
where
    C: BlitColorTarget,
{
    /// Transfers a rectangle of pixels from the `source` onto a `region` of each of the color
    /// buffers in framebuffer, using "nearest" filtering if the `source` and the `region` have
    /// different sizes.
    ///
    /// The image data stored in `source` must be stored in a format that is [BlitColorCompatible]
    /// with each of the color buffers in the framebuffer. If the `source` image has a different
    /// size (width or height) than the `region`, then the `source` will be scaled to match the size
    /// of the `region`. If scaling is required, then "nearest" filtering is used to obtain pixel
    /// values for the resized image, where for each pixel value in the resized image, the value
    /// of the pixel that is at the nearest corresponding relative position is used. See
    /// [blit_color_linear_command] for a similar operation that uses linear filtering instead.
    ///
    /// The `region` of the color buffers is constrained to the area of intersection of all color
    /// buffers; a `region` value of [Region::Fill] will match this area of intersection (note that
    /// the origin of a region is in its bottom-left corner). If a `region` bigger than the
    /// intersection is specified with [Region::Area], then any pixels that would be copied outside
    /// the region of overlap are discarded for all color buffers (even color buffers that would by
    /// themselves have been large enough to contain the `region`). However, the amount of scaling
    /// that is applied is based solely on the size of the `region`, it is not affected by the area
    /// of intersection.
    ///
    /// # Example
    ///
    /// ```
    /// # use web_glitz::rendering::{RenderTarget, FloatAttachment};
    /// # use web_glitz::image::texture_2d::Texture2D;
    /// # use web_glitz::image::format::RGBA8;
    /// # use web_glitz::image::renderbuffer::Renderbuffer;
    /// # fn wrapper(
    /// # mut render_target: RenderTarget<(FloatAttachment<Renderbuffer<RGBA8>>,), ()>,
    /// # texture: Texture2D<RGBA8>
    /// # ) {
    /// use web_glitz::image::Region2D;
    ///
    /// let render_pass = render_target.create_render_pass(|framebuffer| {
    ///     framebuffer.blit_color_nearest_command(Region2D::Fill, &texture.base_level())
    /// });
    /// # }
    /// ```
    ///
    /// Here `render_target` is a [RenderTarget]  or [DefaultRenderTarget] and `texture` is a
    /// [Texture2D].
    ///
    /// # Panics
    ///
    /// Panics if `source` belongs to a different context than the framebuffer.
    pub fn blit_color_nearest_command<S>(&self, region: Region2D, source: &S) -> BlitCommand
    where
        S: BlitColorCompatible<C>,
    {
        let source_descriptor = source.descriptor();

        if source_descriptor.context_id != self.data.context_id {
            panic!("The source image belongs to a different context than the framebuffer.");
        }

        BlitCommand {
            render_pass_id: self.data.render_pass_id,
            read_slot: Gl::COLOR_ATTACHMENT0,
            bitmask: Gl::COLOR_BUFFER_BIT,
            filter: Gl::NEAREST,
            target_region: region,
            target: self.color.descriptor(),
            source: source_descriptor,
        }
    }

    /// Transfers a rectangle of pixels from the `source` onto a `region` of each of the color
    /// buffers in framebuffer, using "linear" filtering if the `source` and the `region` have
    /// different sizes.
    ///
    /// The image data stored in `source` must be stored in a format that is [BlitColorCompatible]
    /// with each of the color buffers in the framebuffer. If the `source` image has a different
    /// size (width or height) than the `region`, then the `source` will be scaled to match the size
    /// of the `region`. If scaling is required, then "linear" filtering is used to obtain pixel
    /// values for the resized image, where for each pixel value in the resized image, the value is
    /// obtained by linear interpolation of the 4 pixels that are nearest to corresponding relative
    /// position in the source image. See [blit_color_nearest_command] for a similar operation that
    /// uses "nearest" filtering instead.
    ///
    /// The `region` of the color buffers is constrained to the area of intersection of all color
    /// buffers; a `region` value of [Region::Fill] will match this area of intersection (note that
    /// the origin of a region is in its bottom-left corner). If a `region` bigger than the
    /// intersection is specified with [Region::Area], then any pixels that would be copied outside
    /// the region of overlap are discarded for all color buffers (even color buffers that would by
    /// themselves have been large enough to contain the `region`). However, the amount of scaling
    /// that is applied is based solely on the size of the `region`, it is not affected by the area
    /// of intersection.
    ///
    /// # Example
    ///
    /// ```
    /// # use web_glitz::rendering::{RenderTarget, FloatAttachment};
    /// # use web_glitz::image::texture_2d::Texture2D;
    /// # use web_glitz::image::renderbuffer::Renderbuffer;
    /// # use web_glitz::image::format::RGBA8;
    /// # fn wrapper(
    /// # mut render_target: RenderTarget<(FloatAttachment<Renderbuffer<RGBA8>>,), ()>,
    /// # texture: Texture2D<RGBA8>
    /// # ) {
    /// use web_glitz::image::Region2D;
    ///
    /// let render_pass = render_target.create_render_pass(|framebuffer| {
    ///     framebuffer.blit_color_linear_command(Region2D::Fill, &texture.base_level())
    /// });
    /// # }
    /// ```
    ///
    /// Here `render_target` is a [RenderTarget] or [DefaultRenderTarget] and `texture` is a
    /// [Texture2D].
    ///
    /// # Panics
    ///
    /// Panics if `source` belongs to a different context than the framebuffer.
    pub fn blit_color_linear_command<S>(&self, region: Region2D, source: &S) -> BlitCommand
    where
        S: BlitColorCompatible<C>,
        S::Format: Filterable,
    {
        let source_descriptor = source.descriptor();

        if source_descriptor.context_id != self.data.context_id {
            panic!("The source image belongs to a different context than the framebuffer.");
        }

        BlitCommand {
            render_pass_id: self.data.render_pass_id,
            read_slot: Gl::COLOR_ATTACHMENT0,
            bitmask: Gl::COLOR_BUFFER_BIT,
            filter: Gl::LINEAR,
            target_region: region,
            target: self.color.descriptor(),
            source: source_descriptor,
        }
    }
}

impl<C, F> Framebuffer<C, DepthStencilBuffer<F>>
where
    F: DepthStencilRenderable,
{
    /// Transfers a rectangle of both depth and stencil values from the `source` depth-stencil image
    /// onto a `region` of the depth-stencil buffer in framebuffer, using "nearest" filtering if the
    /// `source` and the `region` have different sizes.
    ///
    /// The depth-stencil data stored in `source` must be stored in the same format as the storage
    /// format format used by the framebuffer's depth-stencil buffer. If the `source` image has a
    /// different size (width or height) than the `region`, then the `source` will be scaled to
    /// match the size of the `region`. If scaling is required, then "nearest" filtering is used to
    /// obtain pixel values for the resized image, where for each pixel value in the resized image,
    /// the value of the pixel that is at the nearest corresponding relative position is used.
    ///
    /// If a `region` bigger than depth-stencil buffer is specified with [Region::Area], then any
    /// pixels that would be copied outside the depth-stencil buffer will be discarded. However, the
    /// amount of scaling that is applied is based solely on the size of the `region`, it is not
    /// affected by the size of the depth-stencil buffer.
    ///
    /// See also [blit_depth_command] and [blit_stencil_command].
    ///
    /// # Example
    ///
    /// ```
    /// # use web_glitz::rendering::{RenderTarget, DepthStencilAttachment, FloatAttachment};
    /// # use web_glitz::image::format::{Depth24Stencil8, RGBA8};
    /// # use web_glitz::image::renderbuffer::Renderbuffer;
    /// # fn wrapper(
    /// # mut render_target: RenderTarget<(FloatAttachment<Renderbuffer<RGBA8>>,), DepthStencilAttachment<Renderbuffer<Depth24Stencil8>>>,
    /// # renderbuffer: Renderbuffer<Depth24Stencil8>
    /// # ) {
    /// use web_glitz::image::Region2D;
    ///
    /// let render_pass = render_target.create_render_pass(|framebuffer| {
    ///     framebuffer.blit_depth_stencil_command(Region2D::Fill, &renderbuffer)
    /// });
    /// # }
    /// ```
    ///
    /// Here `render_target` is a [RenderTarget] or [DefaultRenderTarget] and `renderbuffer` is a
    /// [Renderbuffer].
    ///
    /// # Panics
    ///
    /// Panics if `source` belongs to a different context than the framebuffer.
    pub fn blit_depth_stencil_command<S>(&self, region: Region2D, source: &S) -> BlitCommand
    where
        S: BlitSource<Format = F>,
    {
        let source_descriptor = source.descriptor();

        if source_descriptor.context_id != self.data.context_id {
            panic!("The source image belongs to a different context than the framebuffer.");
        }

        BlitCommand {
            render_pass_id: self.data.render_pass_id,
            read_slot: Gl::DEPTH_STENCIL_ATTACHMENT,
            bitmask: Gl::DEPTH_BUFFER_BIT & Gl::STENCIL_BUFFER_BIT,
            filter: Gl::NEAREST,
            target_region: region,
            target: BlitTargetDescriptor {
                internal: BlitTargetDescriptorInternal::FBO {
                    width: self.depth_stencil.width(),
                    height: self.depth_stencil.height(),
                },
            },
            source: source_descriptor,
        }
    }

    /// Transfers a rectangle of only depth values from the `source` depth-stencil image onto a
    /// `region` of the depth-stencil buffer in framebuffer, using "nearest" filtering if the
    /// `source` and the `region` have different sizes.
    ///
    /// The depth-stencil data stored in `source` must be stored in the same format as the storage
    /// format format used by the framebuffer's depth-stencil buffer. If the `source` image has a
    /// different size (width or height) than the `region`, then the `source` will be scaled to
    /// match the size of the `region`. If scaling is required, then "nearest" filtering is used to
    /// obtain pixel values for the resized image, where for each pixel value in the resized image,
    /// the value of the pixel that is at the nearest corresponding relative position is used.
    ///
    /// If a `region` bigger than depth-stencil buffer is specified with [Region::Area], then any
    /// pixels that would be copied outside the depth-stencil buffer will be discarded. However, the
    /// amount of scaling that is applied is based solely on the size of the `region`, it is not
    /// affected by the size of the depth-stencil buffer.
    ///
    /// See also [blit_depth_stencil_command] and [blit_stencil_command].
    ///
    /// # Example
    ///
    /// ```
    /// # use web_glitz::rendering::{RenderTarget, DepthStencilAttachment, FloatAttachment};
    /// # use web_glitz::image::format::{Depth24Stencil8, RGBA8};
    /// # use web_glitz::image::renderbuffer::Renderbuffer;
    /// # fn wrapper(
    /// # mut render_target: RenderTarget<(FloatAttachment<Renderbuffer<RGBA8>>,), DepthStencilAttachment<Renderbuffer<Depth24Stencil8>>>,
    /// # renderbuffer: Renderbuffer<Depth24Stencil8>
    /// # ) {
    /// use web_glitz::image::Region2D;
    ///
    /// let render_pass = render_target.create_render_pass(|framebuffer| {
    ///     framebuffer.blit_depth_command(Region2D::Fill, &renderbuffer)
    /// });
    /// # }
    /// ```
    ///
    /// Here `render_target` is a [RenderTarget] or [DefaultRenderTarget] and `renderbuffer` is a
    /// [Renderbuffer].
    ///
    /// # Panics
    ///
    /// Panics if `source` belongs to a different context than the framebuffer.
    pub fn blit_depth_command<S>(&self, region: Region2D, source: &S) -> BlitCommand
    where
        S: BlitSource<Format = F>,
    {
        let source_descriptor = source.descriptor();

        if source_descriptor.context_id != self.data.context_id {
            panic!("The source image belongs to a different context than the framebuffer.");
        }

        BlitCommand {
            render_pass_id: self.data.render_pass_id,
            read_slot: Gl::DEPTH_STENCIL_ATTACHMENT,
            bitmask: Gl::DEPTH_BUFFER_BIT,
            filter: Gl::NEAREST,
            target_region: region,
            target: BlitTargetDescriptor {
                internal: BlitTargetDescriptorInternal::FBO {
                    width: self.depth_stencil.width(),
                    height: self.depth_stencil.height(),
                },
            },
            source: source_descriptor,
        }
    }

    /// Transfers a rectangle of only stencil values from the `source` depth-stencil image onto a
    /// `region` of the depth-stencil buffer in framebuffer, using "nearest" filtering if the
    /// `source` and the `region` have different sizes.
    ///
    /// The depth-stencil data stored in `source` must be stored in the same format as the storage
    /// format format used by the framebuffer's depth-stencil buffer. If the `source` image has a
    /// different size (width or height) than the `region`, then the `source` will be scaled to
    /// match the size of the `region`. If scaling is required, then "nearest" filtering is used to
    /// obtain pixel values for the resized image, where for each pixel value in the resized image,
    /// the value of the pixel that is at the nearest corresponding relative position is used.
    ///
    /// If a `region` bigger than depth-stencil buffer is specified with [Region::Area], then any
    /// pixels that would be copied outside the depth-stencil buffer will be discarded. However, the
    /// amount of scaling that is applied is based solely on the size of the `region`, it is not
    /// affected by the size of the depth-stencil buffer.
    ///
    /// See also [blit_depth_stencil_command] and [blit_depth_command].
    ///
    /// # Example
    ///
    /// ```
    /// # use web_glitz::rendering::{RenderTarget, DepthStencilAttachment, FloatAttachment};
    /// # use web_glitz::image::format::{Depth24Stencil8, RGBA8};
    /// # use web_glitz::image::renderbuffer::Renderbuffer;
    /// # fn wrapper(
    /// # mut render_target: RenderTarget<(FloatAttachment<Renderbuffer<RGBA8>>,), DepthStencilAttachment<Renderbuffer<Depth24Stencil8>>>,
    /// # renderbuffer: Renderbuffer<Depth24Stencil8>
    /// # ) {
    /// use web_glitz::image::Region2D;
    ///
    /// let render_pass = render_target.create_render_pass(|framebuffer| {
    ///     framebuffer.blit_stencil_command(Region2D::Fill, &renderbuffer)
    /// });
    /// # }
    /// ```
    ///
    /// Here `rendering` is a [RenderTarget] or [DefaultRenderTarget] and `renderbuffer` is a
    /// [Renderbuffer].
    ///
    /// # Panics
    ///
    /// Panics if `source` belongs to a different context than the framebuffer.
    pub fn blit_stencil_command<S>(&self, region: Region2D, source: &S) -> BlitCommand
    where
        S: BlitSource<Format = F>,
    {
        let source_descriptor = source.descriptor();

        if source_descriptor.context_id != self.data.context_id {
            panic!("The source image belongs to a different context than the framebuffer.");
        }

        BlitCommand {
            render_pass_id: self.data.render_pass_id,
            read_slot: Gl::DEPTH_STENCIL_ATTACHMENT,
            bitmask: Gl::STENCIL_BUFFER_BIT,
            filter: Gl::NEAREST,
            target_region: region,
            target: BlitTargetDescriptor {
                internal: BlitTargetDescriptorInternal::FBO {
                    width: self.depth_stencil.width(),
                    height: self.depth_stencil.height(),
                },
            },
            source: source_descriptor,
        }
    }
}

impl<C, F> Framebuffer<C, DepthBuffer<F>>
where
    F: DepthRenderable,
{
    /// Transfers a rectangle of depth values from the `source` depth image onto a `region` of the
    /// depth buffer in framebuffer, using "nearest" filtering if the `source` and the `region` have
    /// different sizes.
    ///
    /// The depth data stored in `source` must be stored in the same format as the storage format
    /// format used by the framebuffer's depth buffer. If the `source` image has a different size
    /// (width or height) than the `region`, then the `source` will be scaled to match the size of
    /// the `region`. If scaling is required, then "nearest" filtering is used to obtain pixel
    /// values for the resized image, where for each pixel value in the resized image, the value of
    /// the pixel that is at the nearest corresponding relative position is used.
    ///
    /// If a `region` bigger than depth buffer is specified with [Region::Area], then any pixels
    /// that would be copied outside the depth buffer will be discarded. However, the amount of
    /// scaling that is applied is based solely on the size of the `region`, it is not affected by
    /// the size of the depth buffer.
    ///
    /// # Example
    ///
    /// ```
    /// # use web_glitz::rendering::{RenderTarget, DepthAttachment, FloatAttachment};
    /// # use web_glitz::image::format::{DepthComponent24, RGBA8};
    /// # use web_glitz::image::renderbuffer::Renderbuffer;
    /// # fn wrapper(
    /// # mut render_target: RenderTarget<(FloatAttachment<Renderbuffer<RGBA8>>,), DepthAttachment<Renderbuffer<DepthComponent24>>>,
    /// # renderbuffer: Renderbuffer<DepthComponent24>
    /// # ) {
    /// use web_glitz::image::Region2D;
    ///
    /// let render_pass = render_target.create_render_pass(|framebuffer| {
    ///     framebuffer.blit_depth_command(Region2D::Fill, &renderbuffer)
    /// });
    /// # }
    /// ```
    ///
    /// Here `rendering` is a [RenderTargetDescription] and `renderbuffer` is a [Renderbuffer].
    ///
    /// # Panics
    ///
    /// Panics if `source` belongs to a different context than the framebuffer.
    pub fn blit_depth_command<S>(&self, region: Region2D, source: &S) -> BlitCommand
    where
        S: BlitSource<Format = F>,
    {
        let source_descriptor = source.descriptor();

        if source_descriptor.context_id != self.data.context_id {
            panic!("The source image belongs to a different context than the framebuffer.");
        }

        BlitCommand {
            render_pass_id: self.data.render_pass_id,
            read_slot: Gl::DEPTH_ATTACHMENT,
            bitmask: Gl::DEPTH_BUFFER_BIT,
            filter: Gl::NEAREST,
            target_region: region,
            target: BlitTargetDescriptor {
                internal: BlitTargetDescriptorInternal::FBO {
                    width: self.depth_stencil.width(),
                    height: self.depth_stencil.height(),
                },
            },
            source: source_descriptor,
        }
    }
}

impl<C, F> Framebuffer<C, StencilBuffer<F>>
where
    F: StencilRenderable,
{
    /// Transfers a rectangle of stencil values from the `source` depth image onto a `region` of the
    /// stencil buffer in framebuffer, using "nearest" filtering if the `source` and the `region`
    /// have different sizes.
    ///
    /// The stencil data stored in `source` must be stored in the same format as the storage format
    /// format used by the framebuffer's stencil buffer. If the `source` image has a different size
    /// (width or height) than the `region`, then the `source` will be scaled to match the size of
    /// the `region`. If scaling is required, then "nearest" filtering is used to obtain pixel
    /// values for the resized image, where for each pixel value in the resized image, the value of
    /// the pixel that is at the nearest corresponding relative position is used.
    ///
    /// If a `region` bigger than stencil buffer is specified with [Region::Area], then any pixels
    /// that would be copied outside the stencil buffer will be discarded. However, the amount of
    /// scaling that is applied is based solely on the size of the `region`, it is not affected by
    /// the size of the stencil buffer.
    ///
    /// # Example
    ///
    /// ```
    /// # use web_glitz::rendering::{RenderTarget, StencilAttachment, FloatAttachment};
    /// # use web_glitz::image::format::{StencilIndex8, RGBA8};
    /// # use web_glitz::image::renderbuffer::Renderbuffer;
    /// # fn wrapper(
    /// # mut render_target: RenderTarget<(FloatAttachment<Renderbuffer<RGBA8>>,), StencilAttachment<Renderbuffer<StencilIndex8>>>,
    /// # renderbuffer: Renderbuffer<StencilIndex8>
    /// # ) {
    /// use web_glitz::image::Region2D;
    ///
    /// let render_pass = render_target.create_render_pass(|framebuffer| {
    ///     framebuffer.blit_stencil_command(Region2D::Fill, &renderbuffer)
    /// });
    /// # }
    /// ```
    ///
    /// Here `rendering` is a [RenderTargetDescription] and `renderbuffer` is a [Renderbuffer].
    ///
    /// # Panics
    ///
    /// Panics if `source` belongs to a different context than the framebuffer.
    pub fn blit_stencil_command<S>(&self, region: Region2D, source: &S) -> BlitCommand
    where
        S: BlitSource<Format = F>,
    {
        let source_descriptor = source.descriptor();

        if source_descriptor.context_id != self.data.context_id {
            panic!("The source image belongs to a different context than the framebuffer.");
        }

        BlitCommand {
            render_pass_id: self.data.render_pass_id,
            read_slot: Gl::STENCIL_ATTACHMENT,
            bitmask: Gl::STENCIL_BUFFER_BIT,
            filter: Gl::NEAREST,
            target_region: region,
            target: BlitTargetDescriptor {
                internal: BlitTargetDescriptorInternal::FBO {
                    width: self.depth_stencil.width(),
                    height: self.depth_stencil.height(),
                },
            },
            source: source_descriptor,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct IncompatibleSampleCount {
    framebuffer_samples: usize,
    blit_source_samples: usize,
}

/// Represents a set of image memory buffers that serve as the rendering destination for a
/// [RenderPass].
///
/// The image buffers allocated in the framebuffer correspond to to the images attached to the
/// [RenderTargetDescription] that was used to define the [RenderPass] (see also [RenderTarget]);
/// specifically, [color] provides handles to the color buffers (if any), and [depth_stencil]
/// provides a handle to the depth-stencil buffer (if any).
pub struct MultisampleFramebuffer<C, Ds> {
    pub color: C,
    pub depth_stencil: Ds,
    pub(crate) data: GraphicsPipelineTarget,
    pub(crate) samples: usize,
}

impl<C, Ds> MultisampleFramebuffer<C, Ds> {
    /// Creates a pipeline task using the given `graphics_pipeline`.
    ///
    /// The second parameter `f` must be a function that returns the task that is to be executed
    /// while the `graphics_pipeline` is bound as the active graphics pipeline. This function
    /// will receive a reference to this [ActiveGraphicsPipeline] which may be used to encode
    /// draw commands (see [ActiveGraphicsPipeline::draw_command]). The task returned by the
    /// function typically consists of 1 ore more draw commands that were created in this way. The
    /// current framebuffer serves as the output target for the `graphics_pipeline` (your draw
    /// commands may modify the current framebuffer).
    ///
    /// # Example
    ///
    /// ```
    /// # use web_glitz::rendering::{DefaultRenderTarget, DefaultRGBBuffer};
    /// # use web_glitz::runtime::RenderingContext;
    /// # use web_glitz::buffer::{BufferView, UsageHint};
    /// # use web_glitz::pipeline::graphics::{GraphicsPipeline, Vertex};
    /// # fn wrapper<V>(
    /// #     mut render_target: DefaultRenderTarget<DefaultRGBBuffer, ()>,
    /// #     vertex_buffers: BufferView<[V]>,
    /// #     graphics_pipeline: GraphicsPipeline<V, (), ()>
    /// # )
    /// # where
    /// #     V: Vertex,
    /// # {
    /// # let resources = ();
    /// let render_pass = render_target.create_render_pass(|framebuffer| {
    ///     framebuffer.pipeline_task(&graphics_pipeline, |active_pipeline| {
    ///         active_pipeline.task_builder()
    ///             .bind_vertex_buffers(vertex_buffers)
    ///             .bind_resources(resources)
    ///             .draw(16, 1)
    ///             .finish()
    ///     })
    /// });
    /// # }
    /// ```
    ///
    /// In this example, `context` is a [RenderingContext]; `rendering` is a
    /// [RenderTargetDescription], see also [DefaultRenderTarget] and [RenderTarget];
    /// `graphics_pipeline` is a [GraphicsPipeline], see [GraphicsPipeline] and
    /// [RenderingContext::create_graphics_pipeline] for details; `vertex_stream` is a
    /// [VertexStreamDescription], see [VertexStreamDescription], [VertexArray] and
    /// [RenderingContext::create_vertex_array] for details; `resources` is a user-defined type for
    /// which the [Resources] trait is implemented, see [Resources] for details.
    ///
    /// # Panics
    ///
    /// Panics if the `graphics_pipeline` belongs to a different context than the framebuffer for
    /// which this pipeline task is being created.
    ///
    /// Panics if the task returned by `f` contains commands that were constructed for a different
    /// pipeline task context.
    pub fn pipeline_task<P, V, R, Tf, F, T>(&self, pipeline: &P, f: F) -> PipelineTask<T>
    where
        P: GraphicsPipelineState<V, R, Tf>,
        F: Fn(ActiveGraphicsPipeline<V, R, Tf>) -> T,
        T: GpuTask<PipelineTaskContext>,
    {
        pipeline.pipeline_task(&self.data, f)
    }
}

impl<C, Ds> MultisampleFramebuffer<C, Ds>
where
    C: BlitColorTarget,
{
    /// Returns a command that will transfer a rectangle of pixels from the `source` onto a `region`
    /// of each of the color buffers in framebuffer, using "nearest" filtering if the `source` and
    /// the `region` have different sizes, or returns an error if the number of samples used for the
    /// `source` does not match the number of samples used for the framebuffer.
    ///
    /// Behaves identically to [Framebuffer::blit_color_nearest_command] except for a sample count
    /// compatibility check; see [Framebuffer::blit_color_nearest_command] for further details and
    /// examples.
    pub fn try_blit_color_nearest_command<S>(
        &self,
        region: Region2D,
        source: &S,
    ) -> Result<BlitCommand, IncompatibleSampleCount>
    where
        S: MultisampleBlitColorCompatible<C>,
    {
        let MultisampleBlitSourceDescriptor {
            blit_source_descriptor,
            samples,
        } = source.descriptor();

        if blit_source_descriptor.context_id != self.data.context_id {
            panic!("The source image belongs to a different context than the framebuffer.");
        }

        if samples == self.samples {
            Ok(BlitCommand {
                render_pass_id: self.data.render_pass_id,
                read_slot: Gl::COLOR_ATTACHMENT0,
                bitmask: Gl::COLOR_BUFFER_BIT,
                filter: Gl::NEAREST,
                target_region: region,
                target: self.color.descriptor(),
                source: blit_source_descriptor,
            })
        } else {
            Err(IncompatibleSampleCount {
                framebuffer_samples: self.samples,
                blit_source_samples: samples,
            })
        }
    }

    /// Returns a command that will transfer a rectangle of pixels from the `source` onto a `region`
    /// of each of the color buffers in framebuffer, using "linear" filtering if the `source` and
    /// the `region` have different sizes, or returns an error if the number of samples used for the
    /// `source` does not match the number of samples used for the framebuffer.
    ///
    /// Behaves identically to [Framebuffer::blit_color_linear_command] except for a sample count
    /// compatibility check; see [Framebuffer::blit_color_linear_command] for further details and
    /// examples.
    pub fn try_blit_color_linear_command<S>(
        &self,
        region: Region2D,
        source: &S,
    ) -> Result<BlitCommand, IncompatibleSampleCount>
    where
        S: MultisampleBlitColorCompatible<C>,
        S::Format: Filterable,
    {
        let MultisampleBlitSourceDescriptor {
            blit_source_descriptor,
            samples,
        } = source.descriptor();

        if blit_source_descriptor.context_id != self.data.context_id {
            panic!("The source image belongs to a different context than the framebuffer.");
        }

        if samples == self.samples {
            Ok(BlitCommand {
                render_pass_id: self.data.render_pass_id,
                read_slot: Gl::COLOR_ATTACHMENT0,
                bitmask: Gl::COLOR_BUFFER_BIT,
                filter: Gl::LINEAR,
                target_region: region,
                target: self.color.descriptor(),
                source: blit_source_descriptor,
            })
        } else {
            Err(IncompatibleSampleCount {
                framebuffer_samples: self.samples,
                blit_source_samples: samples,
            })
        }
    }
}

impl<C, F> MultisampleFramebuffer<C, DepthStencilBuffer<F>>
where
    F: DepthStencilRenderable,
{
    /// Returns a command that will transfer a rectangle of both depth and stencil values from the
    /// `source` depth-stencil image onto a `region` of the depth-stencil buffer in framebuffer,
    /// using "nearest" filtering if the `source` and the `region` have different sizes, or returns
    /// an error if the number of samples used for the `source` does not match the number of samples
    /// used for the framebuffer.
    ///
    /// Behaves identically to [Framebuffer::blit_depth_stencil_command] except for a sample count
    /// compatibility check; see [Framebuffer::blit_depth_stencil_command] for further details and
    /// examples.
    pub fn try_blit_depth_stencil_command<S>(
        &self,
        region: Region2D,
        source: &S,
    ) -> Result<BlitCommand, IncompatibleSampleCount>
    where
        S: MultisampleBlitSource<Format = F>,
    {
        let MultisampleBlitSourceDescriptor {
            blit_source_descriptor,
            samples,
        } = source.descriptor();

        if blit_source_descriptor.context_id != self.data.context_id {
            panic!("The source image belongs to a different context than the framebuffer.");
        }

        if samples == self.samples {
            Ok(BlitCommand {
                render_pass_id: self.data.render_pass_id,
                read_slot: Gl::DEPTH_STENCIL_ATTACHMENT,
                bitmask: Gl::DEPTH_BUFFER_BIT & Gl::STENCIL_BUFFER_BIT,
                filter: Gl::NEAREST,
                target_region: region,
                target: BlitTargetDescriptor {
                    internal: BlitTargetDescriptorInternal::FBO {
                        width: self.depth_stencil.width(),
                        height: self.depth_stencil.height(),
                    },
                },
                source: blit_source_descriptor,
            })
        } else {
            Err(IncompatibleSampleCount {
                framebuffer_samples: self.samples,
                blit_source_samples: samples,
            })
        }
    }

    /// Returns a command that will transfer a rectangle of only depth values from the `source`
    /// depth-stencil image onto a `region` of the depth-stencil buffer in framebuffer, using
    /// "nearest" filtering if the `source` and the `region` have different sizes, or returns an
    /// error if the number of samples used for the `source` does not match the number of samples
    /// used for the framebuffer.
    ///
    /// Behaves identically to [Framebuffer::blit_depth_command] except for a sample count
    /// compatibility check; see [Framebuffer::blit_depth_command] for further details and examples.
    pub fn try_blit_depth_command<S>(
        &self,
        region: Region2D,
        source: &S,
    ) -> Result<BlitCommand, IncompatibleSampleCount>
    where
        S: MultisampleBlitSource<Format = F>,
    {
        let MultisampleBlitSourceDescriptor {
            blit_source_descriptor,
            samples,
        } = source.descriptor();

        if blit_source_descriptor.context_id != self.data.context_id {
            panic!("The source image belongs to a different context than the framebuffer.");
        }

        if samples == self.samples {
            Ok(BlitCommand {
                render_pass_id: self.data.render_pass_id,
                read_slot: Gl::DEPTH_STENCIL_ATTACHMENT,
                bitmask: Gl::DEPTH_BUFFER_BIT,
                filter: Gl::NEAREST,
                target_region: region,
                target: BlitTargetDescriptor {
                    internal: BlitTargetDescriptorInternal::FBO {
                        width: self.depth_stencil.width(),
                        height: self.depth_stencil.height(),
                    },
                },
                source: blit_source_descriptor,
            })
        } else {
            Err(IncompatibleSampleCount {
                framebuffer_samples: self.samples,
                blit_source_samples: samples,
            })
        }
    }

    /// Returns a command that will transfer a rectangle of only stencil values from the `source`
    /// depth-stencil image onto a `region` of the depth-stencil buffer in framebuffer, using
    /// "nearest" filtering if the `source` and the `region` have different sizes, or returns an
    /// error if the number of samples used for the `source` does not match the number of samples
    /// used for the framebuffer.
    ///
    /// Behaves identically to [Framebuffer::blit_stencil_command] except for a sample count
    /// compatibility check; see [Framebuffer::blit_stencil_command] for further details and
    /// examples.
    pub fn try_blit_stencil_command<S>(
        &self,
        region: Region2D,
        source: &S,
    ) -> Result<BlitCommand, IncompatibleSampleCount>
    where
        S: MultisampleBlitSource<Format = F>,
    {
        let MultisampleBlitSourceDescriptor {
            blit_source_descriptor,
            samples,
        } = source.descriptor();

        if blit_source_descriptor.context_id != self.data.context_id {
            panic!("The source image belongs to a different context than the framebuffer.");
        }

        if samples == self.samples {
            Ok(BlitCommand {
                render_pass_id: self.data.render_pass_id,
                read_slot: Gl::DEPTH_STENCIL_ATTACHMENT,
                bitmask: Gl::STENCIL_BUFFER_BIT,
                filter: Gl::NEAREST,
                target_region: region,
                target: BlitTargetDescriptor {
                    internal: BlitTargetDescriptorInternal::FBO {
                        width: self.depth_stencil.width(),
                        height: self.depth_stencil.height(),
                    },
                },
                source: blit_source_descriptor,
            })
        } else {
            Err(IncompatibleSampleCount {
                framebuffer_samples: self.samples,
                blit_source_samples: samples,
            })
        }
    }
}

impl<C, F> MultisampleFramebuffer<C, DepthBuffer<F>>
where
    F: DepthRenderable,
{
    /// Returns a command that will transfer a rectangle of depth values from the `source` depth
    /// image onto a `region` of the depth buffer in framebuffer, using "nearest" filtering if the
    /// `source` and the `region` have different sizes, or returns an error if the number of samples
    /// used for the `source` does not match the number of samples used for the framebuffer.
    ///
    /// Behaves identically to [Framebuffer::blit_depth_command] except for a sample count
    /// compatibility check; see [Framebuffer::blit_depth_command] for further details and examples.
    pub fn try_blit_depth_command<S>(
        &self,
        region: Region2D,
        source: &S,
    ) -> Result<BlitCommand, IncompatibleSampleCount>
    where
        S: MultisampleBlitSource<Format = F>,
    {
        let MultisampleBlitSourceDescriptor {
            blit_source_descriptor,
            samples,
        } = source.descriptor();

        if blit_source_descriptor.context_id != self.data.context_id {
            panic!("The source image belongs to a different context than the framebuffer.");
        }

        if samples == self.samples {
            Ok(BlitCommand {
                render_pass_id: self.data.render_pass_id,
                read_slot: Gl::DEPTH_ATTACHMENT,
                bitmask: Gl::DEPTH_BUFFER_BIT,
                filter: Gl::NEAREST,
                target_region: region,
                target: BlitTargetDescriptor {
                    internal: BlitTargetDescriptorInternal::FBO {
                        width: self.depth_stencil.width(),
                        height: self.depth_stencil.height(),
                    },
                },
                source: blit_source_descriptor,
            })
        } else {
            Err(IncompatibleSampleCount {
                framebuffer_samples: self.samples,
                blit_source_samples: samples,
            })
        }
    }
}

impl<C, F> MultisampleFramebuffer<C, StencilBuffer<F>>
where
    F: StencilRenderable,
{
    /// Returns a command that will transfer a rectangle of stencil values from the `source` depth
    /// image onto a `region` of the stencil buffer in framebuffer, using "nearest" filtering if the
    /// `source` and the `region` have different sizes, or returns an error if the number of samples
    /// used for the `source` does not match the number of samples used for the framebuffer.
    ///
    /// Behaves identically to [Framebuffer::blit_stencil_command] except for a sample count
    /// compatibility check; see [Framebuffer::blit_stencil_command] for further details and
    /// examples.
    pub fn try_blit_stencil_command<S>(
        &self,
        region: Region2D,
        source: &S,
    ) -> Result<BlitCommand, IncompatibleSampleCount>
    where
        S: MultisampleBlitSource<Format = F>,
    {
        let MultisampleBlitSourceDescriptor {
            blit_source_descriptor,
            samples,
        } = source.descriptor();

        if blit_source_descriptor.context_id != self.data.context_id {
            panic!("The source image belongs to a different context than the framebuffer.");
        }

        if samples == self.samples {
            Ok(BlitCommand {
                render_pass_id: self.data.render_pass_id,
                read_slot: Gl::STENCIL_ATTACHMENT,
                bitmask: Gl::STENCIL_BUFFER_BIT,
                filter: Gl::NEAREST,
                target_region: region,
                target: BlitTargetDescriptor {
                    internal: BlitTargetDescriptorInternal::FBO {
                        width: self.depth_stencil.width(),
                        height: self.depth_stencil.height(),
                    },
                },
                source: blit_source_descriptor,
            })
        } else {
            Err(IncompatibleSampleCount {
                framebuffer_samples: self.samples,
                blit_source_samples: samples,
            })
        }
    }
}

/// Provides the context necessary for making progress on a [PipelineTask].
pub struct PipelineTaskContext {
    pipeline_task_id: usize,
    connection: *mut Connection,
    attribute_layout: *const VertexInputLayoutDescriptor,
    vertex_buffers: BufferDescriptors,
    index_buffer: Option<IndexDataDescriptor>,
}

impl PipelineTaskContext {
    /// The ID of the [PipelineTask] this [PipelineTaskContext] is associated with.
    pub fn pipeline_task_id(&self) -> usize {
        self.pipeline_task_id
    }

    pub(crate) fn connection_mut(&mut self) -> &mut Connection {
        unsafe { &mut *self.connection }
    }

    /// Unpacks this context into a reference to the raw [web_sys::WebGl2RenderingContext] and a
    /// reference to the WebGlitz state cache for this context.
    ///
    /// # Unsafe
    ///
    /// If state is changed on the [web_sys::WebGl2RenderingContext], than the cache must be updated
    /// accordingly.
    pub unsafe fn unpack(&self) -> (&Gl, &DynamicState) {
        (*self.connection).unpack()
    }

    /// Unpacks this context into a mutable reference to the raw [web_sys::WebGl2RenderingContext]
    /// and a mutable reference to the WebGlitz state cache for this context.
    ///
    /// # Unsafe
    ///
    /// If state is changed on the [web_sys::WebGl2RenderingContext], than the cache must be updated
    /// accordingly.
    pub unsafe fn unpack_mut(&mut self) -> (&mut Gl, &mut DynamicState) {
        (*self.connection).unpack_mut()
    }
}

/// Returned from [Framebuffer::pipeline_task], a series of commands that is executed while a
/// specific [GraphicsPipeline] is bound as the [ActiveGraphicsPipeline].
///
/// See [Framebuffer::pipeline_task].
#[derive(Clone)]
pub struct PipelineTask<T> {
    id: usize,
    render_pass_id: usize,
    task: T,
    program_id: JsId,
    #[allow(dead_code)] // Just holding on to this so it won't get dropped prematurely
    vertex_shader_data: Arc<VertexShaderData>,
    #[allow(dead_code)] // Just holding on to this so it won't get dropped prematurely
    fragment_shader_data: Arc<FragmentShaderData>,
    transform_feedback_data: Arc<UnsafeCell<Option<TransformFeedbackData>>>,
    transform_feedback_buffers: Option<BufferDescriptors>,
    attribute_layout: VertexInputLayoutDescriptor,
    primitive_assembly: PrimitiveAssembly,
    depth_test: Option<DepthTest>,
    stencil_test: Option<StencilTest>,
    scissor_region: Region2D,
    blending: Option<Blending>,
    viewport: Viewport,
    framebuffer_dimensions: Option<(u32, u32)>,
}

impl<T> PipelineTask<T>
where
    T: GpuTask<PipelineTaskContext>,
{
    pub(crate) fn new<V, R, Tf, F>(
        framebuffer_data: &GraphicsPipelineTarget,
        pipeline: &GraphicsPipeline<V, R, Tf>,
        transform_feedback_buffers: Option<BufferDescriptors>,
        f: F,
    ) -> Self
    where
        F: Fn(ActiveGraphicsPipeline<V, R, Tf>) -> T,
    {
        if framebuffer_data.context_id != pipeline.context_id() {
            panic!("The pipeline does not belong to the same context as the framebuffer.");
        }

        let id = framebuffer_data.last_pipeline_task_id.get();

        framebuffer_data.last_pipeline_task_id.set(id + 1);

        let mut hasher = FnvHasher::default();

        (framebuffer_data.render_pass_id, id).hash(&mut hasher);

        let pipeline_task_id = hasher.finish() as usize;

        let task = f(ActiveGraphicsPipeline {
            pipeline_task_id,
            pipeline,
        });

        if task.context_id() != ContextId::Any
            && task.context_id() != ContextId::Id(pipeline_task_id)
        {
            panic!("Task does not belong to the pipeline task context.")
        }

        PipelineTask {
            id: pipeline_task_id,
            render_pass_id: framebuffer_data.render_pass_id,
            task,
            transform_feedback_data: pipeline.transform_feedback_data.clone(),
            transform_feedback_buffers,
            program_id: pipeline.program_id(),
            vertex_shader_data: pipeline.vertex_shader_data.clone(),
            fragment_shader_data: pipeline.fragment_shader_data.clone(),
            attribute_layout: pipeline.vertex_attribute_layout().clone(),
            primitive_assembly: pipeline.primitive_assembly().clone(),
            depth_test: pipeline.depth_test().cloned(),
            stencil_test: pipeline.stencil_test().cloned(),
            scissor_region: pipeline.scissor_region().clone(),
            blending: pipeline.blending().cloned(),
            viewport: pipeline.viewport().clone(),
            framebuffer_dimensions: framebuffer_data.dimensions,
        }
    }
}

unsafe impl<T, O> GpuTask<RenderPassContext> for PipelineTask<T>
where
    T: GpuTask<PipelineTaskContext, Output = O>,
{
    type Output = O;

    fn context_id(&self) -> ContextId {
        ContextId::Id(self.render_pass_id)
    }

    fn progress(&mut self, context: &mut RenderPassContext) -> Progress<Self::Output> {
        let (gl, state) = unsafe { context.unpack_mut() };

        unsafe {
            self.program_id.with_value_unchecked(|program_object| {
                state.use_program(Some(program_object)).apply(gl).unwrap();
            });
        };

        let framebuffer_dimensions = self.framebuffer_dimensions.unwrap_or_else(|| {
            (
                gl.drawing_buffer_width() as u32,
                gl.drawing_buffer_height() as u32,
            )
        });

        let transform_feedback_data = unsafe { &mut *self.transform_feedback_data.get() };

        if let Some(transform_feedback_buffers) = &self.transform_feedback_buffers {
            if let Some(transform_feedback_data) = transform_feedback_data.as_mut() {
                unsafe {
                    transform_feedback_data
                        .id
                        .with_value_unchecked(|transform_feedback| {
                            state
                                .bind_transform_feedback(Some(transform_feedback))
                                .apply(gl)
                                .unwrap();
                        });
                }

                if &transform_feedback_data.buffers != transform_feedback_buffers {
                    for (i, buffer) in transform_feedback_buffers.iter().enumerate() {
                        let offset = buffer.offset_in_bytes;
                        let size = buffer.size_in_bytes;

                        unsafe {
                            buffer
                                .buffer_data
                                .id()
                                .unwrap()
                                .with_value_unchecked(|buffer| {
                                    state
                                        .bind_transform_feedback_buffer_range(
                                            i as u32,
                                            BufferRange::OffsetSize(buffer, offset, size),
                                        )
                                        .apply(gl)
                                        .unwrap();
                                })
                        }
                    }

                    for i in transform_feedback_buffers.len()..transform_feedback_data.buffers.len()
                    {
                        state
                            .bind_transform_feedback_buffer_range(i as u32, BufferRange::None)
                            .apply(gl)
                            .unwrap();
                    }

                    transform_feedback_data.buffers = transform_feedback_buffers.clone();
                }

                match transform_feedback_data.state {
                    TransformFeedbackState::Inactive => {
                        gl.begin_transform_feedback(
                            self.primitive_assembly.transform_feedback_mode(),
                        );
                    }
                    TransformFeedbackState::Paused => {
                        gl.resume_transform_feedback();
                    }
                    TransformFeedbackState::Recording => (),
                }

                transform_feedback_data.state = TransformFeedbackState::Recording;
            } else {
                let transform_feedback = gl.create_transform_feedback().unwrap();

                state
                    .bind_transform_feedback(Some(&transform_feedback))
                    .apply(gl)
                    .unwrap();

                for (i, buffer) in transform_feedback_buffers.iter().enumerate() {
                    let offset = buffer.offset_in_bytes;
                    let size = buffer.size_in_bytes;

                    unsafe {
                        buffer
                            .buffer_data
                            .id()
                            .unwrap()
                            .with_value_unchecked(|buffer| {
                                state
                                    .bind_transform_feedback_buffer_range(
                                        i as u32,
                                        BufferRange::OffsetSize(buffer, offset, size),
                                    )
                                    .apply(gl)
                                    .unwrap();
                            })
                    }
                }

                gl.begin_transform_feedback(self.primitive_assembly.transform_feedback_mode());

                *transform_feedback_data = Some(TransformFeedbackData {
                    id: JsId::from_value(transform_feedback.into()),
                    buffers: transform_feedback_buffers.clone(),
                    state: TransformFeedbackState::Recording,
                });
            }

            // Make sure none of the transform feedback buffers are bound to any other bind slots as
            // this will cause the browser to error in the next draw command.
            state.bind_array_buffer(None).apply(gl).unwrap();
            state.bind_copy_read_buffer(None).apply(gl).unwrap();
            state.bind_copy_write_buffer(None).apply(gl).unwrap();
            state.bind_element_array_buffer(None).apply(gl).unwrap();
            state.bind_pixel_pack_buffer(None).apply(gl).unwrap();
            state.bind_pixel_unpack_buffer(None).apply(gl).unwrap();
        } else {
            if let Some(transform_feedback_data) = transform_feedback_data.as_mut() {
                if transform_feedback_data.state != TransformFeedbackState::Inactive {
                    unsafe {
                        transform_feedback_data
                            .id
                            .with_value_unchecked(|transform_feedback| {
                                state
                                    .bind_transform_feedback(Some(transform_feedback))
                                    .apply(gl)
                                    .unwrap();
                            });

                        gl.end_transform_feedback();

                        // Unbind all transform feedback buffers, otherwise the browser will error
                        // the next time they are used in a draw command.
                        for i in 0..transform_feedback_data.buffers.len() {
                            state
                                .bind_transform_feedback_buffer_range(i as u32, BufferRange::None)
                                .apply(gl)
                                .unwrap();
                        }

                        transform_feedback_data.state = TransformFeedbackState::Inactive;
                    }
                }
            }
        }

        match self.scissor_region {
            Region2D::Area((x, y), width, height) => {
                let (gl, state) = unsafe { context.unpack_mut() };

                state.set_scissor_test_enabled(true).apply(gl).unwrap();
                state
                    .set_scissor_rect((x as i32, y as i32, width, height))
                    .apply(gl)
                    .unwrap();
            }
            Region2D::Fill => {
                state.set_scissor_test_enabled(false).apply(gl).unwrap();
            }
        }

        let connection = context.connection_mut();

        if let Some(face_culling) = self.primitive_assembly.face_culling() {
            face_culling.apply(connection);
        }

        if let Some(winding_order) = self.primitive_assembly.winding_order() {
            winding_order.apply(connection);
        }

        if let Some(line_width) = self.primitive_assembly.line_width() {
            line_width.apply(connection);
        }

        self.viewport.apply(connection, framebuffer_dimensions);

        DepthTest::apply(&self.depth_test, connection);
        StencilTest::apply(&self.stencil_test, connection);
        Blending::apply(&self.blending, connection);

        let res = self.task.progress(&mut PipelineTaskContext {
            pipeline_task_id: self.id,
            connection: context.connection_mut() as *mut Connection,
            attribute_layout: &self.attribute_layout,
            vertex_buffers: BufferDescriptors::new(),
            index_buffer: None,
        });

        if let Some(transform_feedback_data) = transform_feedback_data.as_mut() {
            if transform_feedback_data.state == TransformFeedbackState::Recording {
                let (gl, _) = unsafe { context.unpack_mut() };

                gl.pause_transform_feedback();

                transform_feedback_data.state = TransformFeedbackState::Paused;
            }
        }

        res
    }
}

/// An activated [GraphicsPipeline] which may be used to draw to a [Framebuffer].
///
/// A handle to an [ActiveGraphicsPipeline] is obtained by using a [GraphicsPipeline] to create
/// a [PipelineTask] for a [Framebuffer], see [Framebuffer::pipeline_task].
pub struct ActiveGraphicsPipeline<'a, V, R, Tf> {
    pipeline_task_id: usize,
    pipeline: &'a GraphicsPipeline<V, R, Tf>,
}

impl<'a, V, R, Tf> ActiveGraphicsPipeline<'a, V, R, Tf> {
    /// A builder interface that enforces valid sequencing of pipeline commands.
    ///
    /// Notably, this builder will not allow the sequencing of a [DrawCommand], before vertex
    /// buffers and/or resources have been bound (if the pipeline requires vertex buffers, resp.
    /// resources), see [GraphicsPipelineTaskBuilder::draw]; it will not allow the sequencing of a
    /// [DrawIndexedCommand] before an index buffer has been bound and before vertex buffers and/or
    /// resources have been bound (if the pipeline requires vertex buffers, resp. resources), see
    /// [GraphicsPipelineTaskBuilder::draw_indexed].
    ///
    /// # Example
    ///
    /// ```
    /// # use web_glitz::rendering::DefaultRGBBuffer;
    /// # use web_glitz::rendering::DefaultRenderTarget;
    /// # use web_glitz::runtime::RenderingContext;
    /// # use web_glitz::buffer::BufferView;
    /// # use web_glitz::pipeline::graphics::{GraphicsPipeline, Vertex};
    /// # fn wrapper<Rc, V>(
    /// #     context: &Rc,
    /// #     mut render_target: DefaultRenderTarget<DefaultRGBBuffer, ()>,
    /// #     vertex_buffer: BufferView<[V]>,
    /// #     graphics_pipeline: GraphicsPipeline<V, (), ()>
    /// # )
    /// # where
    /// #     Rc: RenderingContext,
    /// #     V: Vertex,
    /// # {
    /// # let resources = ();
    /// let render_pass = render_target.create_render_pass(|framebuffer| {
    ///     framebuffer.pipeline_task(&graphics_pipeline, |active_pipeline| {
    ///         active_pipeline.task_builder()
    ///             .bind_vertex_buffers(vertex_buffer)
    ///             .bind_resources(resources)
    ///             .draw(16, 1)
    ///             .finish()
    ///     })
    /// });
    /// # }
    /// ```
    pub fn task_builder(
        &self,
    ) -> GraphicsPipelineTaskBuilder<'a, V, R, Unspecified, Unspecified, Unspecified, Empty> {
        GraphicsPipelineTaskBuilder {
            context_id: self.pipeline.context_id(),
            topology: self.pipeline.primitive_assembly().topology(),
            pipeline_task_id: self.pipeline_task_id,
            task: Empty,
            _pipeline: marker::PhantomData,
            _vertex_buffers: marker::PhantomData,
            _index_buffer: marker::PhantomData,
            _resource_bindings: marker::PhantomData,
        }
    }
}

/// A builder interface that enforces valid sequencing of pipeline commands.
///
/// See [ActiveGraphicsPipeline::task_builder].
pub struct GraphicsPipelineTaskBuilder<'a, V, R, Vb, Ib, Rb, T> {
    context_id: usize,
    pipeline_task_id: usize,
    topology: Topology,
    task: T,
    _pipeline: marker::PhantomData<ActiveGraphicsPipeline<'a, V, R, ()>>,
    _vertex_buffers: marker::PhantomData<Vb>,
    _index_buffer: marker::PhantomData<Ib>,
    _resource_bindings: marker::PhantomData<Rb>,
}

impl<'a, V, R, Vb, Ib, Rb, T> GraphicsPipelineTaskBuilder<'a, V, R, Vb, Ib, Rb, T> {
    /// Binds typed a (set of) vertex buffer(s) to the active graphics pipeline.
    ///
    /// When the active graphics pipeline is invoked (see [draw] and [draw_indexed]), then the
    /// `vertex_buffers` define a vertex input array for the pipeline.
    ///
    /// The `vertex_buffers` must be a [TypedVertexBuffers] type with a vertex attribute layout (see
    /// [TypedVertexBuffers::VertexAttributeLayout]) that matches the vertex attribute layout
    /// specified for the pipeline. This is statically verified by the type system; if this
    /// compiles, then this performs no further runtime checks on the compatibility of the vertex
    /// buffers with the active graphics pipeline, it only checks that every buffer that is
    /// bound belongs to the same context as the pipeline. This will not result in invalid behaviour
    /// as long as `vertex_buffers` meets the safety contract on the [TypedVertexBuffers] trait
    /// (implementing [TypedVertexBuffers] is unsafe, but several safe implementations are
    /// provided by this library).
    ///
    /// See also [bind_vertex_buffers_untyped] for an unsafe alternative with relaxed type
    /// constraints.
    ///
    /// # Panics
    ///
    /// Panics if a vertex buffers belongs to different rendering context.
    pub fn bind_vertex_buffers<VbNew>(
        self,
        vertex_buffers: VbNew,
    ) -> GraphicsPipelineTaskBuilder<
        'a,
        V,
        R,
        VbNew,
        Ib,
        Rb,
        Sequence<T, BindVertexBuffersCommand, PipelineTaskContext>,
    >
    where
        V: TypedVertexInputLayout,
        VbNew: TypedVertexBuffers<Layout = V>,
        T: GpuTask<PipelineTaskContext>,
    {
        let vertex_buffers = vertex_buffers
            .encode(&mut VertexBuffersEncodingContext::new())
            .into_descriptors();

        for (i, buffer) in vertex_buffers.iter().enumerate() {
            if buffer.buffer_data.context_id() != self.context_id {
                panic!("Buffer {} belongs to a different context.", i);
            }
        }

        GraphicsPipelineTaskBuilder {
            context_id: self.context_id,
            topology: self.topology,
            pipeline_task_id: self.pipeline_task_id,
            task: sequence(
                self.task,
                BindVertexBuffersCommand {
                    pipeline_task_id: self.pipeline_task_id,
                    vertex_buffers: Some(vertex_buffers),
                },
            ),
            _pipeline: marker::PhantomData,
            _vertex_buffers: marker::PhantomData,
            _index_buffer: marker::PhantomData,
            _resource_bindings: marker::PhantomData,
        }
    }

    /// Binds a (set of) vertex buffer(s) to the active graphics pipeline.
    ///
    /// When the active graphics pipeline is invoked (see [draw] and [draw_indexed]), then the
    /// `vertex_buffers` define a vertex input array for the pipeline.
    ///
    /// # Unsafe
    ///
    /// The vertex buffers must contain data compatible with the vertex input layout specified for
    /// the pipeline.
    ///
    /// This function is an unsafe alternative to [bind_vertex_buffers] with relaxed type
    /// constraints. If your vertex input data has a statically known layout, consider implementing
    /// [TypedVertexInputBuffers] for your data and specifying a [TypedVertexInputLayout] for your
    /// pipeline (see [GraphicsPipelineDescriptorBuilder::typed_vertex_input_layout]; this will
    /// allow you to use [bind_vertex_buffers] instead. Note that [TypedVertexInputBuffers] is
    /// already implemented for any tuple of buffers (up to 16 buffers) where each buffer contains a
    /// slice of [Vertex] types.
    ///
    /// # Panics
    ///
    /// Panics of any of the vertex buffers belong to a different context than the pipeline.
    pub unsafe fn bind_vertex_buffers_untyped<VbNew>(
        self,
        vertex_buffers: VbNew,
    ) -> GraphicsPipelineTaskBuilder<
        'a,
        V,
        R,
        VbNew,
        Ib,
        Rb,
        Sequence<T, BindVertexBuffersCommand, PipelineTaskContext>,
    >
    where
        VbNew: VertexBuffers,
        T: GpuTask<PipelineTaskContext>,
    {
        let vertex_buffers = vertex_buffers
            .encode(&mut VertexBuffersEncodingContext::new())
            .into_descriptors();

        for (i, buffer) in vertex_buffers.iter().enumerate() {
            if buffer.buffer_data.context_id() != self.context_id {
                panic!("Buffer {} belongs to a different context.", i);
            }
        }

        GraphicsPipelineTaskBuilder {
            context_id: self.context_id,
            topology: self.topology,
            pipeline_task_id: self.pipeline_task_id,
            task: sequence(
                self.task,
                BindVertexBuffersCommand {
                    pipeline_task_id: self.pipeline_task_id,
                    vertex_buffers: Some(vertex_buffers),
                },
            ),
            _pipeline: marker::PhantomData,
            _vertex_buffers: marker::PhantomData,
            _index_buffer: marker::PhantomData,
            _resource_bindings: marker::PhantomData,
        }
    }

    /// Binds an index buffer to the graphics pipeline.
    ///
    /// A graphics pipeline typically requires a source of vertex data (see [bind_vertex_buffers]).
    /// This vertex data defines an array of vertices which by itself can serve as the vertex input
    /// stream for the pipeline, where the vertices are simply streamed once in the canonical array
    /// order, see [draw_command]. When an index buffer is specified, then the pipeline may also be
    /// executed in "indexed" mode, see [draw_indexed]. In indexed mode the indices in the index
    /// buffer determine the vertex sequence of the vertex stream. For example, if the first index
    /// is `8`, then the first vertex in the vertex stream is the 9th vertex in the vertex array.
    /// The same index may also occur more than once in the index buffer, in which case the same
    /// vertex will appear more than once in the vertex stream.
    pub fn bind_index_buffer<IbNew>(
        self,
        index_buffer: IbNew,
    ) -> GraphicsPipelineTaskBuilder<
        'a,
        V,
        R,
        Vb,
        IbNew,
        Rb,
        Sequence<T, BindIndexBufferCommand, PipelineTaskContext>,
    >
    where
        IbNew: IndexData,
        T: GpuTask<PipelineTaskContext>,
    {
        let index_data_descriptor = index_buffer.descriptor();

        if index_data_descriptor.buffer_data.context_id() != self.context_id {
            panic!("Index buffer belongs to a different context.");
        }

        GraphicsPipelineTaskBuilder {
            context_id: self.context_id,
            topology: self.topology,
            pipeline_task_id: self.pipeline_task_id,
            task: sequence(
                self.task,
                BindIndexBufferCommand {
                    pipeline_task_id: self.pipeline_task_id,
                    index_buffer: index_data_descriptor,
                },
            ),
            _pipeline: marker::PhantomData,
            _vertex_buffers: marker::PhantomData,
            _index_buffer: marker::PhantomData,
            _resource_bindings: marker::PhantomData,
        }
    }

    /// Binds one or more bind groups containing typed resource groups to the active graphics
    /// pipeline.
    ///
    /// When the pipeline is invoked, each invocation may access the bound resources.
    ///
    /// The `resource_bindings` must implement [TypedResourceBindings] and the
    /// [TypedResourceBindings::Layout] must match the [TypedResourceBindingsLayout] specified for
    /// the pipeline (see [GraphicsPipelineDescriptorBuilder::typed_resource_bindings_layout]); this
    /// is statically verified by the type-checker. No further runtime checks are performed to
    /// ensure compatibility of the resource bindings with the pipeline.
    ///
    /// See also [bind_resources_untyped] for an unsafe alternative with relaxed type constraints.
    ///
    /// # Panics
    ///
    /// Panics if any of the bind groups belong to a different context than the pipeline.
    pub fn bind_resources<RbNew>(
        self,
        resource_bindings: RbNew,
    ) -> GraphicsPipelineTaskBuilder<
        'a,
        V,
        R,
        Vb,
        Ib,
        RbNew,
        Sequence<T, BindResourcesCommand<RbNew::BindGroups>, PipelineTaskContext>,
    >
    where
        R: TypedResourceBindingsLayout,
        RbNew: TypedResourceBindings<Layout = R>,
        T: GpuTask<PipelineTaskContext>,
    {
        GraphicsPipelineTaskBuilder {
            context_id: self.context_id,
            topology: self.topology,
            pipeline_task_id: self.pipeline_task_id,
            task: sequence(
                self.task,
                BindResourcesCommand {
                    pipeline_task_id: self.pipeline_task_id,
                    resource_bindings: resource_bindings
                        .encode(&mut ResourceBindingsEncodingContext::new(self.context_id))
                        .bind_groups,
                },
            ),
            _pipeline: marker::PhantomData,
            _vertex_buffers: marker::PhantomData,
            _index_buffer: marker::PhantomData,
            _resource_bindings: marker::PhantomData,
        }
    }

    /// Binds one or more bind groups to the active graphics pipeline.
    ///
    /// When the pipeline is invoked, each invocation may access the bound resources.
    ///
    /// # Unsafe
    ///
    /// The resource bindings must be compatible with the resource bindings layout specified for the
    /// pipeline.
    ///
    /// This function is an unsafe alternative to `bind_resources` with relaxed type constraints. If
    /// your resource bindings layout is statically know, consider implementing
    /// [TypedResourceBindings] for your bind groups and specifying a [TypedResourceBindingsLayout]
    /// for your pipeline (see [GraphicsPipelineDescriptorBuilder::typed_resource_bindings_layout]);
    /// this will allow you to use [bind_resources] instead. Note that [TypedResourceBindings] is
    /// implemented for any tuple of bind groups where the resources in each bind group implement
    /// [Resources].
    ///
    /// # Panics
    ///
    /// Panics if any of the bind groups belong to a different context than the pipeline.
    pub unsafe fn bind_resources_untyped<RbNew>(
        self,
        resource_bindings: RbNew,
    ) -> GraphicsPipelineTaskBuilder<
        'a,
        V,
        R,
        Vb,
        Ib,
        RbNew,
        Sequence<T, BindResourcesCommand<RbNew::BindGroups>, PipelineTaskContext>,
    >
    where
        RbNew: ResourceBindings,
        T: GpuTask<PipelineTaskContext>,
    {
        GraphicsPipelineTaskBuilder {
            context_id: self.context_id,
            topology: self.topology,
            pipeline_task_id: self.pipeline_task_id,
            task: sequence(
                self.task,
                BindResourcesCommand {
                    pipeline_task_id: self.pipeline_task_id,
                    resource_bindings: resource_bindings
                        .encode(&mut ResourceBindingsEncodingContext::new(self.context_id))
                        .bind_groups,
                },
            ),
            _pipeline: marker::PhantomData,
            _vertex_buffers: marker::PhantomData,
            _index_buffer: marker::PhantomData,
            _resource_bindings: marker::PhantomData,
        }
    }

    /// Creates a [DrawCommand] that will execute the active graphics pipeline, streaming
    /// `vertex_count` vertices for `instance_count` instances from the currently bound vertex
    /// buffers.
    ///
    /// If the pipeline requires vertex buffers, then this command may only be added to the builder
    /// after appropriate vertex buffers have been bound (see [bind_vertex_buffers]). If the
    /// pipeline requires resources, then this command may only be added to the builder after
    /// appropriate resources have been bound (see [bind_resources]).
    ///
    /// This will draw to the framebuffer that created the encapsulating [PipelineTask] (see
    /// [Framebuffer::pipeline_task]). The pipeline's first color output will be stored to the
    /// framebuffer's first color buffer (if present), the second color output will be stored to the
    /// framebuffer's second color buffer (if present), etc. The pipeline's depth test (if enabled)
    /// may update the framebuffer's depth-stencil buffer (if it present and is a [DepthRenderable]
    /// or [DepthStencilRenderable] format, otherwise the depth test will act as if it was
    /// disabled). The pipeline's stencil test (if enabled) may update the framebuffer's
    /// depth-stencil buffer (if it is present and is [StencilRenderable] or
    /// [DepthStencilRenderable] format, otherwise the stencil test will act as if was disabled).
    ///
    /// See also [draw_indexed] for indexed mode drawing with an index buffer.
    ///
    /// # Example
    ///
    /// ```
    /// # use web_glitz::rendering::DefaultRGBBuffer;
    /// # use web_glitz::rendering::DefaultRenderTarget;
    /// # use web_glitz::buffer::{UsageHint, BufferView};
    /// # use web_glitz::pipeline::graphics::{GraphicsPipeline, Vertex};
    /// # fn wrapper<V>(
    /// #     mut render_target: DefaultRenderTarget<DefaultRGBBuffer, ()>,
    /// #     vertex_buffers: BufferView<[V]>,
    /// #     graphics_pipeline: GraphicsPipeline<V, (), ()>
    /// # )
    /// # where
    /// #     V: Vertex,
    /// # {
    /// # let resources = ();
    /// let render_pass = render_target.create_render_pass(|framebuffer| {
    ///     framebuffer.pipeline_task(&graphics_pipeline, |active_pipeline| {
    ///         active_pipeline.task_builder()
    ///             .bind_vertex_buffers(vertex_buffers)
    ///             .bind_resources(resources)
    ///             .draw(16, 1)
    ///             .finish()
    ///     })
    /// });
    /// # }
    /// ```
    ///
    /// In this example `graphics_pipeline` is a [GraphicsPipeline], see [GraphicsPipeline] and
    /// [RenderingContext::create_graphics_pipeline] for details; `vertex_buffers` is a set of
    /// [VertexBuffers]; `resources` is a user-defined type for which the [Resources] trait is
    /// implemented, see [Resources] for details.
    pub fn draw(
        self,
        vertex_count: usize,
        instance_count: usize,
    ) -> GraphicsPipelineTaskBuilder<
        'a,
        V,
        R,
        Vb,
        Ib,
        R,
        Sequence<T, DrawCommand, PipelineTaskContext>,
    >
    where
        Vb: VertexBuffers,
        Rb: ResourceBindings,
        T: GpuTask<PipelineTaskContext>,
    {
        GraphicsPipelineTaskBuilder {
            context_id: self.context_id,
            topology: self.topology,
            pipeline_task_id: self.pipeline_task_id,
            task: sequence(
                self.task,
                DrawCommand {
                    pipeline_task_id: self.pipeline_task_id,
                    topology: self.topology,
                    vertex_count,
                    instance_count,
                },
            ),
            _pipeline: marker::PhantomData,
            _vertex_buffers: marker::PhantomData,
            _index_buffer: marker::PhantomData,
            _resource_bindings: marker::PhantomData,
        }
    }

    /// Creates a [DrawIndexedCommand] that will execute the active graphics pipeline, streaming
    /// `index_count` vertex indices for `instance_count` instances from the currently bound index
    /// buffer, which produces a vertex stream by indexing into the vertex array defined by the
    /// currently bound per-vertex vertex buffers.
    ///
    /// This command may only be added to the builder after an index buffer has been bound (see
    /// [bind_index_buffer]. If the pipeline requires vertex buffers, then this command may only be
    /// added to the builder after appropriate vertex buffers have been bound (see
    /// [bind_vertex_buffers]). If the pipeline requires resources, then this command may only be
    /// added to the builder after appropriate resources have been bound (see [bind_resources]).
    ///
    /// This will draw to the framebuffer that created the encapsulating [PipelineTask] (see
    /// [Framebuffer::pipeline_task]). The pipeline's first color output will be stored to the
    /// framebuffer's first color buffer (if present), the second color output will be stored to the
    /// framebuffer's second color buffer (if present), etc. The pipeline's depth test (if enabled)
    /// may update the framebuffer's depth-stencil buffer (if it present and is a [DepthRenderable]
    /// or [DepthStencilRenderable] format, otherwise the depth test will act as if it was
    /// disabled). The pipeline's stencil test (if enabled) may update the framebuffer's
    /// depth-stencil buffer (if it is present and is [StencilRenderable] or
    /// [DepthStencilRenderable] format, otherwise the stencil test will act as if was disabled).
    ///
    /// See also [draw] for drawing without an index buffer.
    ///
    /// # Example
    ///
    /// ```
    /// # use web_glitz::rendering::DefaultRGBBuffer;
    /// # use web_glitz::rendering::DefaultRenderTarget;
    /// # use web_glitz::buffer::{UsageHint, BufferView};
    /// # use web_glitz::pipeline::graphics::{GraphicsPipeline, Vertex, IndexBufferView};
    /// # fn wrapper<V>(
    /// #     mut render_target: DefaultRenderTarget<DefaultRGBBuffer, ()>,
    /// #     vertex_buffers: BufferView<[V]>,
    /// #     index_buffer: IndexBufferView<u16>,
    /// #     graphics_pipeline: GraphicsPipeline<V, (), ()>
    /// # )
    /// # where
    /// #     V: Vertex,
    /// # {
    /// # let resources = ();
    /// let render_pass = render_target.create_render_pass(|framebuffer| {
    ///     framebuffer.pipeline_task(&graphics_pipeline, |active_pipeline| {
    ///         active_pipeline.task_builder()
    ///             .bind_vertex_buffers(vertex_buffers)
    ///             .bind_index_buffer(index_buffer)
    ///             .bind_resources(resources)
    ///             .draw_indexed(16, 1)
    ///             .finish()
    ///     })
    /// });
    /// # }
    /// ```
    ///
    /// In this example `graphics_pipeline` is a [GraphicsPipeline], see [GraphicsPipeline] and
    /// [RenderingContext::create_graphics_pipeline] for details; `vertex_buffers` is a set of
    /// [VertexBuffers]; `index_buffer` is an [IndexBuffer]; `resources` is a user-defined type for
    /// which the [Resources] trait is implemented, see [Resources] for details.
    pub fn draw_indexed(
        self,
        index_count: usize,
        instance_count: usize,
    ) -> GraphicsPipelineTaskBuilder<
        'a,
        V,
        R,
        Vb,
        Ib,
        R,
        Sequence<T, DrawIndexedCommand, PipelineTaskContext>,
    >
    where
        Vb: VertexBuffers,
        Ib: IndexData,
        Rb: ResourceBindings,
        T: GpuTask<PipelineTaskContext>,
    {
        GraphicsPipelineTaskBuilder {
            context_id: self.context_id,
            topology: self.topology,
            pipeline_task_id: self.pipeline_task_id,
            task: sequence(
                self.task,
                DrawIndexedCommand {
                    pipeline_task_id: self.pipeline_task_id,
                    topology: self.topology,
                    index_count,
                    instance_count,
                },
            ),
            _pipeline: marker::PhantomData,
            _vertex_buffers: marker::PhantomData,
            _index_buffer: marker::PhantomData,
            _resource_bindings: marker::PhantomData,
        }
    }

    /// Finishes the builder and returns the resulting pipeline task.
    pub fn finish(self) -> T {
        self.task
    }
}

/// Command that binds a (set of) vertex buffer(s) to the currently bound graphics pipeline.
///
/// See [GraphicsPipelineTaskBuilder::bind_vertex_buffers].
#[derive(Clone)]
pub struct BindVertexBuffersCommand {
    pipeline_task_id: usize,
    vertex_buffers: Option<BufferDescriptors>,
}

unsafe impl GpuTask<PipelineTaskContext> for BindVertexBuffersCommand {
    type Output = ();

    fn context_id(&self) -> ContextId {
        ContextId::Id(self.pipeline_task_id)
    }

    fn progress(&mut self, execution_context: &mut PipelineTaskContext) -> Progress<Self::Output> {
        execution_context.vertex_buffers =
            self.vertex_buffers.take().expect("Cannot progress twice");

        Progress::Finished(())
    }
}

/// Command that binds an index buffer to the currently bound graphics pipeline.
///
/// See [GraphicsPipelineTaskBuilder::bind_index_buffer].
#[derive(Clone)]
pub struct BindIndexBufferCommand {
    pipeline_task_id: usize,
    index_buffer: IndexDataDescriptor,
}

unsafe impl GpuTask<PipelineTaskContext> for BindIndexBufferCommand {
    type Output = ();

    fn context_id(&self) -> ContextId {
        ContextId::Id(self.pipeline_task_id)
    }

    fn progress(&mut self, execution_context: &mut PipelineTaskContext) -> Progress<Self::Output> {
        execution_context.index_buffer = Some(self.index_buffer.clone());

        Progress::Finished(())
    }
}

/// Command that binds a set of resources to the resource slots of the currently bound pipeline.
///
/// See [GraphicsPipelineTaskBuilder::bind_resources].
#[derive(Clone)]
pub struct BindResourcesCommand<Rb> {
    pipeline_task_id: usize,
    resource_bindings: Rb,
}

unsafe impl<Rb> GpuTask<PipelineTaskContext> for BindResourcesCommand<Rb>
where
    Rb: Borrow<[BindGroupDescriptor]>,
    Rb: Borrow<[BindGroupDescriptor]>,
{
    type Output = ();

    fn context_id(&self) -> ContextId {
        ContextId::Id(self.pipeline_task_id)
    }

    fn progress(&mut self, execution_context: &mut PipelineTaskContext) -> Progress<Self::Output> {
        for descriptor in self.resource_bindings.borrow().iter() {
            descriptor.bind(execution_context.connection_mut());
        }

        Progress::Finished(())
    }
}

/// Command that runs the currently bound graphics pipeline.
///
/// See [GraphicsPipelineTaskBuilder::draw].
#[derive(Clone)]
pub struct DrawCommand {
    pipeline_task_id: usize,
    topology: Topology,
    vertex_count: usize,
    instance_count: usize,
}

unsafe impl GpuTask<PipelineTaskContext> for DrawCommand {
    type Output = ();

    fn context_id(&self) -> ContextId {
        ContextId::Id(self.pipeline_task_id)
    }

    fn progress(&mut self, context: &mut PipelineTaskContext) -> Progress<Self::Output> {
        let (gl, state) = unsafe { (*context.connection).unpack_mut() };

        unsafe {
            state.vertex_array_cache_mut().bind_or_create(
                &*context.attribute_layout,
                &context.vertex_buffers,
                gl,
            );
        }

        if self.instance_count == 1 {
            gl.draw_arrays(self.topology.id(), 0, self.vertex_count as i32);
        } else {
            gl.draw_arrays_instanced(
                self.topology.id(),
                0,
                self.vertex_count as i32,
                self.instance_count as i32,
            );
        }

        Progress::Finished(())
    }
}

/// Command that runs the currently bound graphics pipeline in indexed mode.
///
/// See [GraphicsPipelineTaskBuilder::draw_indexed].
#[derive(Clone)]
pub struct DrawIndexedCommand {
    pipeline_task_id: usize,
    topology: Topology,
    index_count: usize,
    instance_count: usize,
}

unsafe impl GpuTask<PipelineTaskContext> for DrawIndexedCommand {
    type Output = ();

    fn context_id(&self) -> ContextId {
        ContextId::Id(self.pipeline_task_id)
    }

    fn progress(&mut self, context: &mut PipelineTaskContext) -> Progress<Self::Output> {
        let (gl, state) = unsafe { (*context.connection).unpack_mut() };

        if let Some(index_buffer) = &context.index_buffer {
            unsafe {
                state.vertex_array_cache_mut().bind_or_create_indexed(
                    &*context.attribute_layout,
                    &context.vertex_buffers,
                    index_buffer,
                    gl,
                );
            }

            if self.instance_count == 1 {
                gl.draw_elements_with_i32(
                    self.topology.id(),
                    self.index_count as i32,
                    index_buffer.index_type.id(),
                    index_buffer.offset as i32,
                );
            } else {
                gl.draw_elements_instanced_with_i32(
                    self.topology.id(),
                    self.index_count as i32,
                    index_buffer.index_type.id(),
                    index_buffer.offset as i32,
                    self.instance_count as i32,
                );
            }
        } else {
            panic!("No index buffer.");
        }

        Progress::Finished(())
    }
}

/// Helper trait implemented by color buffers that can serve as a target for a [BlitCommand],
/// see [Framebuffer::blit_color_nearest_command] and [Framebuffer::blit_color_linear_command].
pub trait BlitColorTarget {
    /// Encapsulates the information about the color target needed by the [BlitCommand].
    fn descriptor(&self) -> BlitTargetDescriptor;
}

impl BlitColorTarget for DefaultRGBBuffer {
    fn descriptor(&self) -> BlitTargetDescriptor {
        BlitTargetDescriptor {
            internal: BlitTargetDescriptorInternal::Default,
        }
    }
}

impl BlitColorTarget for DefaultRGBABuffer {
    fn descriptor(&self) -> BlitTargetDescriptor {
        BlitTargetDescriptor {
            internal: BlitTargetDescriptorInternal::Default,
        }
    }
}

impl<C> BlitColorTarget for (C,)
where
    C: RenderingOutputBuffer,
{
    fn descriptor(&self) -> BlitTargetDescriptor {
        BlitTargetDescriptor {
            internal: BlitTargetDescriptorInternal::FBO {
                width: self.0.width(),
                height: self.0.height(),
            },
        }
    }
}

macro_rules! impl_blit_color_target {
    ($C0:ident, $($C:ident),*) => {
        impl<$C0, $($C),*> BlitColorTarget for ($C0, $($C),*)
        where
            $C0: RenderingOutputBuffer,
            $($C: RenderingOutputBuffer),*
        {
            fn descriptor(&self) -> BlitTargetDescriptor {
                #[allow(non_snake_case)]
                let ($C0, $($C),*) = self;

                let mut width = $C0.width();

                $(
                    if $C.width() < width {
                        width = $C.width();
                    }
                )*


                let mut height = $C0.height();

                $(
                    if $C.height() < height {
                        height = $C.height();
                    }
                )*

                BlitTargetDescriptor {
                    internal: BlitTargetDescriptorInternal::FBO {
                        width,
                        height,
                    }
                }
            }
        }
    }
}

impl_blit_color_target!(C0, C1);
impl_blit_color_target!(C0, C1, C2);
impl_blit_color_target!(C0, C1, C2, C3);
impl_blit_color_target!(C0, C1, C2, C3, C4);
impl_blit_color_target!(C0, C1, C2, C3, C4, C5);
impl_blit_color_target!(C0, C1, C2, C3, C4, C5, C6);
impl_blit_color_target!(C0, C1, C2, C3, C4, C5, C6, C7);
impl_blit_color_target!(C0, C1, C2, C3, C4, C5, C6, C7, C8);
impl_blit_color_target!(C0, C1, C2, C3, C4, C5, C6, C7, C8, C9);
impl_blit_color_target!(C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10);
impl_blit_color_target!(C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11);
impl_blit_color_target!(C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12);
impl_blit_color_target!(C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12, C13);
impl_blit_color_target!(C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12, C13, C14);
impl_blit_color_target!(C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12, C13, C14, C15);

/// Returned from [BlitColorTarget::descriptor], encapsulates the information about the target
/// buffer needed by the [BlitCommand].
#[derive(Clone)]
pub struct BlitTargetDescriptor {
    internal: BlitTargetDescriptorInternal,
}

#[derive(Clone)]
enum BlitTargetDescriptorInternal {
    Default,
    FBO { width: u32, height: u32 },
}

/// Trait implemented by image reference types that can serve as the image data source for a color
/// [BlitCommand].
///
/// See [Framebuffer::blit_color_nearest_command] and [Framebuffer::blit_color_linear_command].
///
/// # Unsafe
///
/// The [Format] type must match the pixel format the blit source described by the [descriptor].
pub unsafe trait BlitSource {
    /// The image storage format used by the source image.
    type Format: InternalFormat;

    /// Encapsulates the information about the blit source required by the [BlitCommand].
    fn descriptor(&self) -> BlitSourceDescriptor;
}

/// Returned from [BlitSource::descriptor], encapsulates the information about the blit source
/// required by the [BlitCommand].
#[derive(Clone)]
pub struct BlitSourceDescriptor {
    attachment: AttachmentData,
    region: ((u32, u32), u32, u32),
    context_id: usize,
}

unsafe impl<'a, F> BlitSource for Texture2DLevel<'a, F>
where
    F: TextureFormat + 'static,
{
    type Format = F;

    fn descriptor(&self) -> BlitSourceDescriptor {
        BlitSourceDescriptor {
            attachment: Attachment::from_texture_2d_level(self).into_data(),
            region: ((0, 0), self.width(), self.height()),
            context_id: self.texture_data().context_id(),
        }
    }
}

unsafe impl<'a, F> BlitSource for Texture2DLevelSubImage<'a, F>
where
    F: TextureFormat + 'static,
{
    type Format = F;

    fn descriptor(&self) -> BlitSourceDescriptor {
        let origin = match self.region() {
            Region2D::Fill => (0, 0),
            Region2D::Area(origin, ..) => origin,
        };

        BlitSourceDescriptor {
            attachment: Attachment::from_texture_2d_level(&self.level_ref()).into_data(),
            region: (origin, self.width(), self.height()),
            context_id: self.texture_data().context_id(),
        }
    }
}

unsafe impl<'a, F> BlitSource for Texture2DArrayLevelLayer<'a, F>
where
    F: TextureFormat + 'static,
{
    type Format = F;

    fn descriptor(&self) -> BlitSourceDescriptor {
        BlitSourceDescriptor {
            attachment: Attachment::from_texture_2d_array_level_layer(self).into_data(),
            region: ((0, 0), self.width(), self.height()),
            context_id: self.texture_data().context_id(),
        }
    }
}

unsafe impl<'a, F> BlitSource for Texture2DArrayLevelLayerSubImage<'a, F>
where
    F: TextureFormat + 'static,
{
    type Format = F;

    fn descriptor(&self) -> BlitSourceDescriptor {
        let origin = match self.region() {
            Region2D::Fill => (0, 0),
            Region2D::Area(origin, ..) => origin,
        };

        BlitSourceDescriptor {
            attachment: Attachment::from_texture_2d_array_level_layer(&self.level_layer_ref())
                .into_data(),
            region: (origin, self.width(), self.height()),
            context_id: self.texture_data().context_id(),
        }
    }
}

unsafe impl<'a, F> BlitSource for Texture3DLevelLayer<'a, F>
where
    F: TextureFormat + 'static,
{
    type Format = F;

    fn descriptor(&self) -> BlitSourceDescriptor {
        BlitSourceDescriptor {
            attachment: Attachment::from_texture_3d_level_layer(self).into_data(),
            region: ((0, 0), self.width(), self.height()),
            context_id: self.texture_data().context_id(),
        }
    }
}

unsafe impl<'a, F> BlitSource for Texture3DLevelLayerSubImage<'a, F>
where
    F: TextureFormat + 'static,
{
    type Format = F;

    fn descriptor(&self) -> BlitSourceDescriptor {
        let origin = match self.region() {
            Region2D::Fill => (0, 0),
            Region2D::Area(origin, ..) => origin,
        };

        BlitSourceDescriptor {
            attachment: Attachment::from_texture_3d_level_layer(&self.level_layer_ref())
                .into_data(),
            region: (origin, self.width(), self.height()),
            context_id: self.texture_data().context_id(),
        }
    }
}

unsafe impl<'a, F> BlitSource for TextureCubeLevelFace<'a, F>
where
    F: TextureFormat + 'static,
{
    type Format = F;

    fn descriptor(&self) -> BlitSourceDescriptor {
        BlitSourceDescriptor {
            attachment: Attachment::from_texture_cube_level_face(self).into_data(),
            region: ((0, 0), self.width(), self.height()),
            context_id: self.texture_data().context_id(),
        }
    }
}

unsafe impl<'a, F> BlitSource for TextureCubeLevelFaceSubImage<'a, F>
where
    F: TextureFormat + 'static,
{
    type Format = F;

    fn descriptor(&self) -> BlitSourceDescriptor {
        let origin = match self.region() {
            Region2D::Fill => (0, 0),
            Region2D::Area(origin, ..) => origin,
        };

        BlitSourceDescriptor {
            attachment: Attachment::from_texture_cube_level_face(&self.level_face_ref())
                .into_data(),
            region: (origin, self.width(), self.height()),
            context_id: self.texture_data().context_id(),
        }
    }
}

unsafe impl<F> BlitSource for Renderbuffer<F>
where
    F: RenderbufferFormat + 'static,
{
    type Format = F;

    fn descriptor(&self) -> BlitSourceDescriptor {
        BlitSourceDescriptor {
            attachment: Attachment::from_renderbuffer(self).into_data(),
            region: ((0, 0), self.width(), self.height()),
            context_id: self.data().context_id(),
        }
    }
}

unsafe impl<F> BlitSource for Renderbuffer<Multisample<F>>
where
    F: RenderbufferFormat + Multisamplable + 'static,
{
    type Format = F;

    fn descriptor(&self) -> BlitSourceDescriptor {
        BlitSourceDescriptor {
            attachment: Attachment::from_renderbuffer(self).into_data(),
            region: ((0, 0), self.width(), self.height()),
            context_id: self.data().context_id(),
        }
    }
}

/// Trait implemented by multisample image reference types that can serve as the image data source
/// for a color [BlitCommand] on a multisample framebuffer.
///
/// See [MultisampleFramebuffer::try_blit_color_nearest_command] and
/// [MultisampleFramebuffer::try_blit_color_linear_command].
///
/// # Unsafe
///
/// The [Format] type must match the pixel format the blit source described by the [descriptor].
pub unsafe trait MultisampleBlitSource {
    /// The image storage format used by the source image.
    type Format: InternalFormat;

    /// Encapsulates the information about the multisample blit source required by the
    /// [BlitCommand].
    fn descriptor(&self) -> MultisampleBlitSourceDescriptor;
}

unsafe impl<F> MultisampleBlitSource for Renderbuffer<Multisample<F>>
where
    F: RenderbufferFormat + Multisamplable + 'static,
{
    type Format = F;

    fn descriptor(&self) -> MultisampleBlitSourceDescriptor {
        MultisampleBlitSourceDescriptor {
            blit_source_descriptor: BlitSourceDescriptor {
                attachment: Attachment::from_renderbuffer(self).into_data(),
                region: ((0, 0), self.width(), self.height()),
                context_id: self.data().context_id(),
            },
            samples: self.samples(),
        }
    }
}

/// Returned from [BlitSource::descriptor], encapsulates the information about the blit source
/// required by the [BlitCommand].
#[derive(Clone)]
pub struct MultisampleBlitSourceDescriptor {
    blit_source_descriptor: BlitSourceDescriptor,
    samples: usize,
}

/// Marker trait that identifies [BlitSource] types that can be safely blitted to a typed color
/// buffer or set of color buffers.
pub unsafe trait BlitColorCompatible<C>: BlitSource {}

unsafe impl<T> BlitColorCompatible<FloatBuffer<T::Format>> for T
where
    T: BlitSource,
    T::Format: FloatRenderable,
{
}

unsafe impl<T> BlitColorCompatible<DefaultRGBABuffer> for T where T: BlitSource<Format = RGBA8> {}

unsafe impl<T> BlitColorCompatible<DefaultRGBBuffer> for T where T: BlitSource<Format = RGB8> {}

unsafe impl<T> BlitColorCompatible<IntegerBuffer<T::Format>> for T
where
    T: BlitSource,
    T::Format: IntegerRenderable,
{
}

unsafe impl<T> BlitColorCompatible<UnsignedIntegerBuffer<T::Format>> for T
where
    T: BlitSource,
    T::Format: UnsignedIntegerRenderable,
{
}

macro_rules! impl_blit_color_compatible {
    ($C0:ident $(,$C:ident)*) => {
        unsafe impl<T, $C0 $(,$C)*> BlitColorCompatible<($C0 $(,$C)*,)> for T
        where T: BlitColorCompatible<$C0> $(+ BlitColorCompatible<$C>)* {}
    }
}

impl_blit_color_compatible!(C0);
impl_blit_color_compatible!(C0, C1);
impl_blit_color_compatible!(C0, C1, C2);
impl_blit_color_compatible!(C0, C1, C2, C3);
impl_blit_color_compatible!(C0, C1, C2, C3, C4);
impl_blit_color_compatible!(C0, C1, C2, C3, C4, C5);
impl_blit_color_compatible!(C0, C1, C2, C3, C4, C5, C6);
impl_blit_color_compatible!(C0, C1, C2, C3, C4, C5, C6, C7);
impl_blit_color_compatible!(C0, C1, C2, C3, C4, C5, C6, C7, C8);
impl_blit_color_compatible!(C0, C1, C2, C3, C4, C5, C6, C7, C8, C9);
impl_blit_color_compatible!(C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10);
impl_blit_color_compatible!(C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11);
impl_blit_color_compatible!(C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12);
impl_blit_color_compatible!(C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12, C13);
impl_blit_color_compatible!(C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12, C13, C14);
impl_blit_color_compatible!(C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12, C13, C14, C15);

/// Marker trait that identifies [MultisampleBlitSource] types that can be safely blitted to a typed
/// color buffer or set of color buffers.
pub unsafe trait MultisampleBlitColorCompatible<C>: MultisampleBlitSource {}

unsafe impl<T> MultisampleBlitColorCompatible<FloatBuffer<T::Format>> for T
where
    T: MultisampleBlitSource,
    T::Format: FloatRenderable,
{
}

unsafe impl<T> MultisampleBlitColorCompatible<DefaultRGBABuffer> for T where
    T: MultisampleBlitSource<Format = RGBA8>
{
}

unsafe impl<T> MultisampleBlitColorCompatible<DefaultRGBBuffer> for T where
    T: MultisampleBlitSource<Format = RGB8>
{
}

macro_rules! impl_multisample_blit_color_compatible {
    ($C0:ident, $($C:ident),*) => {
        unsafe impl<T, $C0, $($C),*> MultisampleBlitColorCompatible<($C0, $($C),*)> for T
        where T: MultisampleBlitColorCompatible<$C0> $(+ MultisampleBlitColorCompatible<$C>)* {}
    }
}

impl_multisample_blit_color_compatible!(C0, C1);
impl_multisample_blit_color_compatible!(C0, C1, C2);
impl_multisample_blit_color_compatible!(C0, C1, C2, C3);
impl_multisample_blit_color_compatible!(C0, C1, C2, C3, C4);
impl_multisample_blit_color_compatible!(C0, C1, C2, C3, C4, C5);
impl_multisample_blit_color_compatible!(C0, C1, C2, C3, C4, C5, C6);
impl_multisample_blit_color_compatible!(C0, C1, C2, C3, C4, C5, C6, C7);
impl_multisample_blit_color_compatible!(C0, C1, C2, C3, C4, C5, C6, C7, C8);
impl_multisample_blit_color_compatible!(C0, C1, C2, C3, C4, C5, C6, C7, C8, C9);
impl_multisample_blit_color_compatible!(C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10);
impl_multisample_blit_color_compatible!(C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11);
impl_multisample_blit_color_compatible!(C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12);
impl_multisample_blit_color_compatible!(C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12, C13);
impl_multisample_blit_color_compatible!(
    C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12, C13, C14
);
impl_multisample_blit_color_compatible!(
    C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12, C13, C14, C15
);

/// Encapsulates a command that transfers a rectangle of pixels from a source image into the
/// framebuffer.
///
/// See [Framebuffer::blit_color_nearest_command], [Framebuffer::blit_color_linear_command],
/// [Framebuffer::blit_depth_stencil_command], [Framebuffer::blit_depth_command] and
/// [Framebuffer::blit_stencil_command]
#[derive(Clone)]
pub struct BlitCommand {
    render_pass_id: usize,
    read_slot: u32,
    bitmask: u32,
    filter: u32,
    target: BlitTargetDescriptor,
    target_region: Region2D,
    source: BlitSourceDescriptor,
}

unsafe impl GpuTask<RenderPassContext> for BlitCommand {
    type Output = ();

    fn context_id(&self) -> ContextId {
        ContextId::Id(self.render_pass_id)
    }

    fn progress(&mut self, context: &mut RenderPassContext) -> Progress<Self::Output> {
        let (gl, state) = unsafe { context.unpack_mut() };

        state.bind_default_read_framebuffer(gl);

        self.source
            .attachment
            .attach(gl, Gl::READ_FRAMEBUFFER, self.read_slot);

        let ((src_x0, src_y0), src_width, src_height) = self.source.region;
        let src_x1 = src_x0 + src_width;
        let src_y1 = src_y0 + src_height;

        let (dst_x0, dst_y0, dst_x1, dst_y1) = match self.target_region {
            Region2D::Fill => match self.target.internal {
                BlitTargetDescriptorInternal::Default => {
                    (0, 0, gl.drawing_buffer_width(), gl.drawing_buffer_height())
                }
                BlitTargetDescriptorInternal::FBO { width, height } => {
                    (0, 0, width as i32, height as i32)
                }
            },
            Region2D::Area((dst_x0, dst_y0), dst_width, dst_height) => {
                let dst_x1 = dst_x0 + dst_width;
                let dst_y1 = dst_y0 + dst_height;

                (dst_x0 as i32, dst_y0 as i32, dst_x1 as i32, dst_y1 as i32)
            }
        };

        gl.blit_framebuffer(
            src_x0 as i32,
            src_y0 as i32,
            src_x1 as i32,
            src_y1 as i32,
            dst_x0,
            dst_y0,
            dst_x1,
            dst_y1,
            self.bitmask,
            self.filter,
        );

        Progress::Finished(())
    }
}

/// Represents the color buffer for a [DefaultRenderTarget] without an alpha channel.
pub struct DefaultRGBBuffer {
    render_pass_id: usize,
}

impl DefaultRGBBuffer {
    pub(crate) fn new(render_pass_id: usize) -> Self {
        DefaultRGBBuffer { render_pass_id }
    }

    /// Returns a command that clears a `region` of the buffer to `clear_value`.
    ///
    /// All pixels in the region are set to the `clear_value`; values outside the region are
    /// unaffected.
    ///
    /// # Examples
    ///
    /// The following command will clear the entire buffer to "transparent black":
    ///
    /// ```
    /// # use web_glitz::rendering::DefaultRGBBuffer;
    /// # use web_glitz::image::Region2D;
    /// # fn wrapper(buffer: &DefaultRGBBuffer) {
    /// let command = buffer.clear_command([0.0, 0.0, 0.0, 0.0], Region2D::Fill);
    /// # }
    /// ```
    ///
    /// It's also possible to only clear a specific rectangular area of the buffer:
    ///
    /// ```
    /// # use web_glitz::rendering::DefaultRGBBuffer;
    /// # use web_glitz::image::Region2D;
    /// # fn wrapper(buffer: &DefaultRGBBuffer) {
    /// let command = buffer.clear_command([0.0, 0.0, 0.0, 0.0], Region2D::Area((0, 0), 100, 100));
    /// # }
    /// ```
    ///
    /// See also [Region2D].
    pub fn clear_command(&self, clear_value: [f32; 4], region: Region2D) -> ClearFloatCommand {
        ClearFloatCommand {
            render_pass_id: self.render_pass_id,
            buffer_index: 0,
            clear_value,
            region,
        }
    }
}

/// Represents the color buffer for a [DefaultRenderTarget] with an alpha channel.
pub struct DefaultRGBABuffer {
    render_pass_id: usize,
}

impl DefaultRGBABuffer {
    pub(crate) fn new(render_pass_id: usize) -> Self {
        DefaultRGBABuffer { render_pass_id }
    }

    /// Returns a command that clears a `region` of the buffer to `clear_value`.
    ///
    /// All pixels in the region are set to the `clear_value`; values outside the region are
    /// unaffected.
    ///
    /// # Examples
    ///
    /// The following command will clear the entire buffer to "transparent black":
    ///
    /// ```
    /// # use web_glitz::rendering::DefaultRGBABuffer;
    /// # use web_glitz::image::Region2D;
    /// # fn wrapper(buffer: &DefaultRGBABuffer) {
    /// let command = buffer.clear_command([0.0, 0.0, 0.0, 0.0], Region2D::Fill);
    /// # }
    /// ```
    ///
    /// It's also possible to only clear a specific rectangular area of the buffer:
    ///
    /// ```
    /// # use web_glitz::rendering::DefaultRGBABuffer;
    /// # use web_glitz::image::Region2D;
    /// # fn wrapper(buffer: &DefaultRGBABuffer) {
    /// let command = buffer.clear_command([0.0, 0.0, 0.0, 0.0], Region2D::Area((0, 0), 100, 100));
    /// # }
    /// ```
    ///
    /// See also [Region2D].
    pub fn clear_command(&self, clear_value: [f32; 4], region: Region2D) -> ClearFloatCommand {
        ClearFloatCommand {
            render_pass_id: self.render_pass_id,
            buffer_index: 0,
            clear_value,
            region,
        }
    }
}

/// Represents the depth-stencil buffer for a [DefaultRenderTarget] with both a depth and a stencil
/// channel.
pub struct DefaultDepthStencilBuffer {
    render_pass_id: usize,
}

impl DefaultDepthStencilBuffer {
    pub(crate) fn new(render_pass_id: usize) -> Self {
        DefaultDepthStencilBuffer { render_pass_id }
    }

    /// Returns a command that clears all depth values in the `region` to `depth` and all stencil
    /// values in the `region` to `stencil`.
    ///
    /// Values outside the region are unaffected. See also [clear_depth_command] for a command that
    /// clears only depth values, and [clear_stencil_command] for a command that clears only stencil
    /// values.
    ///
    /// # Examples
    ///
    /// The following command will clear the entire depth-stencil buffer to a depth value of `1.0`
    /// and a stencil value of `0`:
    ///
    /// ```
    /// # use web_glitz::rendering::DefaultDepthStencilBuffer;
    /// # use web_glitz::image::Region2D;
    /// # fn wrapper(buffer: &DefaultDepthStencilBuffer) {
    /// let command = buffer.clear_command(1.0, 0, Region2D::Fill);
    /// # }
    /// ```
    ///
    /// It's also possible to only clear a specific rectangular area of the buffer:
    ///
    /// ```
    /// # use web_glitz::rendering::DefaultDepthStencilBuffer;
    /// # use web_glitz::image::Region2D;
    /// # fn wrapper(buffer: &DefaultDepthStencilBuffer) {
    /// let command = buffer.clear_command(1.0, 0, Region2D::Area((0, 0), 100, 100));
    /// # }
    /// ```
    ///
    /// See also [Region2D].
    pub fn clear_command(
        &self,
        depth: f32,
        stencil: i32,
        region: Region2D,
    ) -> ClearDepthStencilCommand {
        ClearDepthStencilCommand {
            render_pass_id: self.render_pass_id,
            depth,
            stencil,
            region,
        }
    }

    /// Returns a command that clears all depth values in the `region` to `depth`.
    ///
    /// Stencil values and depth values outside the region are unaffected. See also
    /// [clear_stencil_command] for a command that clears stencil values, and [clear_command] for
    /// a command that clears both depth and stencil values.
    ///
    /// # Examples
    ///
    /// The following command will clear the entire depth-stencil buffer to a depth value of `1.0`:
    ///
    /// ```
    /// # use web_glitz::rendering::DefaultDepthStencilBuffer;
    /// # use web_glitz::image::Region2D;
    /// # fn wrapper(buffer: &DefaultDepthStencilBuffer) {
    /// let command = buffer.clear_depth_command(1.0, Region2D::Fill);
    /// # }
    /// ```
    ///
    /// The stencil values in the depth-stencil buffer will not change.
    ///
    /// It's also possible to only clear a specific rectangular area of the buffer:
    ///
    /// ```
    /// # use web_glitz::rendering::DefaultDepthStencilBuffer;
    /// # use web_glitz::image::Region2D;
    /// # fn wrapper(buffer: &DefaultDepthStencilBuffer) {
    /// let command = buffer.clear_depth_command(1.0, Region2D::Area((0, 0), 100, 100));
    /// # }
    /// ```
    ///
    /// See also [Region2D].
    pub fn clear_depth_command(&self, depth: f32, region: Region2D) -> ClearDepthCommand {
        ClearDepthCommand {
            render_pass_id: self.render_pass_id,
            depth,
            region,
        }
    }

    /// Returns a command that clears all stencil values in the `region` to `stencil`.
    ///
    /// Depth values and stencil values outside the region are unaffected. See also
    /// [clear_depth_command] for a command that clears depth values, and [clear_command] for
    /// a command that clears both depth and stencil values.
    ///
    /// # Examples
    ///
    /// The following command will clear the entire depth-stencil buffer to a stencil value of `0`:
    ///
    /// ```
    /// # use web_glitz::rendering::DefaultDepthStencilBuffer;
    /// # use web_glitz::image::Region2D;
    /// # fn wrapper(buffer: &DefaultDepthStencilBuffer) {
    /// let command = buffer.clear_stencil_command(0, Region2D::Fill);
    /// # }
    /// ```
    ///
    /// The depth values in the depth-stencil buffer will not change.
    ///
    /// It's also possible to only clear a specific rectangular area of the buffer:
    ///
    /// ```
    /// # use web_glitz::rendering::DefaultDepthStencilBuffer;
    /// # use web_glitz::image::Region2D;
    /// # fn wrapper(buffer: &DefaultDepthStencilBuffer) {
    /// let command = buffer.clear_stencil_command(0, Region2D::Area((0, 0), 100, 100));
    /// # }
    /// ```
    ///
    /// See also [Region2D].
    pub fn clear_stencil_command(&self, stencil: i32, region: Region2D) -> ClearStencilCommand {
        ClearStencilCommand {
            render_pass_id: self.render_pass_id,
            stencil,
            region,
        }
    }
}

/// Represents the depth-stencil buffer for a [DefaultRenderTarget] with only a depth channel and
/// no stencil channel.
pub struct DefaultDepthBuffer {
    render_pass_id: usize,
}

impl DefaultDepthBuffer {
    pub(crate) fn new(render_pass_id: usize) -> Self {
        DefaultDepthBuffer { render_pass_id }
    }

    /// Returns a command that clears all depth values in the `region` to `depth`.
    ///
    /// Values outside the region are unaffected.
    ///
    /// # Examples
    ///
    /// The following command will clear the entire depth-stencil buffer to a depth value of `1.0`:
    ///
    /// ```
    /// # use web_glitz::rendering::DefaultDepthBuffer;
    /// # use web_glitz::image::Region2D;
    /// # fn wrapper(buffer: &DefaultDepthBuffer) {
    /// let command = buffer.clear_command(1.0, Region2D::Fill);
    /// # }
    /// ```
    ///
    /// It's also possible to only clear a specific rectangular area of the buffer:
    ///
    /// ```
    /// # use web_glitz::rendering::DefaultDepthBuffer;
    /// # use web_glitz::image::Region2D;
    /// # fn wrapper(buffer: &DefaultDepthBuffer) {
    /// let command = buffer.clear_command(1.0, Region2D::Area((0, 0), 100, 100));
    /// # }
    /// ```
    ///
    /// See also [Region2D].
    pub fn clear_command(&self, depth: f32, region: Region2D) -> ClearDepthCommand {
        ClearDepthCommand {
            render_pass_id: self.render_pass_id,
            depth,
            region,
        }
    }
}

/// Represents the depth-stencil buffer for a [DefaultRenderTarget] with only a stencil channel and
/// no depth channel.
pub struct DefaultStencilBuffer {
    render_pass_id: usize,
}

impl DefaultStencilBuffer {
    pub(crate) fn new(render_pass_id: usize) -> Self {
        DefaultStencilBuffer { render_pass_id }
    }

    /// Returns a command that clears all stencil values in the `region` to `stencil`.
    ///
    /// Values outside the region are unaffected.
    ///
    /// # Examples
    ///
    /// The following command will clear the entire depth-stencil buffer to a stencil value of `0`:
    ///
    /// ```
    /// # use web_glitz::rendering::DefaultStencilBuffer;
    /// # use web_glitz::image::Region2D;
    /// # fn wrapper(buffer: &DefaultStencilBuffer) {
    /// let command = buffer.clear_command(0, Region2D::Fill);
    /// # }
    /// ```
    ///
    /// It's also possible to only clear a specific rectangular area of the buffer:
    ///
    /// ```
    /// # use web_glitz::rendering::DefaultStencilBuffer;
    /// # use web_glitz::image::Region2D;
    /// # fn wrapper(buffer: &DefaultStencilBuffer) {
    /// let command = buffer.clear_command(0, Region2D::Area((0, 0), 100, 100));
    /// # }
    /// ```
    ///
    /// See also [Region2D].
    pub fn clear_command(&self, stencil: i32, region: Region2D) -> ClearStencilCommand {
        ClearStencilCommand {
            render_pass_id: self.render_pass_id,
            stencil,
            region,
        }
    }
}

/// Trait implemented by types that represent a rendering output buffer in the framebuffer for a
/// custom render target.
pub trait RenderingOutputBuffer {
    /// The type image storage format used by the buffer.
    type Format: InternalFormat;

    /// The width of the buffer (in pixels).
    fn width(&self) -> u32;

    /// The height of the buffer (in pixels).
    fn height(&self) -> u32;
}

/// Represents a color buffer that stores floating point values in a framebuffer for a custom render
/// target.
pub struct FloatBuffer<F> {
    render_pass_id: usize,
    index: i32,
    width: u32,
    height: u32,
    _marker: marker::PhantomData<Box<F>>,
}

impl<F> FloatBuffer<F> {
    pub(crate) fn new(render_pass_id: usize, index: i32, width: u32, height: u32) -> Self {
        FloatBuffer {
            render_pass_id,
            index,
            width,
            height,
            _marker: marker::PhantomData,
        }
    }

    /// Returns a command that clears all pixel values in the `region` to `clear_value`.
    ///
    /// Values outside the region are unaffected.
    ///
    /// # Examples
    ///
    /// The following command will clear the entire buffer to a value of "transparent black":
    ///
    /// ```
    /// # use web_glitz::rendering::FloatBuffer;
    /// # use web_glitz::image::Region2D;
    /// # use web_glitz::image::format::RGBA8;
    /// # fn wrapper(buffer: &FloatBuffer<RGBA8>) {
    /// let command = buffer.clear_command([0.0, 0.0, 0.0, 0.0], Region2D::Fill);
    /// # }
    /// ```
    ///
    /// It's also possible to only clear a specific rectangular area of the buffer:
    ///
    /// ```
    /// # use web_glitz::rendering::FloatBuffer;
    /// # use web_glitz::image::Region2D;
    /// # use web_glitz::image::format::RGBA8;
    /// # fn wrapper(buffer: &FloatBuffer<RGBA8>) {
    /// let command = buffer.clear_command([0.0, 0.0, 0.0, 0.0], Region2D::Area((0, 0), 100, 100));
    /// # }
    /// ```
    ///
    /// See also [Region2D].
    pub fn clear_command(&self, clear_value: [f32; 4], region: Region2D) -> ClearFloatCommand {
        ClearFloatCommand {
            render_pass_id: self.render_pass_id,
            buffer_index: self.index,
            clear_value,
            region,
        }
    }
}

impl<F> RenderingOutputBuffer for FloatBuffer<F>
where
    F: InternalFormat,
{
    type Format = F;

    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }
}

/// Represents a color buffer that stores integer values in a framebuffer for a custom render
/// target.
pub struct IntegerBuffer<F> {
    render_pass_id: usize,
    index: i32,
    width: u32,
    height: u32,
    _marker: marker::PhantomData<Box<F>>,
}

impl<F> IntegerBuffer<F> {
    pub(crate) fn new(render_pass_id: usize, index: i32, width: u32, height: u32) -> Self {
        IntegerBuffer {
            render_pass_id,
            index,
            width,
            height,
            _marker: marker::PhantomData,
        }
    }

    /// Returns a command that clears all pixel values in the `region` to `clear_value`.
    ///
    /// Values outside the region are unaffected.
    ///
    /// # Examples
    ///
    /// The following command will clear all pixels in the buffer to all zeroes:
    ///
    /// ```
    /// # use web_glitz::rendering::IntegerBuffer;
    /// # use web_glitz::image::Region2D;
    /// # use web_glitz::image::format::RGBA8I;
    /// # fn wrapper(buffer: &IntegerBuffer<RGBA8I>) {
    /// let command = buffer.clear_command([0, 0, 0, 0], Region2D::Fill);
    /// # }
    /// ```
    ///
    /// It's also possible to only clear a specific rectangular area of the buffer:
    ///
    /// ```
    /// # use web_glitz::rendering::IntegerBuffer;
    /// # use web_glitz::image::Region2D;
    /// # use web_glitz::image::format::RGBA8I;
    /// # fn wrapper(buffer: &IntegerBuffer<RGBA8I>) {
    /// let command = buffer.clear_command([0, 0, 0, 0], Region2D::Area((0, 0), 100, 100));
    /// # }
    /// ```
    ///
    /// See also [Region2D].
    pub fn clear_command(&self, clear_value: [i32; 4], region: Region2D) -> ClearIntegerCommand {
        ClearIntegerCommand {
            render_pass_id: self.render_pass_id,
            buffer_index: self.index,
            clear_value,
            region,
        }
    }
}
impl<F> RenderingOutputBuffer for IntegerBuffer<F>
where
    F: InternalFormat,
{
    type Format = F;

    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }
}

/// Represents a color buffer that stores unsigned integer values in a framebuffer for a custom
/// render target.
pub struct UnsignedIntegerBuffer<F> {
    render_pass_id: usize,
    index: i32,
    width: u32,
    height: u32,
    _marker: marker::PhantomData<Box<F>>,
}

impl<F> UnsignedIntegerBuffer<F> {
    pub(crate) fn new(render_pass_id: usize, index: i32, width: u32, height: u32) -> Self {
        UnsignedIntegerBuffer {
            render_pass_id,
            index,
            width,
            height,
            _marker: marker::PhantomData,
        }
    }

    /// Returns a command that clears all pixel values in the `region` to `clear_value`.
    ///
    /// Values outside the region are unaffected.
    ///
    /// # Examples
    ///
    /// The following command will clear all pixels in the buffer to all zeroes:
    ///
    /// ```
    /// # use web_glitz::rendering::UnsignedIntegerBuffer;
    /// # use web_glitz::image::Region2D;
    /// # use web_glitz::image::format::RGBA8UI;
    /// # fn wrapper(buffer: &UnsignedIntegerBuffer<RGBA8UI>) {
    /// let command = buffer.clear_command([0, 0, 0, 0], Region2D::Fill);
    /// # }
    /// ```
    ///
    /// It's also possible to only clear a specific rectangular area of the buffer:
    ///
    /// ```
    /// # use web_glitz::rendering::UnsignedIntegerBuffer;
    /// # use web_glitz::image::Region2D;
    /// # use web_glitz::image::format::RGBA8UI;
    /// # fn wrapper(buffer: &UnsignedIntegerBuffer<RGBA8UI>) {
    /// let command = buffer.clear_command([0, 0, 0, 0], Region2D::Area((0, 0), 100, 100));
    /// # }
    /// ```
    ///
    /// See also [Region2D].
    pub fn clear_command(
        &self,
        clear_value: [u32; 4],
        region: Region2D,
    ) -> ClearUnsignedIntegerCommand {
        ClearUnsignedIntegerCommand {
            render_pass_id: self.render_pass_id,
            buffer_index: self.index,
            clear_value,
            region,
        }
    }
}

impl<F> RenderingOutputBuffer for UnsignedIntegerBuffer<F>
where
    F: InternalFormat,
{
    type Format = F;

    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }
}

/// Represents a depth-stencil buffer that stores both depth and stencil values in a framebuffer for
/// a custom render target.
pub struct DepthStencilBuffer<F> {
    render_pass_id: usize,
    width: u32,
    height: u32,
    _marker: marker::PhantomData<Box<F>>,
}

impl<F> DepthStencilBuffer<F> {
    pub(crate) fn new(render_pass_id: usize, width: u32, height: u32) -> Self {
        DepthStencilBuffer {
            render_pass_id,
            width,
            height,
            _marker: marker::PhantomData,
        }
    }

    /// Returns a command that clears all depth values in the `region` to `depth` and all stencil
    /// values in the `region` to `stencil`.
    ///
    /// Values outside the region are unaffected. See also [clear_depth_command] for a command that
    /// clears only depth values, and [clear_stencil_command] for a command that clears only stencil
    /// values.
    ///
    /// # Examples
    ///
    /// The following command will clear the entire depth-stencil buffer to a depth value of `1.0`
    /// and a stencil value of `0`:
    ///
    /// ```
    /// # use web_glitz::rendering::DepthStencilBuffer;
    /// # use web_glitz::image::Region2D;
    /// # use web_glitz::image::format::Depth24Stencil8;
    /// # fn wrapper(buffer: &DepthStencilBuffer<Depth24Stencil8>) {
    /// let command = buffer.clear_command(1.0, 0, Region2D::Fill);
    /// # }
    /// ```
    ///
    /// It's also possible to only clear a specific rectangular area of the buffer:
    ///
    /// ```
    /// # use web_glitz::rendering::DepthStencilBuffer;
    /// # use web_glitz::image::Region2D;
    /// # use web_glitz::image::format::Depth24Stencil8;
    /// # fn wrapper(buffer: &DepthStencilBuffer<Depth24Stencil8>) {
    /// let command = buffer.clear_command(1.0, 0, Region2D::Area((0, 0), 100, 100));
    /// # }
    /// ```
    ///
    /// See also [Region2D].
    pub fn clear_command(
        &self,
        depth: f32,
        stencil: i32,
        region: Region2D,
    ) -> ClearDepthStencilCommand {
        ClearDepthStencilCommand {
            render_pass_id: self.render_pass_id,
            depth,
            stencil,
            region,
        }
    }

    /// Returns a command that clears all depth values in the `region` to `depth`.
    ///
    /// Stencil values and depth values outside the region are unaffected. See also
    /// [clear_stencil_command] for a command that clears stencil values, and [clear_command] for
    /// a command that clears both depth and stencil values.
    ///
    /// # Examples
    ///
    /// The following command will clear the entire depth-stencil buffer to a depth value of `1.0`:
    ///
    /// ```
    /// # use web_glitz::rendering::DepthStencilBuffer;
    /// # use web_glitz::image::Region2D;
    /// # use web_glitz::image::format::Depth24Stencil8;
    /// # fn wrapper(buffer: &DepthStencilBuffer<Depth24Stencil8>) {
    /// let command = buffer.clear_depth_command(1.0, Region2D::Fill);
    /// # }
    /// ```
    ///
    /// The stencil values in the depth-stencil buffer will not change.
    ///
    /// It's also possible to only clear a specific rectangular area of the buffer:
    ///
    /// ```
    /// # use web_glitz::rendering::DepthStencilBuffer;
    /// # use web_glitz::image::Region2D;
    /// # use web_glitz::image::format::Depth24Stencil8;
    /// # fn wrapper(buffer: &DepthStencilBuffer<Depth24Stencil8>) {
    /// let command = buffer.clear_depth_command(1.0, Region2D::Area((0, 0), 100, 100));
    /// # }
    /// ```
    ///
    /// See also [Region2D].
    pub fn clear_depth_command(&self, depth: f32, region: Region2D) -> ClearDepthCommand {
        ClearDepthCommand {
            render_pass_id: self.render_pass_id,
            depth,
            region,
        }
    }

    /// Returns a command that clears all stencil values in the `region` to `stencil`.
    ///
    /// Depth values and stencil values outside the region are unaffected. See also
    /// [clear_depth_command] for a command that clears depth values, and [clear_command] for
    /// a command that clears both depth and stencil values.
    ///
    /// # Examples
    ///
    /// The following command will clear the entire depth-stencil buffer to a stencil value of `0`:
    ///
    /// ```
    /// # use web_glitz::rendering::DepthStencilBuffer;
    /// # use web_glitz::image::Region2D;
    /// # use web_glitz::image::format::Depth24Stencil8;
    /// # fn wrapper(buffer: &DepthStencilBuffer<Depth24Stencil8>) {
    /// let command = buffer.clear_stencil_command(0, Region2D::Fill);
    /// # }
    /// ```
    ///
    /// The depth values in the depth-stencil buffer will not change.
    ///
    /// It's also possible to only clear a specific rectangular area of the buffer:
    ///
    /// ```
    /// # use web_glitz::rendering::DepthStencilBuffer;
    /// # use web_glitz::image::Region2D;
    /// # use web_glitz::image::format::Depth24Stencil8;
    /// # fn wrapper(buffer: &DepthStencilBuffer<Depth24Stencil8>) {
    /// let command = buffer.clear_stencil_command(0, Region2D::Area((0, 0), 100, 100));
    /// # }
    /// ```
    ///
    /// See also [Region2D].
    pub fn clear_stencil_command(&self, stencil: i32, region: Region2D) -> ClearStencilCommand {
        ClearStencilCommand {
            render_pass_id: self.render_pass_id,
            stencil,
            region,
        }
    }
}

impl<F> RenderingOutputBuffer for DepthStencilBuffer<F>
where
    F: InternalFormat,
{
    type Format = F;

    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }
}

/// Represents a depth-stencil buffer that stores only depth values in a framebuffer for a custom
/// render target.
pub struct DepthBuffer<F> {
    render_pass_id: usize,
    width: u32,
    height: u32,
    _marker: marker::PhantomData<Box<F>>,
}

impl<F> DepthBuffer<F> {
    pub(crate) fn new(render_pass_id: usize, width: u32, height: u32) -> Self {
        DepthBuffer {
            render_pass_id,
            width,
            height,
            _marker: marker::PhantomData,
        }
    }

    /// Returns a command that clears all depth values in the `region` to `depth`.
    ///
    /// Values outside the region are unaffected.
    ///
    /// # Examples
    ///
    /// The following command will clear the entire depth-stencil buffer to a depth value of `1.0`:
    ///
    /// ```
    /// # use web_glitz::rendering::DepthBuffer;
    /// # use web_glitz::image::Region2D;
    /// # use web_glitz::image::format::DepthComponent24;
    /// # fn wrapper(buffer: &DepthBuffer<DepthComponent24>) {
    /// let command = buffer.clear_command(1.0, Region2D::Fill);
    /// # }
    /// ```
    ///
    /// It's also possible to only clear a specific rectangular area of the buffer:
    ///
    /// ```
    /// # use web_glitz::rendering::DepthBuffer;
    /// # use web_glitz::image::Region2D;
    /// # use web_glitz::image::format::DepthComponent24;
    /// # fn wrapper(buffer: &DepthBuffer<DepthComponent24>) {
    /// let command = buffer.clear_command(1.0, Region2D::Area((0, 0), 100, 100));
    /// # }
    /// ```
    ///
    /// See also [Region2D].
    pub fn clear_command(&self, depth: f32, region: Region2D) -> ClearDepthCommand {
        ClearDepthCommand {
            render_pass_id: self.render_pass_id,
            depth,
            region,
        }
    }
}

impl<F> RenderingOutputBuffer for DepthBuffer<F>
where
    F: InternalFormat,
{
    type Format = F;

    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }
}

pub struct StencilBuffer<F> {
    render_pass_id: usize,
    width: u32,
    height: u32,
    _marker: marker::PhantomData<Box<F>>,
}

impl<F> StencilBuffer<F> {
    pub(crate) fn new(render_pass_id: usize, width: u32, height: u32) -> Self {
        StencilBuffer {
            render_pass_id,
            width,
            height,
            _marker: marker::PhantomData,
        }
    }

    /// Returns a command that clears all stencil values in the `region` to `stencil`.
    ///
    /// Values outside the region are unaffected.
    ///
    /// # Examples
    ///
    /// The following command will clear the entire depth-stencil buffer to a stencil value of `0`:
    ///
    /// ```
    /// # use web_glitz::rendering::StencilBuffer;
    /// # use web_glitz::image::Region2D;
    /// # use web_glitz::image::format::StencilIndex8;
    /// # fn wrapper(buffer: &StencilBuffer<StencilIndex8>) {
    /// let command = buffer.clear_command(0, Region2D::Fill);
    /// # }
    /// ```
    ///
    /// It's also possible to only clear a specific rectangular area of the buffer:
    ///
    /// ```
    /// # use web_glitz::rendering::StencilBuffer;
    /// # use web_glitz::image::Region2D;
    /// # use web_glitz::image::format::StencilIndex8;
    /// # fn wrapper(buffer: &StencilBuffer<StencilIndex8>) {
    /// let command = buffer.clear_command(0, Region2D::Area((0, 0), 100, 100));
    /// # }
    /// ```
    ///
    /// See also [Region2D].
    pub fn clear_command(&self, stencil: i32, region: Region2D) -> ClearStencilCommand {
        ClearStencilCommand {
            render_pass_id: self.render_pass_id,
            stencil,
            region,
        }
    }
}

impl<F> RenderingOutputBuffer for StencilBuffer<F>
where
    F: InternalFormat,
{
    type Format = F;

    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }
}

/// Command that will clear a region of a color buffer that stores floating point values.
///
/// See [FloatBuffer::clear_command], [DefaultRGBBuffer::clear_command] and
/// [DefaultRGBABuffer::clear_command].
#[derive(Clone)]
pub struct ClearFloatCommand {
    render_pass_id: usize,
    buffer_index: i32,
    clear_value: [f32; 4],
    region: Region2D,
}

unsafe impl GpuTask<RenderPassContext> for ClearFloatCommand {
    type Output = ();

    fn context_id(&self) -> ContextId {
        ContextId::Id(self.render_pass_id)
    }

    fn progress(&mut self, context: &mut RenderPassContext) -> Progress<Self::Output> {
        let (gl, state) = unsafe { context.unpack_mut() };

        match self.region {
            Region2D::Fill => state.set_scissor_test_enabled(false).apply(gl).unwrap(),
            Region2D::Area((x, y), width, height) => {
                state.set_scissor_test_enabled(true).apply(gl).unwrap();
                state
                    .set_scissor_rect((x as i32, y as i32, width, height))
                    .apply(gl)
                    .unwrap();
            }
        }

        gl.clear_bufferfv_with_f32_array(Gl::COLOR, self.buffer_index, &self.clear_value);

        Progress::Finished(())
    }
}

/// Command that will clear a region of a color buffer that stores integer values.
///
/// See [IntegerBuffer::clear_command].
#[derive(Clone)]
pub struct ClearIntegerCommand {
    render_pass_id: usize,
    buffer_index: i32,
    clear_value: [i32; 4],
    region: Region2D,
}

unsafe impl GpuTask<RenderPassContext> for ClearIntegerCommand {
    type Output = ();

    fn context_id(&self) -> ContextId {
        ContextId::Id(self.render_pass_id)
    }

    fn progress(&mut self, context: &mut RenderPassContext) -> Progress<Self::Output> {
        let (gl, state) = unsafe { context.unpack_mut() };

        match self.region {
            Region2D::Fill => state.set_scissor_test_enabled(false).apply(gl).unwrap(),
            Region2D::Area((x, y), width, height) => {
                state.set_scissor_test_enabled(true).apply(gl).unwrap();
                state
                    .set_scissor_rect((x as i32, y as i32, width, height))
                    .apply(gl)
                    .unwrap();
            }
        }

        gl.clear_bufferiv_with_i32_array(Gl::COLOR, self.buffer_index, &self.clear_value);

        Progress::Finished(())
    }
}

/// Command that will clear a region of a color buffer that stores unsigned integer values.
///
/// See [UnsignedIntegerBuffer::clear_command].
#[derive(Clone)]
pub struct ClearUnsignedIntegerCommand {
    render_pass_id: usize,
    buffer_index: i32,
    clear_value: [u32; 4],
    region: Region2D,
}

unsafe impl GpuTask<RenderPassContext> for ClearUnsignedIntegerCommand {
    type Output = ();

    fn context_id(&self) -> ContextId {
        ContextId::Id(self.render_pass_id)
    }

    fn progress(&mut self, context: &mut RenderPassContext) -> Progress<Self::Output> {
        let (gl, state) = unsafe { context.unpack_mut() };

        match self.region {
            Region2D::Fill => state.set_scissor_test_enabled(false).apply(gl).unwrap(),
            Region2D::Area((x, y), width, height) => {
                state.set_scissor_test_enabled(true).apply(gl).unwrap();
                state
                    .set_scissor_rect((x as i32, y as i32, width, height))
                    .apply(gl)
                    .unwrap();
            }
        }

        gl.clear_bufferuiv_with_u32_array(Gl::COLOR, self.buffer_index, &self.clear_value);

        Progress::Finished(())
    }
}

/// Command that will clear the depth and stencil values in a region of a depth-stencil buffer.
///
/// See [DepthStencilBuffer::clear_command] and [DefaultDepthStencilBuffer::clear_command].
#[derive(Clone)]
pub struct ClearDepthStencilCommand {
    render_pass_id: usize,
    depth: f32,
    stencil: i32,
    region: Region2D,
}

unsafe impl GpuTask<RenderPassContext> for ClearDepthStencilCommand {
    type Output = ();

    fn context_id(&self) -> ContextId {
        ContextId::Id(self.render_pass_id)
    }

    fn progress(&mut self, context: &mut RenderPassContext) -> Progress<Self::Output> {
        let (gl, state) = unsafe { context.unpack_mut() };

        match self.region {
            Region2D::Fill => state.set_scissor_test_enabled(false).apply(gl).unwrap(),
            Region2D::Area((x, y), width, height) => {
                state.set_scissor_test_enabled(true).apply(gl).unwrap();
                state
                    .set_scissor_rect((x as i32, y as i32, width, height))
                    .apply(gl)
                    .unwrap();
            }
        }

        gl.clear_bufferfi(Gl::DEPTH_STENCIL, 0, self.depth, self.stencil);

        Progress::Finished(())
    }
}

/// Command that will clear the depth values in a region of a depth-stencil buffer.
///
/// See [DepthStencilBuffer::clear_depth_command], [DepthBuffer::clear_command],
/// [DefaultDepthStencilBuffer::clear_depth_command] and [DefaultDepthBuffer::clear_command].
#[derive(Clone)]
pub struct ClearDepthCommand {
    render_pass_id: usize,
    depth: f32,
    region: Region2D,
}

unsafe impl GpuTask<RenderPassContext> for ClearDepthCommand {
    type Output = ();

    fn context_id(&self) -> ContextId {
        ContextId::Id(self.render_pass_id)
    }

    fn progress(&mut self, context: &mut RenderPassContext) -> Progress<Self::Output> {
        let (gl, state) = unsafe { context.unpack_mut() };

        match self.region {
            Region2D::Fill => state.set_scissor_test_enabled(false).apply(gl).unwrap(),
            Region2D::Area((x, y), width, height) => {
                state.set_scissor_test_enabled(true).apply(gl).unwrap();
                state
                    .set_scissor_rect((x as i32, y as i32, width, height))
                    .apply(gl)
                    .unwrap();
            }
        }

        gl.clear_bufferfv_with_f32_array(Gl::DEPTH, 0, &[self.depth]);

        Progress::Finished(())
    }
}

/// Command that will clear the stencil values in a region of a depth-stencil buffer.
///
/// See [DepthStencilBuffer::clear_stencil_command], [StencilBuffer::clear_command],
/// [DefaultDepthStencilBuffer::clear_stencil_command] and [DefaultStencilBuffer::clear_command].
#[derive(Clone)]
pub struct ClearStencilCommand {
    render_pass_id: usize,
    stencil: i32,
    region: Region2D,
}

unsafe impl GpuTask<RenderPassContext> for ClearStencilCommand {
    type Output = ();

    fn context_id(&self) -> ContextId {
        ContextId::Id(self.render_pass_id)
    }

    fn progress(&mut self, context: &mut RenderPassContext) -> Progress<Self::Output> {
        let (gl, state) = unsafe { context.unpack_mut() };

        match self.region {
            Region2D::Fill => state.set_scissor_test_enabled(false).apply(gl).unwrap(),
            Region2D::Area((x, y), width, height) => {
                state.set_scissor_test_enabled(true).apply(gl).unwrap();
                state
                    .set_scissor_rect((x as i32, y as i32, width, height))
                    .apply(gl)
                    .unwrap();
            }
        }

        gl.clear_bufferiv_with_i32_array(Gl::STENCIL, 0, &[self.stencil]);

        Progress::Finished(())
    }
}
