use std::borrow::Borrow;
use std::pin::Pin;
use std::task::Poll;

use futures::channel::oneshot::Receiver;
use futures::future::Future;
use futures::task::Context;

use web_sys::WebGl2RenderingContext as Gl;

use crate::buffer::{Buffer, IntoBuffer, UsageHint};
use crate::image::format::{RenderbufferFormat, TextureFormat};
use crate::image::renderbuffer::{Renderbuffer, RenderbufferDescriptor};
use crate::image::texture_2d::{Texture2D, Texture2DDescriptor};
use crate::image::texture_2d_array::{Texture2DArray, Texture2DArrayDescriptor};
use crate::image::texture_3d::{Texture3D, Texture3DDescriptor};
use crate::image::texture_cube::{TextureCube, TextureCubeDescriptor};
use crate::image::MaxMipmapLevelsExceeded;
use crate::pipeline::graphics::{
    FragmentShader, GraphicsPipeline, GraphicsPipelineDescriptor, IncompatibleVertexInputLayout,
    IndexBuffer, IndexFormat, ShaderLinkingError, VertexShader,
};
use crate::pipeline::resources::{
    BindGroup, BindableResourceGroup, IncompatibleResources, ResourceSlotIdentifier,
};
use crate::render_pass::{RenderPass, RenderPassContext};
use crate::render_target::RenderTargetDescription;
use crate::runtime::state::{CreateProgramError, DynamicState};
use crate::sampler::{Sampler, SamplerDescriptor, ShadowSampler, ShadowSamplerDescriptor};
use crate::task::GpuTask;

/// Trait implemented by types that can serve as a WebGlitz rendering context.
///
/// The rendering context is the main interface of interaction with the Graphics Processing Unit
/// (GPU). It has 3 main roles:
///
/// 1. Provide information about the abilities of the current GPU connection, see [extensions].
/// 2. Act as a factory for the following WebGlitz objects:
///    - [Buffer]s, see [create_buffer].
///    - [Texture2D]s, see [create_texture_2d].
///    - [Texture2DArray]s, see [create_texture_2d_array].
///    - [Texture3D]s, see [create_texture_3d].
///    - [TextureCube]s, see [create_texture_cube].
///    - [Sampler]s, see [create_sampler].
///    - [ShadowSampler]s, see [create_shadow_sampler].
///    - [Renderbuffer]s, see [create_renderbuffer].
///    - [VertexShader]s, see [create_vertex_shader].
///    - [FragmentShader]s, see [create_fragment_shader].
///    - [GraphicsPipeline]s, see [create_graphics_pipeline].
///    - [VertexArray]s, see [create_vertex_array].
///    - [RenderPass]es, see [create_render_pass].
/// 3. Submission of [GpuTask]s to the GPU with [submit].
pub trait RenderingContext {
    /// Identifier that uniquely identifies this rendering context.
    fn id(&self) -> usize;

    /// Returns information about which extensions are enabled for this rendering context.
    ///
    /// See [Extensions] for details.
    fn extensions(&self) -> &Extensions;

    /// Creates a new group of bindable resources.
    ///
    /// The resulting [BindGroup] may be bound to a pipeline such that all invocations of the
    /// pipeline have access to its resources when the pipeline is executed. See
    /// [GraphicsPipelineTaskBuilder::bind_resources] and
    /// [GraphicsPipelineTaskBuilder::bind_resources_untyped] for details.
    ///
    /// The resources specified for the bind group must implement the [BindableResourceGroup] trait.
    /// See also the [Resources] trait for an automatically derivable implementation of the
    /// [BindableResourceGroup] and its sub-trait [TypedBindableResourceGroup].
    ///
    /// # Example
    ///
    /// This example creates a bind group containg a single uniform buffer. We derive the
    /// [Resources] trait to define a typed bind group layout:
    ///
    /// ```
    /// use web_glitz::buffer::{Buffer, UsageHint};
    /// use web_glitz::std140;
    /// use web_glitz::std140::repr_std140;
    ///
    /// #[derive(web_glitz::derive::Resources)]
    /// struct Resources<'a> {
    ///     #[resource(binding=0, name="SomeUniformBlock")]
    ///     uniform_buffer: &'a Buffer<UniformBlock>,
    /// }
    ///
    /// #[repr_std140]
    /// #[derive(web_glitz::derive::InterfaceBlock)]
    /// struct Uniforms {
    ///     scale: std140::float,
    /// }
    ///
    /// let uniforms = Uniforms {
    ///     scale: std140::float(0.5),
    /// };
    ///
    /// let uniform_buffer = context.create_buffer(uniforms, UsageHint::DynamicDraw);
    ///
    /// let bind_group = context.create_bind_group(Resources {
    ///     uniform_buffer: &uniform_buffer,
    /// });
    /// ```
    fn create_bind_group<T>(&self, resources: T) -> BindGroup<T>
    where
        T: BindableResourceGroup;

    /// Creates a new GPU-accessible memory [Buffer].
    ///
    /// # Examples
    ///
    /// A buffer can store any type that is both [Sized] and [Copy] (although not all types are
    /// usefull in GPU operations). We can for example store an [InterfaceBlock] type (which we
    /// might later use to back a uniform block in a pipeline):
    ///
    /// ```
    /// # #![feature(const_fn)]
    /// # use web_glitz::runtime::RenderingContext;
    /// # fn wrapper<Rc>(context: &Rc) where Rc: RenderingContext {
    /// use web_glitz::buffer::UsageHint;
    /// use web_glitz::std140;
    /// use web_glitz::std140::repr_std140;
    ///
    /// #[repr_std140]
    /// #[derive(web_glitz::derive::InterfaceBlock, Clone, Copy)]
    /// struct Uniforms {
    ///     scale: std140::float,
    /// }
    ///
    /// let uniforms = Uniforms {
    ///     scale: std140::float(0.5),
    /// };
    ///
    /// let uniform_buffer = context.create_buffer(uniforms, UsageHint::DynamicDraw);
    /// # }
    /// ```
    ///
    /// Here `context` is a [RenderingContext]. We use [UsageHint::DynamicDraw] to indicate that we
    /// intend to read this buffer on the GPU and we intend to modify the contents of the buffer
    /// repeatedly (see [UsageHint] for details).
    ///
    /// A buffer can also store an array of any type `T` that is both [Sized] and [Copy], by
    /// initializing a type that implements `Borrow<[T]>`. We can for example store an array of
    /// [Vertex] values (which we might later use to create a [VertexArray]):
    ///
    /// ```
    /// # #![feature(const_fn)]
    /// # use web_glitz::runtime::RenderingContext;
    /// # fn wrapper<Rc>(context: &Rc) where Rc: RenderingContext {
    /// use web_glitz::buffer::{Buffer, UsageHint};
    ///
    /// #[derive(web_glitz::derive::Vertex, Clone, Copy)]
    /// struct Vertex {
    ///     #[vertex_attribute(location = 0, format = "Float2_f32")]
    ///     position: [f32; 2],
    ///     #[vertex_attribute(location = 1, format = "Float3_u8_norm")]
    ///     color: [u8; 3],
    /// }
    ///
    /// let vertex_data = [
    ///     Vertex {
    ///         position: [0.0, 0.5],
    ///         color: [255, 0, 0],
    ///     },
    ///     Vertex {
    ///         position: [-0.5, -0.5],
    ///         color: [0, 255, 0],
    ///     },
    ///     Vertex {
    ///         position: [0.5, -0.5],
    ///         color: [0, 0, 255],
    ///     },
    /// ];
    ///
    /// let vertex_buffer: Buffer<[Vertex]> = context.create_buffer(vertex_data, UsageHint::StaticDraw);
    /// # }
    /// ```
    ///
    /// Note that [create_buffer] takes ownership of the data source (`vertex_data` in the example)
    /// and that the data source must be `'static`. It is however possible to use shared ownership
    /// constructs like [Rc](std::rc::Rc) or [Arc](std::sync::Arc). We use a [UsageHint::StaticDraw]
    /// to once again indiciate that we wish to read this data on the GPU, but this time we don't
    /// intend to modify the data in the buffer later.
    fn create_buffer<D, T>(&self, data: D, usage_hint: UsageHint) -> Buffer<T>
    where
        D: IntoBuffer<T>,
        T: ?Sized;

    /// Creates a new GPU-accessible memory [IndexBuffer].
    ///
    /// # Examples
    ///
    /// An [IndexBuffer] can store a slice of indices that implement [IndexFormat]:
    ///
    /// ```
    /// # #![feature(const_fn)]
    /// # use web_glitz::runtime::RenderingContext;
    /// # use web_glitz::pipeline::graphics::IndexBuffer;
    /// # fn wrapper<Rc>(context: &Rc) where Rc: RenderingContext {
    /// use web_glitz::buffer::UsageHint;
    ///
    /// let index_data: [u16; 4] = [1, 2, 3, 4];
    /// let index_buffer: IndexBuffer<u16> = context.create_index_buffer(index_data, UsageHint::StaticDraw);
    /// # }
    /// ```
    ///
    /// Here `context` is a [RenderingContext]. We use [UsageHint::StaticDraw] to indicate that we
    /// intend to read this buffer on the GPU and we intend to modify the contents of the buffer
    /// only rarely (see [UsageHint] for details).
    ///
    /// Note that [create_buffer] takes ownership of the data source (`index_data` in the example)
    /// and that the data source must be `'static`. It is however possible to use shared ownership
    /// constructs like [Rc](std::rc::Rc) or [Arc](std::sync::Arc).
    fn create_index_buffer<D, T>(&self, data: D, usage_hint: UsageHint) -> IndexBuffer<T>
    where
        D: Borrow<[T]> + 'static,
        T: IndexFormat + 'static;

    /// Creates a new [Renderbuffer].
    ///
    /// # Example
    ///
    /// A renderbuffer is created from a [RenderbufferDescriptor]:
    ///
    /// ```rust
    /// # use web_glitz::runtime::RenderingContext;
    /// # fn wrapper<Rc>(context: &Rc) where Rc: RenderingContext {
    /// use web_glitz::image::format::RGB8;
    /// use web_glitz::image::renderbuffer::RenderbufferDescriptor;
    ///
    /// let renderbuffer = context.create_renderbuffer(&RenderbufferDescriptor {
    ///     format: RGB8,
    ///     width: 256,
    ///     height: 256
    /// });
    /// # }
    /// ```
    ///
    /// Here `context` is a [RenderingContext].
    fn create_renderbuffer<F>(&self, descriptor: &RenderbufferDescriptor<F>) -> Renderbuffer<F>
    where
        F: RenderbufferFormat + 'static;

    /// Creates a new [VertexShader] from source code or returns an error if the source code fails
    /// to compile into a valid vertex shader.
    ///
    /// # Example
    ///
    /// A vertex shader can be created from a source [String] or `&'static str`:
    ///
    /// ```rust
    /// # use web_glitz::runtime::RenderingContext;
    /// # fn wrapper<Rc>(context: &Rc) where Rc: RenderingContext {
    /// let vertex_shader = context.create_vertex_shader("\
    /// #version 300 es
    ///
    /// layout(location=0) in vec2 position;
    /// layout(location=1) in vec3 color;
    ///
    /// out vec3 varying_color;
    ///
    /// void main() {
    ///     varying_color = color;
    ///
    ///     gl_Position = vec4(position, 0, 1);
    /// }
    /// ");
    /// # }
    /// ```
    ///
    /// Here `context` is a [RenderingContext].
    ///
    /// Note that in the example the newline for the first line of the source string is explicitly
    /// escaped with `\`, because the GLSL `#version` directive typically has to appear on the first
    /// line of the shader source.
    ///
    /// You might also store the shader source in a separate file and inline the string during
    /// compilation using Rust's `include_str!` macro:
    ///
    /// ```rust
    /// # use web_glitz::runtime::RenderingContext;
    /// # fn wrapper<Rc>(context: &Rc) where Rc: RenderingContext {
    /// let vertex_shader = context
    ///     .create_vertex_shader(include_str!("../../../examples/0_triangle/src/vertex.glsl"))
    ///     .unwrap();
    /// # }
    /// ```
    fn create_vertex_shader<S>(&self, source: S) -> Result<VertexShader, ShaderCompilationError>
    where
        S: Borrow<str> + 'static;

    /// Creates a new [FragmentShader] from source code or returns an error if the source code fails
    /// to compile into a valid fragment shader.
    ///
    /// # Example
    ///
    /// A fragment shader can be created from a source [String] or `&'static str`:
    ///
    /// ```rust
    /// # use web_glitz::runtime::RenderingContext;
    /// # fn wrapper<Rc>(context: &Rc) where Rc: RenderingContext {
    /// let fragment_shader = context.create_fragment_shader("\
    /// #version 300 es
    /// precision mediump float;
    ///
    /// in vec3 varying_color;
    ///
    /// out vec4 out_color;
    ///
    /// void main() {
    ///     out_color = vec4(varying_color, 1);
    /// }
    /// ");
    /// # }
    /// ```
    ///
    /// Here `context` is a [RenderingContext].
    ///
    /// Note that in the example the newline for the first line of the source string is explicitly
    /// escaped with `\`, because the GLSL `#version` directive typically has to appear on the first
    /// line of the shader source.
    ///
    /// You might also store the shader source in a separate file and inline the string during
    /// compilation using Rust's `include_str!` macro:
    ///
    /// ```rust
    /// # use web_glitz::runtime::RenderingContext;
    /// # fn wrapper<Rc>(context: &Rc) where Rc: RenderingContext {
    /// let fragment_shader = context
    ///     .create_fragment_shader(include_str!("../../../examples/0_triangle/src/fragment.glsl"))
    ///     .unwrap();
    /// # }
    /// ```
    fn create_fragment_shader<S>(
        &self,
        source: S,
    ) -> Result<FragmentShader, ShaderCompilationError>
    where
        S: Borrow<str> + 'static;

    /// Creates a new [GraphicsPipeline] from the given [GraphicsPipelineDescriptor] or returns an
    /// error if no valid pipeline could be created from the descriptor.
    ///
    /// See [GraphicsPipelineDescriptor] and [GraphicsPipelineDescriptorBuilder] for details on
    /// creating a valid descriptor.
    ///
    /// An invalid descriptor will result in a [CreateGraphicsPipelineError]. See the documentation
    /// for the variants of this error for details on the types of errors that may ocurr.
    ///
    /// # Example
    ///
    /// ```
    /// # use web_glitz::runtime::RenderingContext;
    /// # use web_glitz::pipeline::graphics::{VertexShader, FragmentShader, TypedVertexInputLayout};
    /// # use web_glitz::pipeline::resources::TypedResourceBindingsLayout;
    /// # fn wrapper<Rc, MyVertex, MyResources>(
    /// #     context: &Rc,
    /// #     vertex_shader: &VertexShader,
    /// #     fragment_shader: &FragmentShader
    /// # ) where Rc: RenderingContext, MyVertex: TypedVertexInputLayout, MyResources: TypedResourceBindingsLayout {
    /// use web_glitz::pipeline::graphics::{
    ///     GraphicsPipelineDescriptor, PrimitiveAssembly, WindingOrder, CullingMode,
    ///     SlotBindingStrategy, DepthTest
    /// };
    ///
    /// let descriptor = GraphicsPipelineDescriptor::begin()
    ///     .vertex_shader(&vertex_shader)
    ///     .primitive_assembly(PrimitiveAssembly::Triangles {
    ///         winding_order: WindingOrder::CounterClockwise,
    ///         face_culling: CullingMode::None
    ///     })
    ///     .fragment_shader(&fragment_shader)
    ///     .enable_depth_test(DepthTest::default())
    ///     .typed_vertex_attribute_layout::<MyVertex>()
    ///     .typed_resource_bindings_layout::<MyResources>()
    ///     .finish();
    ///
    /// let graphics_pipeline = context.create_graphics_pipeline(&descriptor).unwrap();
    /// # }
    /// ```
    ///
    /// Here `vertex_shader` is a [VertexShader], `fragment_shader` is a [FragmentShader],
    /// `MyVertex` is a type that implements [TypedVertexInputLayout], `MyResources` is a
    /// type that implements [TypedResourceBindingsLayout] and `context` is a [RenderingContext].
    ///
    /// # Panics
    ///
    /// Panics if the [VertexShader] or the [FragmentShader] provided for the pipeline belong to
    /// a different [RenderingContext].
    fn create_graphics_pipeline<V, R, Tf>(
        &self,
        descriptor: &GraphicsPipelineDescriptor<V, R, Tf>,
    ) -> Result<GraphicsPipeline<V, R, Tf>, CreateGraphicsPipelineError>;

    /// Creates a new [RenderPass] that targets the `render_target` and performs the render pass
    /// task produced by `f`.
    ///
    /// The `render_target` must be a [RenderTargetDescription], either the [DefaultRenderTarget]
    /// obtained when the runtime was initialized, or a custom render target (see [RenderTarget] for
    /// details on and examples of creating a valid custom render target). Rendering output buffers
    /// will be allocated in the framebuffer for each of the color images attached to the render
    /// target (if any) and for the depth-stencil image attached to the render target (if any).
    ///
    /// The `f` function must return a [GpuTask] that can be executed in a [RenderPassContext]. This
    /// function will receive a reference to the framebuffer representation associated with the
    /// render target. It may use this reference to construct commands that make up that task, see
    /// [Framebuffer] for details.
    ///
    /// At the start of the render pass, the [LoadOp]s associated with each of the images attached
    /// to the render target will be performed to initialize the framebuffer. Then the task returned
    /// from `f` is executed, which may modify the contents of the framebuffer. Finally, the
    /// [StoreOp]s associated with each of the images attached to the render target will be
    /// performed to store the (modified) contents of the framebuffer back to these images.
    ///
    /// # Example
    ///
    /// ```
    /// # use web_glitz::render_pass::DefaultRGBBuffer;
    /// # use web_glitz::render_target::DefaultRenderTarget;
    /// # use web_glitz::runtime::RenderingContext;
    /// # use web_glitz::vertex::{Vertex, VertexArray};
    /// # use web_glitz::buffer::UsageHint;
    /// # use web_glitz::pipeline::graphics::GraphicsPipeline;
    /// # use web_glitz::pipeline::resources::Resources;
    /// # fn wrapper<Rc, V>(
    /// #     context: &Rc,
    /// #     mut default_render_target: DefaultRenderTarget<DefaultRGBBuffer, ()>,
    /// #     vertex_stream: VertexArray<V>,
    /// #     graphics_pipeline: GraphicsPipeline<V, (), ()>
    /// # )
    /// # where
    /// #     Rc: RenderingContext,
    /// #     V: Vertex,
    /// # {
    /// # let resources = ();
    /// let render_pass = context.create_render_pass(&mut default_render_target, |framebuffer| {
    ///     framebuffer.pipeline_task(&graphics_pipeline, |active_pipeline| {
    ///         active_pipeline.draw_command(&vertex_stream, resources)
    ///     })
    /// });
    /// # }
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if any of the images attached to the render target belong to a different context.
    ///
    /// Panics if the render pass context ID associated with the task returned from `f` does not
    /// match the ID generated for this render pass (the task returned from `f` must not contain
    /// commands that were created for a different render pass).
    fn create_render_pass<R, F, T>(&self, render_target: R, f: F) -> RenderPass<T>
    where
        R: RenderTargetDescription,
        F: FnOnce(&R::Framebuffer) -> T,
        T: GpuTask<RenderPassContext>;

    /// Creates a new [Texture2D] from the given `descriptor`, or returns an error if the descriptor
    /// was invalid.
    ///
    /// See [Texture2DDescriptor] for details on specifying a valid descriptor.
    ///
    /// Returns an error if the descriptor specifies more mipmap levels than the texture's
    /// dimensions support.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use web_glitz::runtime::RenderingContext;
    /// # fn wrapper<Rc>(context: &Rc) where Rc: RenderingContext + Clone + 'static {
    /// use web_glitz::image::MipmapLevels;
    /// use web_glitz::image::format::RGB8;
    /// use web_glitz::image::texture_2d::Texture2DDescriptor;
    ///
    /// let texture = context.create_texture_2d(&Texture2DDescriptor {
    ///     format: RGB8,
    ///     width: 256,
    ///     height: 256,
    ///     levels: MipmapLevels::Complete
    /// }).unwrap();
    /// # }
    /// ```
    fn create_texture_2d<F>(
        &self,
        descriptor: &Texture2DDescriptor<F>,
    ) -> Result<Texture2D<F>, MaxMipmapLevelsExceeded>
    where
        F: TextureFormat + 'static;

    /// Creates a new [Texture2DArray] from the given `descriptor`, or returns an error if the
    /// descriptor was invalid.
    ///
    /// See [Texture2DArrayDescriptor] for details on specifying a valid descriptor.
    ///
    /// Returns an error if the descriptor specifies more mipmap levels than the texture's
    /// dimensions support.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use web_glitz::runtime::RenderingContext;
    /// # fn wrapper<Rc>(context: &Rc) where Rc: RenderingContext + Clone + 'static {
    /// use web_glitz::image::MipmapLevels;
    /// use web_glitz::image::format::RGB8;
    /// use web_glitz::image::texture_2d_array::Texture2DArrayDescriptor;
    ///
    /// let texture = context.create_texture_2d_array(&Texture2DArrayDescriptor {
    ///     format: RGB8,
    ///     width: 256,
    ///     height: 256,
    ///     depth: 16,
    ///     levels: MipmapLevels::Complete
    /// }).unwrap();
    /// # }
    /// ```
    fn create_texture_2d_array<F>(
        &self,
        descriptor: &Texture2DArrayDescriptor<F>,
    ) -> Result<Texture2DArray<F>, MaxMipmapLevelsExceeded>
    where
        F: TextureFormat + 'static;

    /// Creates a new [Texture3D] from the given `descriptor`, or returns an error if the descriptor
    /// was invalid.
    ///
    /// See [Texture3DDescriptor] for details on specifying a valid descriptor.
    ///
    /// Returns an error if the descriptor specifies more mipmap levels than the texture's
    /// dimensions support.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use web_glitz::runtime::RenderingContext;
    /// # fn wrapper<Rc>(context: &Rc) where Rc: RenderingContext + Clone + 'static {
    /// use web_glitz::image::MipmapLevels;
    /// use web_glitz::image::format::RGB8;
    /// use web_glitz::image::texture_3d::Texture3DDescriptor;
    ///
    /// let texture = context.create_texture_3d(&Texture3DDescriptor {
    ///     format: RGB8,
    ///     width: 256,
    ///     height: 256,
    ///     depth: 256,
    ///     levels: MipmapLevels::Complete
    /// }).unwrap();
    /// # }
    /// ```
    fn create_texture_3d<F>(
        &self,
        descriptor: &Texture3DDescriptor<F>,
    ) -> Result<Texture3D<F>, MaxMipmapLevelsExceeded>
    where
        F: TextureFormat + 'static;

    /// Creates a new [TextureCube] from the given `descriptor`, or returns an error if the
    /// descriptor was invalid.
    ///
    /// See [TextureCubeDescriptor] for details on specifying a valid descriptor.
    ///
    /// Returns an error if the descriptor specifies more mipmap levels than the texture's
    /// dimensions support.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use web_glitz::runtime::RenderingContext;
    /// # fn wrapper<Rc>(context: &Rc) where Rc: RenderingContext + Clone + 'static {
    /// use web_glitz::image::MipmapLevels;
    /// use web_glitz::image::format::RGB8;
    /// use web_glitz::image::texture_cube::TextureCubeDescriptor;
    ///
    /// let texture = context.create_texture_cube(&TextureCubeDescriptor {
    ///     format: RGB8,
    ///     width: 256,
    ///     height: 256,
    ///     levels: MipmapLevels::Complete
    /// }).unwrap();
    /// # }
    /// ```
    fn create_texture_cube<F>(
        &self,
        descriptor: &TextureCubeDescriptor<F>,
    ) -> Result<TextureCube<F>, MaxMipmapLevelsExceeded>
    where
        F: TextureFormat + 'static;

    /// Creates a new [Sampler] from the given `descriptor`.
    ///
    /// See [SamplerDescriptor] for details on specifying a descriptor.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use web_glitz::runtime::RenderingContext;
    /// # fn wrapper<Rc>(context: &Rc) where Rc: RenderingContext + Clone + 'static {
    /// use web_glitz::sampler::{
    ///     SamplerDescriptor, MinificationFilter, MagnificationFilter, LODRange, Wrap
    /// };
    ///
    /// let sampler = context.create_sampler(&SamplerDescriptor {
    ///     minification_filter: MinificationFilter::NearestMipmapLinear,
    ///     magnification_filter: MagnificationFilter::Linear,
    ///     lod_range: LODRange::default(),
    ///     wrap_s: Wrap::Repeat,
    ///     wrap_t: Wrap::Repeat,
    ///     wrap_r: Wrap::Repeat,
    /// });
    /// # }
    /// ```
    fn create_sampler(&self, descriptor: &SamplerDescriptor) -> Sampler;

    /// Creates a new [ShadowSampler] from the given `descriptor`.
    ///
    /// See [ShadowSamplerDescriptor] for details on specifying a descriptor.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use web_glitz::runtime::RenderingContext;
    /// # fn wrapper<Rc>(context: &Rc) where Rc: RenderingContext + Clone + 'static {
    /// use web_glitz::sampler::{ShadowSamplerDescriptor, CompareFunction, Wrap};
    ///
    /// let shadow_sampler = context.create_shadow_sampler(&ShadowSamplerDescriptor {
    ///     compare: CompareFunction::LessOrEqual,
    ///     wrap_s: Wrap::Repeat,
    ///     wrap_t: Wrap::Repeat,
    ///     wrap_r: Wrap::Repeat,
    /// });
    /// # }
    /// ```
    fn create_shadow_sampler(&self, descriptor: &ShadowSamplerDescriptor) -> ShadowSampler;

    /// Submits the `task` for execution and returns the output of the task as a [Future] result.
    ///
    /// When the task finishes ([GpuTask::progress] returns [Progress::Finished]), the [Future]
    /// will resolve with the task's output value (see [GpuTask::Output]).
    ///
    /// No guarantees are given about the execution order of tasks that have been submitted
    /// separately: they may be initiated out of order, and progress on separately submitted tasks
    /// may be made concurrently. If you wish to ensure that certain tasks are executed in order,
    /// use a "sequence" combinator, see the module documentation for [web_glitz::task] for details.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use web_glitz::runtime::{Connection, RenderingContext};
    /// # use web_glitz::task::GpuTask;
    /// # fn wrapper<Rc, T>(context: &Rc, task: T) where Rc: RenderingContext, T: GpuTask<Connection> + 'static {
    /// use futures::future::FutureExt;
    ///
    /// use wasm_bindgen_futures::future_to_promise;
    ///
    /// let future_output = context.submit(task);
    ///
    /// future_to_promise(future_output.inspect(|output| {
    ///     // Do something with the output...
    /// }));
    /// # }
    /// ```
    ///
    /// In this example we use [wasm_bindgen_futures::future_to_promise] to run the future returned
    /// by [submit] in a WASM web context and use the `inspect` combinator to do something with the
    /// output value when the future resolves.
    ///
    /// Note that in many cases the output of a task is not relevant (the output is often just the
    /// empty tuple `()`). In this case it is not necessary to ever poll the future for the task to
    /// be executed: any task that is submitted will be executed, regardless of whether the future
    /// returned by [submit] is ever polled or just simply dropped immediately.
    ///
    /// # Panics
    ///
    /// Panics if the task belongs to a different [RenderingContext] ([GpuTask::context_id] returns
    /// a value that is not compatible with this current context).
    fn submit<T>(&self, task: T) -> Execution<T::Output>
    where
        T: GpuTask<Connection> + 'static;
}

/// Encapsulates the [ExtensionState]s for each of the available context extensions for a
/// [RenderingContext].
///
/// Returned from [RenderingContext::extensions].
#[derive(Clone)]
pub struct Extensions {
    texture_float_linear: ExtensionState,
}

impl Extensions {
    /// Returns [ExtensionState::Enabled] if the "texture_float_linear" linear extension is
    /// enabled, or [ExtensionState::Disabled] otherwise.
    pub fn texture_float_linear(&self) -> ExtensionState {
        self.texture_float_linear
    }
}

impl Default for Extensions {
    fn default() -> Self {
        Extensions {
            texture_float_linear: ExtensionState::Disabled,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ExtensionState {
    Enabled,
    Disabled,
}

impl ExtensionState {
    pub fn is_enabled(&self) -> bool {
        *self == ExtensionState::Enabled
    }

    pub fn is_disabled(&self) -> bool {
        *self == ExtensionState::Disabled
    }
}

#[derive(PartialEq, Debug)]
pub struct ShaderCompilationError(pub(crate) String);

/// Error returned from [RenderingContext::create_graphics_pipeline].
#[derive(Debug)]
pub enum CreateGraphicsPipelineError {
    /// Variant that is returned when the programmable shader stages fail to link into a valid
    /// program.
    ///
    /// Typically the result of a prior stage's outputs not matching the succeeding stage's inputs.
    ShaderLinkingError(ShaderLinkingError),

    /// Variant that is returned when any of the programmable shader stages define an uniform type
    /// that is not supported by WebGlitz.
    ///
    /// Note that WebGlitz does not support non-opaque uniform types (such as `float`, `vec4`,
    /// `mat4`) outside of uniform blocks, only opaque (texture/shader types) are supported. All
    /// basic non-opaque uniform slots must be declared as part of a uniform block.
    UnsupportedUniformType(ResourceSlotIdentifier, &'static str),

    /// Variant that is returned when the input attribute layout declared for the pipeline (see
    /// [GraphicsPipelineBuilder::vertex_input_layout]) does not match the actual input attribute
    /// layout as defined by the shader code.
    IncompatibleInputAttributeLayout(IncompatibleVertexInputLayout),

    /// Variant that is returned when the resource layout declared for the pipeline (see
    /// [GraphicsPipelineBuilder::resource_layout]) does not match the resource layout as defined by
    /// the shader code.
    IncompatibleResources(IncompatibleResources),

    TransformFeedbackTypeMismatch(String),
}

impl From<CreateProgramError> for CreateGraphicsPipelineError {
    fn from(err: CreateProgramError) -> Self {
        match err {
            CreateProgramError::ShaderLinkingError(error) => {
                CreateGraphicsPipelineError::ShaderLinkingError(ShaderLinkingError { error })
            }
            CreateProgramError::UnsupportedUniformType(identifier, error) => {
                CreateGraphicsPipelineError::UnsupportedUniformType(identifier, error)
            }
        }
    }
}

impl From<ShaderLinkingError> for CreateGraphicsPipelineError {
    fn from(error: ShaderLinkingError) -> Self {
        CreateGraphicsPipelineError::ShaderLinkingError(error)
    }
}

impl From<IncompatibleVertexInputLayout> for CreateGraphicsPipelineError {
    fn from(error: IncompatibleVertexInputLayout) -> Self {
        CreateGraphicsPipelineError::IncompatibleInputAttributeLayout(error)
    }
}

impl From<IncompatibleResources> for CreateGraphicsPipelineError {
    fn from(error: IncompatibleResources) -> Self {
        CreateGraphicsPipelineError::IncompatibleResources(error)
    }
}

/// Returned from [RenderingContext::submit], future result of the [GpuTask] that was submitted
/// that will resolve when the task finishes executing.
///
/// See [RenderingContext::submit].
pub enum Execution<O> {
    /// Variant returned when the task finished immediately upon submission.
    Ready(Option<O>),

    /// Variant returned when the task did not finish immediately upon submission.
    Pending(Receiver<O>),
}

impl<O> Future for Execution<O> {
    type Output = O;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<O> {
        match unsafe { self.get_unchecked_mut() } {
            Execution::Ready(ref mut output) => {
                let output = output
                    .take()
                    .expect("Cannot poll Execution more than once after its ready");

                Poll::Ready(output)
            }
            Execution::Pending(ref mut recv) => match Pin::new(recv).poll(cx) {
                Poll::Ready(Ok(output)) => Poll::Ready(output),
                Poll::Pending => Poll::Pending,
                _ => unreachable!(),
            },
        }
    }
}

impl<T> From<T> for Execution<T> {
    fn from(value: T) -> Self {
        Execution::Ready(Some(value))
    }
}

impl<T> From<Receiver<T>> for Execution<T> {
    fn from(recv: Receiver<T>) -> Self {
        Execution::Pending(recv)
    }
}

/// Encapsulates the raw [WebGl2RenderingContext] and its current state.
///
/// Can be unpacked into the raw [WebGl2RenderingContext] and its current state, see [unpack] and
/// [unpack_mut].
///
/// Acts as the base execution context for [GpuTask]s that can be submitted to
/// [RenderingContext::submit]. You may create a custom `GpuTask<Connection>`, which will receive
/// a mutable reference to the current connection by the task executed associated with the context
/// (see [GpuTask::progress]). This is WebGlitz's primary escape hatch for dropping down to a bare
/// [WebGl2RenderingContext] for functionality that either is not supported by WebGlitz's or comes
/// with unacceptable overhead for your use-case.
pub struct Connection {
    context_id: usize,
    gl: Gl,
    state: DynamicState,
}

impl Connection {
    /// Creates a new connection from a raw [WebGl2RenderingContext] and its current `state` for a
    /// context with the given `context_id`.
    ///
    /// The `context_id` should be unique (no other [RenderingContext] with that ID exists).
    ///
    /// # Unsafe
    ///
    /// The `state` must accurately reflect the current state of the [WebGl2RenderingContext].
    pub unsafe fn new(context_id: usize, gl: Gl, state: DynamicState) -> Self {
        Connection {
            context_id,
            gl,
            state,
        }
    }

    /// The unique identifier for the [RenderingContext] with which this [Connection] is associated.
    pub fn context_id(&self) -> usize {
        self.context_id
    }

    /// Unpacks the connection into a reference to the raw [WebGl2RenderingContext] and its
    /// [DynamicState].
    ///
    /// # Unsafe
    ///
    /// The [WebGl2RenderingContext]'s state must remain unchanged or must be restored to its prior
    /// state before progress can be made on another [GpuTask] (this typically means before your
    /// [GpuTask::progress] implementation returns).
    pub unsafe fn unpack(&self) -> (&Gl, &DynamicState) {
        (&self.gl, &self.state)
    }

    /// Unpacks the connection into a mutable reference to the raw [WebGl2RenderingContext] and its
    /// [DynamicState].
    ///
    /// # Unsafe
    ///
    /// If the [WebGl2RenderingContext]'s state is changed, then the `state` must also be updated
    /// to accurately reflect that changed state, before progress can be made on another [GpuTask]
    /// (this typically means before your [GpuTask::progress] implementation returns).
    ///
    /// It is advisable to first update the state on the [DynamicState]. This will return a
    /// [ContextUpdate] which can then be applied to the [WebGl2RenderingContext]:
    ///
    /// ```
    /// # use web_glitz::runtime::Connection;
    /// # fn wrapper(connection: &mut Connection) {
    /// use web_glitz::runtime::state::ContextUpdate;
    ///
    /// unsafe {
    ///     let (gl, state) = connection.unpack_mut();
    ///
    ///     let context_update = state.set_clear_color([0.0, 1.0, 0.0, 1.0]);
    ///
    ///     context_update.apply(gl);
    /// }
    /// # }
    /// ```
    pub unsafe fn unpack_mut(&mut self) -> (&mut Gl, &mut DynamicState) {
        (&mut self.gl, &mut self.state)
    }
}
