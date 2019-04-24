use std::borrow::Borrow;

use crate::vertex::{Vertex, VertexAttributeDescriptor};

/// Describes a [VertexAttributeLayout] attached to a type.
///
/// The attribute layout is described by sequence of grouped [VertexAttributeDescriptor]s.
/// Together with a sequence of [VertexInputDescriptor]s, these may be used to describe the vertex
/// input state for a [VertexArray], see [VertexInputStateDescription].
///
/// # Unsafe
///
/// The value returned by [input_attribute_bindings] must describe the same attribute layout on
/// every invocation.
pub unsafe trait VertexAttributeLayout {
    /// The type returned by [input_attribute_bindings].
    type InputAttributeBindings: Borrow<[&'static [VertexAttributeDescriptor]]> + 'static;

    /// Returns a sequence of grouped [VertexAttributeDescriptor]s.
    ///
    /// Together with a sequence of [VertexInputDescriptor]s, these may be used to describe the
    /// vertex input state for a [VertexArray], see [VertexInputStateDescription].
    fn input_attribute_bindings() -> Self::InputAttributeBindings;
}

macro_rules! impl_vertex_attribute_layout {
    ($n:tt, $($T:ident),*) => {
        unsafe impl<$($T),*> VertexAttributeLayout for ($($T),*) where $($T: Vertex),* {
            type InputAttributeBindings = [&'static [VertexAttributeDescriptor]; $n];

            fn input_attribute_bindings() -> Self::InputAttributeBindings {
                [
                    $($T::attribute_descriptors()),*
                ]
            }
        }
    }
}

impl_vertex_attribute_layout!(1, T0);
impl_vertex_attribute_layout!(2, T0, T1);
impl_vertex_attribute_layout!(3, T0, T1, T2);
impl_vertex_attribute_layout!(4, T0, T1, T2, T3);
impl_vertex_attribute_layout!(5, T0, T1, T2, T3, T4);
impl_vertex_attribute_layout!(6, T0, T1, T2, T3, T4, T5);
impl_vertex_attribute_layout!(7, T0, T1, T2, T3, T4, T5, T6);
impl_vertex_attribute_layout!(8, T0, T1, T2, T3, T4, T5, T6, T7);
impl_vertex_attribute_layout!(9, T0, T1, T2, T3, T4, T5, T6, T7, T8);
impl_vertex_attribute_layout!(10, T0, T1, T2, T3, T4, T5, T6, T7, T8, T9);
impl_vertex_attribute_layout!(11, T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
impl_vertex_attribute_layout!(12, T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);
impl_vertex_attribute_layout!(13, T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12);
impl_vertex_attribute_layout!(14, T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13);
impl_vertex_attribute_layout!(15, T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14);
impl_vertex_attribute_layout!(
    16, T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15
);
