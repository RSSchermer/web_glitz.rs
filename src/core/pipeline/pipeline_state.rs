
pub struct GraphicsPipeline {
    vertex_shader: Shader,
    primitive_assembly: PrimitiveAssembly,
    rasterizer: Rasterizer,
    scissor_test: Option<ScreenRegion>,
    stencil_test: Option<StencilTest>,
    depth_test: Option<DepthTest>,
    fragment_shader: Shader,
    blending: Option<Blending>,
    viewport: Viewport
}

impl From<GraphicsPipeline> for GraphicsPipelineBuilder<Shader, Topology, impl Unspecified, Rasterizer, Shader>{
    fn from(pipeline: GraphicsPipeline) -> GraphicsPipelineBuilder<Shader, Topology, impl Unspecified, Rasterizer, Shader> {
        GraphicsPipelineBuilder {
            vertex_shader: pipeline.vertex_shader,
            primitive_assembly: pipeline.primitive_assembly,
            transform_feedback: (),
            rasterizer: pipeline.rasterizer,
            scissor_test: pipeline.scissor_test,
            stencil_test: pipeline.stencil_test,
            depth_test: pipeline.depth_test,
            fragment_shader: pipeline.fragment_shader,
            blending: pipeline.blending,
            viewport: pipeline.viewport
        }
    }
}

pub struct TFGraphicsPipeline {
    graphics_pipeline: GraphicsPipeline,
    transform_feedback: TransformFeedback,
}

impl TFGraphicsPipeline {

}

pub struct RecordingTFGraphicsPipeline {
    graphics_pipeline: TFGraphicsPipeline,
    recording_session: TransformFeedbackSession,
}

pub struct PausedTFGraphicsPipeline {
    graphics_pipeline: TFGraphicsPipeline,
    recording_session: TransformFeedbackSession,
}

pub struct TFComputePipeline {
    shader: Shader,
    primitive_assembly: PrimitiveAssembly,
    transform_feedback: TransformFeedback,
}

pub struct GraphicsPipelineBuilder<VertexShader, PrimitiveAssembly, TransformFeedback, Rasterize, FragmentShader> {
    vertex_shader: VertexShader,
    primitive_assembly: PrimitiveAssembly,
    transform_feedback: TransformFeedback,
    rasterizer: Rasterize,
    scissor_test: Option<ScreenRegion>,
    stencil_test: Option<StencilTest>,
    depth_test: Option<DepthTest>,
    fragment_shader: FragmentShader,
    blending: Option<Blending>,
    viewport: Option<ScreenRegion>
}

trait Unspecified {}

impl Unspecified for () {}

impl<VertexShader, PrimitiveAssembly, TransformFeedback, FragmentShader> GraphicsPipelineBuilder<VertexShader, PrimitiveAssembly, TransformFeedback, Rasterizer, FragmentShader> where VertexShader: Unspecified, PrimitiveAssembly: Unspecified, TransformFeedback: Unspecified, FragmentShader: Unspecified {
    pub fn new() -> Self {
        GraphicsPipelineBuilder {
            vertex_shader: (),
            primitive_assembly: (),
            transform_feedback: (),
            rasterizer: Rasterizer::default(),
            scissor_test: None,
            stencil_test: None,
            depth_test: None,
            fragment_shader: (),
            blending: None,
            viewport: Viewport::default()
        }
    }
}

impl<VertexShader, PrimitiveAssembly, TransformFeedback, Rasterize, FragmentShader> GraphicsPipelineBuilder<VertexShader, PrimitiveAssembly, TransformFeedback, Rasterize, FragmentShader> {
    pub fn vertex_shader(self, shader: Shader) -> GraphicsPipelineBuilder<Shader, PrimitiveAssembly, TransformFeedback, Rasterize, FragmentShader> {
        GraphicsPipelineBuilder {
            vertex_shader: shader,
            .. self
        }
    }

    pub fn primitive_assembly(self, topology: Topology) -> GraphicsPipelineBuilder<VertexShader, Topology, TransformFeedback, Rasterize, FragmentShader> {
        GraphicsPipelineBuilder {
            primitive_assembly: topology,
            .. self
        }
    }

    pub fn transform_feedback(self, varyings: TransformFeedbackVaryings) -> GraphicsPipelineBuilder<VertexShader, PrimitiveAssembly, TransformFeedbackVaryings, Rasterize, FragmentShader> {
        GraphicsPipelineBuilder {
            transform_feedback: varyings,
            .. self
        }
    }

    pub fn rasterizer(self, rasterizer: Rasterizer) -> GraphicsPipelineBuilder<VertexShader, PrimitiveAssembly, TransformFeedback, Rasterizer, FragmentShader> {
        GraphicsPipelineBuilder {
            rasterizer,
            .. self
        }
    }

    pub fn disable_rasterizer(self) -> GraphicsPipelineBuilder<VertexShader, PrimitiveAssembly, TransformFeedback, (), FragmentShader> {
        GraphicsPipelineBuilder {
            rasterizer: (),
            .. self
        }
    }

    pub fn scissor_test(self, region: ScreenRegion) -> Self {
        self.scissor_test = Some(ScreenRegion);

        self
    }

    pub fn disable_scissor_test(self) -> Self {
        self.scissor_test = None;

        self
    }

    pub fn stencil_test(self, stencil_test: StencilTest) -> Self {
        self.stencil_test = Some(StencilTest);

        self
    }

    pub fn disable_stencil_test(self) -> Self {
        self.stencil_test = None;

        self
    }

    pub fn depth_test(self, depth_test: DepthTest) -> Self {
        self.depth_test = Some(DepthTest);

        self
    }

    pub fn disable_depth_test(self) -> Self {
        self.depth_test = None;

        self
    }

    pub fn fragment_shader(self, shader: Shader) -> GraphicsPipelineBuilder<VertexShader, PrimitiveAssembly, TransformFeedback, Rasterize, Shader> {
        GraphicsPipelineBuilder {
            fragment_shader: shader,
            .. self
        }
    }

    pub fn blending(self, blending: Blending) -> Self {
        self.blending = Some(Blending);

        self
    }

    pub fn disable_blending(self) -> Self {
        self.blending = None;

        self
    }
}

impl<TransformFeedback> GraphicsPipelineBuilder<Shader, Topology, TransformFeedback, Rasterizer, Shader> where TransformFeedback: Unspecified {
    pub fn build(self) -> GraphicsPipeline {
        GraphicsPipeline {
            vertex_shader: self.vertex_shader,
            primitive_assembly: self.primitive_assembly,
            rasterizer: self.rasterizer,
            scissor_test: self.scissor_test,
            stencil_test: self.stencil_test,
            depth_test: self.depth_test,
            fragment_shader: self.fragment_shader,
            blending: self.blending,
            viewport: self.viewport
        }
    }
}

impl GraphicsPipelineBuilder<Shader, Topology, TransformFeedbackVaryings, Rasterizer, Shader> {
    pub fn build(self) -> TFGraphicsPipeline {
        TFGraphicsPipeline {
            graphics_pipeline: GraphicsPipeline {
                vertex_shader: self.vertex_shader,
                primitive_assembly: self.primitive_assembly,
                rasterizer: self.rasterizer,
                scissor_test: self.scissor_test,
                stencil_test: self.stencil_test,
                depth_test: self.depth_test,
                fragment_shader: self.fragment_shader,
                blending: self.blending,
                viewport: self.viewport
            },
            transform_feedback: self.transform_feedback,
        }
    }
}

pub struct TFComputePipelineBuilder<Shader, PrimitiveAssembly, TransformFeedback> {
    shader: Shader,
    primitive_assembly: PrimitiveAssembly,
    transform_feedback: TransformFeedback,
}

impl TFComputePipelineBuilder<Shader, Topology, TransformFeedbackVaryings> {
    pub fn build(self) -> TFComputePipeline {
        TFComputePipeline {
            shader: self.shader,
            primitive_assembly: self.primitive_assembly,
            transform_feedback: self.transform_feedback
        }
    }
}
