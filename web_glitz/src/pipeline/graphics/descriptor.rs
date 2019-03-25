use std::marker;
use std::sync::Arc;

use crate::image::Region2D;
use crate::pipeline::graphics::fragment_test::{DepthTest, StencilTest};
use crate::pipeline::graphics::line_width::LineWidth;
use crate::pipeline::graphics::primitive_assembly::PrimitiveAssembly;
use crate::pipeline::graphics::shader::{FragmentShader, VertexShader, VertexShaderData, FragmentShaderData};
use crate::pipeline::graphics::vertex_input::InputAttributeLayout;
use crate::pipeline::graphics::viewport::Viewport;
use crate::pipeline::graphics::Blending;
use crate::pipeline::resources::Resources;

pub struct GraphicsPipelineDescriptor<Il, R, Tf> {
    _vertex_input_layout: marker::PhantomData<Il>,
    _resource_layout: marker::PhantomData<R>,
    _transform_feedback: marker::PhantomData<Tf>,
    pub(crate) vertex_shader_data: Arc<VertexShaderData>,
    pub(crate) fragment_shader_data: Arc<FragmentShaderData>,
    pub(crate) primitive_assembly: PrimitiveAssembly,
    pub(crate) depth_test: Option<DepthTest>,
    pub(crate) stencil_test: Option<StencilTest>,
    pub(crate) scissor_region: Region2D,
    pub(crate) blending: Option<Blending>,
    pub(crate) line_width: LineWidth,
    pub(crate) viewport: Viewport,
    pub(crate) binding_strategy: BindingStrategy,
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
            scissor_region: Region2D::Fill,
            blending: None,
            line_width: LineWidth::default(),
            viewport: Viewport::Auto,
            binding_strategy: BindingStrategy::Check,
        }
    }
}

pub struct GraphicsPipelineDescriptorBuilder<Vs, Pa, Fs, Il, R, Tf> {
    _vertex_shader: marker::PhantomData<Vs>,
    _primitive_assembly: marker::PhantomData<Pa>,
    _fragment_shader: marker::PhantomData<Fs>,
    _transform_feedback: marker::PhantomData<Tf>,
    _vertex_input_layout: marker::PhantomData<Il>,
    _resource_layout: marker::PhantomData<R>,
    vertex_shader: Option<Arc<VertexShaderData>>,
    fragment_shader: Option<Arc<FragmentShaderData>>,
    primitive_assembly: Option<PrimitiveAssembly>,
    depth_test: Option<DepthTest>,
    stencil_test: Option<StencilTest>,
    scissor_region: Region2D,
    blending: Option<Blending>,
    line_width: LineWidth,
    viewport: Viewport,
    binding_strategy: BindingStrategy,
}

impl<Vs, Pa, Fs, Il, R, Tf> GraphicsPipelineDescriptorBuilder<Vs, Pa, Fs, Il, R, Tf> {
    pub fn vertex_shader(
        self,
        vertex_shader: &VertexShader,
    ) -> GraphicsPipelineDescriptorBuilder<VertexShader, Pa, Fs, Il, R, Tf> {
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
            scissor_region: self.scissor_region,
            blending: self.blending,
            line_width: self.line_width,
            viewport: self.viewport,
            binding_strategy: self.binding_strategy,
        }
    }

    pub fn primitive_assembly(
        self,
        primitive_assembly: PrimitiveAssembly,
    ) -> GraphicsPipelineDescriptorBuilder<Vs, PrimitiveAssembly, Fs, Il, R, Tf> {
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
            scissor_region: self.scissor_region,
            blending: self.blending,
            line_width: self.line_width,
            viewport: self.viewport,
            binding_strategy: self.binding_strategy,
        }
    }

    pub fn fragment_shader(
        self,
        fragment_shader: &FragmentShader,
    ) -> GraphicsPipelineDescriptorBuilder<Vs, Pa, FragmentShader, Il, R, Tf> {
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
            scissor_region: self.scissor_region,
            blending: self.blending,
            line_width: self.line_width,
            viewport: self.viewport,
            binding_strategy: self.binding_strategy,
        }
    }

    pub fn vertex_input_layout<T>(self) -> GraphicsPipelineDescriptorBuilder<Vs, Pa, Fs, T, R, Tf>
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
            scissor_region: self.scissor_region,
            blending: self.blending,
            line_width: self.line_width,
            viewport: self.viewport,
            binding_strategy: self.binding_strategy,
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
            scissor_region: self.scissor_region,
            blending: self.blending,
            line_width: self.line_width,
            viewport: self.viewport,
            binding_strategy: strategy,
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

    pub fn scissor_region(self, scissor_region: Region2D) -> Self {
        GraphicsPipelineDescriptorBuilder {
            scissor_region,
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

    pub fn viewport(self, viewport: Viewport) -> Self {
        GraphicsPipelineDescriptorBuilder {
            viewport,
            ..self
        }
    }
}

impl<Il, R>
    GraphicsPipelineDescriptorBuilder<VertexShader, PrimitiveAssembly, FragmentShader, Il, R, ()>
where
    Il: InputAttributeLayout,
    R: Resources,
{
    fn finish(self) -> GraphicsPipelineDescriptor<Il, R, ()> {
        GraphicsPipelineDescriptor {
            _vertex_input_layout: marker::PhantomData,
            _resource_layout: marker::PhantomData,
            _transform_feedback: marker::PhantomData,
            vertex_shader_data: self.vertex_shader.unwrap(),
            fragment_shader_data: self.fragment_shader.unwrap(),
            primitive_assembly: self.primitive_assembly.unwrap(),
            depth_test: self.depth_test,
            stencil_test: self.stencil_test,
            scissor_region: self.scissor_region,
            blending: self.blending,
            line_width: self.line_width,
            viewport: self.viewport,
            binding_strategy: self.binding_strategy,
        }
    }
}
