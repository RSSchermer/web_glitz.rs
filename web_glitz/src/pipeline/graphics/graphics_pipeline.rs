use std::cell::UnsafeCell;
use std::hash::{Hash, Hasher};
use std::marker;
use std::sync::Arc;

use fnv::FnvHasher;
use wasm_bindgen::convert::IntoWasmAbi;
use wasm_bindgen::JsCast;

use crate::image::Region2D;
use crate::pipeline::graphics::descriptor::ResourceBindingsLayoutKind;
use crate::pipeline::graphics::shader::{FragmentShaderData, VertexShaderData};
use crate::pipeline::graphics::util::BufferDescriptors;
use crate::pipeline::graphics::{
    Blending, DepthTest, GraphicsPipelineDescriptor, PrimitiveAssembly, StencilTest,
    TransformFeedbackBuffersEncodingContext, TransformFeedbackLayoutDescriptor,
    TypedTransformFeedbackBuffers, TypedTransformFeedbackLayout, Untyped,
    VertexInputLayoutDescriptor, Viewport,
};
use crate::pipeline::resources::resource_slot::{SlotBindingUpdater, SlotType};
use crate::pipeline::resources::{
    IncompatibleResources, ResourceBindingsLayoutDescriptor, ResourceSlotKind, ResourceSlotType,
    TypedResourceBindingsLayout, TypedResourceBindingsLayoutDescriptor,
};
use crate::runtime::state::{ContextUpdate, DynamicState, ProgramKey};
use crate::runtime::{Connection, CreateGraphicsPipelineError, RenderingContext};
use crate::task::{ContextId, GpuTask, Progress};
use crate::util::JsId;

/// Encapsulates the state for a graphics pipeline.
///
/// See [RenderingContext::create_graphics_pipeline] for details on how a graphics pipeline is
/// constructed. See [Framebuffer::pipeline_task] for details on how a graphics pipeline may be used
/// to draw to a framebuffer.
pub struct GraphicsPipeline<V, R, Tf> {
    _vertex_attribute_layout_marker: marker::PhantomData<V>,
    _resources_marker: marker::PhantomData<R>,
    _transform_feedback_varyings_marker: marker::PhantomData<Tf>,
    context_id: usize,
    dropper: Box<dyn GraphicsPipelineDropper>,
    #[allow(dead_code)] // Just holding on to this so it won't get dropped prematurely
    pub(crate) vertex_shader_data: Arc<VertexShaderData>,
    #[allow(dead_code)] // Just holding on to this so it won't get dropped prematurely
    pub(crate) fragment_shader_data: Arc<FragmentShaderData>,
    vertex_attribute_layout: VertexInputLayoutDescriptor,
    transform_feedback_layout: Option<TransformFeedbackLayoutDescriptor>,
    resource_bindings_layout: ResourceBindingsLayoutKind,
    primitive_assembly: PrimitiveAssembly,
    program_id: JsId,
    depth_test: Option<DepthTest>,
    stencil_test: Option<StencilTest>,
    scissor_region: Region2D,
    blending: Option<Blending>,
    viewport: Viewport,
    pub(crate) transform_feedback_data: Arc<UnsafeCell<Option<TransformFeedbackData>>>,
}

impl<V, R, Tf> GraphicsPipeline<V, R, Tf> {
    pub(crate) fn context_id(&self) -> usize {
        self.context_id
    }

    pub(crate) fn program_id(&self) -> JsId {
        self.program_id
    }

    /// Returns a description of the vertex input layout expected by the pipeline.
    ///
    /// See [VertexInputLayoutDescriptor] for details.
    pub fn vertex_attribute_layout(&self) -> &VertexInputLayoutDescriptor {
        &self.vertex_attribute_layout
    }

    /// Returns a description of the transform feedback layout used by the pipeline if the pipeline
    /// is capable of recording transform feedback, or `None` otherwise.
    ///
    /// See [TransformFeedbackLayoutDescriptor] for details.
    pub fn transform_feedback_layout(&self) -> Option<&TransformFeedbackLayoutDescriptor> {
        self.transform_feedback_layout.as_ref()
    }

    /// Returns the primitive assembly configuration used by the pipeline.
    ///
    /// See [PrimitiveAssembly] for details.
    pub fn primitive_assembly(&self) -> &PrimitiveAssembly {
        &self.primitive_assembly
    }

    /// Returns the depth test configuration used by the pipeline if the depth test is enabled, or
    /// `None` otherwise.
    ///
    /// See [DepthTest] for details.
    pub fn depth_test(&self) -> Option<&DepthTest> {
        self.depth_test.as_ref()
    }

    /// Returns the stencil test configuration used by the pipeline if the depth test is enabled, or
    /// `None` otherwise.
    ///
    /// See [StencilTest] for details.
    pub fn stencil_test(&self) -> Option<&StencilTest> {
        self.stencil_test.as_ref()
    }

    /// Returns the scissor region applied by this pipeline when outputting to a framebuffer.
    ///
    /// Fragments outside this region are discarded before the fragment processing stages.
    pub fn scissor_region(&self) -> &Region2D {
        &self.scissor_region
    }

    /// Returns the blending configuration used by the pipeline if the depth test is enabled, or
    /// `None` otherwise.
    ///
    /// See [Blending] for details.
    pub fn blending(&self) -> Option<&Blending> {
        self.blending.as_ref()
    }

    /// Returns the viewport configuration used by the pipeline.
    ///
    /// See [Viewport] for details.
    pub fn viewport(&self) -> &Viewport {
        &self.viewport
    }

    /// Returns a wrapped representation of this graphics pipeline that will record the output of
    /// the vertex transformation stage(s) for the pipeline in the attached
    /// `transform_feedback_buffers`.
    pub fn record_transform_feedback<Fb>(
        &mut self,
        transform_feedback_buffers: Fb,
    ) -> RecordTransformFeedback<V, R, Tf, Fb>
    where
        Tf: TypedTransformFeedbackLayout,
        Fb: TypedTransformFeedbackBuffers<Layout = Tf>,
    {
        RecordTransformFeedback {
            pipeline: self,
            buffers: transform_feedback_buffers
                .encode(&mut TransformFeedbackBuffersEncodingContext::new())
                .into_descriptors(),
            _marker: marker::PhantomData,
        }
    }
}

impl<V, Tf> GraphicsPipeline<V, Untyped, Tf> {
    /// Returns a minimal description of the resource bindings layout used by this pipeline.
    ///
    /// See [ResourceBindingsLayoutDescriptor] for details.
    pub fn resource_bindings_layout(&self) -> &ResourceBindingsLayoutDescriptor {
        match &self.resource_bindings_layout {
            ResourceBindingsLayoutKind::Minimal(layout) => layout,
            _ => unreachable!(),
        }
    }
}

impl<V, R, Tf> GraphicsPipeline<V, R, Tf>
where
    R: TypedResourceBindingsLayout,
{
    /// Returns a typed description of the resource bindings layout used by this pipeline.
    ///
    /// See [TypedResourceBindingsLayoutDescriptor] for details.
    pub fn resource_bindings_layout(&self) -> &TypedResourceBindingsLayoutDescriptor {
        match &self.resource_bindings_layout {
            ResourceBindingsLayoutKind::Typed(layout) => layout,
            _ => unreachable!(),
        }
    }
}

impl<V, R, Tf> GraphicsPipeline<V, R, Tf> {
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

        // TODO: need to reference state later, but keep reference to the program as well. I'm sure
        // there some obvious better way to do this, but I'm too tired to see it right now. This
        // should be safe for now (as we're referencing different parts of `state`).
        let mut program_cache = unsafe { (&mut *(state as *mut DynamicState)).program_cache_mut() };

        let transform_feedback_layout_key =
            descriptor.transform_feedback_layout.as_ref().map(|layout| {
                let mut hasher = FnvHasher::default();

                layout.hash(&mut hasher);

                hasher.finish()
            });

        let program = program_cache.get_or_create(
            ProgramKey {
                vertex_shader_id: descriptor.vertex_shader_data.id().unwrap(),
                fragment_shader_id: descriptor.fragment_shader_data.id().unwrap(),
                resource_bindings_layout: descriptor.resource_bindings_layout.key(),
                transform_feedback_layout_key,
            },
            &descriptor.transform_feedback_layout,
            gl,
        )?;

        let program_object = program.gl_object();

        if let Some(layout) = &descriptor.transform_feedback_layout {
            layout.check_compatibility(program_object, gl)?;
        }

        descriptor
            .vertex_attribute_layout
            .check_compatibility(program.attribute_slot_descriptors())?;

        state.use_program(Some(program_object)).apply(gl).unwrap();

        let updater = SlotBindingUpdater::new(gl, program_object);

        match &descriptor.resource_bindings_layout {
            ResourceBindingsLayoutKind::Minimal(layout) => {
                let bind_groups = layout.bind_groups();
                let mut iter = bind_groups.iter();

                let bind_group_0 = iter
                    .next()
                    .filter(|g| g.bind_group_index() == 0)
                    .ok_or(IncompatibleResources::MissingBindGroup(0))?;

                let bind_group_1 = iter
                    .next()
                    .filter(|g| g.bind_group_index() == 1)
                    .ok_or(IncompatibleResources::MissingBindGroup(1))?;

                'outer: for slot in program.resource_slot_descriptors() {
                    if slot.slot_type().is_kind(ResourceSlotKind::UniformBuffer) {
                        'inner: for descriptor in bind_group_0.slots() {
                            if &descriptor.slot_identifier == slot.identifier() {
                                if !descriptor.slot_kind.is_uniform_buffer() {
                                    return Err(IncompatibleResources::ResourceTypeMismatch(
                                        slot.identifier().clone(),
                                    )
                                    .into());
                                }

                                updater.update_slot_binding(slot, descriptor.slot_index as u32);

                                continue 'outer;
                            }
                        }

                        return Err(IncompatibleResources::MissingResource(
                            slot.identifier().clone(),
                        )
                        .into());
                    } else if slot.slot_type().is_kind(ResourceSlotKind::SampledTexture) {
                        'inner: for descriptor in bind_group_1.slots() {
                            if &descriptor.slot_identifier == slot.identifier() {
                                if !descriptor.slot_kind.is_sampled_texture() {
                                    return Err(IncompatibleResources::ResourceTypeMismatch(
                                        slot.identifier().clone(),
                                    )
                                    .into());
                                }

                                updater.update_slot_binding(slot, descriptor.slot_index as u32);

                                continue 'outer;
                            }
                        }

                        return Err(IncompatibleResources::MissingResource(
                            slot.identifier().clone(),
                        )
                        .into());
                    }

                    return Err(
                        IncompatibleResources::MissingResource(slot.identifier().clone()).into(),
                    );
                }
            }
            ResourceBindingsLayoutKind::Typed(layout) => {
                let mut iter = layout.bind_groups().iter();

                let bind_group_0 = iter
                    .next()
                    .filter(|g| g.bind_group_index() == 0)
                    .ok_or(IncompatibleResources::MissingBindGroup(0))?;

                let bind_group_1 = iter
                    .next()
                    .filter(|g| g.bind_group_index() == 1)
                    .ok_or(IncompatibleResources::MissingBindGroup(1))?;

                'outer: for slot in program.resource_slot_descriptors() {
                    match slot.slot_type() {
                        SlotType::UniformBlock(uniform_block_slot) => {
                            'inner: for descriptor in bind_group_0.slots() {
                                if &descriptor.slot_identifier == slot.identifier() {
                                    if let ResourceSlotType::UniformBuffer(memory_units) =
                                        descriptor.slot_type
                                    {
                                        uniform_block_slot.compatibility(memory_units).map_err(
                                            |e| {
                                                IncompatibleResources::IncompatibleInterface(
                                                    slot.identifier().clone(),
                                                    e,
                                                )
                                            },
                                        )?;
                                    } else {
                                        return Err(IncompatibleResources::ResourceTypeMismatch(
                                            slot.identifier().clone(),
                                        )
                                        .into());
                                    }

                                    updater.update_slot_binding(slot, descriptor.slot_index as u32);

                                    continue 'outer;
                                }
                            }
                        }
                        SlotType::TextureSampler(texture_sampler_slot) => {
                            'inner: for descriptor in bind_group_1.slots() {
                                if &descriptor.slot_identifier == slot.identifier() {
                                    if let ResourceSlotType::SampledTexture(tpe) =
                                        descriptor.slot_type
                                    {
                                        if tpe == texture_sampler_slot.kind() {
                                            updater.update_slot_binding(
                                                slot,
                                                descriptor.slot_index as u32,
                                            );

                                            continue 'outer;
                                        }
                                    }

                                    return Err(IncompatibleResources::ResourceTypeMismatch(
                                        slot.identifier().clone(),
                                    )
                                    .into());
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(GraphicsPipeline {
            _vertex_attribute_layout_marker: marker::PhantomData,
            _resources_marker: marker::PhantomData,
            _transform_feedback_varyings_marker: marker::PhantomData,
            context_id: context.id(),
            dropper: Box::new(context.clone()),
            vertex_shader_data: descriptor.vertex_shader_data.clone(),
            fragment_shader_data: descriptor.fragment_shader_data.clone(),
            vertex_attribute_layout: descriptor.vertex_attribute_layout.clone(),
            transform_feedback_layout: descriptor.transform_feedback_layout.clone(),
            resource_bindings_layout: descriptor.resource_bindings_layout.clone(),
            primitive_assembly: descriptor.primitive_assembly.clone(),
            program_id: JsId::from_abi(program_object.into_abi()),
            depth_test: descriptor.depth_test.clone(),
            stencil_test: descriptor.stencil_test.clone(),
            scissor_region: descriptor.scissor_region.clone(),
            blending: descriptor.blending.clone(),
            viewport: descriptor.viewport.clone(),
            transform_feedback_data: Arc::new(UnsafeCell::new(None)),
        })
    }
}

pub struct RecordTransformFeedback<'a, V, R, Tf, Fb> {
    pub(crate) pipeline: &'a mut GraphicsPipeline<V, R, Tf>,
    pub(crate) buffers: BufferDescriptors,
    _marker: marker::PhantomData<Fb>,
}

pub(crate) struct TransformFeedbackData {
    pub(crate) id: JsId,
    pub(crate) state: TransformFeedbackState,
    pub(crate) buffers: BufferDescriptors,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(crate) enum TransformFeedbackState {
    Inactive,
    Recording,
    Paused,
}

/// Error returned when trying to create a graphics pipeline and the shaders fail to link.
///
/// See [RenderingContext::create_graphics_pipeline].
#[derive(Debug)]
pub struct ShaderLinkingError {
    pub(crate) error: String,
}

trait GraphicsPipelineDropper {
    fn drop_graphics_pipeline(
        &self,
        id: JsId,
        transform_feedback_data: Arc<UnsafeCell<Option<TransformFeedbackData>>>,
    );
}

impl<T> GraphicsPipelineDropper for T
where
    T: RenderingContext,
{
    fn drop_graphics_pipeline(
        &self,
        program_id: JsId,
        transform_feedback_data: Arc<UnsafeCell<Option<TransformFeedbackData>>>,
    ) {
        self.submit(GraphicsPipelineDropCommand {
            program_id,
            transform_feedback_data,
        });
    }
}

impl<V, R, Tf> Drop for GraphicsPipeline<V, R, Tf> {
    fn drop(&mut self) {
        self.dropper
            .drop_graphics_pipeline(self.program_id, self.transform_feedback_data.clone());
    }
}

struct GraphicsPipelineDropCommand {
    program_id: JsId,
    transform_feedback_data: Arc<UnsafeCell<Option<TransformFeedbackData>>>,
}

unsafe impl GpuTask<Connection> for GraphicsPipelineDropCommand {
    type Output = ();

    fn context_id(&self) -> ContextId {
        ContextId::Any
    }

    fn progress(&mut self, connection: &mut Connection) -> Progress<Self::Output> {
        unsafe { JsId::into_value(self.program_id) };

        let transform_feedback_data = unsafe { &mut *self.transform_feedback_data.get() };

        if let Some(transform_feedback_data) = transform_feedback_data.as_ref() {
            let (gl, state) = unsafe { connection.unpack_mut() };

            unsafe {
                let transform_feedback =
                    JsId::into_value(transform_feedback_data.id).unchecked_into();

                if transform_feedback_data.state != TransformFeedbackState::Inactive {
                    state
                        .bind_transform_feedback(Some(&transform_feedback))
                        .apply(gl)
                        .unwrap();

                    gl.end_transform_feedback();
                }

                state.bind_transform_feedback(None).apply(gl).unwrap();

                gl.delete_transform_feedback(Some(&transform_feedback));
            }
        }

        Progress::Finished(())
    }
}
