use std::mem;
use std::sync::Arc;

use web_sys::WebGl2RenderingContext as Gl;

use crate::buffer::{Buffer, BufferData, BufferView};

/// Trait implemented for types that can be used as indices for a [VertexArray] encoded in the
/// associated [IndexType].
pub unsafe trait IndexFormat {
    /// The [IndexType] associated with this [IndexFormat].
    const TYPE: IndexType;
}

unsafe impl IndexFormat for u8 {
    const TYPE: IndexType = IndexType::UnsignedByte;
}

unsafe impl IndexFormat for u16 {
    const TYPE: IndexType = IndexType::UnsignedShort;
}

unsafe impl IndexFormat for u32 {
    const TYPE: IndexType = IndexType::UnsignedInt;
}

/// Trait implemented for types that describe a data source for [VertexArray] indices.
///
/// Types that implement this trait can be used as an index source for a [VertexArray].
///
/// If [descriptor] instead returns `None`, then the [VertexArray] is not indexed: the vertices in
/// the vertex stream appear in an order defined by the [VertexArray]'s vertex input state.
///
/// If [descriptor] returns an [IndexBufferDescriptor], then the [VertexArray] is indexed using the
/// indices in the index buffer: the indices specify the vertices that appear in a vertex stream
/// produced from the [VertexArray]. For example, if the first index is `8`, then the first vertex
/// in the vertex stream is the 9th vertex defined by the [VertexArray]'s vertex input state.
///
/// This trait is notably implemented for the empty tuple `()`, which indicates the absence of an
/// index buffer ([descriptor] returns `None`): any [VertexArrayDescriptor] that uses `()` for its
/// indices, will produce a [VertexArray] that does not use indexing.
pub unsafe trait IndexBufferDescription {
    /// Returns an [IndexBufferDescriptor] if this [IndexBufferDescription] describes a source of
    /// index data for a [VertexArray], or `None` if a [VertexArray] using this description does
    /// not use indexing.
    fn descriptor(&self) -> Option<IndexBufferDescriptor>;
}

/// Describes a [Buffer] region that contains data that may be used to index a [VertexArray].
///
/// See also [IndexBufferDescription].
pub struct IndexBufferDescriptor {
    pub(crate) buffer_data: Arc<BufferData>,
    pub(crate) index_type: IndexType,
    pub(crate) offset: u32,
    pub(crate) len: u32,
}

/// Enumerates the available type encodings for [VertexArray] indices.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum IndexType {
    UnsignedByte,
    UnsignedShort,
    UnsignedInt,
}

impl IndexType {
    pub(crate) fn id(&self) -> u32 {
        match self {
            IndexType::UnsignedByte => Gl::UNSIGNED_BYTE,
            IndexType::UnsignedShort => Gl::UNSIGNED_SHORT,
            IndexType::UnsignedInt => Gl::UNSIGNED_INT,
        }
    }

    pub(crate) fn size_in_bytes(&self) -> u32 {
        match self {
            IndexType::UnsignedByte => 1,
            IndexType::UnsignedShort => 2,
            IndexType::UnsignedInt => 4,
        }
    }
}

unsafe impl<'a> IndexBufferDescription for () {
    fn descriptor(&self) -> Option<IndexBufferDescriptor> {
        None
    }
}

unsafe impl<'a, F> IndexBufferDescription for &'a Buffer<[F]>
where
    F: IndexFormat,
{
    fn descriptor(&self) -> Option<IndexBufferDescriptor> {
        Some(IndexBufferDescriptor {
            buffer_data: self.data().clone(),
            index_type: F::TYPE,
            offset: 0,
            len: self.len() as u32,
        })
    }
}

unsafe impl<'a, F> IndexBufferDescription for BufferView<'a, [F]>
where
    F: IndexFormat,
{
    fn descriptor(&self) -> Option<IndexBufferDescriptor> {
        Some(IndexBufferDescriptor {
            buffer_data: self.buffer_data().clone(),
            index_type: F::TYPE,
            offset: (self.offset_in_bytes() / mem::size_of::<F>()) as u32,
            len: self.len() as u32,
        })
    }
}
