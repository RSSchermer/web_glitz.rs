use crate::pipeline::graphics::vertex_input::{InputAttributeLayout, VertexInputStreamDescription};
use crate::pipeline::graphics::{
    vertex_input, BindingStrategy, GraphicsPipelineDescriptor, Topology,
};
use crate::pipeline::resources;
use crate::pipeline::resources::bind_group_encoding::BindingDescriptor;
use crate::pipeline::resources::resource_slot::{
    Identifier, SlotBindingChecker, SlotBindingUpdater,
};
use crate::pipeline::resources::Resources;
use crate::runtime::state::{Program, ProgramKey};
use crate::runtime::{Connection, RenderingContext};
use crate::task::GpuTask;
use std::any::TypeId;
use std::borrow::Borrow;
use std::marker;
use std::sync::Arc;
use crate::pipeline::graphics::shader::ShaderData;

pub struct GraphicsPipeline<Il, R, Tf> {
    _input_attribute_layout_marker: marker::PhantomData<Il>,
    _resources_marker: marker::PhantomData<R>,
    _transform_feedback_varyings_marker: marker::PhantomData<Tf>,
    vertex_shader: Arc<ShaderData>,
    fragment_shader: Arc<ShaderData>,
}

impl<Il, R, Tf> GraphicsPipeline<Il, R, Tf>
where
    Il: InputAttributeLayout,
    R: Resources,
{
    pub(crate) fn create<Rc>(
        context: &Rc,
        connection: &mut Connection,
        descriptor: &GraphicsPipelineDescriptor<Il, R, Tf>,
    ) -> Result<Self, CreateGraphicsPipelineError>
    where
        Rc: RenderingContext + Clone + 'static,
    {
        let (gl, state) = unsafe { connection.unpack_mut() };

        if descriptor.vertex_shader.context_id() != context.id() {
            panic!("Vertex shader does not belong to the context.");
        }

        if descriptor.fragment_shader.context_id() != context.id() {
            panic!("Fragment shader does not belong to the context.");
        }

        let program = state.program_cache_mut().get_or_create(ProgramKey {
            vertex_shader_id: descriptor.vertex_shader.id().unwrap(),
            fragment_shader_id: descriptor.fragment_shader.id().unwrap(),
            resources_type_id: TypeId::of::<R>(),
        }, gl)?;

        Il::check_compatibility(program.attribute_slot_descriptors())?;

        let bindings_confirmer = match descriptor.binding_strategy {
            BindingStrategy::Check => SlotBindingChecker::new(gl, program.gl_object()),
            BindingStrategy::Update => SlotBindingUpdater::new(gl, program.gl_object()),
        };

        R::confirm_slot_bindings(&bindings_confirmer, program.resource_slot_descriptors())?;

        GraphicsPipeline {
            _input_attribute_layout_marker: marker::PhantomData,
            _resources_marker: marker::PhantomData,
            _transform_feedback_varyings_marker: marker::PhantomData,
            vertex_shader: descriptor.vertex_shader.data().clone(),
            fragment_shader: descriptor.fragment_shader.data().clone(),
        }
    }

//    pub(crate) fn create_unchecked<Rc>(
//        context: &Rc,
//        connection: &mut Connection,
//        descriptor: &GraphicsPipelineDescriptor<Il, R, Tf>,
//    ) -> Self
//    where
//        Rc: RenderingContext + Clone + 'static,
//    {
//        let (gl, state) = unsafe { connection.unpack_mut() };
//
//        let program = state
//            .program_cache()
//            .get_or_create_unchecked(ProgramDescriptor {
//                vertex_shader: &descriptor.vertex_shader,
//                fragment_shader: &descriptor.fragment_shader,
//                resources_type: TypeId::of::<R>(),
//            })?;
//
//        match descriptor.binding_strategy {
//            BindingStrategy::Update => {
//                let confirmer = SlotBindingUpdater::new(gl, program.gl_object());
//
//                R::confirm_slot_bindings(confirmer, program.resource_slot_descriptors())
//            }
//            _ => (),
//        };
//
//        GraphicsPipeline {
//            _input_attribute_layout_marker: marker::PhantomData,
//            _resources_marker: marker::PhantomData,
//            _transform_feedback_varyings_marker: marker::PhantomData,
//            vertex_shader: descriptor.vertex_shader.data().clone(),
//            fragment_shader: descriptor.fragment_shader.data().clone(),
//        }
//    }
}

pub struct ShaderLinkingError {
    error: String,
}

pub enum CreateGraphicsPipelineError {
    VertexShaderContextMismatch,
    FragmentShaderContextMismatch,
    ShaderLinkingError(ShaderLinkingError),
    IncompatibleInputAttributeLayout(vertex_input::Incompatible),
    IncompatibleResources(resources::Incompatible),
}

impl From<ShaderLinkingError> for CreateGraphicsPipelineError {
    fn from(error: ShaderLinkingError) -> Self {
        CreateGraphicsPipelineError::ShaderLinkingError(error)
    }
}

impl From<vertex_input::Incompatible> for CreateGraphicsPipelineError {
    fn from(error: vertex_input::Incompatible) -> Self {
        CreateGraphicsPipelineError::IncompatibleInputAttributeLayout(error)
    }
}

impl From<resources::Incompatible> for CreateGraphicsPipelineError {
    fn from(error: resources::Incompatible) -> Self {
        CreateGraphicsPipelineError::IncompatibleResources(error)
    }
}

pub struct ActiveGraphicsPipeline<Il, R> {
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
    pub fn draw_command<V>(&self, vertex_input_stream: V, resources: R) -> DrawCommand<R::Bindings>
    where
        V: VertexInputStreamDescription<Layout = Il>,
    {
        unimplemented!()
    }
}

pub struct DrawCommand<B> {
    topology: Topology,
    binding_group: B,
}

impl<B> GpuTask<Connection> for DrawCommand<B>
where
    B: Borrow<[BindingDescriptor]>,
{
    type Output = ();
}
