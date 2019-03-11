use crate::image::Region2D;
use crate::pipeline::graphics::fragment_test::{DepthTest, StencilTest};
use crate::pipeline::graphics::line_width::LineWidth;
use crate::pipeline::graphics::primitive_assembly::PrimitiveAssembly;
use crate::pipeline::graphics::shader::{FragmentShader, ShaderData, VertexShader};
use crate::pipeline::graphics::vertex_input::InputAttributeLayout;
use crate::pipeline::graphics::viewport::Viewport;
use crate::pipeline::resources::Resources;
use std::marker;
use std::sync::Arc;

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
    blending: Option<Blending>,
    line_width: LineWidth,
    viewport: Viewport,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum BindingStrategy {
    Check,
    Update,
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
            blending: None,
            line_width: LineWidth::default(),
            viewport: Viewport::Auto,
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
    blending: Option<Blending>,
    line_width: LineWidth,
    viewport: Viewport,
}

impl<Vs, Pa, Fs, Il, Rl, Tf> GraphicsPipelineDescriptorBuilder<Vs, Pa, Fs, Il, Rl, Tf> {
    pub fn vertex_shader(
        self,
        vertex_shader: &VertexShader,
    ) -> GraphicsPipelineDescriptorBuilder<VertexShader, Pa, Fs, Il, Rl, Tf> {
        GraphicsPipelineDescriptorBuilder {
            _vertex_shader: marker::PhantomData,
            _primitive_assembly: marker::PhantomData,
            _fragment_shader: marker::PhantomData,
            _transform_feedback: marker::PhantomData,
            _vertex_input_layout: marker::PhantomData,
            _resource_layout: marker::PhantomData,
            vertex_shader: Some(vertex_shader.data().clone()),
            primitive_assembly: self.primitive_assembly,
            fragment_shader: self.fragment_shader,
            depth_test: self.depth_test,
            stencil_test: self.stencil_test,
            scissor_test: self.scissor_test,
            blending: self.blending,
            line_width: self.line_width,
            viewport: self.viewport,
        }
    }

    pub fn primitive_assembly(
        self,
        primitive_assembly: PrimitiveAssembly,
    ) -> GraphicsPipelineDescriptorBuilder<Vs, PrimitiveAssembly, Fs, Il, Rl, Tf> {
        GraphicsPipelineDescriptorBuilder {
            _vertex_shader: marker::PhantomData,
            _primitive_assembly: marker::PhantomData,
            _fragment_shader: marker::PhantomData,
            _transform_feedback: marker::PhantomData,
            _vertex_input_layout: marker::PhantomData,
            _resource_layout: marker::PhantomData,
            vertex_shader: self.vertex_shader,
            primitive_assembly: Some(primitive_assembly),
            fragment_shader: self.fragment_shader,
            depth_test: self.depth_test,
            stencil_test: self.stencil_test,
            scissor_test: self.scissor_test,
            blending: self.blending,
            line_width: self.line_width,
            viewport: self.viewport,
        }
    }

    pub fn fragment_shader(
        self,
        fragment_shader: &FragmentShader,
    ) -> GraphicsPipelineDescriptorBuilder<Vs, Pa, FragmentShader, Il, Rl, Tf> {
        GraphicsPipelineDescriptorBuilder {
            _vertex_shader: marker::PhantomData,
            _primitive_assembly: marker::PhantomData,
            _fragment_shader: marker::PhantomData,
            _transform_feedback: marker::PhantomData,
            _vertex_input_layout: marker::PhantomData,
            _resource_layout: marker::PhantomData,
            vertex_shader: self.vertex_shader,
            primitive_assembly: self.primitive_assembly,
            fragment_shader: Some(fragment_shader.data().clone()),
            depth_test: self.depth_test,
            stencil_test: self.stencil_test,
            scissor_test: self.scissor_test,
            blending: self.blending,
            line_width: self.line_width,
            viewport: self.viewport,
        }
    }

    pub fn vertex_input_layout<T>(self) -> GraphicsPipelineDescriptorBuilder<Vs, Pa, Fs, T, Rl, Tf>
    where
        T: InputAttributeLayout,
    {
        GraphicsPipelineDescriptorBuilder {
            _vertex_shader: marker::PhantomData,
            _primitive_assembly: marker::PhantomData,
            _fragment_shader: marker::PhantomData,
            _transform_feedback: marker::PhantomData,
            _vertex_input_layout: marker::PhantomData,
            _resource_layout: marker::PhantomData,
            vertex_shader: self.vertex_shader,
            primitive_assembly: self.primitive_assembly,
            fragment_shader: self.fragment_shader,
            depth_test: self.depth_test,
            stencil_test: self.stencil_test,
            scissor_test: self.scissor_test,
            blending: self.blending,
            line_width: self.line_width,
            viewport: self.viewport,
        }
    }

    pub fn resource_layout<T>(
        self,
        strategy: BindingStrategy,
    ) -> GraphicsPipelineDescriptorBuilder<Vs, Pa, Fs, Il, T, Tf>
    where
        T: Resources,
    {
        GraphicsPipelineDescriptorBuilder {
            _vertex_shader: marker::PhantomData,
            _primitive_assembly: marker::PhantomData,
            _fragment_shader: marker::PhantomData,
            _transform_feedback: marker::PhantomData,
            _vertex_input_layout: marker::PhantomData,
            _resource_layout: marker::PhantomData,
            vertex_shader: self.vertex_shader,
            primitive_assembly: self.primitive_assembly,
            fragment_shader: self.fragment_shader,
            depth_test: self.depth_test,
            stencil_test: self.stencil_test,
            scissor_test: self.scissor_test,
            blending: self.blending,
            line_width: self.line_width,
            viewport: self.viewport,
        }
    }

    pub fn enable_depth_test(self, depth_test: DepthTest) -> Self {
        GraphicsPipelineDescriptorBuilder {
            depth_test: Some(depth_test),
            ..self
        }
    }

    pub fn disable_depth_test(self) -> Self {
        GraphicsPipelineDescriptorBuilder {
            depth_test: None,
            ..self
        }
    }

    pub fn enable_stencil_test(self, stencil_test: StencilTest) -> Self {
        GraphicsPipelineDescriptorBuilder {
            stencil_test: Some(stencil_test),
            ..self
        }
    }

    pub fn disable_stencil_test(self) -> Self {
        GraphicsPipelineDescriptorBuilder {
            stencil_test: None,
            ..self
        }
    }

    pub fn enable_scissor_test(self, scissor_region: Region2D) -> Self {
        GraphicsPipelineDescriptorBuilder {
            scissor_test: Some(scissor_region),
            ..self
        }
    }

    pub fn disable_scissor_test(self) -> Self {
        GraphicsPipelineDescriptorBuilder {
            scissor_test: None,
            ..self
        }
    }

    pub fn enable_blending(self, blending: Blending) -> Self {
        GraphicsPipelineDescriptorBuilder {
            blending: Some(blending),
            ..self
        }
    }

    pub fn disable_blending(self) -> Self {
        GraphicsPipelineDescriptorBuilder {
            blending: None,
            ..self
        }
    }

    pub fn line_width(self, line_width: LineWidth) -> Self {
        GraphicsPipelineDescriptorBuilder { line_width, ..self }
    }
}
