use std::sync::Arc;
use crate::vertex::vertex_array::VertexArrayData;
use crate::vertex::{IndexType, VertexArray, VertexArraySlice, Instanced};

pub trait VertexStreamDescription {
    type AttributeLayout;

    fn descriptor(&self) -> VertexStreamDescriptor;
}

#[derive(Clone)]
pub struct VertexStreamDescriptor {
    pub(crate) vertex_array_data: Arc<VertexArrayData>,
    pub(crate) offset: usize,
    pub(crate) count: usize,
    pub(crate) instance_count: usize,
}

impl VertexStreamDescriptor {
    pub(crate) fn index_type(&self) -> Option<IndexType> {
        self.vertex_array_data.index_type
    }
}

impl<L> VertexStreamDescription for VertexArray<L> {
    type AttributeLayout = L;

    fn descriptor(&self) -> VertexStreamDescriptor {
        VertexStreamDescriptor {
            vertex_array_data: self.data.clone(),
            offset: 0,
            count: self.len,
            instance_count: 1,
        }
    }
}

impl<'a, L> VertexStreamDescription for VertexArraySlice<'a, L> {
    type AttributeLayout = L;

    fn descriptor(&self) -> VertexStreamDescriptor {
        VertexStreamDescriptor {
            vertex_array_data: self.vertex_array.data.clone(),
            offset: self.offset,
            count: self.len,
            instance_count: 1,
        }
    }
}

impl<'a, L> VertexStreamDescription for Instanced<VertexArraySlice<'a, L>> {
    type AttributeLayout = L;

    fn descriptor(&self) -> VertexStreamDescriptor {
        let Instanced(slice, instance_count) = self;

        VertexStreamDescriptor {
            vertex_array_data: slice.vertex_array.data.clone(),
            offset: slice.offset,
            count: slice.len,
            instance_count: *instance_count,
        }
    }
}
