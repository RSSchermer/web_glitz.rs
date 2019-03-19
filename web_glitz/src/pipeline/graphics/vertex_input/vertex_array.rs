use crate::buffer::{Buffer, BufferData, BufferView};
use crate::pipeline::graphics::vertex_input::input_attribute_layout::InputAttributeLayout;
use crate::pipeline::graphics::vertex_input::{
    InputRate, VertexBufferDescription, VertexInputAttributeDescriptor,
};
use crate::runtime::{Connection, RenderingContext};
use crate::task::{GpuTask, Progress, ContextId};
use crate::util::{arc_get_mut_unchecked, JsId};
use fnv::FnvHasher;
use js_sys::buffer;
use std::sync::Arc;
use std::{marker, mem};

struct VertexBufferDescriptor {
    buffer_data: Arc<BufferData>,
    offset_in_bytes: u32,
    stride_in_bytes: i32,
    size_in_bytes: u32,
    input_rate: InputRate,
    attribute_offset: usize,
    attribute_count: usize,
}

pub struct VertexBuffersDescriptor {
    buffer_descriptors: Vec<VertexBufferDescriptor>,
    attribute_descriptors: Vec<VertexInputAttributeDescriptor>,
}

pub unsafe trait VertexBuffersDescription {
    type Layout;

    fn descriptor(&self) -> VertexBuffersDescriptor;
}

macro_rules! impl_vertex_buffers_description {
    ($($T:ident),*) => {
        unsafe impl<$($T),*> VertexBuffersDescription for ($($T),*)
        where
            $($T: VertexBufferDescription),*
        {
            type Layout = ($($T::Vertex),*);

            fn descriptor(&self) -> VertexBuffersDescriptor {
                let ($($T::Vertex),*) = self;
                let mut buffer_count = 0;
                let mut attribute_count = 0;

                $(
                    buffer_count += 1;
                    attribute_count += $T.input_attribute_descriptors().len();
                )*

                let mut buffer_descriptors = Vec::with_capacity(buffer_count);
                let mut attribute_descriptors = Vec::with_capacity(attribute_count);

                let mut attribute_offset = 0;

                $(
                    let attribute_count = $T.input_attribute_descriptors().len();

                    buffer_descriptors.push(VertexBufferDescriptor {
                        buffer_data: $T.buffer_view().buffer_data().clone(),
                        offset_in_bytes: $T.offset_in_bytes(),
                        stride_in_bytes: $T.stride_in_bytes() as i32,
                        size_in_bytes: $T.size_in_bytes(),
                        input_rate: $T.input_rate(),
                        attribute_offset,
                        attribute_count
                    });

                    attribute_offset += attribute_count;

                    for descriptor in $T.input_attribute_descriptors().iter() {
                        attribute_descriptors.push(descriptor.clone())
                    }
                )*

                VertexBuffersDescriptor {
                    buffer_descriptors,
                    attribute_descriptors
                }
            }
        }
    }
}

impl_vertex_buffers_description!(T0);
impl_vertex_buffers_description!(T0, T1);
impl_vertex_buffers_description!(T0, T1, T2);
impl_vertex_buffers_description!(T0, T1, T2, T3);
impl_vertex_buffers_description!(T0, T1, T2, T3, T4);
impl_vertex_buffers_description!(T0, T1, T2, T3, T4, T5);
impl_vertex_buffers_description!(T0, T1, T2, T3, T4, T5, T6);
impl_vertex_buffers_description!(T0, T1, T2, T3, T4, T5, T6, T7);
impl_vertex_buffers_description!(T0, T1, T2, T3, T4, T5, T6, T7, T8);
impl_vertex_buffers_description!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9);
impl_vertex_buffers_description!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
impl_vertex_buffers_description!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);
impl_vertex_buffers_description!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12);
impl_vertex_buffers_description!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13);
impl_vertex_buffers_description!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14);
impl_vertex_buffers_description!(
    T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15
);

pub unsafe trait IndexFormat {
    fn kind() -> IndexFormatKind;
}

unsafe impl IndexFormat for u8 {
    fn kind() -> IndexFormatKind {
        IndexFormatKind::UnsignedByte
    }
}

unsafe impl IndexFormat for u16 {
    fn kind() -> IndexFormatKind {
        IndexFormatKind::UnsignedShort
    }
}

unsafe impl IndexFormat for u32 {
    fn kind() -> IndexFormatKind {
        IndexFormatKind::UnsignedInt
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum IndexFormatKind {
    UnsignedByte,
    UnsignedShort,
    UnsignedInt
}

impl IndexFormatKind {
    pub(crate) fn id(&self) -> u32 {
        match self {
            IndexFormatKind::UnsignedByte => Gl::UNSIGNED_BYTE,
            IndexFormatKind::UnsignedShort => Gl::UNSIGNED_SHORT,
            IndexFormatKind::UnsignedInt => Gl::UNSIGNED_INT,
        }
    }

    pub(crate) fn size_in_bytes(&self) -> u32 {
        match self {
            IndexFormatKind::UnsignedByte => 1,
            IndexFormatKind::UnsignedShort => 2,
            IndexFormatKind::UnsignedInt => 4,
        }
    }
}

pub unsafe trait IndexBufferDescription {
    type Format: IndexFormat;

    fn descriptor(&self) -> IndexBufferDescriptor;
}

pub struct IndexBufferDescriptor {
    buffer_data: Arc<Bufferdata>,
    format_id: u32,
    offset: u32,
    len: u32,
}

unsafe impl<F> IndexBufferDescription for Buffer<[F]>
where
    F: IndexFormat,
{
    type Format = F;

    fn descriptor(&self) -> IndexBufferDescriptor {
        IndexBufferDescriptor {
            buffer_data: self.data().clone(),
            format_id: F::id(),
            offset: 0,
            len: self.len() as u32,
        }
    }
}

unsafe impl<F> IndexBufferDescription for BufferView<[F]>
where
    F: IndexFormat,
{
    type Format = F;

    fn descriptor(&self) -> IndexBufferDescriptor {
        IndexBufferDescriptor {
            buffer_data: self.data().clone(),
            format_id: F::id(),
            offset: (self.offset_in_bytes() / mem::size_of::<F>()) as u32,
            len: self.len() as u32,
        }
    }
}

pub struct VertexArrayDescriptor<V, I>
where
    V: VertexBufferDescription,
    I: IndexBufferDescription,
{
    pub vertex_buffers: V,
    pub index_buffer: Option<I>,
}

pub struct VertexArray<L> {
    data: Arc<VertexArrayData>,
    len: usize,
    _marker: marker::PhantomData<L>,
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
    vertex_buffer_pointers: Vec<Arc<BufferData>>,
    index_buffer_pointer: Option<Arc<BufferData>>,
    index_format_kind: Option<IndexFormatKind>,
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

impl<L> VertexArray<L> {
    pub(crate) fn new<Rc, V, I>(context: Rc, descriptor: VertexArrayDescriptor<V, I>) -> Self
    where
        Rc: RenderingContext + Clone + 'static,
        V: VertexBuffersDescription<Layout = L>,
        I: IndexBufferDescription,
    {
        let VertexArrayDescriptor {
            vertex_buffers,
            index_buffer,
        } = descriptor;

        let VertexBuffersDescriptor {
            buffer_descriptors,
            attribute_descriptors,
        } = vertex_buffers.descriptor();

        let mut buffer_pointers = Vec::with_capacity(buffer_descriptors.len());
        let mut vertex_count = None;

        for descriptor in buffer_descriptors.iter() {
            let buffer_len = descriptor.size_in_bytes / descriptor.stride_in_bytes();

            if let Some(len) = vertex_count {
                if buffer_len < len {
                    vertex_count = Some(buffer_len)
                }
            } else {
                vertex_count = Some(buffer_len)
            }

            buffer_pointers.push(descriptor.buffer_data.clone());
        }

        let index_buffer_descriptor = index_buffer.map(|b| b.descriptor());

        let data = Arc::new(VertexArrayData {
            id: None,
            context_id: context.id(),
            dropper: Box::new(context.clone()),
            vertex_buffer_pointers: buffer_pointers,
            index_buffer_pointer: index_buffer_descriptor.map(|d| d.buffer_data.clone()),
            index_format_kind: index_buffer_descriptor.map(|d| <I::Format as IndexFormat>::kind()),
        });

        let len = if let Some(descriptor) = index_buffer_descriptor {
            descriptor.len
        } else {
            vertex_count.unwrap_or(0)
        } as usize;

        context.submit(VertexArrayAllocateCommand {
            data: data.clone(),
            buffer_descriptors,
            attribute_descriptors,
            index_buffer_descriptor,
        });

        VertexArray {
            data,
            len,
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
        rante.range(VertexArraySlice {
            vertex_array: self,
            offset: 0,
            len: self.len,
        })
    }

    pub unsafe fn range_unchecked<R>(&self, range: R) -> Option<VertexArraySlice<L>>
    where
        R: VertexArrayRange,
    {
        rante.range_unchecked(VertexArraySlice {
            vertex_array: self,
            offset: 0,
            len: self.len,
        })
    }

    pub fn instanced(&self, instance_count: usize) -> Instanced<VertexArraySlice<L>> {
        Instanced {
            vertex_array: VertexArraySlice {
                vertex_array: self,
                offset: 0,
                len: self.len,
            },
        }
    }
}

pub trait VertexArrayRange {
    fn range<'a, L>(self, vertex_array: VertexArraySlice<L>) -> Option<VertexArraySlice<L>>;

    unsafe fn range_unchecked<'a, V, I>(
        self,
        vertex_array: VertexArraySlice<L>,
    ) -> VertexArraySlice<L>;
}

#[derive(Clone, Copy)]
pub struct VertexArraySlice<'a, L> {
    vertex_array: &'a VertexArray<L>,
    offset: usize,
    len: usize,
}

impl<L> VertexArraySlice<L> {
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn range<R>(&self, range: R) -> Option<VertexArraySlice<L>>
    where
        R: VertexArrayRange,
    {
        range.range(self)
    }

    pub unsafe fn range_unchecked<R>(&self, range: R) -> Option<VertexArraySlice<L>>
    where
        R: VertexArrayRange,
    {
        range.range_unchecked(self)
    }

    pub fn instanced(&self, instance_count: usize) -> InstancedVertexArraySlice<L> {
        InstancedVertexArraySlice {
            vertex_array: self,
            offset: self.offset,
            len: self.len,
            instance_count: usize,
        }
    }
}

#[derive(Clone, Copy)]
pub struct InstancedVertexArraySlice<'a, L> {
    vertex_array: &'a VertexArray<L>,
    offset: usize,
    len: usize,
    instance_count: usize,
}

pub trait VertexInputStreamDescription {
    type Layout;

    fn descriptor(&self) -> VertexInputStreamDescriptor;
}

#[derive(Clone)]
pub struct VertexInputStreamDescriptor {
    pub(crate) vertex_array_data: Arc<VertexArrayData>,
    pub(crate) offset: usize,
    pub(crate) count: usize,
    pub(crate) instance_count: usize,
}

impl VertexInputStreamDescriptor {
    pub(crate) fn index_format_kind(&self) -> Option<IndexFormatKind> {
        self.vertex_array_data.index_format_kind
    }
}

impl<L> VertexInputStreamDescription for VertexArray<L> {
    type Layout = L;

    fn descriptor(&self) -> VertexInputStreamDescriptor {
        VertexInputStreamDescriptor {
            vertex_array_data: self.data.clone(),
            offset: 0,
            count: self.len,
            instance_count: 1,
        }
    }
}

impl<'a, L> VertexInputStreamDescription for VertexArraySlice<'a, L> {
    type Layout = L;

    fn descriptor(&self) -> VertexInputStreamDescriptor {
        VertexInputStreamDescriptor {
            vertex_array_data: self.vertex_array.data.clone(),
            offset: self.offset,
            count: self.len,
            instance_count: 1,
        }
    }
}

impl<'a, L> VertexInputStreamDescription for InstancedVertexArraySlice<'a, L> {
    type Layout = L;

    fn descriptor(&self) -> VertexInputStreamDescriptor {
        VertexInputStreamDescriptor {
            vertex_array_data: self.vertex_array.data.clone(),
            offset: self.offset,
            count: self.len,
            instance_count: self.instance_count,
        }
    }
}

struct VertexArrayAllocateCommand {
    data: Arc<VertexArrayData>,
    buffer_descriptors: Vec<VertexBufferDescriptor>,
    attribute_descriptors: Vec<VertexInputAttributeDescriptor>,
    index_buffer_descriptor: Option<IndexBufferDescriptor>,
}

unsafe impl GpuTask<Connection> for VertexArrayAllocateCommand {
    type Output = ();

    fn context_id(&self) -> ContextId {
        ContextId::Any
    }

    fn progress(&mut self, connection: &mut Connection) -> Progress<Self::Output> {
        let (gl, state) = unsafe { connection.unpack_mut() };
        let mut data = unsafe { arc_get_mut_unchecked(&mut self.data) };
        let vao = gl.create_vertex_array().unwrap();

        state.set_bound_vertex_array(Some(&vao)).apply(gl).unwrap();

        for buffer_descriptor in self.buffer_descriptors.iter() {
            buffer_descriptor
                .buffer_data
                .id()
                .unwrap()
                .with_value(|buffer| {
                    state
                        .set_bound_array_buffer(Some(buffer))
                        .apply(gl)
                        .unwrap();
                });

            for i in 0..buffer_descriptor.attribute_count {
                self.attribute_descriptors[i + buffer_descriptor.attribute_offset].apply(
                    gl,
                    buffer_descriptor.stride_in_bytes,
                    buffer_descriptor.offset_in_bytes as i32,
                    buffer_descriptor.input_rate,
                );
            }
        }

        if let Some(index_buffer_descriptor) = &self.index_buffer_descriptor {
            index_buffer_descriptor
                .buffer_data
                .id()
                .unwrap()
                .with_value(|buffer| {
                    state
                        .set_bound_element_array_buffer(Some(buffer))
                        .apply(gl)
                        .unwrap();
                });
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
