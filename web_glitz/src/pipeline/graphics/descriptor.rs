use std::marker;
use std::sync::Arc;

use crate::image::Region2D;
use crate::pipeline::graphics::shader::{FragmentShaderData, VertexShaderData};
use crate::pipeline::graphics::{
    Blending, DepthTest, FragmentShader, PrimitiveAssembly, StencilTest,
    TransformFeedbackLayoutDescriptor, TypedTransformFeedbackLayout, TypedVertexInputLayout,
    Untyped, VertexInputLayoutDescriptor, VertexShader, Viewport,
};
use crate::pipeline::resources::Resources;

/// Enumerates the strategies available to map pipeline resource slots to binding indices.
///
/// See [GraphicsPipelineDescriptorBuilder::resource_layout].
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum SlotBindingStrategy {
    Check,
    Update,
}

/// Provides a description from which a [GraphicsPipeline] may be created.
///
/// See [RenderingContext::create_graphics_pipeline] for details on how a
/// [GraphicsPipelineDescriptor] may be used to create a [GraphicsPipeline].
///
/// A [GraphicsPipelineDescriptor] is constructed through a [GraphicsPipelineDescriptorBuilder]. You
/// may begin building a new [GraphicsPipelineDescriptor] with [GraphicsPipelineDescriptor::begin].
/// See [GraphicsPipelineDescriptorBuilder] for details on how a [GraphicsPipelineDescriptor] is
/// specified.
pub struct GraphicsPipelineDescriptor<V, R, Tf> {
    _vertex_attribute_layout: marker::PhantomData<V>,
    _resource_layout: marker::PhantomData<R>,
    _transform_feedback: marker::PhantomData<Tf>,
    pub(crate) vertex_shader_data: Arc<VertexShaderData>,
    pub(crate) fragment_shader_data: Arc<FragmentShaderData>,
    pub(crate) vertex_attribute_layout: VertexInputLayoutDescriptor,
    pub(crate) transform_feedback_layout: Option<TransformFeedbackLayoutDescriptor>,
    pub(crate) primitive_assembly: PrimitiveAssembly,
    pub(crate) depth_test: Option<DepthTest>,
    pub(crate) stencil_test: Option<StencilTest>,
    pub(crate) scissor_region: Region2D,
    pub(crate) blending: Option<Blending>,
    pub(crate) viewport: Viewport,
    pub(crate) binding_strategy: SlotBindingStrategy,
}

impl GraphicsPipelineDescriptor<(), (), ()> {
    /// Begins building a new [GraphicsPipelineDescriptor].
    ///
    /// Returns a [GraphicsPipelineDescriptorBuilder], see [GraphicsPipelineDescriptorBuilder] for
    /// details on it is used to construct a buildable [GraphicsPipelineDescriptor].
    pub fn begin() -> GraphicsPipelineDescriptorBuilder<(), (), (), (), (), ()> {
        GraphicsPipelineDescriptorBuilder {
            _vertex_shader: marker::PhantomData,
            _primitive_assembly: marker::PhantomData,
            _fragment_shader: marker::PhantomData,
            _transform_feedback: marker::PhantomData,
            _vertex_attribute_layout: marker::PhantomData,
            _resource_layout: marker::PhantomData,
            vertex_shader: None,
            fragment_shader: None,
            vertex_input_layout: ().into(),
            transform_feedback_layout: None,
            primitive_assembly: None,
            depth_test: None,
            stencil_test: None,
            scissor_region: Region2D::Fill,
            blending: None,
            viewport: Viewport::Auto,
            binding_strategy: SlotBindingStrategy::Check,
        }
    }
}

/// Type checked builder for a [GraphicsPipelineDescriptor].
///
/// The following behaviours of the graphics pipeline can be configured:
///
/// - The vertex shader stage can be specified with [vertex_shader]. See [VertexShader] for details
///   on the vertex shader stage. Must be set explicitly, has no default value.
/// - The primitive assembly shader stage can be specified with [primitive_assembly]. See
///   [PrimitiveAssembly] on the primitive assembly stage. Must be set explicitly, has no default
///   value.
/// - The fragment shader stage can be specified with [fragment_shader]. See [FragmentShader] for
///   details on the fragment shader stage. Must be set explicitly, has no default value.
/// - The depth test can be enabled with [enable_depth_test]. See [DepthTest] for details on the
///   depth test. Does not need to be set explicitly, will default to disabled.
/// - The stencil test can be enabled with [enable_stencil_test]. See [StencilTest] for details on
///   the stencil test. Does not need to be set explicitly, will default to disabled.
/// - A scissor region may be specified with [scissor_region]. Any fragments outside the scissor
///   region will be discarded before the fragment processing stages. If not set explicitly, the
///   the scissor region defaults to [Region2D::Fill] and will match the size of the framebuffer
///   that is the current draw target.
/// - Blending can be enabled with [enable_blending]. See [Blending] for details on blending. Does
///   not need to be set explicitly, will default to disabled.
/// - The viewport may be specified with [viewport]. See [Viewport] for details on the viewport. If
///   no viewport is explicitly specified, then the viewport will default to [Viewport::Auto].
///
/// Additionally, static type information about the interface of the graphics pipeline with external
/// data sources must be specified:
///
/// - The vertex input layout type must be specified with [vertex_input_layout]. This must be a
///   type that implements [AttributeSlotLayoutCompatible]. Note that
///   [AttributeSlotLayoutCompatible] is implemented for any type that implements [Vertex] and any
///   tuple of types that implement [Vertex] (e.g. `(Vertex1, Vertex2, Vertex3)`).
/// - The resource layout must be specified with [resource_layout]. This must be a type that
///   implements [Resources].
///
/// When the descriptor that results from this builder is used to create a graphics pipeline (see
/// [RenderingContext::create_graphics_pipeline]), the vertex input layout and resource layout
/// associated with these types is checked against the actual vertex input layout and resource
/// layout defined by the pipeline's programmable shader stages (by reflecting on the code for these
/// shader stages). If the layout defined by these types and the actual layout defined by the
/// shader stages do not match, then pipeline creation will fail and an error is returned instead.
/// If the layouts do match, then vertex input streams and resources of the specified types may be
/// safely bound to the pipeline when creating draw commands (see [PipelineTask::draw_command])
/// without additional runtime checks.
///
/// Finally, the [GraphicsPipelineDescriptor] may be finalized by calling [finish]. [finish] may
/// only be called if at least the following have been explicitly specified:
///
/// - The vertex shader with [vertex_shader].
/// - The primitive assembly algorithm with [primitive_assembly].
/// - The fragment shader with [fragment_shader].
/// - The vertex input layout type with [vertex_input_layout].
/// - The resource layout type with [resource_layout].
///
/// # Example
///
/// ```
/// # use web_glitz::pipeline::graphics::{VertexShader, FragmentShader};
/// # use web_glitz::pipeline::resources::Resources;
/// # use web_glitz::vertex::Vertex;
/// # fn wrapper<MyVertex: Vertex, MyResources: Resources>(
/// #     vertex_shader: VertexShader,
/// #     fragment_shader: FragmentShader
/// # ) {
/// use web_glitz::pipeline::graphics::{
///     GraphicsPipelineDescriptor, PrimitiveAssembly, WindingOrder, CullingMode,
///     SlotBindingStrategy, DepthTest
/// };
///
/// let graphics_pipeline_descriptor = GraphicsPipelineDescriptor::begin()
///     .vertex_shader(&vertex_shader)
///     .primitive_assembly(PrimitiveAssembly::Triangles {
///         winding_order: WindingOrder::CounterClockwise,
///         face_culling: CullingMode::None
///     })
///     .fragment_shader(&fragment_shader)
///     .enable_depth_test(DepthTest::default())
///     .typed_vertex_attribute_layout::<MyVertex>()
///     .resource_layout::<MyResources>(SlotBindingStrategy::Update)
///     .finish();
/// # }
/// ```
///
/// Here `vertex_shader` is a [VertexShader], `fragment_shader` is a [FragmentShader], `MyVertex` is
/// a type that implements [AttributeSlotLayoutCompatible] and `MyResources` is a type that
/// implements [Resources].
pub struct GraphicsPipelineDescriptorBuilder<Vs, Pa, Fs, V, R, Tf> {
    _vertex_shader: marker::PhantomData<Vs>,
    _primitive_assembly: marker::PhantomData<Pa>,
    _fragment_shader: marker::PhantomData<Fs>,
    _transform_feedback: marker::PhantomData<Tf>,
    _vertex_attribute_layout: marker::PhantomData<V>,
    _resource_layout: marker::PhantomData<R>,
    vertex_shader: Option<Arc<VertexShaderData>>,
    vertex_input_layout: VertexInputLayoutDescriptor,
    transform_feedback_layout: Option<TransformFeedbackLayoutDescriptor>,
    fragment_shader: Option<Arc<FragmentShaderData>>,
    primitive_assembly: Option<PrimitiveAssembly>,
    depth_test: Option<DepthTest>,
    stencil_test: Option<StencilTest>,
    scissor_region: Region2D,
    blending: Option<Blending>,
    viewport: Viewport,
    binding_strategy: SlotBindingStrategy,
}

impl<Vs, Pa, Fs, V, R, Tf> GraphicsPipelineDescriptorBuilder<Vs, Pa, Fs, V, R, Tf> {
    /// Specifies the [VertexShader] that any graphics pipeline created using the descriptor will
    /// use.
    ///
    /// See [VertexShader] for details on vertex shaders.
    pub fn vertex_shader(
        self,
        vertex_shader: &VertexShader,
    ) -> GraphicsPipelineDescriptorBuilder<VertexShader, Pa, Fs, V, R, Tf> {
        GraphicsPipelineDescriptorBuilder {
            _vertex_shader: marker::PhantomData,
            _primitive_assembly: marker::PhantomData,
            _fragment_shader: marker::PhantomData,
            _transform_feedback: marker::PhantomData,
            _vertex_attribute_layout: marker::PhantomData,
            _resource_layout: marker::PhantomData,
            vertex_shader: Some(vertex_shader.data().clone()),
            vertex_input_layout: self.vertex_input_layout,
            transform_feedback_layout: self.transform_feedback_layout,
            primitive_assembly: self.primitive_assembly,
            fragment_shader: self.fragment_shader,
            depth_test: self.depth_test,
            stencil_test: self.stencil_test,
            scissor_region: self.scissor_region,
            blending: self.blending,
            viewport: self.viewport,
            binding_strategy: self.binding_strategy,
        }
    }

    /// Specifies the [PrimitiveAssembly] algorithm that any graphics pipeline created using the
    /// descriptor will use.
    ///
    /// See [PrimitiveAssembly] for details on primitive assembly.
    pub fn primitive_assembly(
        self,
        primitive_assembly: PrimitiveAssembly,
    ) -> GraphicsPipelineDescriptorBuilder<Vs, PrimitiveAssembly, Fs, V, R, Tf> {
        GraphicsPipelineDescriptorBuilder {
            _vertex_shader: marker::PhantomData,
            _primitive_assembly: marker::PhantomData,
            _fragment_shader: marker::PhantomData,
            _transform_feedback: marker::PhantomData,
            _vertex_attribute_layout: marker::PhantomData,
            _resource_layout: marker::PhantomData,
            vertex_shader: self.vertex_shader,
            vertex_input_layout: self.vertex_input_layout,
            transform_feedback_layout: self.transform_feedback_layout,
            primitive_assembly: Some(primitive_assembly),
            fragment_shader: self.fragment_shader,
            depth_test: self.depth_test,
            stencil_test: self.stencil_test,
            scissor_region: self.scissor_region,
            blending: self.blending,
            viewport: self.viewport,
            binding_strategy: self.binding_strategy,
        }
    }

    /// Specifies the [FragmentShader] that any graphics pipeline created using the descriptor will
    /// use.
    ///
    /// See [FragmentShader] for details on fragment shaders.
    pub fn fragment_shader(
        self,
        fragment_shader: &FragmentShader,
    ) -> GraphicsPipelineDescriptorBuilder<Vs, Pa, FragmentShader, V, R, Tf> {
        GraphicsPipelineDescriptorBuilder {
            _vertex_shader: marker::PhantomData,
            _primitive_assembly: marker::PhantomData,
            _fragment_shader: marker::PhantomData,
            _transform_feedback: marker::PhantomData,
            _vertex_attribute_layout: marker::PhantomData,
            _resource_layout: marker::PhantomData,
            vertex_shader: self.vertex_shader,
            vertex_input_layout: self.vertex_input_layout,
            transform_feedback_layout: self.transform_feedback_layout,
            primitive_assembly: self.primitive_assembly,
            fragment_shader: Some(fragment_shader.data().clone()),
            depth_test: self.depth_test,
            stencil_test: self.stencil_test,
            scissor_region: self.scissor_region,
            blending: self.blending,
            viewport: self.viewport,
            binding_strategy: self.binding_strategy,
        }
    }

    /// Specifies a [TypedVertexAttributeLayout] type that determines the vertex input layout for
    /// any graphics pipeline created from the descriptor.
    ///
    /// When the descriptor that results from this builder is used to create a graphics pipeline
    /// (see [RenderingContext::create_graphics_pipeline]), the vertex input layout associated with
    /// this type is checked against the actual vertex input layout defined by the pipeline's
    /// programmable shader stages (by reflecting on the code for these shader stages). If the
    /// layout defined by this type and the actual layout defined by the shader stages do not match,
    /// then pipeline creation will fail and an error is returned instead. If the layouts do match,
    /// then vertex input streams that use the type as their vertex type may be safely bound to the
    /// pipeline when creating draw commands without additional runtime checks (see
    /// [PipelineTask::draw_command]).
    ///
    /// Note that [TypedVertexAttributeLayout] is implemented for any type that implements [Vertex]
    /// and any tuple of types that implement [Vertex] (e.g. `(Vertex1, Vertex2)` where both
    /// `Vertex1` and `Vertex2` are types that implement [Vertex]).
    pub fn typed_vertex_attribute_layout<T>(
        self,
    ) -> GraphicsPipelineDescriptorBuilder<Vs, Pa, Fs, T, R, Tf>
    where
        T: TypedVertexInputLayout,
    {
        GraphicsPipelineDescriptorBuilder {
            _vertex_shader: marker::PhantomData,
            _primitive_assembly: marker::PhantomData,
            _fragment_shader: marker::PhantomData,
            _transform_feedback: marker::PhantomData,
            _vertex_attribute_layout: marker::PhantomData,
            _resource_layout: marker::PhantomData,
            vertex_shader: self.vertex_shader,
            vertex_input_layout: T::LAYOUT_DESCRIPTION.into(),
            transform_feedback_layout: self.transform_feedback_layout,
            primitive_assembly: self.primitive_assembly,
            fragment_shader: self.fragment_shader,
            depth_test: self.depth_test,
            stencil_test: self.stencil_test,
            scissor_region: self.scissor_region,
            blending: self.blending,
            viewport: self.viewport,
            binding_strategy: self.binding_strategy,
        }
    }

    pub fn untyped_vertex_attribute_layout(
        self,
        vertex_attribute_layout: VertexInputLayoutDescriptor,
    ) -> GraphicsPipelineDescriptorBuilder<Vs, Pa, Fs, Untyped, R, Tf> {
        GraphicsPipelineDescriptorBuilder {
            _vertex_shader: marker::PhantomData,
            _primitive_assembly: marker::PhantomData,
            _fragment_shader: marker::PhantomData,
            _transform_feedback: marker::PhantomData,
            _vertex_attribute_layout: marker::PhantomData,
            _resource_layout: marker::PhantomData,
            vertex_shader: self.vertex_shader,
            vertex_input_layout: vertex_attribute_layout,
            transform_feedback_layout: self.transform_feedback_layout,
            primitive_assembly: self.primitive_assembly,
            fragment_shader: self.fragment_shader,
            depth_test: self.depth_test,
            stencil_test: self.stencil_test,
            scissor_region: self.scissor_region,
            blending: self.blending,
            viewport: self.viewport,
            binding_strategy: self.binding_strategy,
        }
    }

    /// Specifies a [TypedTransformFeedbackLayout] type that determines the layout of the transform
    /// feedback produced by any pipeline created using this descriptor.
    ///
    /// When the descriptor that results from this builder is used to create a graphics pipeline
    /// (see [RenderingContext::create_graphics_pipeline]), the transform feedback layout associated
    /// with this type is checked against the actual transform feedback layout defined by the
    /// pipeline's programmable shader stages (by reflecting on the code for these shader stages).
    /// If the layout defined by this type and the actual layout defined by the shader stages do not
    /// match, then pipeline creation will fail and an error is returned instead. If the layouts do
    /// match, then [TypedTransformFeedbackBuffers] that match the layout  may be safely bound to
    /// the pipeline without additional runtime checks (see
    /// [GraphicsPipeline::record_transform_feedback]).
    ///
    /// Note that [TypedTransformFeedbackLayout] is implemented for any type that implements
    /// [TransformFeedback] and any tuple of types that implement [TransformFeedback] (e.g.
    /// `(TransformFeedback1, TransformFeedback2)` where both `TransformFeedback1` and
    /// `TransformFeedback2` are types that implement [TransformFeedback]).
    pub fn typed_transform_feedback_layout<T>(
        self,
    ) -> GraphicsPipelineDescriptorBuilder<Vs, Pa, Fs, V, R, T>
    where
        T: TypedTransformFeedbackLayout,
    {
        GraphicsPipelineDescriptorBuilder {
            _vertex_shader: marker::PhantomData,
            _primitive_assembly: marker::PhantomData,
            _fragment_shader: marker::PhantomData,
            _transform_feedback: marker::PhantomData,
            _vertex_attribute_layout: marker::PhantomData,
            _resource_layout: marker::PhantomData,
            vertex_shader: self.vertex_shader,
            vertex_input_layout: self.vertex_input_layout,
            transform_feedback_layout: Some(T::LAYOUT_DESCRIPTION.into()),
            primitive_assembly: self.primitive_assembly,
            fragment_shader: self.fragment_shader,
            depth_test: self.depth_test,
            stencil_test: self.stencil_test,
            scissor_region: self.scissor_region,
            blending: self.blending,
            viewport: self.viewport,
            binding_strategy: self.binding_strategy,
        }
    }

    pub fn untyped_transform_feedback_layout(
        self,
        transform_feedback_layout: TransformFeedbackLayoutDescriptor,
    ) -> GraphicsPipelineDescriptorBuilder<Vs, Pa, Fs, V, R, Untyped> {
        GraphicsPipelineDescriptorBuilder {
            _vertex_shader: marker::PhantomData,
            _primitive_assembly: marker::PhantomData,
            _fragment_shader: marker::PhantomData,
            _transform_feedback: marker::PhantomData,
            _vertex_attribute_layout: marker::PhantomData,
            _resource_layout: marker::PhantomData,
            vertex_shader: self.vertex_shader,
            vertex_input_layout: self.vertex_input_layout,
            transform_feedback_layout: Some(transform_feedback_layout),
            primitive_assembly: self.primitive_assembly,
            fragment_shader: self.fragment_shader,
            depth_test: self.depth_test,
            stencil_test: self.stencil_test,
            scissor_region: self.scissor_region,
            blending: self.blending,
            viewport: self.viewport,
            binding_strategy: self.binding_strategy,
        }
    }

    /// Specifies a [Resources] type that determines the resource layout and resource binding
    /// strategy for any graphics pipeline created from the descriptor.
    ///
    /// When the descriptor that results from this builder is used to create a graphics pipeline
    /// (see [RenderingContext::create_graphics_pipeline]), the resource layout associated with
    /// this type is checked against the actual resource layout defined by the pipeline's
    /// programmable shader stages (by reflecting on the code for these shader stages). If the
    /// layout defined by this type and the actual layout defined by the shader stages do not match,
    /// then pipeline creation will fail and an error is returned instead. If the layouts do match,
    /// then instances of this type may be safely bound to the pipeline's resource slots when
    /// creating draw commands without additional runtime checks (see [PipelineTask::draw_command]).
    ///
    /// The `strategy` specified the strategy used to match the pipeline's resource slots to the
    /// resource bindings provided by the [Resources] type. If [SlotBindingStrategy::Check] is used,
    /// then [RenderingContext::create_graphics_pipeline] will verify whether or not the default
    /// binding indices used by the pipeline's resource slots are identical to the resource
    /// associated with that slot name on the [Resources] type. If any slot is incompatible, then
    /// [RenderingContext::create_graphics_pipeline] will fail and return an error. If
    /// [SlotBindingStrategy::Update] is used, then [RenderingContext::create_graphics_pipeline]
    /// will override the pipeline's default slot binding indices with the indices associated with
    /// the corresponding resource on the [Resources] type.
    pub fn resource_layout<T>(
        self,
        strategy: SlotBindingStrategy,
    ) -> GraphicsPipelineDescriptorBuilder<Vs, Pa, Fs, V, T, Tf>
    where
        T: Resources,
    {
        GraphicsPipelineDescriptorBuilder {
            _vertex_shader: marker::PhantomData,
            _primitive_assembly: marker::PhantomData,
            _fragment_shader: marker::PhantomData,
            _transform_feedback: marker::PhantomData,
            _vertex_attribute_layout: marker::PhantomData,
            _resource_layout: marker::PhantomData,
            vertex_shader: self.vertex_shader,
            vertex_input_layout: self.vertex_input_layout,
            transform_feedback_layout: self.transform_feedback_layout,
            primitive_assembly: self.primitive_assembly,
            fragment_shader: self.fragment_shader,
            depth_test: self.depth_test,
            stencil_test: self.stencil_test,
            scissor_region: self.scissor_region,
            blending: self.blending,
            viewport: self.viewport,
            binding_strategy: strategy,
        }
    }

    /// Enables the depth test for any graphics pipeline created from the descriptor.
    ///
    /// See [DepthTest] for details on the depth test.
    pub fn enable_depth_test(self, depth_test: DepthTest) -> Self {
        GraphicsPipelineDescriptorBuilder {
            depth_test: Some(depth_test),
            ..self
        }
    }

    /// Enables the depth test for any graphics pipeline created from the descriptor.
    ///
    /// See [StencilTest] for details on the depth test.
    pub fn enable_stencil_test(self, stencil_test: StencilTest) -> Self {
        GraphicsPipelineDescriptorBuilder {
            stencil_test: Some(stencil_test),
            ..self
        }
    }

    /// Sets the scissor region used by any graphics pipeline created from the descriptor.
    ///
    /// Any fragments outside the scissor region will be discarded before the fragment processing
    /// stages. If not set explicitly, the scissor region defaults to [Region2D::Fill] and will
    /// match the size of the framebuffer that is the current draw target.
    pub fn scissor_region(self, scissor_region: Region2D) -> Self {
        GraphicsPipelineDescriptorBuilder {
            scissor_region,
            ..self
        }
    }

    /// Enables blending for any graphics pipeline created from the descriptor.
    ///
    /// See [Blending] for details on blending.
    pub fn enable_blending(self, blending: Blending) -> Self {
        GraphicsPipelineDescriptorBuilder {
            blending: Some(blending),
            ..self
        }
    }

    /// Sets the viewport used by any graphics pipeline created from the descriptor.
    ///
    /// See [Viewport] for details on the viewport. Defaults to [Viewport::Auto].
    pub fn viewport(self, viewport: Viewport) -> Self {
        GraphicsPipelineDescriptorBuilder { viewport, ..self }
    }
}

impl<V, R, Tf>
    GraphicsPipelineDescriptorBuilder<VertexShader, PrimitiveAssembly, FragmentShader, V, R, Tf>
where
    R: Resources,
{
    /// Finishes building and returns the [GraphicsPipelineDescriptor].
    pub fn finish(self) -> GraphicsPipelineDescriptor<V, R, Tf> {
        GraphicsPipelineDescriptor {
            _vertex_attribute_layout: marker::PhantomData,
            _resource_layout: marker::PhantomData,
            _transform_feedback: marker::PhantomData,
            vertex_shader_data: self.vertex_shader.unwrap(),
            fragment_shader_data: self.fragment_shader.unwrap(),
            vertex_attribute_layout: self.vertex_input_layout,
            transform_feedback_layout: self.transform_feedback_layout,
            primitive_assembly: self.primitive_assembly.unwrap(),
            depth_test: self.depth_test,
            stencil_test: self.stencil_test,
            scissor_region: self.scissor_region,
            blending: self.blending,
            viewport: self.viewport,
            binding_strategy: self.binding_strategy,
        }
    }
}
