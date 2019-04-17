use crate::vertex::{VertexAttributeLayout, Vertex};
use std::borrow::Borrow;
use std::sync::Arc;
use crate::buffer::{BufferData, BufferView, Buffer};
use std::mem;
use crate::vertex::attribute_format::AttributeFormat;

use web_sys::WebGl2RenderingContext as Gl;

pub unsafe trait VertexInputStateDescription {
    type AttributeLayout: VertexAttributeLayout;

    type InputDescriptors: Borrow<[VertexInputDescriptor]> + 'static;

    fn vertex_input_descriptors(&self) -> Self::InputDescriptors;
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum InputRate {
    PerVertex,
    PerInstance,
}

#[derive(Clone)]
pub struct VertexInputDescriptor {
    pub(crate) buffer_data: Arc<BufferData>,
    pub(crate) stride_in_bytes: u8,
    pub(crate) offset_in_bytes: u32,
    pub(crate) size_in_bytes: u32,
    pub(crate) input_rate: InputRate,
}

impl VertexInputDescriptor {
    pub fn from_buffer_view<T>(buffer_view: BufferView<[T]>, input_rate: InputRate) -> Self {
        VertexInputDescriptor {
            buffer_data: buffer_view.buffer_data().clone(),
            stride_in_bytes: mem::size_of::<T>() as u8,
            offset_in_bytes: buffer_view.offset_in_bytes() as u32,
            size_in_bytes: (mem::size_of::<T>() * buffer_view.len()) as u32,
            input_rate
        }
    }
}

unsafe impl<'a, T> VertexInputStateDescription for &'a Buffer<[T]>
    where
        T: Vertex,
{
    type AttributeLayout = T;

    type InputDescriptors = [VertexInputDescriptor; 1];

    fn vertex_input_descriptors(&self) -> Self::InputDescriptors {
        [
            VertexInputDescriptor::from_buffer_view(self.view(), InputRate::PerVertex)
        ]
    }
}

unsafe impl<'a, T> VertexInputStateDescription for BufferView<'a, [T]>
    where
        T: Vertex,
{
    type AttributeLayout = T;

    type InputDescriptors = [VertexInputDescriptor; 1];

    fn vertex_input_descriptors(&self) -> Self::InputDescriptors {
        [
            VertexInputDescriptor::from_buffer_view(self.clone(), InputRate::PerVertex)
        ]
    }
}

pub struct PerInstance<T>(T);

unsafe impl<'a, T> VertexInputStateDescription for PerInstance<&'a Buffer<[T]>>
    where
        T: Vertex,
{
    type AttributeLayout = T;

    type InputDescriptors = [VertexInputDescriptor; 1];

    fn vertex_input_descriptors(&self) -> Self::InputDescriptors {
        [
            VertexInputDescriptor::from_buffer_view(self.0.view(), InputRate::PerInstance)
        ]
    }
}

unsafe impl<'a, T> VertexInputStateDescription for PerInstance<BufferView<'a, [T]>>
    where
        T: Vertex,
{
    type AttributeLayout = T;

    type InputDescriptors = [VertexInputDescriptor; 1];

    fn vertex_input_descriptors(&self) -> Self::InputDescriptors {
        [
            VertexInputDescriptor::from_buffer_view(self.0.clone(), InputRate::PerInstance)
        ]
    }
}

macro_rules! impl_vertex_input_state_description {
    ($($T:ident),*) => {
        unsafe impl<$($T),*> VertexInputStateDescription for ($($T),*)
            where
                $($T: VertexInputStateDescription, $T::AttributeLayout: Vertex),*
        {
            type AttributeLayout = ($($T::AttributeLayout),*);

            type InputDescriptors = Vec<VertexInputDescriptor>;

            #[allow(unused_assignments)]
            fn vertex_input_descriptors(&self) -> Self::InputDescriptors {
                let mut vec = Vec::new();

                #[allow(unused_parens, non_snake_case)]
                let ($($T),*) = self;

                $(
                    for descriptor in $T.vertex_input_descriptors().borrow().iter() {
                        vec.push(descriptor.clone());
                    }
                )*

                vec
            }
        }
    }
}

impl_vertex_input_state_description!(T0, T1);
impl_vertex_input_state_description!(T0, T1, T2);
impl_vertex_input_state_description!(T0, T1, T2, T3);
impl_vertex_input_state_description!(T0, T1, T2, T3, T4);
impl_vertex_input_state_description!(T0, T1, T2, T3, T4, T5);
impl_vertex_input_state_description!(T0, T1, T2, T3, T4, T5, T6);
impl_vertex_input_state_description!(T0, T1, T2, T3, T4, T5, T6, T7);
impl_vertex_input_state_description!(T0, T1, T2, T3, T4, T5, T6, T7, T8);
impl_vertex_input_state_description!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9);
impl_vertex_input_state_description!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
impl_vertex_input_state_description!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);
impl_vertex_input_state_description!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12);
impl_vertex_input_state_description!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13);
impl_vertex_input_state_description!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14);
impl_vertex_input_state_description!(
    T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15
);

#[derive(Clone, PartialEq, Debug)]
pub struct VertexAttributeDescriptor {
    pub location: u32,
    pub offset: u8,
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
                    base_offset_in_bytes + self.offset as i32 as i32,
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
                    base_offset_in_bytes + self.offset as i32,
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
                    base_offset_in_bytes + self.offset as i32,
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
                    base_offset_in_bytes + self.offset as i32,
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
                    base_offset_in_bytes + self.offset as i32,
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
                    base_offset_in_bytes + self.offset as i32,
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
                    base_offset_in_bytes + self.offset as i32,
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
                    base_offset_in_bytes + self.offset as i32,
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
                    base_offset_in_bytes + self.offset as i32,
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
                    base_offset_in_bytes + self.offset as i32,
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
                    base_offset_in_bytes + self.offset as i32,
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
                    base_offset_in_bytes + self.offset as i32,
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
                    base_offset_in_bytes + self.offset as i32,
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
                    base_offset_in_bytes + self.offset as i32,
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
                    base_offset_in_bytes + self.offset as i32,
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
                    base_offset_in_bytes + self.offset as i32,
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
                    base_offset_in_bytes + self.offset as i32,
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
                    base_offset_in_bytes + self.offset as i32,
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
                    base_offset_in_bytes + self.offset as i32,
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
                    base_offset_in_bytes + self.offset as i32,
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
                    base_offset_in_bytes + self.offset as i32,
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
                    base_offset_in_bytes + self.offset as i32,
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
                    base_offset_in_bytes + self.offset as i32,
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
                    base_offset_in_bytes + self.offset as i32,
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
                    base_offset_in_bytes + self.offset as i32,
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
                    base_offset_in_bytes + self.offset as i32,
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
                    base_offset_in_bytes + self.offset as i32,
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
                    base_offset_in_bytes + self.offset as i32,
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
                    base_offset_in_bytes + self.offset as i32,
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
                    base_offset_in_bytes + self.offset as i32,
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
                    base_offset_in_bytes + self.offset as i32,
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
                    base_offset_in_bytes + self.offset as i32,
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
                    base_offset_in_bytes + self.offset as i32,
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
                    base_offset_in_bytes + self.offset as i32,
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
                    base_offset_in_bytes + self.offset as i32,
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
                    base_offset_in_bytes + self.offset as i32,
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
                    base_offset_in_bytes + self.offset as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 4 * 2,
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
                    base_offset_in_bytes + self.offset as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 1 * 2,
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
                    base_offset_in_bytes + self.offset as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 1 * 2,
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
                    base_offset_in_bytes + self.offset as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 2 * 2,
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
                    base_offset_in_bytes + self.offset as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 2 * 2,
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
                    base_offset_in_bytes + self.offset as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 1 * 2,
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
                    base_offset_in_bytes + self.offset as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 1 * 2,
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
                    base_offset_in_bytes + self.offset as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 2 * 2,
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
                    base_offset_in_bytes + self.offset as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 2 * 2,
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
                    base_offset_in_bytes + self.offset as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 4 * 3,
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
                    base_offset_in_bytes + self.offset as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 1 * 3,
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
                    base_offset_in_bytes + self.offset as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 1 * 3,
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
                    base_offset_in_bytes + self.offset as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 2 * 3,
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
                    base_offset_in_bytes + self.offset as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 2 * 3,
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
                    base_offset_in_bytes + self.offset as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 1 * 3,
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
                    base_offset_in_bytes + self.offset as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 1 * 3,
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
                    base_offset_in_bytes + self.offset as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 2 * 3,
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
                    base_offset_in_bytes + self.offset as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 2 * 3,
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
                    base_offset_in_bytes + self.offset as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 4 * 4,
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
                    base_offset_in_bytes + self.offset as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 1 * 4,
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
                    base_offset_in_bytes + self.offset as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 1 * 4,
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
                    base_offset_in_bytes + self.offset as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 2 * 4,
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
                    base_offset_in_bytes + self.offset as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 2 * 4,
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
                    base_offset_in_bytes + self.offset as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 1 * 4,
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
                    base_offset_in_bytes + self.offset as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 1 * 4,
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
                    base_offset_in_bytes + self.offset as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 2 * 4,
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
                    base_offset_in_bytes + self.offset as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 2 * 4,
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
                    base_offset_in_bytes + self.offset as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 4 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    2,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 4 * 2 * 2,
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
                    base_offset_in_bytes + self.offset as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 1 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    2,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 1 * 2 * 2,
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
                    base_offset_in_bytes + self.offset as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 1 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    2,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 1 * 2 * 2,
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
                    base_offset_in_bytes + self.offset as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 2 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    2,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 2 * 2 * 2,
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
                    base_offset_in_bytes + self.offset as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 2 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    2,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 2 * 2 * 2,
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
                    base_offset_in_bytes + self.offset as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 1 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    2,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 1 * 2 * 2,
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
                    base_offset_in_bytes + self.offset as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 1 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    2,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 1 * 2 * 2,
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
                    base_offset_in_bytes + self.offset as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 2 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    2,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 2 * 2 * 2,
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
                    base_offset_in_bytes + self.offset as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 2 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    2,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 2 * 2 * 2,
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
                    base_offset_in_bytes + self.offset as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 4 * 3,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    3,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 4 * 3 * 2,
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
                    base_offset_in_bytes + self.offset as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 1 * 3,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    3,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 1 * 3 * 2,
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
                    base_offset_in_bytes + self.offset as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 1 * 3,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    3,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 1 * 3 * 2,
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
                    base_offset_in_bytes + self.offset as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 2 * 3,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    3,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 2 * 3 * 2,
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
                    base_offset_in_bytes + self.offset as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 2 * 3,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    3,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 2 * 3 * 2,
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
                    base_offset_in_bytes + self.offset as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 1 * 3,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    3,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 1 * 3 * 2,
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
                    base_offset_in_bytes + self.offset as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 1 * 3,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    3,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 1 * 3 * 2,
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
                    base_offset_in_bytes + self.offset as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 2 * 3,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    3,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 2 * 3 * 2,
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
                    base_offset_in_bytes + self.offset as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 2 * 3,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    3,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 2 * 3 * 2,
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
                    base_offset_in_bytes + self.offset as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 4 * 4,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    4,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 4 * 4 * 2,
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
                    base_offset_in_bytes + self.offset as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 1 * 4,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    4,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 1 * 4 * 2,
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
                    base_offset_in_bytes + self.offset as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 1 * 4,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    4,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 1 * 4 * 2,
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
                    base_offset_in_bytes + self.offset as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 2 * 4,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    4,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 2 * 4 * 2,
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
                    base_offset_in_bytes + self.offset as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 2 * 4,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    4,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 2 * 4 * 2,
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
                    base_offset_in_bytes + self.offset as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 1 * 4,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    4,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 1 * 4 * 2,
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
                    base_offset_in_bytes + self.offset as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 1 * 4,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    4,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 1 * 4 * 2,
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
                    base_offset_in_bytes + self.offset as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 2 * 4,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    4,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 2 * 4 * 2,
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
                    base_offset_in_bytes + self.offset as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 2 * 4,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    4,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 2 * 4 * 2,
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
                    base_offset_in_bytes + self.offset as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 4 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    2,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 4 * 2 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    2,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 4 * 2 * 3,
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
                    base_offset_in_bytes + self.offset as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 1 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    2,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 1 * 2 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    2,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 1 * 2 * 3,
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
                    base_offset_in_bytes + self.offset as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 1 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    2,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 1 * 2 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    2,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 1 * 2 * 3,
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
                    base_offset_in_bytes + self.offset as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 2 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    2,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 2 * 2 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    2,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 2 * 2 * 3,
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
                    base_offset_in_bytes + self.offset as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 2 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    2,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 2 * 2 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    2,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 2 * 2 * 3,
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
                    base_offset_in_bytes + self.offset as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 1 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    2,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 1 * 2 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    2,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 1 * 2 * 3,
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
                    base_offset_in_bytes + self.offset as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 1 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    2,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 1 * 2 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    2,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 1 * 2 * 3,
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
                    base_offset_in_bytes + self.offset as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 2 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    2,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 2 * 2 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    2,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 2 * 2 * 3,
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
                    base_offset_in_bytes + self.offset as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 2 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    2,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 2 * 2 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    2,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 2 * 2 * 3,
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
                    base_offset_in_bytes + self.offset as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 4 * 3,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    3,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 4 * 3 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    3,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 4 * 3 * 3,
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
                    base_offset_in_bytes + self.offset as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 1 * 3,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    3,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 1 * 3 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    3,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 1 * 3 * 3,
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
                    base_offset_in_bytes + self.offset as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 1 * 3,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    3,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 1 * 3 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    3,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 1 * 3 * 3,
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
                    base_offset_in_bytes + self.offset as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 2 * 3,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    3,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 2 * 3 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    3,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 2 * 3 * 3,
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
                    base_offset_in_bytes + self.offset as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 2 * 3,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    3,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 2 * 3 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    3,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 2 * 3 * 3,
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
                    base_offset_in_bytes + self.offset as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 1 * 3,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    3,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 1 * 3 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    3,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 1 * 3 * 3,
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
                    base_offset_in_bytes + self.offset as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 1 * 3,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    3,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 1 * 3 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    3,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 1 * 3 * 3,
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
                    base_offset_in_bytes + self.offset as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 2 * 3,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    3,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 2 * 3 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    3,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 2 * 3 * 3,
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
                    base_offset_in_bytes + self.offset as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 2 * 3,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    3,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 2 * 3 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    3,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 2 * 3 * 3,
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
                    base_offset_in_bytes + self.offset as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 4 * 4,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    4,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 4 * 4 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    4,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 4 * 4 * 3,
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
                    base_offset_in_bytes + self.offset as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 1 * 4,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    4,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 1 * 4 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    4,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 1 * 4 * 3,
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
                    base_offset_in_bytes + self.offset as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 1 * 4,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    4,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 1 * 4 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    4,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 1 * 4 * 3,
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
                    base_offset_in_bytes + self.offset as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 2 * 4,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    4,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 2 * 4 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    4,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 2 * 4 * 3,
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
                    base_offset_in_bytes + self.offset as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 2 * 4,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    4,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 2 * 4 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    4,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 2 * 4 * 3,
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
                    base_offset_in_bytes + self.offset as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 1 * 4,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    4,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 1 * 4 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    4,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 1 * 4 * 3,
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
                    base_offset_in_bytes + self.offset as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 1 * 4,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    4,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 1 * 4 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    4,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 1 * 4 * 3,
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
                    base_offset_in_bytes + self.offset as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 2 * 4,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    4,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 2 * 4 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    4,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 2 * 4 * 3,
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
                    base_offset_in_bytes + self.offset as i32,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 2 * 4,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    4,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 2 * 4 * 2,
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    4,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset as i32 + 2 * 4 * 3,
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
                    base_offset_in_bytes + self.offset as i32,
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
                    base_offset_in_bytes + self.offset as i32,
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
                    base_offset_in_bytes + self.offset as i32,
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
                    base_offset_in_bytes + self.offset as i32,
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
                    base_offset_in_bytes + self.offset as i32,
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
                    base_offset_in_bytes + self.offset as i32,
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
                    base_offset_in_bytes + self.offset as i32,
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
                    base_offset_in_bytes + self.offset as i32,
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
                    base_offset_in_bytes + self.offset as i32,
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
                    base_offset_in_bytes + self.offset as i32,
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
                    base_offset_in_bytes + self.offset as i32,
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
                    base_offset_in_bytes + self.offset as i32,
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
                    base_offset_in_bytes + self.offset as i32,
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
                    base_offset_in_bytes + self.offset as i32,
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
                    base_offset_in_bytes + self.offset as i32,
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
                    base_offset_in_bytes + self.offset as i32,
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
                    base_offset_in_bytes + self.offset as i32,
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
                    base_offset_in_bytes + self.offset as i32,
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
                    base_offset_in_bytes + self.offset as i32,
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
                    base_offset_in_bytes + self.offset as i32,
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
                    base_offset_in_bytes + self.offset as i32,
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
                    base_offset_in_bytes + self.offset as i32,
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
                    base_offset_in_bytes + self.offset as i32,
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
                    base_offset_in_bytes + self.offset as i32,
                );

                gl.enable_vertex_attrib_array(self.location);

                if input_rate == InputRate::PerInstance {
                    gl.vertex_attrib_divisor(self.location, 1);
                }
            }
        }
    }
}
