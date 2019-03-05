pub struct GraphicsPipelineDescriptor<Il, Rl, Tf> {
    _vertex_input_layout: marker::PhantomData<Il>,
    _resource_layout: marker::PhantomData<Rl>,
    _transform_feedback: marker::PhantomData<Tf>,
    vertex_shader: Arc<ShaderData>,
    fragment_shader: Arc<ShaderData>,
    primitive_assembly: PrimitiveAssembly,
    depth_test: Option<DepthTest>,
    stencil_test: Option<StencilTest>,
    scissor_test: Option<Region2D>,
    blending: Option<Blending>
}

impl GraphicsPipelineDescriptor<(), (), ()> {
    pub fn begin() -> GraphicsPipelineDescriptorBuilder<(), (), (), (), (), ()> {
        GraphicsPipelineDescriptorBuilder {
            _vertex_shader: marker::PhantomData,
            _primitive_assembly: marker::PhantomData,
            _fragment_shader: marker::PhantomData,
            _transform_feedback: marker::PhantomData,
            _vertex_input_layout: marker::PhantomData,
            _resource_layout: marker::PhantomData,
            vertex_shader: None,
            fragment_shader: None,
            primitive_assembly: None,
            depth_test: None,
            stencil_test: None,
            scissor_test: None,
            blending: None
        }
    }
}

pub struct GraphicsPipelineDescriptorBuilder<Vs, Pa, Fs, Il, Rl, Tf> {
    _vertex_shader: marker::PhantomData<Vs>,
    _primitive_assembly: marker::PhantomData<Pa>,
    _fragment_shader: marker::PhantomData<Fs>,
    _transform_feedback: marker::PhantomData<Tf>,
    _vertex_input_layout: marker::PhantomData<Il>,
    _resource_layout: marker::PhantomData<Rl>,
    vertex_shader: Option<Arc<ShaderData>>,
    fragment_shader: Option<Arc<ShaderData>>,
    primitive_assembly: Option<PrimitiveAssembly>,
    depth_test: Option<DepthTest>,
    stencil_test: Option<StencilTest>,
    scissor_test: Option<Region2D>,
    blending: Option<Blending>
}

impl<Vs, Pa, Fs, Il, Rl, Tf> GraphicsPipelineDescriptorBuilder<Vs, Pa, Fs, Il, Rl, Tf> {
    pub fn vertex_shader(self, vertex_shader: &VertexShader) -> GraphicsPipelineDescriptorBuilder<VertexShader, Pa, Fs, Il, Rl, Tf> {
        GraphicsPipelineDescriptorBuilder {
            _vertex_shader: marker::PhantomData,
            _primitive_assembly: marker::PhantomData,
            _fragment_shader: marker::PhantomData,
            _transform_feedback: marker::PhantomData,
            _vertex_input_layout: marker::PhantomData,
            _resource_layout: marker::PhantomData,
            vertex_shader: Some(VertexShader.data().clone()),
            primitive_assembly: self.primitive_assembly,
            fragment_shader: self.fragment_shader,
            depth_test: self.depth_test,
            stencil_test: self.stencil_test,
            scissor_test: self.scissor_test,
            blending: self.blending
        }
    }

    pub fn primitive_assembly(self, primitive_assembly: &PrimitiveAssembly) -> GraphicsPipelineDescriptorBuilder<Vs, PrimitiveAssembly, Fs, Il, Rl, Tf> {
        GraphicsPipelineDescriptorBuilder {
            _vertex_shader: marker::PhantomData,
            _primitive_assembly: marker::PhantomData,
            _fragment_shader: marker::PhantomData,
            _transform_feedback: marker::PhantomData,
            _vertex_input_layout: marker::PhantomData,
            _resource_layout: marker::PhantomData,
            vertex_shader: self.vertex_shader,
            primitive_assembly: Some(primitive_assembly.clone()),
            fragment_shader: self.fragment_shader,
            depth_test: self.depth_test,
            stencil_test: self.stencil_test,
            scissor_test: self.scissor_test,
            blending: self.blending
        }
    }
}
