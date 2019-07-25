use crate::buffer::{Buffer, BufferView};
use crate::pipeline::graphics::util::{BufferDescriptor, BufferDescriptors};
use crate::pipeline::graphics::{TransformFeedback, TypedTransformFeedbackLayout};

/// Encodes a description of a (set of) buffer(s) or buffer region(s) that can record the output
/// feedback from the transform stage of a graphics pipeline.
pub trait TransformFeedbackBuffers {
    fn encode<'a>(
        &self,
        context: &'a mut TransformFeedbackBuffersEncodingContext,
    ) -> TransformFeedbackBuffersEncoding<'a>;
}

/// Helper trait for the implementation of [FeedbackBuffers] for tuple types.
pub trait TransformFeedbackBuffer {
    fn encode(&self, encoding: &mut TransformFeedbackBuffersEncoding);
}

/// Sub-trait of [TransformFeedbackBuffers], where a type statically describes the feedback
/// attribute layout supported by the feedback buffers.
///
/// Transform feedback buffers that implement this trait may be bound to graphics pipelines with a
/// matching [TypedTransformFeedbackLayout] without further runtime checks.
///
/// # Unsafe
///
/// This trait must only by implemented for [FeedbackBuffers] types if the feedback buffers encoding
/// for any instance of the the type is guaranteed to be bit-wise compatible with the output
/// feedback recorded from the graphics pipeline.
pub unsafe trait TypedTransformFeedbackBuffers: TransformFeedbackBuffers {
    /// A type statically associated with a feedback attribute layout with which any instance of
    /// these [TypedFeedbackBuffers] is compatible.
    type Layout: TypedTransformFeedbackLayout;
}

/// Helper trait for the implementation of [TypedFeedbackBuffers] for tuple types.
pub unsafe trait TypedTransformFeedbackBuffer: TransformFeedbackBuffer {
    type TransformFeedback: TransformFeedback;
}

// Note that currently the FeedbackBuffersEncodingContext's only use is to serve as a form of
// lifetime erasure, it ensures if a buffer is mutable borrowed for transform feedback, then it
// should be impossible to create an IndexBufferEncoding for that pipeline task that also uses that
// buffer in safe Rust, without having to keep the actual borrow of that buffer alive (the resulting
// pipeline task needs to be `'static`).

/// Context required for the creation of a new [TransformFeedbackBuffersEncoding].
///
/// See [TransformFeedbackBuffersEncoding::new].
pub struct TransformFeedbackBuffersEncodingContext(());

impl TransformFeedbackBuffersEncodingContext {
    pub(crate) fn new() -> Self {
        TransformFeedbackBuffersEncodingContext(())
    }
}

/// An encoding of a description of a (set of) buffer(s) or buffer region(s) that can serve as the
/// feedback input data source(s) for a graphics pipeline.
///
/// See also [TransformFeedbackBuffers].
///
/// Contains slots for up to 16 buffers or buffer regions.
pub struct TransformFeedbackBuffersEncoding<'a> {
    #[allow(unused)]
    context: &'a mut TransformFeedbackBuffersEncodingContext,
    descriptors: BufferDescriptors,
}

impl<'a> TransformFeedbackBuffersEncoding<'a> {
    /// Returns a new empty [TransformFeedbackBuffersEncoding] for the given `context`.
    pub fn new(context: &'a mut TransformFeedbackBuffersEncodingContext) -> Self {
        TransformFeedbackBuffersEncoding {
            context,
            descriptors: BufferDescriptors::new(),
        }
    }

    /// Adds a new buffer or buffer region to the description in the next free binding slot.
    ///
    /// # Panics
    ///
    /// Panics if called when all 16 feedback buffer slots have already been filled.
    pub fn add_feedback_buffer<'b, V, T>(&mut self, buffer: V)
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

impl<'a, T> TransformFeedbackBuffer for &'a Buffer<[T]>
where
    T: TransformFeedback,
{
    fn encode(&self, encoding: &mut TransformFeedbackBuffersEncoding) {
        encoding.add_feedback_buffer(*self);
    }
}

unsafe impl<'a, T> TypedTransformFeedbackBuffer for &'a Buffer<[T]>
where
    T: TransformFeedback,
{
    type TransformFeedback = T;
}

impl<'a, T> TransformFeedbackBuffer for BufferView<'a, [T]>
where
    T: TransformFeedback,
{
    fn encode(&self, encoding: &mut TransformFeedbackBuffersEncoding) {
        encoding.add_feedback_buffer(*self);
    }
}

unsafe impl<'a, T> TypedTransformFeedbackBuffer for BufferView<'a, [T]>
where
    T: TransformFeedback,
{
    type TransformFeedback = T;
}

macro_rules! impl_transform_feedback_buffers {
    ($($T:ident),*) => {
        impl<$($T),*> TransformFeedbackBuffers for ($($T),*)
        where
            $($T: TransformFeedbackBuffer),*
        {
            fn encode<'a>(
                &self,
                context: &'a mut TransformFeedbackBuffersEncodingContext
            ) -> TransformFeedbackBuffersEncoding<'a> {
                let mut encoding = TransformFeedbackBuffersEncoding::new(context);

                #[allow(unused_parens, non_snake_case)]
                let ($($T),*) = self;

                $(
                    $T.encode(&mut encoding);
                )*

                encoding
            }
        }

        unsafe impl<$($T),*> TypedTransformFeedbackBuffers for ($($T),*)
        where
            $($T: TypedTransformFeedbackBuffer),*
        {
            type Layout = ($($T::TransformFeedback),*);
        }
    }
}

impl_transform_feedback_buffers!(T0);
impl_transform_feedback_buffers!(T0, T1);
impl_transform_feedback_buffers!(T0, T1, T2);
impl_transform_feedback_buffers!(T0, T1, T2, T3);
impl_transform_feedback_buffers!(T0, T1, T2, T3, T4);
impl_transform_feedback_buffers!(T0, T1, T2, T3, T4, T5);
impl_transform_feedback_buffers!(T0, T1, T2, T3, T4, T5, T6);
impl_transform_feedback_buffers!(T0, T1, T2, T3, T4, T5, T6, T7);
impl_transform_feedback_buffers!(T0, T1, T2, T3, T4, T5, T6, T7, T8);
impl_transform_feedback_buffers!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9);
impl_transform_feedback_buffers!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
impl_transform_feedback_buffers!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);
impl_transform_feedback_buffers!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12);
impl_transform_feedback_buffers!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13);
impl_transform_feedback_buffers!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14);
impl_transform_feedback_buffers!(
    T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15
);
