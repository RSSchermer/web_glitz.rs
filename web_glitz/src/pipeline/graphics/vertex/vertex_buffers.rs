use crate::buffer::{Buffer, BufferView};
use crate::pipeline::graphics::util::{BufferDescriptor, BufferDescriptors};
use crate::pipeline::graphics::{TypedVertexInputLayout, Vertex};

/// Encodes a description of a (set of) buffer(s) or buffer region(s) that can serve as the vertex
/// input data source(s) for a graphics pipeline.
pub trait VertexBuffers {
    fn encode<'a>(self, context: &'a mut VertexBuffersEncodingContext)
        -> VertexBuffersEncoding<'a>;
}

/// Helper trait for the implementation of [VertexBuffers] for tuple types.
pub trait VertexBuffer {
    fn encode(self, encoding: &mut VertexBuffersEncoding);
}

/// Sub-trait of [VertexBuffers], where a type statically describes the vertex attribute layout
/// supported by the vertex buffers.
///
/// Vertex buffers that implement this trait may be bound to graphics pipelines with a matching
/// [TypedVertexAttributeLayout] without further runtime checks.
///
/// # Unsafe
///
/// This trait must only by implemented for [VertexBuffers] types if the vertex buffers encoding
/// for any instance of the the type is guaranteed to provide compatible vertex input data for
/// each of the [VertexAttributeDescriptors] specified by the [Layout].
pub unsafe trait TypedVertexBuffers: VertexBuffers {
    /// A type statically associated with a vertex attribute layout with which any instance of these
    /// [TypedVertexBuffers] is compatible.
    type Layout: TypedVertexInputLayout;
}

/// Helper trait for the implementation of [TypedVertexBuffers] for tuple types.
pub unsafe trait TypedVertexBuffer: VertexBuffer {
    type Vertex: Vertex;
}

// Note that currently the VertexBuffersEncodingContext's only use is to serve as a form of lifetime
// erasure, it ensures if a buffer is mutable borrowed for transform feedback, then it should be
// impossible to create an IndexBufferEncoding for that pipeline task that also uses that buffer
// in safe Rust, without having to keep the actual borrow of that buffer alive (the resulting
// pipeline task needs to be `'static`).

/// Context required for the creation of a new [VertexBuffersEncoding].
///
/// See [VertexBuffersEncoding::new].
pub struct VertexBuffersEncodingContext(());

impl VertexBuffersEncodingContext {
    pub(crate) fn new() -> Self {
        VertexBuffersEncodingContext(())
    }
}

/// An encoding of a description of a (set of) buffer(s) or buffer region(s) that can serve as the
/// vertex input data source(s) for a graphics pipeline.
///
/// See also [VertexBuffers].
///
/// Contains slots for up to 16 buffers or buffer regions.
pub struct VertexBuffersEncoding<'a> {
    #[allow(unused)]
    context: &'a mut VertexBuffersEncodingContext,
    descriptors: BufferDescriptors,
}

impl<'a> VertexBuffersEncoding<'a> {
    /// Returns a new empty [VertexBuffersEncoding] for the given `context`.
    pub fn new(context: &'a mut VertexBuffersEncodingContext) -> Self {
        VertexBuffersEncoding {
            context,
            descriptors: BufferDescriptors::new(),
        }
    }

    /// Adds a new buffer or buffer region to the description in the next free binding slot.
    ///
    /// # Panics
    ///
    /// Panics if called when all 16 vertex buffer slots have already been filled.
    pub fn add_vertex_buffer<'b, V, T>(&mut self, buffer: V)
    where
        V: Into<BufferView<'b, [T]>>,
        T: 'b,
    {
        self.descriptors
            .push(BufferDescriptor::from_buffer_view(buffer.into()));
    }

    pub(crate) fn into_descriptors(self) -> BufferDescriptors {
        self.descriptors
    }
}

impl<'a, T> VertexBuffer for &'a Buffer<[T]>
where
    T: Vertex,
{
    fn encode(self, encoding: &mut VertexBuffersEncoding) {
        encoding.add_vertex_buffer(self);
    }
}

unsafe impl<'a, T> TypedVertexBuffer for &'a Buffer<[T]>
where
    T: Vertex,
{
    type Vertex = T;
}

impl<'a, T> VertexBuffer for BufferView<'a, [T]>
where
    T: Vertex,
{
    fn encode(self, encoding: &mut VertexBuffersEncoding) {
        encoding.add_vertex_buffer(self);
    }
}

unsafe impl<'a, T> TypedVertexBuffer for BufferView<'a, [T]>
where
    T: Vertex,
{
    type Vertex = T;
}

macro_rules! impl_vertex_buffers {
    ($($T:ident),*) => {
        #[allow(unused_parens)]
        impl<$($T),*> VertexBuffers for ($($T),*)
        where
            $($T: VertexBuffer),*
        {
            fn encode<'a>(self, context: &'a mut VertexBuffersEncodingContext) -> VertexBuffersEncoding<'a> {
                let mut encoding = VertexBuffersEncoding::new(context);

                #[allow(unused_parens, non_snake_case)]
                let ($($T),*) = self;

                $(
                    $T.encode(&mut encoding);
                )*

                encoding
            }
        }

        #[allow(unused_parens)]
        unsafe impl<$($T),*> TypedVertexBuffers for ($($T),*)
        where
            $($T: TypedVertexBuffer),*
        {
            #[allow(unused_parens)]
            type Layout = ($($T::Vertex),*);
        }
    }
}

impl_vertex_buffers!(T0);
impl_vertex_buffers!(T0, T1);
impl_vertex_buffers!(T0, T1, T2);
impl_vertex_buffers!(T0, T1, T2, T3);
impl_vertex_buffers!(T0, T1, T2, T3, T4);
impl_vertex_buffers!(T0, T1, T2, T3, T4, T5);
impl_vertex_buffers!(T0, T1, T2, T3, T4, T5, T6);
impl_vertex_buffers!(T0, T1, T2, T3, T4, T5, T6, T7);
impl_vertex_buffers!(T0, T1, T2, T3, T4, T5, T6, T7, T8);
impl_vertex_buffers!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9);
impl_vertex_buffers!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
impl_vertex_buffers!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);
impl_vertex_buffers!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12);
impl_vertex_buffers!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13);
impl_vertex_buffers!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14);
impl_vertex_buffers!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15);
