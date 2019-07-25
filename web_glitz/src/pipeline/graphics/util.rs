use std::hash::{Hash, Hasher};
use std::mem::{ManuallyDrop, MaybeUninit};
use std::ops::Deref;
use std::sync::Arc;
use std::{mem, ptr};

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

impl Hash for BufferDescriptor {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.buffer_data.id().hash(state);
        self.offset_in_bytes.hash(state);
        self.size_in_bytes.hash(state);
    }
}

pub(crate) struct BufferDescriptors {
    storage: ManuallyDrop<[BufferDescriptor; 16]>,
    len: usize,
}

impl BufferDescriptors {
    pub fn new() -> Self {
        BufferDescriptors {
            storage: unsafe {
                ManuallyDrop::new([
                    MaybeUninit::uninit().assume_init(),
                    MaybeUninit::uninit().assume_init(),
                    MaybeUninit::uninit().assume_init(),
                    MaybeUninit::uninit().assume_init(),
                    MaybeUninit::uninit().assume_init(),
                    MaybeUninit::uninit().assume_init(),
                    MaybeUninit::uninit().assume_init(),
                    MaybeUninit::uninit().assume_init(),
                    MaybeUninit::uninit().assume_init(),
                    MaybeUninit::uninit().assume_init(),
                    MaybeUninit::uninit().assume_init(),
                    MaybeUninit::uninit().assume_init(),
                    MaybeUninit::uninit().assume_init(),
                    MaybeUninit::uninit().assume_init(),
                    MaybeUninit::uninit().assume_init(),
                    MaybeUninit::uninit().assume_init(),
                ])
            },
            len: 0,
        }
    }

    pub fn push(&mut self, descriptor: BufferDescriptor) {
        self.storage[self.len] = descriptor;
        self.len += 1;
    }
}

impl Deref for BufferDescriptors {
    type Target = [BufferDescriptor];

    fn deref(&self) -> &Self::Target {
        &self.storage[0..self.len]
    }
}

impl Drop for BufferDescriptors {
    fn drop(&mut self) {
        for vertex_buffer in self.storage[0..self.len].iter_mut() {
            unsafe {
                ptr::drop_in_place(vertex_buffer as *mut BufferDescriptor);
            }
        }
    }
}
