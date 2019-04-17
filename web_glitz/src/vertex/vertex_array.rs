use crate::vertex::{VertexInputStateDescription, IndexBufferDescription, VertexAttributeLayout, IndexBufferDescriptor, VertexInputDescriptor, IndexType};
use std::sync::Arc;
use std::marker;
use crate::runtime::{RenderingContext, Connection};
use crate::util::{JsId, arc_get_mut_unchecked};
use crate::buffer::BufferData;
use crate::vertex::vertex_input_state_description::VertexAttributeDescriptor;
use std::borrow::Borrow;
use crate::task::{GpuTask, ContextId, Progress};
use crate::runtime::state::ContextUpdate;

use wasm_bindgen::JsCast;

pub struct VertexArrayDescriptor<V, I>
    where
        V: VertexInputStateDescription,
        I: IndexBufferDescription,
{
    pub vertex_input_state: V,
    pub indices: I,
}

pub struct Instanced<T>(pub(crate) T, pub(crate) usize);

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
            L: VertexAttributeLayout
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
                    id: None,
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
                    id: None,
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

    pub fn len(&self) -> usize {
        self.len
    }

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

pub trait VertexArrayRange {
    fn range<'a, L>(
        self,
        vertex_array: &VertexArraySlice<'a, L>,
    ) -> Option<VertexArraySlice<'a, L>>;

    unsafe fn range_unchecked<'a, L>(
        self,
        vertex_array: &VertexArraySlice<'a, L>,
    ) -> VertexArraySlice<'a, L>;
}

#[derive(Clone, Copy)]
pub struct VertexArraySlice<'a, L> {
    pub(crate) vertex_array: &'a VertexArray<L>,
    pub(crate) offset: usize,
    pub(crate) len: usize,
}

impl<'a, L> VertexArraySlice<'a, L> {
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn range<R>(&self, range: R) -> Option<VertexArraySlice<L>>
        where
            R: VertexArrayRange,
    {
        range.range(self)
    }

    pub unsafe fn range_unchecked<R>(&self, range: R) -> VertexArraySlice<L>
        where
            R: VertexArrayRange,
    {
        range.range_unchecked(self)
    }

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
    id: Option<JsId>,
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
        self.id
    }

    pub(crate) fn context_id(&self) -> usize {
        self.context_id
    }
}

impl Drop for VertexArrayData {
    fn drop(&mut self) {
        if let Some(id) = self.id {
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

unsafe impl<A, I> GpuTask<Connection> for VertexArrayAllocateCommand<A, I> where A: Borrow<[&'static [VertexAttributeDescriptor]]>, I: Borrow<[VertexInputDescriptor]> {
    type Output = ();

    fn context_id(&self) -> ContextId {
        ContextId::Any
    }

    fn progress(&mut self, connection: &mut Connection) -> Progress<Self::Output> {
        let (gl, state) = unsafe { connection.unpack_mut() };
        let data = unsafe { arc_get_mut_unchecked(&mut self.data) };
        let vao = gl.create_vertex_array().unwrap();

        state.set_bound_vertex_array(Some(&vao)).apply(gl).unwrap();

        let iter = self.attribute_layout.borrow().iter().zip(self.input_descriptors.borrow().iter());

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
                attribute_descriptor.apply(gl, input_descriptor.stride_in_bytes as i32,
                                           input_descriptor.offset_in_bytes as i32,
                                           input_descriptor.input_rate);
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

        data.id = Some(JsId::from_value(vao.into()));

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
