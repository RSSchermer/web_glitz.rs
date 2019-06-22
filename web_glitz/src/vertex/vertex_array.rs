use crate::buffer::BufferData;
use crate::runtime::state::ContextUpdate;
use crate::runtime::{Connection, RenderingContext};
use crate::task::{ContextId, GpuTask, Progress};
use crate::util::JsId;
use crate::vertex::{
    IndexBufferDescription, IndexBufferDescriptor, IndexType, VertexAttributeDescriptor,
    VertexAttributeLayout, VertexInputDescriptor, VertexInputStateDescription,
};
use std::borrow::Borrow;
use std::marker;
use std::sync::Arc;

use std::cell::UnsafeCell;
use wasm_bindgen::JsCast;

/// Provides the information necessary for the creation of a [VertexArray].
///
/// See [RenderingContext::create_vertex_array] for examples of its use.
pub struct VertexArrayDescriptor<V, I>
where
    V: VertexInputStateDescription,
    I: IndexBufferDescription,
{
    /// The vertex input state for the [VertexArray], must be a [VertexInputStateDescription].
    ///
    /// See [VertexInputStateDescription] for details on defining a valid description. Note that
    /// [VertexInputStateDescription] is implemented for a [Buffer] that holds any array of a type
    /// that implements [Vertex] (e.g. `Buffer<[MyVertex]>` where `MyVertex` implements [Vertex]).
    /// It is also implemented for any tuple of [Buffer]s that hold arrays of types that implement
    /// [Vertex] (e.g. `(Buffer<[MyVertexA]>, Buffer<[MyVertexB]>)` where `MyVertexA` implements
    /// [Vertex] and `MyVertexB` implements [Vertex]).
    pub vertex_input_state: V,

    /// The index data for the [VertexArray], must be an [IndexBufferDescription].
    ///
    /// See [IndexBufferDescription] for details.
    pub indices: I,
}

/// Wraps a [VertexArraySlice] to represent an instanced vertex stream.
///
/// See [VertexArray::instanced] and [VertexArraySlice::instanced].
pub struct Instanced<T>(pub(crate) T, pub(crate) usize);

/// Encapsulates the state of a [VertexArray] from which vertices may be streamed as the input to
/// a graphics pipeline.
///
/// See [RenderingContext::create_vertex_array] for details on how a [VertexArray] is created.
///
/// A [VertexArray] may act as a [VertexStreamDescription] for a draw command:
///
/// ```
/// # use web_glitz::render_pass::{DefaultRenderTarget, DefaultRGBBuffer};
/// # use web_glitz::runtime::RenderingContext;
/// # use web_glitz::vertex::{Vertex, VertexArrayDescriptor};
/// # use web_glitz::buffer::UsageHint;
/// # use web_glitz::pipeline::graphics::GraphicsPipeline;
/// # fn wrapper<Rc, V>(
/// #     context: &Rc,
/// #     mut render_target: DefaultRenderTarget<DefaultRGBBuffer, ()>,
/// #     vertex_data: [V; 100],
/// #     graphics_pipeline: GraphicsPipeline<V, (), ()>
/// # )
/// # where
/// #     Rc: RenderingContext,
/// #     V: Vertex
/// # {
/// # let vertex_buffer = context.create_buffer(vertex_data, UsageHint::StaticDraw);
/// let vertex_array = context.create_vertex_array(&VertexArrayDescriptor {
///     vertex_input_state: &vertex_buffer,
///     indices: ()
/// });
///
/// let render_pass = context.create_render_pass(&mut render_target, |framebuffer| {
///     framebuffer.pipeline_task(&graphics_pipeline, |active_pipeline| {
///         active_pipeline.draw_command(&vertex_array, &());
///     })
/// });
/// # }
/// ```
///
/// Here `context` is a [RenderingContext], `vertex_buffer` is a [Buffer] that holds an array of
/// a type that implements [Vertex], `render_target` is a [RenderTargetDescription] and
/// `graphics_pipeline` is a [GraphicsPipeline] with a compatible vertex input layout  (see
/// [PipelineTask::draw_command] for details).
///
/// It is also possible to select just a sub-range of the [VertexArray] with [range] and use only
/// that sub-range as a [VertexStreamDescription], see [range] for details and an example.
///
/// Lastly, a [VertexStreamDescription] that uses instancing can be created with [instanced], see
/// [instanced] for details and an example (this also works in a similar way with just a sub-range,
/// see [VertexArrayRange::instanced]).
pub struct VertexArray<L> {
    pub(crate) data: Arc<VertexArrayData>,
    pub(crate) len: usize,
    _marker: marker::PhantomData<L>,
}

impl<L> VertexArray<L> {
    pub(crate) fn new<Rc, V, I>(context: &Rc, descriptor: &VertexArrayDescriptor<V, I>) -> Self
    where
        Rc: RenderingContext + Clone + 'static,
        V: VertexInputStateDescription<AttributeLayout = L>,
        I: IndexBufferDescription,
        L: VertexAttributeLayout,
    {
        let VertexArrayDescriptor {
            vertex_input_state,
            indices,
        } = descriptor;

        let attribute_layout = L::input_attribute_bindings();
        let input_descriptors = vertex_input_state.vertex_input_descriptors();
        let input_slice = input_descriptors.borrow();

        let mut buffer_pointers = Vec::with_capacity(input_slice.len());
        let mut vertex_count = None;

        for (_, input) in attribute_layout.borrow().iter().zip(input_slice.iter()) {
            let buffer_len = input.size_in_bytes / input.stride_in_bytes as u32;

            if let Some(len) = vertex_count {
                if buffer_len < len {
                    vertex_count = Some(buffer_len)
                }
            } else {
                vertex_count = Some(buffer_len)
            }

            buffer_pointers.push(input.buffer_data.clone());
        }

        let index_buffer_descriptor = indices.descriptor();

        let (data, len) = if let Some(ref index_buffer_descriptor) = index_buffer_descriptor {
            (
                Arc::new(VertexArrayData {
                    id: UnsafeCell::new(None),
                    context_id: context.id(),
                    dropper: Box::new(context.clone()),
                    vertex_buffer_pointers: buffer_pointers,
                    index_buffer_pointer: Some(index_buffer_descriptor.buffer_data.clone()),
                    index_type: Some(index_buffer_descriptor.index_type),
                    offset: index_buffer_descriptor.offset,
                }),
                index_buffer_descriptor.len,
            )
        } else {
            (
                Arc::new(VertexArrayData {
                    id: UnsafeCell::new(None),
                    context_id: context.id(),
                    dropper: Box::new(context.clone()),
                    vertex_buffer_pointers: buffer_pointers,
                    index_buffer_pointer: None,
                    index_type: None,
                    offset: 0,
                }),
                vertex_count.unwrap_or(0),
            )
        };

        context.submit(VertexArrayAllocateCommand {
            data: data.clone(),
            input_descriptors,
            attribute_layout,
            index_buffer_descriptor,
        });

        VertexArray {
            data,
            len: len as usize,
            _marker: marker::PhantomData,
        }
    }

    /// Returns the number of vertices that can be streamed from this [VertexArray] without
    /// instancing.
    ///
    /// If this vertex array uses indexing, then this number is equal to the number of indices. If
    /// the vertex array does not use indexing, then the vertex input source that can provide the
    /// fewest input sets determines this number and is calculated as
    /// `size_in_bytes / stride_in_bytes` (rounded down), where `size_in_bytes` is the size in
    /// bytes of the vertex input source (see [VertexInputDescriptor::size_in_bytes] and
    /// `stride_in_bytes` is the stride in bytes of the vertex input source (see
    /// [VertexInputDescriptor::stride_in_bytes]).
    ///
    /// This number is based only on how many "per vertex" attribute inputs can be generated from
    /// the input data, "per instance" attribute inputs do not affect this count (see
    /// [VertexInputDescriptor::input_rate]).
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns a reference to a sub-range of the [VertexArray], or `None` if the range is out of
    /// bounds.
    ///
    /// This may be used to limit the vertices streamed from this array to only a sub-range of the
    /// array when it is used as a vertex input stream description for a graphics pipeline.
    ///
    /// See also [range_unchecked] for an unsafe variant that does not do any bounds checking.
    ///
    /// # Example
    ///
    /// ```
    /// # use web_glitz::render_pass::{DefaultRenderTarget, DefaultRGBBuffer};
    /// # use web_glitz::runtime::RenderingContext;
    /// # use web_glitz::vertex::{Vertex, VertexArrayDescriptor};
    /// # use web_glitz::buffer::UsageHint;
    /// # use web_glitz::pipeline::graphics::GraphicsPipeline;
    /// # fn wrapper<Rc, V>(
    /// #     context: &Rc,
    /// #     render_target: DefaultRenderTarget<DefaultRGBBuffer, ()>,
    /// #     vertex_data: [V; 100],
    /// #     graphics_pipeline: GraphicsPipeline<V, (), ()>
    /// # )
    /// # where
    /// #     Rc: RenderingContext,
    /// #     V: Vertex
    /// # {
    /// # let vertex_buffer = context.create_buffer(vertex_data, UsageHint::StaticDraw);
    /// let vertex_array = context.create_vertex_array(&VertexArrayDescriptor {
    ///     vertex_input_state: &vertex_buffer,
    ///     indices: ()
    /// });
    ///
    /// assert_eq!(vertex_array.len(), 100);
    ///
    /// let range_in_bounds = vertex_array.range(10..20);
    ///
    /// assert!(range_in_bounds.is_some());
    ///
    /// let range_out_of_bounds = vertex_array.range(10..101);
    ///
    /// assert!(range_out_of_bounds.is_none());
    ///
    /// let range = range_in_bounds.unwrap();
    ///
    /// assert_eq!(range.len(), 10);
    ///
    /// let render_pass = context.create_render_pass(render_target, |framebuffer| {
    ///     framebuffer.pipeline_task(&graphics_pipeline, |active_pipeline| {
    ///         active_pipeline.draw_command(&range, &());
    ///     })
    /// });
    /// # }
    /// ```
    pub fn range<R>(&self, range: R) -> Option<VertexArraySlice<L>>
    where
        R: VertexArrayRange,
    {
        range.range(&VertexArraySlice {
            vertex_array: self,
            offset: 0,
            len: self.len,
        })
    }

    /// Returns a reference to a sub-range of the [VertexArray] without doing any bounds checks.
    ///
    /// This may be used to limit the vertices streamed from this array to only a sub-range of the
    /// array when it is used as a vertex input stream description for a graphics pipeline.
    ///
    /// See also [range] for a safe variant that does bounds checking.
    ///
    /// # Example
    ///
    /// ```
    /// # use web_glitz::render_pass::{DefaultRenderTarget, DefaultRGBBuffer};
    /// # use web_glitz::runtime::RenderingContext;
    /// # use web_glitz::vertex::{Vertex, VertexArrayDescriptor};
    /// # use web_glitz::buffer::UsageHint;
    /// # use web_glitz::pipeline::graphics::GraphicsPipeline;
    /// # fn wrapper<Rc, V>(
    /// #     context: &Rc,
    /// #     render_target: DefaultRenderTarget<DefaultRGBBuffer, ()>,
    /// #     vertex_data: [V; 100],
    /// #     graphics_pipeline: GraphicsPipeline<V, (), ()>
    /// # )
    /// # where
    /// #     Rc: RenderingContext,
    /// #     V: Vertex
    /// # {
    /// # let vertex_buffer = context.create_buffer(vertex_data, UsageHint::StaticDraw);
    /// let vertex_array = context.create_vertex_array(&VertexArrayDescriptor {
    ///     vertex_input_state: &vertex_buffer,
    ///     indices: ()
    /// });
    ///
    /// assert_eq!(vertex_array.len(), 100);
    ///
    /// let range = unsafe { vertex_array.range_unchecked(10..20) };
    ///
    /// assert_eq!(range.len(), 10);
    ///
    /// let render_pass = context.create_render_pass(render_target, |framebuffer| {
    ///     framebuffer.pipeline_task(&graphics_pipeline, |active_pipeline| {
    ///         active_pipeline.draw_command(&range, &());
    ///     })
    /// });
    /// # }
    /// ```
    pub unsafe fn range_unchecked<R>(&self, range: R) -> VertexArraySlice<L>
    where
        R: VertexArrayRange,
    {
        range.range_unchecked(&VertexArraySlice {
            vertex_array: self,
            offset: 0,
            len: self.len,
        })
    }

    /// Wraps this [VertexArray] to represent an instanced vertex stream with the given
    /// `instance_count`.
    ///
    /// This may be used to describe an instanced vertex input stream for a graphics pipeline.
    ///
    /// # Example
    ///
    /// ```
    /// # use web_glitz::render_pass::{DefaultRenderTarget, DefaultRGBBuffer};
    /// # use web_glitz::runtime::RenderingContext;
    /// # use web_glitz::vertex::{Vertex, VertexArrayDescriptor};
    /// # use web_glitz::buffer::UsageHint;
    /// # use web_glitz::pipeline::graphics::GraphicsPipeline;
    /// # fn wrapper<Rc, V>(
    /// #     context: &Rc,
    /// #     render_target: DefaultRenderTarget<DefaultRGBBuffer, ()>,
    /// #     vertex_data: [V; 100],
    /// #     graphics_pipeline: GraphicsPipeline<V, (), ()>
    /// # )
    /// # where
    /// #     Rc: RenderingContext,
    /// #     V: Vertex
    /// # {
    /// # let vertex_buffer = context.create_buffer(vertex_data, UsageHint::StaticDraw);
    /// let vertex_array = context.create_vertex_array(&VertexArrayDescriptor {
    ///     vertex_input_state: &vertex_buffer,
    ///     indices: ()
    /// });
    ///
    /// let render_pass = context.create_render_pass(render_target, |framebuffer| {
    ///     framebuffer.pipeline_task(&graphics_pipeline, |active_pipeline| {
    ///         active_pipeline.draw_command(&vertex_array.instanced(10), &());
    ///     })
    /// });
    /// # }
    /// ```
    pub fn instanced(&self, instance_count: usize) -> Instanced<VertexArraySlice<L>> {
        Instanced(
            VertexArraySlice {
                vertex_array: self,
                offset: 0,
                len: self.len,
            },
            instance_count,
        )
    }
}

/// Helper trait for selecting a sub-range of a [VertexArray].
///
/// See [VertexArray::range] and [VertexArray::range_unchecked].
pub trait VertexArrayRange {
    /// Returns the output for this operation if in bounds, or `None` otherwise.
    fn range<'a, L>(
        self,
        vertex_array: &VertexArraySlice<'a, L>,
    ) -> Option<VertexArraySlice<'a, L>>;

    /// Returns the output for this operation, without performing any bounds checking.
    unsafe fn range_unchecked<'a, L>(
        self,
        vertex_array: &VertexArraySlice<'a, L>,
    ) -> VertexArraySlice<'a, L>;
}

/// A reference to a sub-range of the [VertexArray].
///
/// See [VertexArray::range] and [VertexArray::range_unchecked].
#[derive(Clone, Copy)]
pub struct VertexArraySlice<'a, L> {
    pub(crate) vertex_array: &'a VertexArray<L>,
    pub(crate) offset: usize,
    pub(crate) len: usize,
}

impl<'a, L> VertexArraySlice<'a, L> {
    /// Returns the number of vertices that can be streamed from this [VertexArraySlice] without
    /// instancing.
    ///
    /// See also [VertexArray::len].
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns a reference to a sub-range of the [VertexArray], or `None` if the range is out of
    /// bounds.
    ///
    /// See also [VertexArray::range].
    pub fn range<R>(&self, range: R) -> Option<VertexArraySlice<L>>
    where
        R: VertexArrayRange,
    {
        range.range(self)
    }

    /// Returns a reference to a sub-range of the [VertexArray] without doing any bounds checks.
    ///
    /// See also [VertexArray::range_unchecked].
    pub unsafe fn range_unchecked<R>(&self, range: R) -> VertexArraySlice<L>
    where
        R: VertexArrayRange,
    {
        range.range_unchecked(self)
    }

    /// Wraps this [VertexArraySlice] to represent an instanced vertex stream with the given
    /// `instance_count`.
    ///
    /// See also [VertexArray::instanced].
    pub fn instanced(&self, instance_count: usize) -> Instanced<VertexArraySlice<L>> {
        Instanced(
            VertexArraySlice {
                vertex_array: self.vertex_array,
                offset: self.offset,
                len: self.len,
            },
            instance_count,
        )
    }
}

trait VertexArrayObjectDropper {
    fn drop_vertex_array_object(&self, id: JsId);
}

impl<T> VertexArrayObjectDropper for T
where
    T: RenderingContext,
{
    fn drop_vertex_array_object(&self, id: JsId) {
        self.submit(VertexArrayDropCommand { id });
    }
}

pub(crate) struct VertexArrayData {
    id: UnsafeCell<Option<JsId>>,
    context_id: usize,
    dropper: Box<VertexArrayObjectDropper>,
    #[allow(dead_code)] // Just holding on to these so they don't get dropped prematurely
    vertex_buffer_pointers: Vec<Arc<BufferData>>,
    #[allow(dead_code)] // Just holding on to this so it doesn't get dropped prematurely
    index_buffer_pointer: Option<Arc<BufferData>>,
    pub(crate) index_type: Option<IndexType>,
    pub(crate) offset: u32,
}

impl VertexArrayData {
    pub(crate) fn id(&self) -> Option<JsId> {
        unsafe { *self.id.get() }
    }

    pub(crate) fn context_id(&self) -> usize {
        self.context_id
    }
}

impl Drop for VertexArrayData {
    fn drop(&mut self) {
        if let Some(id) = self.id() {
            self.dropper.drop_vertex_array_object(id);
        }
    }
}

struct VertexArrayAllocateCommand<A, I> {
    data: Arc<VertexArrayData>,
    input_descriptors: I,
    attribute_layout: A,
    index_buffer_descriptor: Option<IndexBufferDescriptor>,
}

unsafe impl<A, I> GpuTask<Connection> for VertexArrayAllocateCommand<A, I>
where
    A: Borrow<[&'static [VertexAttributeDescriptor]]>,
    I: Borrow<[VertexInputDescriptor]>,
{
    type Output = ();

    fn context_id(&self) -> ContextId {
        ContextId::Any
    }

    fn progress(&mut self, connection: &mut Connection) -> Progress<Self::Output> {
        let (gl, state) = unsafe { connection.unpack_mut() };
        let data = &self.data;
        let vao = gl.create_vertex_array().unwrap();

        state.set_bound_vertex_array(Some(&vao)).apply(gl).unwrap();

        let iter = self
            .attribute_layout
            .borrow()
            .iter()
            .zip(self.input_descriptors.borrow().iter());

        for (bind_group, input_descriptor) in iter {
            unsafe {
                input_descriptor
                    .buffer_data
                    .id()
                    .unwrap()
                    .with_value_unchecked(|buffer| {
                        state
                            .set_bound_array_buffer(Some(buffer))
                            .apply(gl)
                            .unwrap();
                    });
            }

            for attribute_descriptor in bind_group.iter() {
                attribute_descriptor.apply(
                    gl,
                    input_descriptor.stride_in_bytes as i32,
                    input_descriptor.offset_in_bytes as i32,
                    input_descriptor.input_rate,
                );
            }
        }

        if let Some(index_buffer_descriptor) = &self.index_buffer_descriptor {
            unsafe {
                index_buffer_descriptor
                    .buffer_data
                    .id()
                    .unwrap()
                    .with_value_unchecked(|buffer| {
                        state
                            .set_bound_element_array_buffer(Some(buffer))
                            .apply(gl)
                            .unwrap();
                    });
            }
        }

        unsafe {
            *data.id.get() = Some(JsId::from_value(vao.into()));
        }

        Progress::Finished(())
    }
}

struct VertexArrayDropCommand {
    id: JsId,
}

unsafe impl GpuTask<Connection> for VertexArrayDropCommand {
    type Output = ();

    fn context_id(&self) -> ContextId {
        ContextId::Any
    }

    fn progress(&mut self, connection: &mut Connection) -> Progress<Self::Output> {
        let (gl, _) = unsafe { connection.unpack() };
        let value = unsafe { JsId::into_value(self.id) };

        gl.delete_vertex_array(Some(&value.unchecked_into()));

        Progress::Finished(())
    }
}
