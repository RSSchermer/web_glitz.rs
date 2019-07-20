use std::mem;
use std::sync::Arc;

use web_sys::WebGl2RenderingContext as Gl;

use crate::buffer::{Buffer, BufferData, BufferView};
use std::hash::{Hash, Hasher};

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

/// Describes a data source that can be used to provide indexing data to a draw command.
///
/// See [ActiveGraphicsPipeline::bind_index_buffer] for details.
/// Types that implement this trait can be used as an index source for a [VertexArray].
///
/// If [descriptor] instead returns `None`, then the [VertexArray] is not indexed: the vertices in
/// the vertex stream appear in an order defined by the [VertexArray]'s vertex input state.
///
/// If [descriptor] returns an [IndexBufferDescriptor], then the [VertexArray] is indexed using the
/// indices in the index buffer:
pub unsafe trait IndexBuffer {
    /// Returns an [IndexBufferDescriptor].
    fn encode<'a>(&self, context: &'a mut IndexBufferEncodingContext) -> IndexBufferEncoding<'a>;
}

pub struct IndexBufferEncodingContext(());

impl IndexBufferEncodingContext {
    pub(crate) fn new() -> Self {
        IndexBufferEncodingContext(())
    }
}

pub struct IndexBufferEncoding<'a> {
    context: &'a mut IndexBufferEncodingContext,
    descriptor: IndexBufferDescriptor
}

impl<'a> IndexBufferEncoding<'a> {
    pub fn from_typed_index_buffer<'b, B, T>(context: &'a mut IndexBufferEncodingContext, buffer: B) -> Self where B: Into<BufferView<'b, [T]>>, T: IndexFormat + 'b {
        let view = buffer.into();

        IndexBufferEncoding {
            context,
            descriptor: IndexBufferDescriptor {
                buffer_data: view.buffer_data().clone(),
                index_type: T::TYPE,
                offset: view.offset_in_bytes() as u32,
                len: view.len() as u32
            }
        }
    }

    pub(crate) fn into_descriptor(self) -> IndexBufferDescriptor {
        self.descriptor
    }
}

/// Describes a [Buffer] region that contains data that may be used to index a [VertexArray].
///
/// See also [IndexBufferDescription].
#[derive(Clone)]
pub(crate) struct IndexBufferDescriptor {
    pub(crate) buffer_data: Arc<BufferData>,
    pub(crate) index_type: IndexType,
    pub(crate) offset: u32,
    pub(crate) len: u32,
}

impl Hash for IndexBufferDescriptor {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.buffer_data.id().unwrap().hash(state);
        self.index_type.hash(state);
        self.offset.hash(state);
        self.len.hash(state);
    }
}

/// Enumerates the available type encodings for [VertexArray] indices.
#[derive(Clone, Copy, PartialEq, Hash, Debug)]
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

unsafe impl<'a, F> IndexBuffer for &'a Buffer<[F]>
where
    F: IndexFormat,
{
    fn encode<'b>(&self, context: &'b mut IndexBufferEncodingContext) -> IndexBufferEncoding<'b> {
        IndexBufferEncoding::from_typed_index_buffer(context, *self)
    }
}

unsafe impl<'a, F> IndexBuffer for BufferView<'a, [F]>
where
    F: IndexFormat,
{
    fn encode<'b>(&self, context: &'b mut IndexBufferEncodingContext) -> IndexBufferEncoding<'b> {
        IndexBufferEncoding::from_typed_index_buffer(context, *self)
    }
}
