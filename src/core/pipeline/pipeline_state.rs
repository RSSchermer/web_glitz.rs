
pub struct DrawPipeline {
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

pub struct FeedbackPipeline {
    vertex_shader: Shader,
    primitive_assembly: PrimitiveAssembly,
    transform_feedback: TransformFeedback,
}

pub struct DrawFeedbackPipeline {
    vertex_shader: Shader,
    primitive_assembly: PrimitiveAssembly,
    transform_feedback: TransformFeedback,
    rasterizer: Rasterizer,
    scissor_test: Option<ScreenRegion>,
    stencil_test: Option<StencilTest>,
    depth_test: Option<DepthTest>,
    fragment_shader: Shader,
    blending: Option<Blending>,
    viewport: Viewport
}

pub struct PipelineBuilder<VertexShader, PrimitiveAssembly, TransformFeedback, Rasterize, FragmentShader> {
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

impl<VertexShader, PrimitiveAssembly, TransformFeedback, FragmentShader> PipelineBuilder<VertexShader, PrimitiveAssembly, TransformFeedback, Rasterizer, FragmentShader> where VertexShader: Unspecified, PrimitiveAssembly: Unspecified, TransformFeedback: Unspecified, FragmentShader: Unspecified {
    pub fn new() -> Self {
        PipelineBuilder {
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

impl<VertexShader, PrimitiveAssembly, TransformFeedback, Rasterize, FragmentShader> PipelineBuilder<VertexShader, PrimitiveAssembly, TransformFeedback, Rasterize, FragmentShader> {
    pub fn vertex_shader(self, shader: Shader) -> PipelineBuilder<Shader, PrimitiveAssembly, TransformFeedback, Rasterize, FragmentShader> {
        PipelineBuilder {
            vertex_shader: shader,
            .. self
        }
    }

    pub fn primitive_assembly(self, topology: Topology) -> PipelineBuilder<VertexShader, Topology, TransformFeedback, Rasterize, FragmentShader> {
        PipelineBuilder {
            primitive_assembly: topology,
            .. self
        }
    }

    pub fn transform_feedback(self, varyings: TransformFeedbackVaryings) -> PipelineBuilder<VertexShader, PrimitiveAssembly, TransformFeedbackVaryings, Rasterize, FragmentShader> {
        PipelineBuilder {
            transform_feedback: varyings,
            .. self
        }
    }

    pub fn rasterizer(self, rasterizer: Rasterizer) -> PipelineBuilder<VertexShader, PrimitiveAssembly, TransformFeedback, Rasterizer, FragmentShader> {
        PipelineBuilder {
            rasterizer,
            .. self
        }
    }

    pub fn disable_rasterizer(self) -> PipelineBuilder<VertexShader, PrimitiveAssembly, TransformFeedback, (), FragmentShader> {
        PipelineBuilder {
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

    pub fn fragment_shader(self, shader: Shader) -> PipelineBuilder<VertexShader, PrimitiveAssembly, TransformFeedback, Rasterize, Shader> {
        PipelineBuilder {
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

impl<Rasterize, FragmentShader> PipelineBuilder<Shader, Topology, TransformFeedbackVaryings, Rasterize, FragmentShader> where Rasterize: Unspecified {
    pub fn build(self) -> FeedbackPipeline {
        FeedbackPipeline {
            vertex_shader: self.vertex_shader,
            primitive_assembly: self.primitive_assembly,
            transform_feedback: self.transform_feedback
        }
    }
}

impl<TransformFeedback> PipelineBuilder<Shader, Topology, TransformFeedback, Rasterizer, Shader> where TransformFeedback: Unspecified {
    pub fn build(self) -> DrawPipeline {
        DrawPipeline {
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

impl PipelineBuilder<Shader, Topology, TransformFeedbackVaryings, Rasterizer, Shader> {
    pub fn build(self) -> DrawFeedbackPipeline {
        DrawFeedbackPipeline {
            vertex_shader: self.vertex_shader,
            primitive_assembly: self.primitive_assembly,
            transform_feedback: self.transform_feedback,
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


