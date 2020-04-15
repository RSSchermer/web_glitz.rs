use std::hash::{Hash, Hasher};
use std::mem;
use std::sync::Arc;

use crate::buffer::{BufferData, BufferView};

/// Describes an input source for vertex attribute data.
#[derive(Clone)]
pub(crate) struct BufferDescriptor {
    pub(crate) buffer_data: Arc<BufferData>,
    pub(crate) offset_in_bytes: u32,
    pub(crate) size_in_bytes: u32,
}

impl BufferDescriptor {
    /// Creates a [VertexInputDescriptor] from a [BufferView] of a typed slice, with the given
    /// [InputRate].
    ///
    /// The [offset_in_bytes] of the [VertexInputDescriptor] is the offset in bytes of the [Buffer]
    /// region viewed by the [BufferView] relative to the start of the buffer. The [size_in_bytes]
    /// of the [VertexInputDescriptor] is the size in bytes of the buffer region viewed by the
    /// [BufferView]. The [stride_in_bytes] of the [VertexInputDescriptor] is
    /// `std::mem::size_of::<T>`, where `T` is the element type of the slice viewed by the
    /// [BufferView].
    pub(crate) fn from_buffer_view<T>(buffer_view: BufferView<[T]>) -> Self {
        BufferDescriptor {
            buffer_data: buffer_view.buffer_data().clone(),
            offset_in_bytes: buffer_view.offset_in_bytes() as u32,
            size_in_bytes: (mem::size_of::<T>() * buffer_view.len()) as u32,
        }
    }
}

impl PartialEq for BufferDescriptor {
    fn eq(&self, other: &Self) -> bool {
        self.offset_in_bytes == other.offset_in_bytes
            && self.size_in_bytes == other.size_in_bytes
            && self.buffer_data.id() == other.buffer_data.id()
    }
}

impl Hash for BufferDescriptor {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.buffer_data.id().hash(state);
        self.offset_in_bytes.hash(state);
        self.size_in_bytes.hash(state);
    }
}
