use crate::pipeline::graphics::vertex_input::{InputAttributeLayout, VertexInputStreamDescription, VertexInputAttributeDescriptor, VertexInputStreamDescriptor};
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
use crate::task::{GpuTask, ContextId, Progress};
use std::any::TypeId;
use std::borrow::Borrow;
use std::marker;
use std::sync::Arc;
use crate::pipeline::graphics::shader::ShaderData;
use crate::pipeline::graphics::vertex_input::vertex_array::VertexArrayData;

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

pub struct PipelineTaskContext<'a> {
    connection: &'a mut Connection
}

pub struct ActiveGraphicsPipeline<Il, R> {
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
    pub fn draw_command<V>(&self, vertex_input_stream: V, resources: R) -> DrawCommand<R::Bindings>
    where
        V: VertexInputStreamDescription<Layout = Il>,
    {
        unimplemented!()
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

    fn progress(&self, context: &mut PipelineTaskContext<'a>) -> Progress<Self::Output> {
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

            let offset = offset * format.size_in_bytes();

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
