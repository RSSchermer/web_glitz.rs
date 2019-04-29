use std::borrow::Borrow;
use std::cell::Cell;
use std::hash::{Hash, Hasher};
use std::marker;

use fnv::FnvHasher;

use web_sys::WebGl2RenderingContext as Gl;

use crate::image::format::{
    DepthRenderable, DepthStencilRenderable, Filterable, FloatRenderable, IntegerRenderable,
    InternalFormat, RenderbufferFormat, StencilRenderable, TextureFormat,
    UnsignedIntegerRenderable,
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
use crate::pipeline::graphics::primitive_assembly::Topology;
use crate::pipeline::graphics::{
    AttributeSlotLayoutCompatible, Blending, DepthTest, GraphicsPipeline, PrimitiveAssembly,
    StencilTest, Viewport,
};
use crate::pipeline::resources::bind_group_encoding::{
    BindGroupEncodingContext, BindingDescriptor,
};
use crate::pipeline::resources::Resources;
use crate::render_pass::RenderPassContext;
use crate::render_target::attachable_image_ref::{AttachableImageData, AttachableImageRef};
use crate::runtime::state::ContextUpdate;
use crate::runtime::Connection;
use crate::task::{ContextId, GpuTask, Progress};
use crate::util::{slice_make_mut, JsId};
use crate::vertex::{VertexStreamDescription, VertexStreamDescriptor};

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
    pub(crate) dimensions: Option<(u32, u32)>,
    pub(crate) context_id: usize,
    pub(crate) render_pass_id: usize,
    pub(crate) last_pipeline_task_id: Cell<usize>,
}

impl<C, Ds> Framebuffer<C, Ds> {
    pub fn pipeline_task<V, R, Tf, F, T>(
        &self,
        pipeline: &GraphicsPipeline<V, R, Tf>,
        f: F,
    ) -> PipelineTask<T>
    where
        F: Fn(&ActiveGraphicsPipeline<V, R>) -> T,
        for<'a> T: GpuTask<PipelineTaskContext<'a>>,
    {
        if self.context_id != pipeline.context_id() {
            panic!("The pipeline does not belong to the same context as the framebuffer.");
        }

        let id = self.last_pipeline_task_id.get();

        self.last_pipeline_task_id.set(id + 1);

        let mut hasher = FnvHasher::default();

        (self.render_pass_id, id).hash(&mut hasher);

        let pipeline_task_id = hasher.finish() as usize;

        let task = f(&ActiveGraphicsPipeline {
            context_id: self.context_id,
            topology: pipeline.primitive_assembly().topology(),
            pipeline_task_id,
            _vertex_attribute_layout_marker: marker::PhantomData,
            _resources_marker: marker::PhantomData,
        });

        if task.context_id() != ContextId::Id(pipeline_task_id) {
            panic!("Task does not belong to the pipeline task context.")
        }

        PipelineTask {
            render_pass_id: self.render_pass_id,
            task,
            program_id: pipeline.program_id(),
            primitive_assembly: pipeline.primitive_assembly().clone(),
            depth_test: pipeline.depth_test().clone(),
            stencil_test: pipeline.stencil_test().clone(),
            scissor_region: pipeline.scissor_region().clone(),
            blending: pipeline.blending().clone(),
            viewport: pipeline.viewport().clone(),
            framebuffer_dimensions: self.dimensions,
        }
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
    /// # use web_glitz::render_target::DefaultRenderTarget;
    /// # use web_glitz::render_pass::DefaultRGBABuffer;
    /// # use web_glitz::image::texture_2d::Texture2D;
    /// # use web_glitz::image::format::RGBA8;
    /// # use web_glitz::runtime::RenderingContext;
    /// # fn wrapper<Rc>(
    /// # context: &Rc,
    /// # mut render_target: DefaultRenderTarget<DefaultRGBABuffer, ()>,
    /// # texture: Texture2D<RGBA8>
    /// # ) where Rc: RenderingContext {
    /// let render_pass = context.create_render_pass(&mut render_target, |framebuffer| {
    ///     framebuffer.blit_color_nearest_command(Region::Fill, texture.base_level())
    /// });
    /// # }
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if `source` belongs to a different context than the framebuffer.
    pub fn blit_color_nearest_command<S>(
        &self,
        region: Region2D,
        source: &S,
    ) -> BlitCommand
    where
        S: BlitColorCompatible<C>,
    {
        let source_descriptor = source.descriptor();

        if source_descriptor.context_id != self.context_id {
            panic!("The source image belongs to a different context than the framebuffer.");
        }

        BlitCommand {
            render_pass_id: self.render_pass_id,
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
    /// # use web_glitz::render_target::DefaultRenderTarget;
    /// # use web_glitz::render_pass::DefaultRGBABuffer;
    /// # use web_glitz::image::texture_2d::Texture2D;
    /// # use web_glitz::image::format::RGBA8;
    /// # use web_glitz::runtime::RenderingContext;
    /// # fn wrapper<Rc>(
    /// # context: &Rc,
    /// # mut render_target: DefaultRenderTarget<DefaultRGBABuffer, ()>,
    /// # texture: Texture2D<RGBA8>
    /// # ) where Rc: RenderingContext {
    /// let render_pass = context.create_render_pass(&mut render_target, |framebuffer| {
    ///     framebuffer.blit_color_linear_command(Region::Fill, texture.base_level())
    /// });
    /// # }
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if `source` belongs to a different context than the framebuffer.
    pub fn blit_color_linear_command<S>(
        &self,
        region: Region2D,
        source: &S,
    ) -> BlitCommand
    where
        S: BlitColorCompatible<C>,
        S::Format: Filterable,
    {
        let source_descriptor = source.descriptor();

        if source_descriptor.context_id != self.context_id {
            panic!("The source image belongs to a different context than the framebuffer.");
        }

        BlitCommand {
            render_pass_id: self.render_pass_id,
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
    /// # use web_glitz::render_target::RenderTargetDescription;
    /// # use web_glitz::render_pass::{ Framebuffer, DepthStencilBuffer};
    /// # use web_glitz::image::format::Depth24Stencil8;
    /// # use web_glitz::runtime::RenderingContext;
    /// # use web_glitz::image::renderbuffer::Renderbuffer;
    /// # fn wrapper<Rc, T>(
    /// # context: &Rc,
    /// # mut render_target: T,
    /// # renderbuffer: Renderbuffer<Depth24Stencil8>
    /// # ) where
    /// # Rc: RenderingContext,
    /// # T: RenderTargetDescription<Framebuffer=Framebuffer<(), DepthStencilBuffer<Depth24Stencil8>>> {
    /// let render_pass = context.create_render_pass(&mut render_target, |framebuffer| {
    ///     framebuffer.blit_depth_stencil_command(Region::Fill, &renderbuffer)
    /// });
    /// # }
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if `source` belongs to a different context than the framebuffer.
    pub fn blit_depth_stencil_command<S>(
        &self,
        region: Region2D,
        source: &S,
    ) -> BlitCommand
    where
        S: BlitSource<Format = F>,
    {
        let source_descriptor = source.descriptor();

        if source_descriptor.context_id != self.context_id {
            panic!("The source image belongs to a different context than the framebuffer.");
        }

        BlitCommand {
            render_pass_id: self.render_pass_id,
            read_slot: Gl::DEPTH_STENCIL_ATTACHMENT,
            bitmask: Gl::DEPTH_BUFFER_BIT & Gl::STENCIL_BUFFER_BIT,
            filter: Gl::NEAREST,
            target_region: region,
            target: self.depth_stencil.descriptor(),
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
    /// # use web_glitz::render_target::RenderTargetDescription;
    /// # use web_glitz::render_pass::{ Framebuffer, DepthStencilBuffer};
    /// # use web_glitz::image::format::Depth24Stencil8;
    /// # use web_glitz::runtime::RenderingContext;
    /// # use web_glitz::image::renderbuffer::Renderbuffer;
    /// # fn wrapper<Rc, T>(
    /// # context: &Rc,
    /// # mut render_target: T,
    /// # renderbuffer: Renderbuffer<Depth24Stencil8>
    /// # ) where
    /// # Rc: RenderingContext,
    /// # T: RenderTargetDescription<Framebuffer=Framebuffer<(), DepthStencilBuffer<Depth24Stencil8>>> {
    /// let render_pass = context.create_render_pass(&mut render_target, |framebuffer| {
    ///     framebuffer.blit_depth_command(Region::Fill, &renderbuffer)
    /// });
    /// # }
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if `source` belongs to a different context than the framebuffer.
    pub fn blit_depth_command<S>(
        &self,
        region: Region2D,
        source: &S,
    ) -> BlitCommand
    where
        S: BlitSource<Format = F>,
    {
        let source_descriptor = source.descriptor();

        if source_descriptor.context_id != self.context_id {
            panic!("The source image belongs to a different context than the framebuffer.");
        }

        BlitCommand {
            render_pass_id: self.render_pass_id,
            read_slot: Gl::DEPTH_STENCIL_ATTACHMENT,
            bitmask: Gl::DEPTH_BUFFER_BIT,
            filter: Gl::NEAREST,
            target_region: region,
            target: self.depth_stencil.descriptor(),
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
    /// # use web_glitz::render_target::RenderTargetDescription;
    /// # use web_glitz::render_pass::{ Framebuffer, DepthStencilBuffer};
    /// # use web_glitz::image::format::Depth24Stencil8;
    /// # use web_glitz::runtime::RenderingContext;
    /// # use web_glitz::image::renderbuffer::Renderbuffer;
    /// # fn wrapper<Rc, T>(
    /// # context: &Rc,
    /// # mut render_target: T,
    /// # renderbuffer: Renderbuffer<Depth24Stencil8>
    /// # ) where
    /// # Rc: RenderingContext,
    /// # T: RenderTargetDescription<Framebuffer=Framebuffer<(), DepthStencilBuffer<Depth24Stencil8>>> {
    /// let render_pass = context.create_render_pass(&mut render_target, |framebuffer| {
    ///     framebuffer.blit_depth_command(Region::Fill, &renderbuffer)
    /// });
    /// # }
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if `source` belongs to a different context than the framebuffer.
    pub fn blit_stencil_command<S>(
        &self,
        region: Region2D,
        source: &S,
    ) -> BlitCommand
    where
        S: BlitSource<Format = F>,
    {
        let source_descriptor = source.descriptor();

        if source_descriptor.context_id != self.context_id {
            panic!("The source image belongs to a different context than the framebuffer.");
        }

        BlitCommand {
            render_pass_id: self.render_pass_id,
            read_slot: Gl::DEPTH_STENCIL_ATTACHMENT,
            bitmask: Gl::STENCIL_BUFFER_BIT,
            filter: Gl::NEAREST,
            target_region: region,
            target: self.depth_stencil.descriptor(),
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
    /// # use web_glitz::render_target::RenderTargetDescription;
    /// # use web_glitz::render_pass::{Framebuffer, DepthBuffer};
    /// # use web_glitz::image::format::DepthComponent24;
    /// # use web_glitz::runtime::RenderingContext;
    /// # use web_glitz::image::renderbuffer::Renderbuffer;
    /// # fn wrapper<Rc, T>(
    /// # context: &Rc,
    /// # mut render_target: T,
    /// # renderbuffer: Renderbuffer<DepthComponent24>
    /// # ) where
    /// # Rc: RenderingContext,
    /// # T: RenderTargetDescription<Framebuffer=Framebuffer<(), DepthBuffer<DepthComponent24>>> {
    /// let render_pass = context.create_render_pass(&mut render_target, |framebuffer| {
    ///     framebuffer.blit_depth_command(Region::Fill, &renderbuffer)
    /// });
    /// # }
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if `source` belongs to a different context than the framebuffer.
    pub fn blit_depth_command<S>(
        &self,
        region: Region2D,
        source: &S,
    ) -> BlitCommand
    where
        S: BlitSource<Format = F>,
    {
        let source_descriptor = source.descriptor();

        if source_descriptor.context_id != self.context_id {
            panic!("The source image belongs to a different context than the framebuffer.");
        }

        BlitCommand {
            render_pass_id: self.render_pass_id,
            read_slot: Gl::DEPTH_ATTACHMENT,
            bitmask: Gl::DEPTH_BUFFER_BIT,
            filter: Gl::NEAREST,
            target_region: region,
            target: self.depth_stencil.descriptor(),
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
    /// # use web_glitz::render_target::RenderTargetDescription;
    /// # use web_glitz::render_pass::{Framebuffer, StencilBuffer};
    /// # use web_glitz::image::format::StencilIndex8;
    /// # use web_glitz::runtime::RenderingContext;
    /// # use web_glitz::image::renderbuffer::Renderbuffer;
    /// # fn wrapper<Rc, T>(
    /// # context: &Rc,
    /// # mut render_target: T,
    /// # renderbuffer: Renderbuffer<StencilIndex8>
    /// # ) where
    /// # Rc: RenderingContext,
    /// # T: RenderTargetDescription<Framebuffer=Framebuffer<(), StencilBuffer<StencilIndex8>>> {
    /// let render_pass = context.create_render_pass(&mut render_target, |framebuffer| {
    ///     framebuffer.blit_stencil_command(Region::Fill, &renderbuffer)
    /// });
    /// # }
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if `source` belongs to a different context than the framebuffer.
    pub fn blit_stencil_command<S>(
        &self,
        region: Region2D,
        source: &S,
    ) -> BlitCommand
    where
        S: BlitSource<Format = F>,
    {
        let source_descriptor = source.descriptor();

        if source_descriptor.context_id != self.context_id {
            panic!("The source image belongs to a different context than the framebuffer.");
        }

        BlitCommand {
            render_pass_id: self.render_pass_id,
            read_slot: Gl::STENCIL_ATTACHMENT,
            bitmask: Gl::STENCIL_BUFFER_BIT,
            filter: Gl::NEAREST,
            target_region: region,
            target: self.depth_stencil.descriptor(),
            source: source_descriptor,
        }
    }
}

/// Provides the context necessary for making progress on a [PipelineTask].
pub struct PipelineTaskContext<'a> {
    connection: &'a mut Connection,
}

/// Returned from [Framebuffer::pipeline_task], a series of commands that is executed while a
/// specific [GraphicsPipeline] is bound as the [ActiveGraphicsPipeline].
///
/// See [Framebuffer::pipeline_task].
pub struct PipelineTask<T> {
    render_pass_id: usize,
    task: T,
    program_id: JsId,
    primitive_assembly: PrimitiveAssembly,
    depth_test: Option<DepthTest>,
    stencil_test: Option<StencilTest>,
    scissor_region: Region2D,
    blending: Option<Blending>,
    viewport: Viewport,
    framebuffer_dimensions: Option<(u32, u32)>,
}

unsafe impl<'a, T, O> GpuTask<RenderPassContext<'a>> for PipelineTask<T>
where
    for<'b> T: GpuTask<PipelineTaskContext<'b>, Output = O>,
{
    type Output = O;

    fn context_id(&self) -> ContextId {
        ContextId::Id(self.render_pass_id)
    }

    fn progress(&mut self, context: &mut RenderPassContext<'a>) -> Progress<Self::Output> {
        let (gl, state) = unsafe { context.unpack_mut() };

        unsafe {
            self.program_id.with_value_unchecked(|program_object| {
                state
                    .set_active_program(Some(program_object))
                    .apply(gl)
                    .unwrap();
            })
        };

        let framebuffer_dimensions = self.framebuffer_dimensions.unwrap_or_else(|| {
            (
                gl.drawing_buffer_width() as u32,
                gl.drawing_buffer_height() as u32,
            )
        });

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

        self.task.progress(&mut PipelineTaskContext {
            connection: context.connection_mut(),
        })
    }
}

/// An activated [GraphicsPipeline] which may be used to draw to a [Framebuffer].
///
/// A handle to an [ActiveGraphicsPipeline] is obtained by using a [GraphicsPipeline] to create
/// a [PipelineTask] for a [Framebuffer], see [Framebuffer::pipeline_task].
pub struct ActiveGraphicsPipeline<V, R> {
    context_id: usize,
    topology: Topology,
    pipeline_task_id: usize,
    _vertex_attribute_layout_marker: marker::PhantomData<V>,
    _resources_marker: marker::PhantomData<R>,
}

impl<V, R> ActiveGraphicsPipeline<V, R>
where
    V: AttributeSlotLayoutCompatible,
    R: Resources,
{
    /// Creates a [DrawCommand] that will execute this [ActiveGraphicsPipeline], using the
    /// [vertex_input_stream] as input and with the [resources] bound to the pipeline's resource
    /// slots.
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
    /// # Example
    ///
    /// ```
    /// # use web_glitz::render_pass::{DefaultRenderTarget, DefaultRGBBuffer};
    /// # use web_glitz::runtime::RenderingContext;
    /// # use web_glitz::vertex::{Vertex, VertexArray};
    /// # use web_glitz::buffer::UsageHint;
    /// # use web_glitz::pipeline::graphics::GraphicsPipeline;
    /// # use web_glitz::pipeline::resources::Resources;
    /// # fn wrapper<Rc, V, R>(
    /// #     context: &Rc,
    /// #     mut render_target: DefaultRenderTarget<DefaultRGBBuffer, ()>,
    /// #     vertex_stream: VertexArray<V>,
    /// #     resources: R,
    /// #     graphics_pipeline: GraphicsPipeline<V, R, ()>
    /// # )
    /// # where
    /// #     Rc: RenderingContext,
    /// #     V: Vertex,
    /// #     R: Resources
    /// # {
    /// let render_pass = context.create_render_pass(&mut render_target, |framebuffer| {
    ///     framebuffer.pipeline_task(&graphics_pipeline, |active_pipeline| {
    ///         active_pipeline.draw_command(&vertex_stream, &resources);
    ///     })
    /// });
    /// # }
    /// ```
    ///
    /// In this example `graphics_pipeline` is a [GraphicsPipeline] , see [GraphicsPipeline] and
    /// [RenderingContext::create_graphics_pipeline] for details; `vertex_stream` is a
    /// [VertexStreamDescription], see [VertexStreamDescription], [VertexArray] and
    /// [RenderingContext::create_vertex_array] for details; `resources` is a user-defined type for
    /// which the [Resources] trait is implemented, see [Resources] for details.
    ///
    /// # Panic
    ///
    /// - Panics when [vertex_input_stream] uses a [VertexArray] that belongs to a different context
    ///   than this [ActiveGraphicsPipeline].
    /// - Panics when [resources] specifies a resource that belongs to a different context than this
    ///   [ActiveGraphicsPipeline].
    pub fn draw_command<Vs>(
        &self,
        vertex_input_stream: &Vs,
        resources: &R,
    ) -> DrawCommand<R::Bindings>
    where
        Vs: VertexStreamDescription<AttributeLayout = V>,
        R: Resources,
    {
        let input_stream_descriptor = vertex_input_stream.descriptor();

        if input_stream_descriptor.vertex_array_data.context_id() != self.context_id {
            panic!("Vertex array does not belong to the same context as the pipeline.");
        }

        DrawCommand {
            pipeline_task_id: self.pipeline_task_id,
            vertex_stream_descriptor: vertex_input_stream.descriptor(),
            topology: self.topology,
            binding_group: resources
                .encode_bind_group(&mut BindGroupEncodingContext::new(self.context_id))
                .into_descriptors(),
        }
    }
}

/// Returned from [ActiveGraphicsPipeline::draw_command].
///
/// See [ActiveGraphicsPipeline::draw_command] for details.
pub struct DrawCommand<B> {
    pipeline_task_id: usize,
    vertex_stream_descriptor: VertexStreamDescriptor,
    topology: Topology,
    binding_group: B,
}

unsafe impl<'a, B> GpuTask<PipelineTaskContext<'a>> for DrawCommand<B>
where
    B: Borrow<[BindingDescriptor]>,
{
    type Output = ();

    fn context_id(&self) -> ContextId {
        ContextId::Id(self.pipeline_task_id)
    }

    fn progress(&mut self, context: &mut PipelineTaskContext<'a>) -> Progress<Self::Output> {
        let (gl, state) = unsafe { context.connection.unpack_mut() };

        unsafe {
            self.vertex_stream_descriptor
                .vertex_array_data
                .id()
                .unwrap()
                .with_value_unchecked(|vao| {
                    state.set_bound_vertex_array(Some(vao)).apply(gl).unwrap();
                })
        }

        for descriptor in self.binding_group.borrow().iter() {
            descriptor.bind(context.connection);
        }

        let (gl, _) = unsafe { context.connection.unpack_mut() };

        if let Some(index_type) = self.vertex_stream_descriptor.index_type() {
            let VertexStreamDescriptor {
                offset,
                count,
                instance_count,
                ref vertex_array_data,
                ..
            } = self.vertex_stream_descriptor;

            let offset = vertex_array_data.offset + offset as u32 * index_type.size_in_bytes();

            if instance_count == 1 {
                gl.draw_elements_with_i32(
                    self.topology.id(),
                    count as i32,
                    index_type.id(),
                    offset as i32,
                );
            } else {
                gl.draw_elements_instanced_with_i32(
                    self.topology.id(),
                    count as i32,
                    index_type.id(),
                    offset as i32,
                    instance_count as i32,
                );
            }
        } else {
            let VertexStreamDescriptor {
                offset,
                count,
                instance_count,
                ..
            } = self.vertex_stream_descriptor;

            if instance_count == 1 {
                gl.draw_arrays(self.topology.id(), offset as i32, count as i32);
            } else {
                gl.draw_arrays_instanced(
                    self.topology.id(),
                    offset as i32,
                    count as i32,
                    instance_count as i32,
                );
            }
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

impl<C> BlitColorTarget for C
where
    C: RenderBuffer,
{
    fn descriptor(&self) -> BlitTargetDescriptor {
        BlitTargetDescriptor {
            internal: BlitTargetDescriptorInternal::FBO {
                width: self.width(),
                height: self.height(),
            },
        }
    }
}

macro_rules! impl_blit_color_target {
    ($C0:ident, $($C:ident),*) => {
        impl<$C0, $($C),*> BlitColorTarget for ($C0, $($C),*)
        where
            $C0: RenderBuffer,
            $($C: RenderBuffer),*
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
pub struct BlitTargetDescriptor {
    internal: BlitTargetDescriptorInternal,
}

enum BlitTargetDescriptorInternal {
    Default,
    FBO { width: u32, height: u32 },
}

/// Trait implemented by image reference types that can serve as the image data source for a color
/// [BlitCommand].
///
/// See [Framebuffer::blit_color_nearest_command] and [Framebuffer::blit_color_linear_command].
pub trait BlitSource {
    /// The image storage format used by the source image.
    type Format: InternalFormat;

    /// Encapsulates the information about the blit source required by the [BlitCommand].
    fn descriptor(&self) -> BlitSourceDescriptor;
}

/// Returned from [BlitSource::descriptor], encapsulates the information about the blit source
/// required by the [BlitCommand].
pub struct BlitSourceDescriptor {
    attachment: AttachableImageData,
    region: ((u32, u32), u32, u32),
    context_id: usize,
}

impl<'a, F> BlitSource for Texture2DLevel<'a, F>
where
    F: TextureFormat + 'static,
{
    type Format = F;

    fn descriptor(&self) -> BlitSourceDescriptor {
        BlitSourceDescriptor {
            attachment: AttachableImageRef::from_texture_2d_level(self).into_data(),
            region: ((0, 0), self.width(), self.height()),
            context_id: self.texture_data().context_id(),
        }
    }
}

impl<'a, F> BlitSource for Texture2DLevelSubImage<'a, F>
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
            attachment: AttachableImageRef::from_texture_2d_level(&self.level_ref()).into_data(),
            region: (origin, self.width(), self.height()),
            context_id: self.texture_data().context_id(),
        }
    }
}

impl<'a, F> BlitSource for Texture2DArrayLevelLayer<'a, F>
where
    F: TextureFormat + 'static,
{
    type Format = F;

    fn descriptor(&self) -> BlitSourceDescriptor {
        BlitSourceDescriptor {
            attachment: AttachableImageRef::from_texture_2d_array_level_layer(self).into_data(),
            region: ((0, 0), self.width(), self.height()),
            context_id: self.texture_data().context_id(),
        }
    }
}

impl<'a, F> BlitSource for Texture2DArrayLevelLayerSubImage<'a, F>
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
            attachment: AttachableImageRef::from_texture_2d_array_level_layer(
                &self.level_layer_ref(),
            )
            .into_data(),
            region: (origin, self.width(), self.height()),
            context_id: self.texture_data().context_id(),
        }
    }
}

impl<'a, F> BlitSource for Texture3DLevelLayer<'a, F>
where
    F: TextureFormat + 'static,
{
    type Format = F;

    fn descriptor(&self) -> BlitSourceDescriptor {
        BlitSourceDescriptor {
            attachment: AttachableImageRef::from_texture_3d_level_layer(self).into_data(),
            region: ((0, 0), self.width(), self.height()),
            context_id: self.texture_data().context_id(),
        }
    }
}

impl<'a, F> BlitSource for Texture3DLevelLayerSubImage<'a, F>
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
            attachment: AttachableImageRef::from_texture_3d_level_layer(&self.level_layer_ref())
                .into_data(),
            region: (origin, self.width(), self.height()),
            context_id: self.texture_data().context_id(),
        }
    }
}

impl<'a, F> BlitSource for TextureCubeLevelFace<'a, F>
where
    F: TextureFormat + 'static,
{
    type Format = F;

    fn descriptor(&self) -> BlitSourceDescriptor {
        BlitSourceDescriptor {
            attachment: AttachableImageRef::from_texture_cube_level_face(self).into_data(),
            region: ((0, 0), self.width(), self.height()),
            context_id: self.texture_data().context_id(),
        }
    }
}

impl<'a, F> BlitSource for TextureCubeLevelFaceSubImage<'a, F>
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
            attachment: AttachableImageRef::from_texture_cube_level_face(&self.level_face_ref())
                .into_data(),
            region: (origin, self.width(), self.height()),
            context_id: self.texture_data().context_id(),
        }
    }
}

impl<F> BlitSource for Renderbuffer<F>
where
    F: RenderbufferFormat + 'static,
{
    type Format = F;

    fn descriptor(&self) -> BlitSourceDescriptor {
        BlitSourceDescriptor {
            attachment: AttachableImageRef::from_renderbuffer(self).into_data(),
            region: ((0, 0), self.width(), self.height()),
            context_id: self.data().context_id(),
        }
    }
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
    ($C0:ident, $($C:ident),*) => {
        unsafe impl<T, $C0, $($C),*> BlitColorCompatible<($C0, $($C),*)> for T
        where T: BlitColorCompatible<$C0> $(+ BlitColorCompatible<$C>)* {}
    }
}

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

/// Encapsulates a command that transfers a rectangle of pixels from a source image into the
/// framebuffer.
///
/// See [Framebuffer::blit_color_nearest_command], [Framebuffer::blit_color_linear_command],
/// [Framebuffer::blit_depth_stencil_command], [Framebuffer::blit_depth_command] and
/// [Framebuffer::blit_stencil_command]
pub struct BlitCommand {
    render_pass_id: usize,
    read_slot: u32,
    bitmask: u32,
    filter: u32,
    target: BlitTargetDescriptor,
    target_region: Region2D,
    source: BlitSourceDescriptor,
}

unsafe impl<'a> GpuTask<RenderPassContext<'a>> for BlitCommand {
    type Output = ();

    fn context_id(&self) -> ContextId {
        ContextId::Id(self.render_pass_id)
    }

    fn progress(&mut self, context: &mut RenderPassContext) -> Progress<Self::Output> {
        let (gl, state) = unsafe { context.unpack_mut() };

        state.bind_read_framebuffer(gl);

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
    /// # use web_glitz::render_pass::DefaultRGBBuffer;
    /// # use web_glitz::image::Region2D;
    /// # fn wrapper(buffer: &DefaultRGBBuffer) {
    /// let command = buffer.clear_command([0.0, 0.0, 0.0, 0.0], Region2D::Fill);
    /// # }
    /// ```
    ///
    /// It's also possible to only clear a specific rectangular area of the buffer:
    ///
    /// ```
    /// # use web_glitz::render_pass::DefaultRGBBuffer;
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
    /// # use web_glitz::render_pass::DefaultRGBABuffer;
    /// # use web_glitz::image::Region2D;
    /// # fn wrapper(buffer: &DefaultRGBABuffer) {
    /// let command = buffer.clear_command([0.0, 0.0, 0.0, 0.0], Region2D::Fill);
    /// # }
    /// ```
    ///
    /// It's also possible to only clear a specific rectangular area of the buffer:
    ///
    /// ```
    /// # use web_glitz::render_pass::DefaultRGBABuffer;
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
    /// # use web_glitz::render_pass::DefaultDepthStencilBuffer;
    /// # use web_glitz::image::Region2D;
    /// # fn wrapper(buffer: &DefaultDepthStencilBuffer) {
    /// let command = buffer.clear_command(1.0, 0, Region2D::Fill);
    /// # }
    /// ```
    ///
    /// It's also possible to only clear a specific rectangular area of the buffer:
    ///
    /// ```
    /// # use web_glitz::render_pass::DefaultDepthStencilBuffer;
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
    /// # use web_glitz::render_pass::DefaultDepthStencilBuffer;
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
    /// # use web_glitz::render_pass::DefaultDepthStencilBuffer;
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
    /// # use web_glitz::render_pass::DefaultDepthStencilBuffer;
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
    /// # use web_glitz::render_pass::DefaultDepthStencilBuffer;
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
    /// # use web_glitz::render_pass::DefaultDepthBuffer;
    /// # use web_glitz::image::Region2D;
    /// # fn wrapper(buffer: &DefaultDepthBuffer) {
    /// let command = buffer.clear_command(1.0, Region2D::Fill);
    /// # }
    /// ```
    ///
    /// It's also possible to only clear a specific rectangular area of the buffer:
    ///
    /// ```
    /// # use web_glitz::render_pass::DefaultDepthBuffer;
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
    /// # use web_glitz::render_pass::DefaultStencilBuffer;
    /// # use web_glitz::image::Region2D;
    /// # fn wrapper(buffer: &DefaultStencilBuffer) {
    /// let command = buffer.clear_command(0, Region2D::Fill);
    /// # }
    /// ```
    ///
    /// It's also possible to only clear a specific rectangular area of the buffer:
    ///
    /// ```
    /// # use web_glitz::render_pass::DefaultStencilBuffer;
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

/// Trait implemented by types that represent a buffer in the framebuffer for a custom render
/// target.
pub trait RenderBuffer {
    /// The type image storage format used by the buffer.
    type Format: InternalFormat;

    /// The width of the buffer (in pixels).
    fn width(&self) -> u32;

    /// The height of the buffer (in pixels).
    fn height(&self) -> u32;
}

/// Represents a color buffer that stores floating point values in a framebuffer for a custom render
/// target.
pub struct FloatBuffer<F>
where
    F: FloatRenderable,
{
    render_pass_id: usize,
    index: i32,
    width: u32,
    height: u32,
    _marker: marker::PhantomData<Box<F>>,
}

impl<F> FloatBuffer<F>
where
    F: FloatRenderable,
{
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
    /// # use web_glitz::render_pass::FloatBuffer;
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
    /// # use web_glitz::render_pass::FloatBuffer;
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

impl<F> RenderBuffer for FloatBuffer<F>
where
    F: FloatRenderable,
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
pub struct IntegerBuffer<F>
where
    F: IntegerRenderable,
{
    render_pass_id: usize,
    index: i32,
    width: u32,
    height: u32,
    _marker: marker::PhantomData<Box<F>>,
}

impl<F> IntegerBuffer<F>
where
    F: IntegerRenderable,
{
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
    /// # use web_glitz::render_pass::IntegerBuffer;
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
    /// # use web_glitz::render_pass::IntegerBuffer;
    /// # use web_glitz::image::Region2D;
    /// # use web_glitz::image::format::RGBA8I;
    /// # fn wrapper(buffer: &IntegerBuffer<RGBA8I>) {
    /// let command = buffer.clear_command([0, 0, 0, 0], Region2D::Area((0, 0), 100, 100));
    /// # }
    /// ```
    ///
    /// See also [Region2D].
    pub fn clear_command(
        &self,
        clear_value: [i32; 4],
        region: Region2D,
    ) -> ClearIntegerCommand {
        ClearIntegerCommand {
            render_pass_id: self.render_pass_id,
            buffer_index: self.index,
            clear_value,
            region,
        }
    }
}
impl<F> RenderBuffer for IntegerBuffer<F>
where
    F: IntegerRenderable,
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
pub struct UnsignedIntegerBuffer<F>
where
    F: UnsignedIntegerRenderable,
{
    render_pass_id: usize,
    index: i32,
    width: u32,
    height: u32,
    _marker: marker::PhantomData<Box<F>>,
}

impl<F> UnsignedIntegerBuffer<F>
where
    F: UnsignedIntegerRenderable,
{
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
    /// # use web_glitz::render_pass::UnsignedIntegerBuffer;
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
    /// # use web_glitz::render_pass::UnsignedIntegerBuffer;
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

impl<F> RenderBuffer for UnsignedIntegerBuffer<F>
where
    F: UnsignedIntegerRenderable,
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
pub struct DepthStencilBuffer<F>
where
    F: DepthStencilRenderable,
{
    render_pass_id: usize,
    width: u32,
    height: u32,
    _marker: marker::PhantomData<Box<F>>,
}

impl<F> DepthStencilBuffer<F>
where
    F: DepthStencilRenderable,
{
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
    /// # use web_glitz::render_pass::DepthStencilBuffer;
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
    /// # use web_glitz::render_pass::DepthStencilBuffer;
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
    /// # use web_glitz::render_pass::DepthStencilBuffer;
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
    /// # use web_glitz::render_pass::DepthStencilBuffer;
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
    /// # use web_glitz::render_pass::DepthStencilBuffer;
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
    /// # use web_glitz::render_pass::DepthStencilBuffer;
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

impl<F> RenderBuffer for DepthStencilBuffer<F>
where
    F: DepthStencilRenderable,
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
pub struct DepthBuffer<F>
where
    F: DepthRenderable,
{
    render_pass_id: usize,
    width: u32,
    height: u32,
    _marker: marker::PhantomData<Box<F>>,
}

impl<F> DepthBuffer<F>
where
    F: DepthRenderable,
{
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
    /// # use web_glitz::render_pass::DepthBuffer;
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
    /// # use web_glitz::render_pass::DepthBuffer;
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

impl<F> RenderBuffer for DepthBuffer<F>
where
    F: DepthRenderable,
{
    type Format = F;

    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }
}

pub struct StencilBuffer<F>
where
    F: StencilRenderable,
{
    render_pass_id: usize,
    width: u32,
    height: u32,
    _marker: marker::PhantomData<Box<F>>,
}

impl<F> StencilBuffer<F>
where
    F: StencilRenderable,
{
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
    /// # use web_glitz::render_pass::StencilBuffer;
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
    /// # use web_glitz::render_pass::StencilBuffer;
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

impl<F> RenderBuffer for StencilBuffer<F>
where
    F: StencilRenderable,
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
pub struct ClearFloatCommand {
    render_pass_id: usize,
    buffer_index: i32,
    clear_value: [f32; 4],
    region: Region2D,
}

unsafe impl<'a> GpuTask<RenderPassContext<'a>> for ClearFloatCommand {
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

        gl.clear_bufferfv_with_f32_array(Gl::COLOR, self.buffer_index, unsafe {
            slice_make_mut(&self.clear_value)
        });

        Progress::Finished(())
    }
}

/// Command that will clear a region of a color buffer that stores integer values.
///
/// See [IntegerBuffer::clear_command].
pub struct ClearIntegerCommand {
    render_pass_id: usize,
    buffer_index: i32,
    clear_value: [i32; 4],
    region: Region2D,
}

unsafe impl<'a> GpuTask<RenderPassContext<'a>> for ClearIntegerCommand {
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

        gl.clear_bufferiv_with_i32_array(Gl::COLOR, self.buffer_index, unsafe {
            slice_make_mut(&self.clear_value)
        });

        Progress::Finished(())
    }
}

/// Command that will clear a region of a color buffer that stores unsigned integer values.
///
/// See [UnsignedIntegerBuffer::clear_command].
pub struct ClearUnsignedIntegerCommand {
    render_pass_id: usize,
    buffer_index: i32,
    clear_value: [u32; 4],
    region: Region2D,
}

unsafe impl<'a> GpuTask<RenderPassContext<'a>> for ClearUnsignedIntegerCommand {
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

        gl.clear_bufferuiv_with_u32_array(Gl::COLOR, self.buffer_index, unsafe {
            slice_make_mut(&self.clear_value)
        });

        Progress::Finished(())
    }
}

/// Command that will clear the depth and stencil values in a region of a depth-stencil buffer.
///
/// See [DepthStencilBuffer::clear_command] and [DefaultDepthStencilBuffer::clear_command].
pub struct ClearDepthStencilCommand {
    render_pass_id: usize,
    depth: f32,
    stencil: i32,
    region: Region2D,
}

unsafe impl<'a> GpuTask<RenderPassContext<'a>> for ClearDepthStencilCommand {
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
pub struct ClearDepthCommand {
    render_pass_id: usize,
    depth: f32,
    region: Region2D,
}

unsafe impl<'a> GpuTask<RenderPassContext<'a>> for ClearDepthCommand {
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

        gl.clear_bufferfv_with_f32_array(Gl::DEPTH, 0, unsafe { slice_make_mut(&[self.depth]) });

        Progress::Finished(())
    }
}

/// Command that will clear the stencil values in a region of a depth-stencil buffer.
///
/// See [DepthStencilBuffer::clear_stencil_command], [StencilBuffer::clear_command],
/// [DefaultDepthStencilBuffer::clear_stencil_command] and [DefaultStencilBuffer::clear_command].
pub struct ClearStencilCommand {
    render_pass_id: usize,
    stencil: i32,
    region: Region2D,
}

unsafe impl<'a> GpuTask<RenderPassContext<'a>> for ClearStencilCommand {
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

        gl.clear_bufferiv_with_i32_array(Gl::STENCIL, 0, unsafe {
            slice_make_mut(&[self.stencil])
        });

        Progress::Finished(())
    }
}
