use crate::buffer::{BufferData, Buffer, BufferView};
use std::sync::Arc;
use std::mem;

use web_sys::WebGl2RenderingContext as Gl;

pub unsafe trait IndexFormat {
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

pub unsafe trait IndexBufferDescription {
    type Format: IndexFormat;

    fn descriptor(&self) -> Option<IndexBufferDescriptor>;
}

pub struct IndexBufferDescriptor {
    pub(crate) buffer_data: Arc<BufferData>,
    pub(crate) index_type: IndexType,
    pub(crate) offset: u32,
    pub(crate) len: u32,
}

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
    type Format = u16;

    fn descriptor(&self) -> Option<IndexBufferDescriptor> {
        None
    }
}

unsafe impl<'a, F> IndexBufferDescription for &'a Buffer<[F]>
    where
        F: IndexFormat,
{
    type Format = F;

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
    type Format = F;

    fn descriptor(&self) -> Option<IndexBufferDescriptor> {
        Some(IndexBufferDescriptor {
            buffer_data: self.buffer_data().clone(),
            index_type: F::TYPE,
            offset: (self.offset_in_bytes() / mem::size_of::<F>()) as u32,
            len: self.len() as u32,
        })
    }
}
