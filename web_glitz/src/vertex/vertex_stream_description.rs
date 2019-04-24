use crate::vertex::vertex_array::VertexArrayData;
use crate::vertex::{IndexType, Instanced, VertexArray, VertexArraySlice};
use std::sync::Arc;

/// Describes a stream of vertices that may serve as the input for a graphics pipeline.
pub trait VertexStreamDescription {
    /// Type associated with the vertex attribute layout of vertices in the vertex stream.
    type AttributeLayout;

    /// Returns a descriptor that encapsulates the state necessary for drawing with this vertex
    /// stream.
    fn descriptor(&self) -> VertexStreamDescriptor;
}

/// Describes a vertex stream that may serve as the input for a graphics pipeline.
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
