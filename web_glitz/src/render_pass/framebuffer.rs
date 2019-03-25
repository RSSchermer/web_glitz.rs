use std::marker;

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
use crate::render_pass::{
    FramebufferAttachment, IntoFramebufferAttachment, RenderPassContext, RenderPassMismatch,
};
use crate::runtime::state::ContextUpdate;
use crate::task::{GpuTask, Progress, ContextId};
use crate::util::{slice_make_mut, JsId};
use crate::pipeline::graphics::{GraphicsPipeline, Topology, DepthTest, StencilTest, Blending, LineWidth, Viewport, PrimitiveAssembly};
use crate::pipeline::graphics::vertex_input::{InputAttributeLayout, VertexInputStreamDescriptor, VertexInputStreamDescription};
use crate::pipeline::resources::bind_group_encoding::{BindingDescriptor, BindGroupEncodingContext};
use std::borrow::Borrow;
use crate::pipeline::resources::Resources;
use crate::runtime::Connection;
use fnv::FnvHasher;
use std::hash::{Hash, Hasher};
use crate::sampler::IncompatibleSampler::ContextMismatch;

pub struct BlitSourceContextMismatch;

pub struct Framebuffer<C, Ds> {
    pub color: C,
    pub depth_stencil: Ds,
    pub(crate) dimensions: Option<(u32, u32)>,
    pub(crate) context_id: usize,
    pub(crate) render_pass_id: usize,
    pub(crate) last_id: usize
}

impl<C, Ds> Framebuffer<C, Ds> {
    pub fn pipeline_task<Vl, R, Tf, F, T>(&mut self, pipeline: &GraphicsPipeline<Vl, R, Tf>, f: F) -> PipelineTask<T>
        where F: Fn(&ActiveGraphicsPipeline<Vl, R>) -> T, for<'a> T: GpuTask<PipelineTaskContext<'a>>
    {
        let id = self.last_id;

        self.last_id += 1;

        let mut hasher = FnvHasher::default();

        (self.render_pass_id, id).hash(&mut hasher);

        let pipeline_task_id = hasher.finish() as usize;

        let task = f(&ActiveGraphicsPipeline {
            context_id: self.context_id,
            topology: pipeline.primitive_assembly().topology,
            pipeline_task_id,
            _input_attribute_layout_marker: marker::PhantomData,
            _resources_marker: marker::PhantomData
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
            line_width: pipeline.line_width().clone(),
            viewport: pipeline.viewport().clone(),
            framebuffer_dimensions: self.dimensions
        }
    }
}

impl<C, Ds> Framebuffer<C, Ds>
where
    C: BlitColorTarget,
{
    pub fn blit_color_command<S>(
        &self,
        region: Region2D,
        source: &S,
    ) -> Result<BlitCommand, BlitSourceContextMismatch>
    where
        S: BlitColorCompatible<C>,
    {
        let source_descriptor = source.descriptor();

        if source_descriptor.context_id != self.context_id {
            return Err(BlitSourceContextMismatch);
        }

        Ok(BlitCommand {
            render_pass_id: self.render_pass_id,
            read_slot: Gl::COLOR_ATTACHMENT0,
            bitmask: Gl::COLOR_BUFFER_BIT,
            filter: Gl::NEAREST,
            target_region: region,
            target: self.color.descriptor(),
            source: source_descriptor,
        })
    }

    pub fn blit_color_linear_command<S>(
        &self,
        region: Region2D,
        source: &S,
    ) -> Result<BlitCommand, BlitSourceContextMismatch>
    where
        S: BlitColorCompatible<C>,
        S::Format: Filterable,
    {
        let source_descriptor = source.descriptor();

        if source_descriptor.context_id != self.context_id {
            return Err(BlitSourceContextMismatch);
        }

        Ok(BlitCommand {
            render_pass_id: self.render_pass_id,
            read_slot: Gl::COLOR_ATTACHMENT0,
            bitmask: Gl::COLOR_BUFFER_BIT,
            filter: Gl::LINEAR,
            target_region: region,
            target: self.color.descriptor(),
            source: source_descriptor,
        })
    }
}

impl<C, F> Framebuffer<C, DepthStencilBuffer<F>>
where
    F: DepthStencilRenderable,
{
    pub fn blit_depth_stencil_command<S>(
        &self,
        region: Region2D,
        source: &S,
    ) -> Result<BlitCommand, BlitSourceContextMismatch>
    where
        S: BlitSource<Format = F>,
    {
        let source_descriptor = source.descriptor();

        if source_descriptor.context_id != self.context_id {
            return Err(BlitSourceContextMismatch);
        }

        Ok(BlitCommand {
            render_pass_id: self.render_pass_id,
            read_slot: Gl::DEPTH_STENCIL_ATTACHMENT,
            bitmask: Gl::DEPTH_BUFFER_BIT & Gl::STENCIL_BUFFER_BIT,
            filter: Gl::NEAREST,
            target_region: region,
            target: self.depth_stencil.descriptor(),
            source: source_descriptor,
        })
    }

    pub fn blit_depth_command<S>(
        &self,
        region: Region2D,
        source: &S,
    ) -> Result<BlitCommand, BlitSourceContextMismatch>
    where
        S: BlitSource<Format = F>,
    {
        let source_descriptor = source.descriptor();

        if source_descriptor.context_id != self.context_id {
            return Err(BlitSourceContextMismatch);
        }

        Ok(BlitCommand {
            render_pass_id: self.render_pass_id,
            read_slot: Gl::DEPTH_STENCIL_ATTACHMENT,
            bitmask: Gl::DEPTH_BUFFER_BIT,
            filter: Gl::NEAREST,
            target_region: region,
            target: self.depth_stencil.descriptor(),
            source: source_descriptor,
        })
    }

    pub fn blit_stencil_command<S>(
        &self,
        region: Region2D,
        source: &S,
    ) -> Result<BlitCommand, BlitSourceContextMismatch>
    where
        S: BlitSource<Format = F>,
    {
        let source_descriptor = source.descriptor();

        if source_descriptor.context_id != self.context_id {
            return Err(BlitSourceContextMismatch);
        }

        Ok(BlitCommand {
            render_pass_id: self.render_pass_id,
            read_slot: Gl::DEPTH_STENCIL_ATTACHMENT,
            bitmask: Gl::STENCIL_BUFFER_BIT,
            filter: Gl::NEAREST,
            target_region: region,
            target: self.depth_stencil.descriptor(),
            source: source_descriptor,
        })
    }
}

impl<C, F> Framebuffer<C, DepthBuffer<F>>
where
    F: DepthRenderable,
{
    pub fn blit_depth_command<S>(
        &self,
        region: Region2D,
        source: &S,
    ) -> Result<BlitCommand, BlitSourceContextMismatch>
    where
        S: BlitSource<Format = F>,
    {
        let source_descriptor = source.descriptor();

        if source_descriptor.context_id != self.context_id {
            return Err(BlitSourceContextMismatch);
        }

        Ok(BlitCommand {
            render_pass_id: self.render_pass_id,
            read_slot: Gl::DEPTH_ATTACHMENT,
            bitmask: Gl::DEPTH_BUFFER_BIT,
            filter: Gl::NEAREST,
            target_region: region,
            target: self.depth_stencil.descriptor(),
            source: source_descriptor,
        })
    }
}

impl<C, F> Framebuffer<C, StencilBuffer<F>>
where
    F: StencilRenderable,
{
    pub fn blit_stencil_command<S>(
        &self,
        region: Region2D,
        source: &S,
    ) -> Result<BlitCommand, BlitSourceContextMismatch>
    where
        S: BlitSource<Format = F>,
    {
        let source_descriptor = source.descriptor();

        if source_descriptor.context_id != self.context_id {
            return Err(BlitSourceContextMismatch);
        }

        Ok(BlitCommand {
            render_pass_id: self.render_pass_id,
            read_slot: Gl::STENCIL_ATTACHMENT,
            bitmask: Gl::STENCIL_BUFFER_BIT,
            filter: Gl::NEAREST,
            target_region: region,
            target: self.depth_stencil.descriptor(),
            source: source_descriptor,
        })
    }
}

pub struct PipelineTaskContext<'a> {
    connection: &'a mut Connection
}

pub struct PipelineTask<T> {
    render_pass_id: usize,
    task: T,
    program_id: JsId,
    primitive_assembly: PrimitiveAssembly,
    depth_test: Option<DepthTest>,
    stencil_test: Option<StencilTest>,
    scissor_region: Region2D,
    blending: Option<Blending>,
    line_width: LineWidth,
    viewport: Viewport,
    framebuffer_dimensions: Option<(u32, u32)>
}

unsafe impl<'a, T, O> GpuTask<RenderPassContext<'a>> for PipelineTask<T> where for <'b> T: GpuTask<PipelineTaskContext<'b>, Output=O> {
    type Output = O;

    fn context_id(&self) -> ContextId {
        ContextId::Id(self.render_pass_id)
    }

    fn progress(&mut self, context: &mut RenderPassContext<'a>) -> Progress<Self::Output> {
        let (gl, state) = unsafe { context.unpack_mut() };

        unsafe {
            self.program_id.with_value_unchecked(|program_object| {
                state.set_active_program(Some(program_object)).apply(gl).unwrap();
            })
        };

        let framebuffer_dimensions = self.framebuffer_dimensions.unwrap_or_else(|| {
            (gl.drawing_buffer_width() as u32, gl.drawing_buffer_height() as u32)
        });

        match self.scissor_region {
            Region2D::Area((x, y), width, height) => {
                let (gl, state) = unsafe { context.unpack_mut() };

                state.set_scissor_test_enabled(true).apply(gl).unwrap();
                state.set_scissor_rect((x as i32, y as i32, width, height)).apply(gl).unwrap();
            },
            Region2D::Fill => {
                state.set_scissor_test_enabled(false).apply(gl).unwrap();
            }
        }

        let connection = context.connection_mut();

        self.primitive_assembly.face_culling.apply(connection);
        self.primitive_assembly.winding_order.apply(connection);

        DepthTest::apply(&self.depth_test, connection);
        StencilTest::apply(&self.stencil_test, connection);
        Blending::apply(&self.blending, connection);

        self.line_width.apply(connection);
        self.viewport.apply(connection, framebuffer_dimensions);

        self.task.progress(&mut PipelineTaskContext {
            connection: context.connection_mut()
        })
    }
}

pub struct ActiveGraphicsPipeline<Il, R> {
    context_id: usize,
    topology: Topology,
    pipeline_task_id: usize,
    _input_attribute_layout_marker: marker::PhantomData<Il>,
    _resources_marker: marker::PhantomData<R>,
}

impl<Il, R> ActiveGraphicsPipeline<Il, R>
    where
        Il: InputAttributeLayout,
        R: Resources,
{
    /// Creates a [DrawCommand] that will execute this [ActiveGraphicsPipeline] on the
    /// [vertex_input_stream] with the [resources] bound to the pipeline's resource slots.
    ///
    /// # Panic
    ///
    /// - Panics when [vertex_input_stream] uses a [VertexArray] that belongs to a different context
    ///   than this [ActiveGraphicsPipeline].
    /// - Panics when [resources] specifies a resource that belongs to a different context than this
    ///   [ActiveGraphicsPipeline].
    pub fn draw_command<V>(&self, vertex_input_stream: &V, resources: &R) -> DrawCommand<R::Bindings>
        where
            V: VertexInputStreamDescription<Layout = Il>,
            R: Resources
    {
        DrawCommand {
            pipeline_task_id: self.pipeline_task_id,
            vertex_input_stream_descriptor: vertex_input_stream.descriptor(),
            topology: self.topology,
            binding_group: resources.encode_bind_group(&mut BindGroupEncodingContext::new(self.context_id)).into_descriptors()
        }
    }
}

pub struct DrawCommand<B> {
    pipeline_task_id: usize,
    vertex_input_stream_descriptor: VertexInputStreamDescriptor,
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
            self.vertex_input_stream_descriptor.vertex_array_data.id().unwrap().with_value_unchecked(|vao| {
                state.set_bound_vertex_array(Some(vao)).apply(gl).unwrap();
            })
        }

        for descriptor in self.binding_group.borrow().iter() {
            descriptor.bind(context.connection);
        }

        let (gl, _) = unsafe { context.connection.unpack_mut() };


        if let Some(format) = self.vertex_input_stream_descriptor.index_format_kind() {
            let VertexInputStreamDescriptor {
                offset,
                count,
                instance_count,
                ..
            } = self.vertex_input_stream_descriptor;

            let offset = offset as u32 * format.size_in_bytes();

            if instance_count == 1 {
                gl.draw_elements_with_i32(self.topology.id(), count as i32, format.id(), offset as i32);
            } else {
                gl.draw_elements_instanced_with_i32(self.topology.id(), count as i32, format.id(), offset as i32, instance_count as i32);
            }
        } else {
            let VertexInputStreamDescriptor {
                offset,
                count,
                instance_count,
                ..
            } = self.vertex_input_stream_descriptor;

            if instance_count == 1 {
                gl.draw_arrays(self.topology.id(), offset as i32, count as i32);
            } else {
                gl.draw_arrays_instanced(self.topology.id(), offset as i32, count as i32, instance_count as i32);
            }
        }

        Progress::Finished(())
    }
}

pub trait BlitColorTarget {
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

pub struct BlitTargetDescriptor {
    internal: BlitTargetDescriptorInternal,
}

enum BlitTargetDescriptorInternal {
    Default,
    FBO { width: u32, height: u32 },
}

pub trait BlitSource {
    type Format: InternalFormat;

    fn descriptor(&self) -> BlitSourceDescriptor;
}

pub struct BlitSourceDescriptor {
    attachment: FramebufferAttachment,
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
            attachment: FramebufferAttachment::from_texture_2d_level(self),
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
            attachment: FramebufferAttachment::from_texture_2d_level(&self.level_ref()),
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
            attachment: FramebufferAttachment::from_texture_2d_array_level_layer(self),
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
            attachment: FramebufferAttachment::from_texture_2d_array_level_layer(
                &self.level_layer_ref(),
            ),
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
            attachment: FramebufferAttachment::from_texture_3d_level_layer(self),
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
            attachment: FramebufferAttachment::from_texture_3d_level_layer(&self.level_layer_ref()),
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
            attachment: FramebufferAttachment::from_texture_cube_level_face(self),
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
            attachment: FramebufferAttachment::from_texture_cube_level_face(&self.level_face_ref()),
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
            attachment: FramebufferAttachment::from_renderbuffer(self),
            region: ((0, 0), self.width(), self.height()),
            context_id: self.data().context_id(),
        }
    }
}

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

pub struct BlitColorCommand {
    region: ((u32, u32), u32, u32),
    source: BlitSourceDescriptor,
}

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

pub struct DefaultRGBBuffer {
    render_pass_id: usize,
}

impl DefaultRGBBuffer {
    pub(crate) fn new(render_pass_id: usize) -> Self {
        DefaultRGBBuffer { render_pass_id }
    }

    pub fn clear_command(&self, clear_value: [f32; 4], region: Region2D) -> ClearFloatCommand {
        ClearFloatCommand {
            render_pass_id: self.render_pass_id,
            buffer_index: 0,
            clear_value,
            region,
        }
    }
}

pub struct DefaultRGBABuffer {
    render_pass_id: usize,
}

impl DefaultRGBABuffer {
    pub(crate) fn new(render_pass_id: usize) -> Self {
        DefaultRGBABuffer { render_pass_id }
    }

    pub fn clear_command(&self, clear_value: [f32; 4], region: Region2D) -> ClearFloatCommand {
        ClearFloatCommand {
            render_pass_id: self.render_pass_id,
            buffer_index: 0,
            clear_value,
            region,
        }
    }
}

pub struct DefaultDepthStencilBuffer {
    render_pass_id: usize,
}

impl DefaultDepthStencilBuffer {
    pub(crate) fn new(render_pass_id: usize) -> Self {
        DefaultDepthStencilBuffer { render_pass_id }
    }

    pub fn clear_command(
        &mut self,
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

    pub fn clear_depth_command(&mut self, depth: f32, region: Region2D) -> ClearDepthCommand {
        ClearDepthCommand {
            render_pass_id: self.render_pass_id,
            depth,
            region,
        }
    }

    pub fn clear_stencil_command(&mut self, stencil: i32, region: Region2D) -> ClearStencilCommand {
        ClearStencilCommand {
            render_pass_id: self.render_pass_id,
            stencil,
            region,
        }
    }
}

pub struct DefaultDepthBuffer {
    render_pass_id: usize,
}

impl DefaultDepthBuffer {
    pub(crate) fn new(render_pass_id: usize) -> Self {
        DefaultDepthBuffer { render_pass_id }
    }

    pub fn clear_command(&mut self, depth: f32, region: Region2D) -> ClearDepthCommand {
        ClearDepthCommand {
            render_pass_id: self.render_pass_id,
            depth,
            region,
        }
    }
}

pub struct DefaultStencilBuffer {
    render_pass_id: usize,
}

impl DefaultStencilBuffer {
    pub(crate) fn new(render_pass_id: usize) -> Self {
        DefaultStencilBuffer { render_pass_id }
    }

    pub fn clear_command(&mut self, stencil: i32, region: Region2D) -> ClearStencilCommand {
        ClearStencilCommand {
            render_pass_id: self.render_pass_id,
            stencil,
            region,
        }
    }
}

pub trait RenderBuffer {
    type Format: InternalFormat;

    fn width(&self) -> u32;

    fn height(&self) -> u32;
}

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

    pub fn clear_command(&mut self, clear_value: [f32; 4], region: Region2D) -> ClearFloatCommand {
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

    pub fn clear_command(
        &mut self,
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

    pub fn clear_command(
        &mut self,
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

    pub fn clear_command(
        &mut self,
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

    pub fn clear_depth_command(&mut self, depth: f32, region: Region2D) -> ClearDepthCommand {
        ClearDepthCommand {
            render_pass_id: self.render_pass_id,
            depth,
            region,
        }
    }

    pub fn clear_stencil_command(&mut self, stencil: i32, region: Region2D) -> ClearStencilCommand {
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

    pub fn clear_command(&mut self, depth: f32, region: Region2D) -> ClearDepthCommand {
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

    pub fn clear_command(&mut self, stencil: i32, region: Region2D) -> ClearStencilCommand {
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
