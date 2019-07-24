use std::hash::{Hash, Hasher};
use std::mem::{ManuallyDrop, MaybeUninit};
use std::ops::Deref;
use std::sync::Arc;
use std::{mem, ptr};

use web_sys::WebGl2RenderingContext as Gl;

use crate::buffer::{Buffer, BufferData, BufferView};
use crate::pipeline::graphics::attribute_format::AttributeFormat;
use crate::pipeline::graphics::{TypedVertexAttributeLayout, Vertex};

/// Encodes a description of a (set of) buffer(s) or buffer region(s) that can serve as the vertex
/// input data source(s) for a graphics pipeline.
pub trait VertexBuffers {
    fn encode<'a>(
        &self,
        context: &'a mut VertexBuffersEncodingContext,
    ) -> VertexBuffersEncoding<'a>;
}

/// Helper trait for the implementation of [VertexBuffers] for tuple types.
pub trait VertexBuffer {
    fn encode(&self, encoding: &mut VertexBuffersEncoding);
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
/// each of the [VertexAttributeDescriptors] specified by the [VertexAttributeLayout].
pub unsafe trait TypedVertexBuffers: VertexBuffers {
    /// A type statically associated with a vertex attribute layout with which any instance of these
    /// [TypedVertexBuffers] is compatible.
    type VertexAttributeLayout: TypedVertexAttributeLayout;
}

/// Helper trait for the implementation of [TypedVertexBuffers] for tuple types.
pub unsafe trait TypedVertexBuffer: VertexBuffer {
    type Vertex: Vertex;
}

/// Describes the input rate for a [VertexInputDescriptor] when it is used as a data source for
/// a vertex stream.
///
/// See also [VertexStreamDescription] and [VertexStreamDescriptor].
#[derive(Clone, Copy, PartialEq, Hash, Debug)]
pub enum InputRate {
    /// Any attribute defined on the [VertexInputDescriptor] will advance to its next value for
    /// every vertex in a vertex stream; the attribute resets to its first value for every instance
    /// in a vertex stream that defines more than 1 instance.
    PerVertex,

    /// Any attribute defined on the [VertexInputDescriptor] will advance to its next value for
    /// every instance in a vertex stream that defines more than 1 instance; all vertices for the
    /// instance will use the same attribute value.
    PerInstance,
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
    descriptors: VertexBufferDescriptors,
}

impl<'a> VertexBuffersEncoding<'a> {
    /// Returns a new empty [VertexBuffersEncoding] for the given `context`.
    pub fn new(context: &'a mut VertexBuffersEncodingContext) -> Self {
        VertexBuffersEncoding {
            context,
            descriptors: VertexBufferDescriptors::new(),
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
            .push(VertexBufferDescriptor::from_buffer_view(buffer.into()));
    }

    pub(crate) fn into_descriptors(self) -> VertexBufferDescriptors {
        self.descriptors
    }
}

pub(crate) struct VertexBufferDescriptors {
    storage: ManuallyDrop<[VertexBufferDescriptor; 16]>,
    len: usize,
}

impl VertexBufferDescriptors {
    pub fn new() -> Self {
        VertexBufferDescriptors {
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

    pub fn push(&mut self, descriptor: VertexBufferDescriptor) {
        self.storage[self.len] = descriptor;
        self.len += 1;
    }
}

impl Deref for VertexBufferDescriptors {
    type Target = [VertexBufferDescriptor];

    fn deref(&self) -> &Self::Target {
        &self.storage[0..self.len]
    }
}

impl Drop for VertexBufferDescriptors {
    fn drop(&mut self) {
        for vertex_buffer in self.storage[0..self.len].iter_mut() {
            unsafe {
                ptr::drop_in_place(vertex_buffer as *mut VertexBufferDescriptor);
            }
        }
    }
}

/// Describes an input source for vertex attribute data.
#[derive(Clone)]
pub(crate) struct VertexBufferDescriptor {
    pub(crate) buffer_data: Arc<BufferData>,
    pub(crate) offset_in_bytes: u32,
    pub(crate) size_in_bytes: u32,
}

impl VertexBufferDescriptor {
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
        VertexBufferDescriptor {
            buffer_data: buffer_view.buffer_data().clone(),
            offset_in_bytes: buffer_view.offset_in_bytes() as u32,
            size_in_bytes: (mem::size_of::<T>() * buffer_view.len()) as u32,
        }
    }
}

impl Hash for VertexBufferDescriptor {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.buffer_data.id().hash(state);
        self.offset_in_bytes.hash(state);
        self.size_in_bytes.hash(state);
    }
}

impl<'a, T> VertexBuffer for &'a Buffer<[T]>
where
    T: Vertex,
{
    fn encode(&self, encoding: &mut VertexBuffersEncoding) {
        encoding.add_vertex_buffer(*self);
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
    fn encode(&self, encoding: &mut VertexBuffersEncoding) {
        encoding.add_vertex_buffer(*self);
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
        impl<$($T),*> VertexBuffers for ($($T),*)
        where
            $($T: VertexBuffer),*
        {
            fn encode<'a>(&self, context: &'a mut VertexBuffersEncodingContext) -> VertexBuffersEncoding<'a> {
                let mut encoding = VertexBuffersEncoding::new(context);

                #[allow(unused_parens, non_snake_case)]
                let ($($T),*) = self;

                $(
                    $T.encode(&mut encoding);
                )*

                encoding
            }
        }

        unsafe impl<$($T),*> TypedVertexBuffers for ($($T),*)
        where
            $($T: TypedVertexBuffer),*
        {
            type VertexAttributeLayout = ($($T::Vertex),*);
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

/// Describes how the data for an `in` attribute in a [VertexShader] is sourced from a
/// [VertexInputDescriptor].
///
/// See also [VertexInputStateDescription].
#[derive(Clone, Copy, PartialEq, Hash, Debug)]
pub struct VertexAttributeDescriptor {
    /// The shader location of the attribute described by this [VertexAttributeDescriptor].
    ///
    /// For example, if the location is `1`, then this [VertexAttributeDescriptor] describes how
    /// data is sourced for values of the attribute at shader attribute location `1`. In GLSL, the
    /// location for an attribute may be specified with the `layout` qualifier:
    ///
    /// ```glsl
    /// layout(location=1) in vec4 position;
    /// ```
    pub location: u32,

    /// The offset in bytes of the first value in the attribute value sequence relative to the start
    /// of a [VertexInputDescriptor].
    ///
    /// The byte-sequence for the first attribute value begins at this offset. Subsequent attribute
    /// values are obtained by adding a stride to the base offset, see
    /// [VertexInputDescriptor::stride_in_bytes].
    pub offset_in_bytes: u8,

    /// The data format in which the attribute values are stored.
    ///
    /// Should be a format that is compatible with the type used for the attribute in the shader,
    /// see also [AttributeFormat::is_compatible].
    pub format: AttributeFormat,
}

impl VertexAttributeDescriptor {
    pub(crate) fn apply(
        &self,
        gl: &Gl,
        stride_in_bytes: i32,
        base_offset_in_bytes: i32,
        input_rate: InputRate,
    ) {
        match self.format {
            AttributeFormat::Float_f32 => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    1,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Float_i8_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    1,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Float_i8_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    1,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);
            }
            AttributeFormat::Float_i16_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    1,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Float_i16_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    1,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Float_u8_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    1,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Float_u8_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    1,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Float_u16_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    1,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Float_u16_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    1,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Float2_f32 => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Float2_i8_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Float2_i8_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Float2_i16_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Float2_i16_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Float2_u8_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Float2_u8_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Float2_u16_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Float2_u16_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Float3_f32 => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Float3_i8_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Float3_i8_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Float3_i16_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Float3_i16_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Float3_u8_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Float3_u8_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Float3_u16_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Float3_u16_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Float4_f32 => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Float4_i8_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Float4_i8_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Float4_i16_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Float4_i16_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Float4_u8_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Float4_u8_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Float4_u16_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Float4_u16_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Float2x2_f32 => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 4 * 2,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                }
            }
            AttributeFormat::Float2x2_i8_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 2,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                }
            }
            AttributeFormat::Float2x2_i8_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 2,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                }
            }
            AttributeFormat::Float2x2_i16_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 2,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                }
            }
            AttributeFormat::Float2x2_i16_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 2,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                }
            }
            AttributeFormat::Float2x2_u8_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 2,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                }
            }
            AttributeFormat::Float2x2_u8_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 2,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                }
            }
            AttributeFormat::Float2x2_u16_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 2,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                }
            }
            AttributeFormat::Float2x2_u16_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 2,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                }
            }
            AttributeFormat::Float2x3_f32 => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 4 * 3,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                }
            }
            AttributeFormat::Float2x3_i8_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 3,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                }
            }
            AttributeFormat::Float2x3_i8_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 3,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                }
            }
            AttributeFormat::Float2x3_i16_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 3,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                }
            }
            AttributeFormat::Float2x3_i16_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 3,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                }
            }
            AttributeFormat::Float2x3_u8_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 3,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                }
            }
            AttributeFormat::Float2x3_u8_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 3,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                }
            }
            AttributeFormat::Float2x3_u16_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 3,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                }
            }
            AttributeFormat::Float2x3_u16_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 3,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                }
            }
            AttributeFormat::Float2x4_f32 => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 4 * 4,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                }
            }
            AttributeFormat::Float2x4_i8_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 4,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                }
            }
            AttributeFormat::Float2x4_i8_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 4,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                }
            }
            AttributeFormat::Float2x4_i16_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 4,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                }
            }
            AttributeFormat::Float2x4_i16_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 4,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                }
            }
            AttributeFormat::Float2x4_u8_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 4,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                }
            }
            AttributeFormat::Float2x4_u8_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 4,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                }
            }
            AttributeFormat::Float2x4_u16_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 4,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                }
            }
            AttributeFormat::Float2x4_u16_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 4,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                }
            }
            AttributeFormat::Float3x2_f32 => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 4 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    2,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 4 * 2 * 2,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                }
            }
            AttributeFormat::Float3x2_i8_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    2,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 2 * 2,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                }
            }
            AttributeFormat::Float3x2_i8_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    2,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 2 * 2,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                }
            }
            AttributeFormat::Float3x2_i16_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    2,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 2 * 2,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                }
            }
            AttributeFormat::Float3x2_i16_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    2,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 2 * 2,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                }
            }
            AttributeFormat::Float3x2_u8_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    2,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 2 * 2,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                }
            }
            AttributeFormat::Float3x2_u8_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    2,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 2 * 2,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                }
            }
            AttributeFormat::Float3x2_u16_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    2,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 2 * 2,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                }
            }
            AttributeFormat::Float3x2_u16_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    2,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 2 * 2,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                }
            }
            AttributeFormat::Float3x3_f32 => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 4 * 3,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    3,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 4 * 3 * 2,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                }
            }
            AttributeFormat::Float3x3_i8_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 3,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    3,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 3 * 2,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                }
            }
            AttributeFormat::Float3x3_i8_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 3,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    3,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 3 * 2,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                }
            }
            AttributeFormat::Float3x3_i16_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 3,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    3,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 3 * 2,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                }
            }
            AttributeFormat::Float3x3_i16_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 3,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    3,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 3 * 2,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                }
            }
            AttributeFormat::Float3x3_u8_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 3,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    3,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 3 * 2,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                }
            }
            AttributeFormat::Float3x3_u8_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 3,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    3,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 3 * 2,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                }
            }
            AttributeFormat::Float3x3_u16_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 3,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    3,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 3 * 2,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                }
            }
            AttributeFormat::Float3x3_u16_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 3,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    3,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 3 * 2,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                }
            }
            AttributeFormat::Float3x4_f32 => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 4 * 4,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    4,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 4 * 4 * 2,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                }
            }
            AttributeFormat::Float3x4_i8_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 4,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    4,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 4 * 2,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                }
            }
            AttributeFormat::Float3x4_i8_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 4,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    4,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 4 * 2,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                }
            }
            AttributeFormat::Float3x4_i16_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 4,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    4,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 4 * 2,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                }
            }
            AttributeFormat::Float3x4_i16_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 4,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    4,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 4 * 2,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                }
            }
            AttributeFormat::Float3x4_u8_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 4,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    4,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 4 * 2,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                }
            }
            AttributeFormat::Float3x4_u8_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 4,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    4,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 4 * 2,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                }
            }
            AttributeFormat::Float3x4_u16_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 4,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    4,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 4 * 2,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                }
            }
            AttributeFormat::Float3x4_u16_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 4,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    4,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 4 * 2,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                }
            }
            AttributeFormat::Float4x2_f32 => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 4 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    2,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 4 * 2 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    2,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 4 * 2 * 3,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
                gl.enable_vertex_attrib_array(self.location + 3);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                    gl.vertex_attrib_divisor(self.location + 3, 1);
                }
            }
            AttributeFormat::Float4x2_i8_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    2,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 2 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    2,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 2 * 3,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
                gl.enable_vertex_attrib_array(self.location + 3);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                    gl.vertex_attrib_divisor(self.location + 3, 1);
                }
            }
            AttributeFormat::Float4x2_i8_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    2,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 2 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    2,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 2 * 3,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
                gl.enable_vertex_attrib_array(self.location + 3);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                    gl.vertex_attrib_divisor(self.location + 3, 1);
                }
            }
            AttributeFormat::Float4x2_i16_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    2,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 2 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    2,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 2 * 3,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
                gl.enable_vertex_attrib_array(self.location + 3);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                    gl.vertex_attrib_divisor(self.location + 3, 1);
                }
            }
            AttributeFormat::Float4x2_i16_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    2,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 2 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    2,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 2 * 3,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
                gl.enable_vertex_attrib_array(self.location + 3);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                    gl.vertex_attrib_divisor(self.location + 3, 1);
                }
            }
            AttributeFormat::Float4x2_u8_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    2,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 2 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    2,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 2 * 3,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
                gl.enable_vertex_attrib_array(self.location + 3);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                    gl.vertex_attrib_divisor(self.location + 3, 1);
                }
            }
            AttributeFormat::Float4x2_u8_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    2,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 2 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    2,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 2 * 3,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
                gl.enable_vertex_attrib_array(self.location + 3);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                    gl.vertex_attrib_divisor(self.location + 3, 1);
                }
            }
            AttributeFormat::Float4x2_u16_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    2,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 2 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    2,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 2 * 3,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
                gl.enable_vertex_attrib_array(self.location + 3);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                    gl.vertex_attrib_divisor(self.location + 3, 1);
                }
            }
            AttributeFormat::Float4x2_u16_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    2,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 2 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    2,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 2 * 3,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
                gl.enable_vertex_attrib_array(self.location + 3);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                    gl.vertex_attrib_divisor(self.location + 3, 1);
                }
            }
            AttributeFormat::Float4x3_f32 => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 4 * 3,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    3,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 4 * 3 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    3,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 4 * 3 * 3,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
                gl.enable_vertex_attrib_array(self.location + 3);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                    gl.vertex_attrib_divisor(self.location + 3, 1);
                }
            }
            AttributeFormat::Float4x3_i8_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 3,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    3,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 3 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    3,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 3 * 3,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
                gl.enable_vertex_attrib_array(self.location + 3);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                    gl.vertex_attrib_divisor(self.location + 3, 1);
                }
            }
            AttributeFormat::Float4x3_i8_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 3,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    3,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 3 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    3,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 3 * 3,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
                gl.enable_vertex_attrib_array(self.location + 3);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                    gl.vertex_attrib_divisor(self.location + 3, 1);
                }
            }
            AttributeFormat::Float4x3_i16_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 3,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    3,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 3 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    3,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 3 * 3,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
                gl.enable_vertex_attrib_array(self.location + 3);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                    gl.vertex_attrib_divisor(self.location + 3, 1);
                }
            }
            AttributeFormat::Float4x3_i16_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 3,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    3,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 3 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    3,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 3 * 3,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
                gl.enable_vertex_attrib_array(self.location + 3);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                    gl.vertex_attrib_divisor(self.location + 3, 1);
                }
            }
            AttributeFormat::Float4x3_u8_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 3,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    3,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 3 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    3,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 3 * 3,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
                gl.enable_vertex_attrib_array(self.location + 3);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                    gl.vertex_attrib_divisor(self.location + 3, 1);
                }
            }
            AttributeFormat::Float4x3_u8_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 3,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    3,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 3 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    3,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 3 * 3,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
                gl.enable_vertex_attrib_array(self.location + 3);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                    gl.vertex_attrib_divisor(self.location + 3, 1);
                }
            }
            AttributeFormat::Float4x3_u16_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 3,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    3,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 3 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    3,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 3 * 3,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
                gl.enable_vertex_attrib_array(self.location + 3);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                    gl.vertex_attrib_divisor(self.location + 3, 1);
                }
            }
            AttributeFormat::Float4x3_u16_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 3,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    3,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 3 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    3,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 3 * 3,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
                gl.enable_vertex_attrib_array(self.location + 3);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                    gl.vertex_attrib_divisor(self.location + 3, 1);
                }
            }
            AttributeFormat::Float4x4_f32 => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 4 * 4,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    4,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 4 * 4 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    4,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 4 * 4 * 3,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
                gl.enable_vertex_attrib_array(self.location + 3);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                    gl.vertex_attrib_divisor(self.location + 3, 1);
                }
            }
            AttributeFormat::Float4x4_i8_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 4,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    4,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 4 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    4,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 4 * 3,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
                gl.enable_vertex_attrib_array(self.location + 3);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                    gl.vertex_attrib_divisor(self.location + 3, 1);
                }
            }
            AttributeFormat::Float4x4_i8_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 4,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    4,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 4 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    4,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 4 * 3,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
                gl.enable_vertex_attrib_array(self.location + 3);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                    gl.vertex_attrib_divisor(self.location + 3, 1);
                }
            }
            AttributeFormat::Float4x4_i16_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 4,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    4,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 4 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    4,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 4 * 3,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
                gl.enable_vertex_attrib_array(self.location + 3);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                    gl.vertex_attrib_divisor(self.location + 3, 1);
                }
            }
            AttributeFormat::Float4x4_i16_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 4,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    4,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 4 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    4,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 4 * 3,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
                gl.enable_vertex_attrib_array(self.location + 3);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                    gl.vertex_attrib_divisor(self.location + 3, 1);
                }
            }
            AttributeFormat::Float4x4_u8_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 4,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    4,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 4 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    4,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 4 * 3,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
                gl.enable_vertex_attrib_array(self.location + 3);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                    gl.vertex_attrib_divisor(self.location + 3, 1);
                }
            }
            AttributeFormat::Float4x4_u8_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 4,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    4,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 4 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    4,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 1 * 4 * 3,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
                gl.enable_vertex_attrib_array(self.location + 3);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                    gl.vertex_attrib_divisor(self.location + 3, 1);
                }
            }
            AttributeFormat::Float4x4_u16_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 4,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    4,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 4 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    4,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 4 * 3,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
                gl.enable_vertex_attrib_array(self.location + 3);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                    gl.vertex_attrib_divisor(self.location + 3, 1);
                }
            }
            AttributeFormat::Float4x4_u16_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 4,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    4,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 4 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    4,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32 + 2 * 4 * 3,
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
                gl.enable_vertex_attrib_array(self.location + 3);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                    gl.vertex_attrib_divisor(self.location + 1, 1);
                    gl.vertex_attrib_divisor(self.location + 2, 1);
                    gl.vertex_attrib_divisor(self.location + 3, 1);
                }
            }
            AttributeFormat::Integer_i8 => {
                gl.vertex_attrib_i_pointer_with_i32(
                    self.location,
                    1,
                    Gl::BYTE,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Integer_u8 => {
                gl.vertex_attrib_i_pointer_with_i32(
                    self.location,
                    1,
                    Gl::UNSIGNED_BYTE,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Integer_i16 => {
                gl.vertex_attrib_i_pointer_with_i32(
                    self.location,
                    1,
                    Gl::SHORT,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Integer_u16 => {
                gl.vertex_attrib_i_pointer_with_i32(
                    self.location,
                    1,
                    Gl::UNSIGNED_SHORT,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Integer_i32 => {
                gl.vertex_attrib_i_pointer_with_i32(
                    self.location,
                    1,
                    Gl::INT,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Integer_u32 => {
                gl.vertex_attrib_i_pointer_with_i32(
                    self.location,
                    1,
                    Gl::UNSIGNED_INT,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Integer2_i8 => {
                gl.vertex_attrib_i_pointer_with_i32(
                    self.location,
                    2,
                    Gl::BYTE,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Integer2_u8 => {
                gl.vertex_attrib_i_pointer_with_i32(
                    self.location,
                    2,
                    Gl::UNSIGNED_BYTE,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Integer2_i16 => {
                gl.vertex_attrib_i_pointer_with_i32(
                    self.location,
                    2,
                    Gl::SHORT,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Integer2_u16 => {
                gl.vertex_attrib_i_pointer_with_i32(
                    self.location,
                    2,
                    Gl::UNSIGNED_SHORT,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Integer2_i32 => {
                gl.vertex_attrib_i_pointer_with_i32(
                    self.location,
                    2,
                    Gl::INT,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Integer2_u32 => {
                gl.vertex_attrib_i_pointer_with_i32(
                    self.location,
                    2,
                    Gl::UNSIGNED_INT,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Integer3_i8 => {
                gl.vertex_attrib_i_pointer_with_i32(
                    self.location,
                    3,
                    Gl::BYTE,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Integer3_u8 => {
                gl.vertex_attrib_i_pointer_with_i32(
                    self.location,
                    3,
                    Gl::UNSIGNED_BYTE,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Integer3_i16 => {
                gl.vertex_attrib_i_pointer_with_i32(
                    self.location,
                    3,
                    Gl::SHORT,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Integer3_u16 => {
                gl.vertex_attrib_i_pointer_with_i32(
                    self.location,
                    3,
                    Gl::UNSIGNED_SHORT,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Integer3_i32 => {
                gl.vertex_attrib_i_pointer_with_i32(
                    self.location,
                    3,
                    Gl::INT,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Integer3_u32 => {
                gl.vertex_attrib_i_pointer_with_i32(
                    self.location,
                    3,
                    Gl::UNSIGNED_INT,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Integer4_i8 => {
                gl.vertex_attrib_i_pointer_with_i32(
                    self.location,
                    4,
                    Gl::BYTE,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Integer4_u8 => {
                gl.vertex_attrib_i_pointer_with_i32(
                    self.location,
                    4,
                    Gl::UNSIGNED_BYTE,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Integer4_i16 => {
                gl.vertex_attrib_i_pointer_with_i32(
                    self.location,
                    4,
                    Gl::SHORT,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Integer4_u16 => {
                gl.vertex_attrib_i_pointer_with_i32(
                    self.location,
                    4,
                    Gl::UNSIGNED_SHORT,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Integer4_i32 => {
                gl.vertex_attrib_i_pointer_with_i32(
                    self.location,
                    4,
                    Gl::INT,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
            AttributeFormat::Integer4_u32 => {
                gl.vertex_attrib_i_pointer_with_i32(
                    self.location,
                    4,
                    Gl::UNSIGNED_INT,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset_in_bytes as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
        }
    }
}
