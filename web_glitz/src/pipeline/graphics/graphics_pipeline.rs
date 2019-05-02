use std::any::TypeId;
use std::marker;
use std::sync::Arc;

use crate::image::Region2D;
use crate::pipeline::graphics::shader::{FragmentShaderData, VertexShaderData};
use crate::pipeline::graphics::AttributeSlotLayoutCompatible;
use crate::pipeline::graphics::{
    SlotBindingStrategy, Blending, DepthTest, GraphicsPipelineDescriptor, PrimitiveAssembly,
    StencilTest, Viewport,
};
use crate::pipeline::resources::resource_slot::{SlotBindingChecker, SlotBindingUpdater};
use crate::pipeline::resources::Resources;
use crate::runtime::state::ProgramKey;
use crate::runtime::{Connection, CreateGraphicsPipelineError, RenderingContext};
use crate::task::{ContextId, GpuTask, Progress};
use crate::util::JsId;

/// Encapsulates the state for a graphics pipeline.
///
/// See [RenderingContext::create_graphics_pipeline] for details on how a graphics pipeline is
/// constructed. See [Framebuffer::pipeline_task] for details on how a graphics pipeline may be used
/// to draw to the framebuffer.
pub struct GraphicsPipeline<V, R, Tf> {
    _vertex_attribute_layout_marker: marker::PhantomData<V>,
    _resources_marker: marker::PhantomData<R>,
    _transform_feedback_varyings_marker: marker::PhantomData<Tf>,
    context_id: usize,
    dropper: Box<ProgramObjectDropper>,
    #[allow(dead_code)] // Just holding on to this so it won't get dropped prematurely
    vertex_shader_data: Arc<VertexShaderData>,
    #[allow(dead_code)] // Just holding on to this so it won't get dropped prematurely
    fragment_shader_data: Arc<FragmentShaderData>,
    primitive_assembly: PrimitiveAssembly,
    program_id: JsId,
    depth_test: Option<DepthTest>,
    stencil_test: Option<StencilTest>,
    scissor_region: Region2D,
    blending: Option<Blending>,
    viewport: Viewport,
}

impl<V, R, Tf> GraphicsPipeline<V, R, Tf> {
    pub(crate) fn context_id(&self) -> usize {
        self.context_id
    }

    pub(crate) fn program_id(&self) -> JsId {
        self.program_id
    }

    pub(crate) fn primitive_assembly(&self) -> &PrimitiveAssembly {
        &self.primitive_assembly
    }

    pub(crate) fn depth_test(&self) -> &Option<DepthTest> {
        &self.depth_test
    }

    pub(crate) fn stencil_test(&self) -> &Option<StencilTest> {
        &self.stencil_test
    }

    pub(crate) fn scissor_region(&self) -> &Region2D {
        &self.scissor_region
    }

    pub(crate) fn blending(&self) -> &Option<Blending> {
        &self.blending
    }

    pub(crate) fn viewport(&self) -> &Viewport {
        &self.viewport
    }
}

impl<V, R, Tf> GraphicsPipeline<V, R, Tf>
where
    V: AttributeSlotLayoutCompatible,
    R: Resources + 'static,
{
    pub(crate) fn create<Rc>(
        context: &Rc,
        connection: &mut Connection,
        descriptor: &GraphicsPipelineDescriptor<V, R, Tf>,
    ) -> Result<Self, CreateGraphicsPipelineError>
    where
        Rc: RenderingContext + Clone + 'static,
    {
        let (gl, state) = unsafe { connection.unpack_mut() };

        if descriptor.vertex_shader_data.context_id() != context.id() {
            panic!("Vertex shader does not belong to the context.");
        }

        if descriptor.fragment_shader_data.context_id() != context.id() {
            panic!("Fragment shader does not belong to the context.");
        }

        let mut program_cache = state.program_cache_mut();
        let program = program_cache.get_or_create(
            ProgramKey {
                vertex_shader_id: descriptor.vertex_shader_data.id().unwrap(),
                fragment_shader_id: descriptor.fragment_shader_data.id().unwrap(),
                resources_type_id: TypeId::of::<R>(),
            },
            gl,
        )?;

        V::check_compatibility(program.attribute_slot_descriptors())?;

        match descriptor.binding_strategy {
            SlotBindingStrategy::Check => {
                let confirmer = SlotBindingChecker::new(gl, program.gl_object());

                R::confirm_slot_bindings(&confirmer, program.resource_slot_descriptors())?;
            }
            SlotBindingStrategy::Update => {
                let confirmer = SlotBindingUpdater::new(gl, program.gl_object());

                R::confirm_slot_bindings(&confirmer, program.resource_slot_descriptors())?;
            }
        };

        Ok(GraphicsPipeline {
            _vertex_attribute_layout_marker: marker::PhantomData,
            _resources_marker: marker::PhantomData,
            _transform_feedback_varyings_marker: marker::PhantomData,
            context_id: context.id(),
            dropper: Box::new(context.clone()),
            vertex_shader_data: descriptor.vertex_shader_data.clone(),
            fragment_shader_data: descriptor.fragment_shader_data.clone(),
            primitive_assembly: descriptor.primitive_assembly.clone(),
            program_id: JsId::from_value(program.gl_object().into()),
            depth_test: descriptor.depth_test.clone(),
            stencil_test: descriptor.stencil_test.clone(),
            scissor_region: descriptor.scissor_region.clone(),
            blending: descriptor.blending.clone(),
            viewport: descriptor.viewport.clone(),
        })
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

/// Error returned when trying to create a graphics pipeline and the shaders fail to link.
///
/// See [RenderingContext::create_graphics_pipeline].
#[derive(Debug)]
pub struct ShaderLinkingError {
    pub(crate) error: String,
}

trait ProgramObjectDropper {
    fn drop_program_object(&self, id: JsId);
}

impl<T> ProgramObjectDropper for T
where
    T: RenderingContext,
{
    fn drop_program_object(&self, id: JsId) {
        self.submit(ProgramObjectDropCommand { id });
    }
}

struct ProgramObjectDropCommand {
    id: JsId,
}

unsafe impl GpuTask<Connection> for ProgramObjectDropCommand {
    type Output = ();

    fn context_id(&self) -> ContextId {
        ContextId::Any
    }

    fn progress(&mut self, _connection: &mut Connection) -> Progress<Self::Output> {
        unsafe { JsId::into_value(self.id) };

        Progress::Finished(())
    }
}

impl<V, R, Tf> Drop for GraphicsPipeline<V, R, Tf> {
    fn drop(&mut self) {
        self.dropper.drop_program_object(self.program_id);
    }
}
