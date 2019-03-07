use crate::buffer::{BufferView, Buffer, BufferData};
use crate::pipeline::graphics::vertex_input::Vertex;
use std::mem;
use crate::pipeline::graphics::vertex_input::input_attribute_layout::AttributeType;
use std::sync::Arc;

pub unsafe trait VertexBufferDescription {
    type Vertex;

    fn buffer_view(&self) -> BufferView<[Self::Vertex]>;

    fn stride_in_bytes(&self) -> u8;

    fn size_in_bytes(&self) -> usize;

    fn input_rate(&self) -> InputRate;

    fn input_attribute_descriptors(&self) -> &[VertexInputAttributeDescriptor];
}

pub enum InputRate {
    PerVertex,
    PerInstance,
}

unsafe impl<T> VertexBufferDescription for Buffer<[T]> where T: Vertex {
    type Vertex = T;

    fn buffer_view(&self) -> BufferView<[T]> {
        self.view()
    }

    fn stride_in_bytes(&self) -> u8 {
        mem::size_of::<T>() as u8
    }

    fn size_in_bytes(&self) -> usize {
        mem::size_of::<T>() * self.len()
    }

    fn input_rate(&self) -> InputRate {
        InputRate::PerVertex
    }

    fn input_attribute_descriptors(&self) -> &[VertexInputAttributeDescriptor] {
        T::input_attribute_descriptors()
    }
}

unsafe impl<T> VertexBufferDescription for BufferView<[T]> where T: Vertex {
    type Vertex = T;

    fn buffer_view(&self) -> BufferView<[T]> {
        self.clone()
    }

    fn stride_in_bytes(&self) -> u8 {
        mem::size_of::<T>() as u8
    }

    fn size_in_bytes(&self) -> usize {
        mem::size_of::<T>() * self.len()
    }

    fn input_rate(&self) -> InputRate {
        InputRate::PerVertex
    }

    fn input_attribute_descriptors(&self) -> &[VertexInputAttributeDescriptor] {
        T::input_attribute_descriptors()
    }
}

pub struct PerInstance<T>(T);

unsafe impl<T> VertexBufferDescription for PerInstance<Buffer<[T]>> where T: Vertex {
    type Vertex = T;

    fn buffer_view(&self) -> BufferView<[T]> {
        self.0.view()
    }

    fn stride_in_bytes(&self) -> u8 {
        mem::size_of::<T>() as u8
    }

    fn size_in_bytes(&self) -> usize {
        mem::size_of::<T>() * self.0.len()
    }

    fn input_rate(&self) -> InputRate {
        InputRate::PerInstance
    }

    fn input_attribute_descriptors(&self) -> &[VertexInputAttributeDescriptor] {
        T::input_attribute_descriptors()
    }
}

unsafe impl<T> VertexBufferDescription for PerInstance<BufferView<[T]>> where T: Vertex {
    type Vertex = T;

    fn buffer_view(&self) -> BufferView<[T]> {
        self.0.clone()
    }

    fn stride_in_bytes(&self) -> u8 {
        mem::size_of::<T>() as u8
    }

    fn size_in_bytes(&self) -> usize {
        mem::size_of::<T>() * self.0.len()
    }

    fn input_rate(&self) -> InputRate {
        InputRate::PerInstance
    }

    fn input_attribute_descriptors(&self) -> &[VertexInputAttributeDescriptor] {
        T::input_attribute_descriptors()
    }
}

#[derive(PartialEq, Debug)]
pub struct VertexInputAttributeDescriptor {
    pub location: u32,
    pub offset: u8,
    pub format: AttributeFormat,
}

impl VertexInputAttributeDescriptor {
    pub(crate) fn apply(&self, gl: &Gl, stride_in_bytes: i32, base_offset_in_bytes: i32) {
        match self.format {
            AttributeFormat::Float_f32 => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    1,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.enable_vertex_attrib_array(self.location);
            },
            AttributeFormat::Float_i8_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    1,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.enable_vertex_attrib_array(self.location);
            },
            AttributeFormat::Float_i8_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    1,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.enable_vertex_attrib_array(self.location);
            },
            AttributeFormat::Float_i16_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    1,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.enable_vertex_attrib_array(self.location);
            },
            AttributeFormat::Float_i16_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    1,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.enable_vertex_attrib_array(self.location);
            },
            AttributeFormat::Float_u8_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    1,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.enable_vertex_attrib_array(self.location);
            },
            AttributeFormat::Float_u8_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    1,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.enable_vertex_attrib_array(self.location);
            },
            AttributeFormat::Float_u16_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    1,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.enable_vertex_attrib_array(self.location);
            },
            AttributeFormat::Float_u16_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    1,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.enable_vertex_attrib_array(self.location);
            },
            AttributeFormat::Float2_f32 => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.enable_vertex_attrib_array(self.location);
            },
            AttributeFormat::Float2_i8_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.enable_vertex_attrib_array(self.location);
            },
            AttributeFormat::Float2_i8_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.enable_vertex_attrib_array(self.location);
            },
            AttributeFormat::Float2_i16_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.enable_vertex_attrib_array(self.location);
            },
            AttributeFormat::Float2_i16_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.enable_vertex_attrib_array(self.location);
            },
            AttributeFormat::Float2_u8_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.enable_vertex_attrib_array(self.location);
            },
            AttributeFormat::Float2_u8_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.enable_vertex_attrib_array(self.location);
            },
            AttributeFormat::Float2_u16_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.enable_vertex_attrib_array(self.location);
            },
            AttributeFormat::Float2_u16_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.enable_vertex_attrib_array(self.location);
            },
            AttributeFormat::Float3_f32 => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.enable_vertex_attrib_array(self.location);
            },
            AttributeFormat::Float3_i8_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.enable_vertex_attrib_array(self.location);
            },
            AttributeFormat::Float3_i8_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.enable_vertex_attrib_array(self.location);
            },
            AttributeFormat::Float3_i16_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.enable_vertex_attrib_array(self.location);
            },
            AttributeFormat::Float3_i16_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.enable_vertex_attrib_array(self.location);
            },
            AttributeFormat::Float3_u8_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.enable_vertex_attrib_array(self.location);
            },
            AttributeFormat::Float3_u8_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.enable_vertex_attrib_array(self.location);
            },
            AttributeFormat::Float3_u16_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.enable_vertex_attrib_array(self.location);
            },
            AttributeFormat::Float3_u16_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.enable_vertex_attrib_array(self.location);
            },
            AttributeFormat::Float4_f32 => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.enable_vertex_attrib_array(self.location);
            },
            AttributeFormat::Float4_i8_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.enable_vertex_attrib_array(self.location);
            },
            AttributeFormat::Float4_i8_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.enable_vertex_attrib_array(self.location);
            },
            AttributeFormat::Float4_i16_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.enable_vertex_attrib_array(self.location);
            },
            AttributeFormat::Float4_i16_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.enable_vertex_attrib_array(self.location);
            },
            AttributeFormat::Float4_u8_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.enable_vertex_attrib_array(self.location);
            },
            AttributeFormat::Float4_u8_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.enable_vertex_attrib_array(self.location);
            },
            AttributeFormat::Float4_u16_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.enable_vertex_attrib_array(self.location);
            },
            AttributeFormat::Float4_u16_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.enable_vertex_attrib_array(self.location);
            },
            AttributeFormat::Float2x2_f32 => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 4 * 2
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
            },
            AttributeFormat::Float2x2_i8_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 1 * 2
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
            },
            AttributeFormat::Float2x2_i8_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 1 * 2
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
            },
            AttributeFormat::Float2x2_i16_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 2 * 2
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
            },
            AttributeFormat::Float2x2_i16_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 2 * 2
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
            },
            AttributeFormat::Float2x2_u8_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 1 * 2
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
            },
            AttributeFormat::Float2x2_u8_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 1 * 2
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
            },
            AttributeFormat::Float2x2_u16_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 2 * 2
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
            },
            AttributeFormat::Float2x2_u16_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 2 * 2
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
            },
            AttributeFormat::Float2x3_f32 => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 4 * 3
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
            },
            AttributeFormat::Float2x3_i8_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 1 * 3
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
            },
            AttributeFormat::Float2x3_i8_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 1 * 3
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
            },
            AttributeFormat::Float2x3_i16_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 2 * 3
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
            },
            AttributeFormat::Float2x3_i16_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 2 * 3
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
            },
            AttributeFormat::Float2x3_u8_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 1 * 3
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
            },
            AttributeFormat::Float2x3_u8_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 1 * 3
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
            },
            AttributeFormat::Float2x3_u16_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 2 * 3
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
            },
            AttributeFormat::Float2x3_u16_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 2 * 3
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
            },
            AttributeFormat::Float2x4_f32 => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 4 * 4
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
            },
            AttributeFormat::Float2x4_i8_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 1 * 4
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
            },
            AttributeFormat::Float2x4_i8_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 1 * 4
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
            },
            AttributeFormat::Float2x4_i16_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 2 * 4
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
            },
            AttributeFormat::Float2x4_i16_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 2 * 4
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
            },
            AttributeFormat::Float2x4_u8_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 1 * 4
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
            },
            AttributeFormat::Float2x4_u8_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 1 * 4
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
            },
            AttributeFormat::Float2x4_u16_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 2 * 4
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
            },
            AttributeFormat::Float2x4_u16_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 2 * 4
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
            },
            AttributeFormat::Float3x2_f32 => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 4 * 2
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    2,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 4 * 2 * 2
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
            },
            AttributeFormat::Float3x2_i8_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 1 * 2
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    2,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 1 * 2 * 2
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
            },
            AttributeFormat::Float3x2_i8_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 1 * 2
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    2,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 1 * 2 * 2
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
            },
            AttributeFormat::Float3x2_i16_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 2 * 2
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    2,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 2 * 2 * 2
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
            },
            AttributeFormat::Float3x2_i16_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 2 * 2
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    2,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 2 * 2 * 2
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
            },
            AttributeFormat::Float3x2_u8_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 1 * 2
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    2,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 1 * 2 * 2
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
            },
            AttributeFormat::Float3x2_u8_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 1 * 2
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    2,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 1 * 2 * 2
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
            },
            AttributeFormat::Float3x2_u16_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 2 * 2
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    2,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 2 * 2 * 2
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
            },
            AttributeFormat::Float3x2_u16_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 2 * 2
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    2,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 2 * 2 * 2
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
            },
            AttributeFormat::Float3x3_f32 => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 4 * 3
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    3,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 4 * 3 * 2
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
            },
            AttributeFormat::Float3x3_i8_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 1 * 3
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    3,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 1 * 3 * 2
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
            },
            AttributeFormat::Float3x3_i8_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 1 * 3
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    3,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 1 * 3 * 2
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
            },
            AttributeFormat::Float3x3_i16_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 2 * 3
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    3,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 2 * 3 * 2
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
            },
            AttributeFormat::Float3x3_i16_norm => {
                l.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 2 * 3
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    3,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 2 * 3 * 2
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
            },
            AttributeFormat::Float3x3_u8_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 1 * 3
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    3,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 1 * 3 * 2
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
            },
            AttributeFormat::Float3x3_u8_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 1 * 3
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    3,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 1 * 3 * 2
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
            },
            AttributeFormat::Float3x3_u16_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 2 * 3
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    3,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 2 * 3 * 2
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
            },
            AttributeFormat::Float3x3_u16_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 2 * 3
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    3,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 2 * 3 * 2
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
            },
            AttributeFormat::Float3x4_f32 => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 4 * 4
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    4,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 4 * 4 * 2
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
            },
            AttributeFormat::Float3x4_i8_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 1 * 4
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    4,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 1 * 4 * 2
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
            },
            AttributeFormat::Float3x4_i8_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 1 * 4
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    4,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 1 * 4 * 2
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
            },
            AttributeFormat::Float3x4_i16_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 2 * 4
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    4,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 2 * 4 * 2
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
            },
            AttributeFormat::Float3x4_i16_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 2 * 4
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    4,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 2 * 4 * 2
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
            },
            AttributeFormat::Float3x4_u8_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 1 * 4
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    4,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 1 * 4 * 2
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
            },
            AttributeFormat::Float3x4_u8_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 1 * 4
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    4,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 1 * 4 * 2
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
            },
            AttributeFormat::Float3x4_u16_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 2 * 4
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    4,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 2 * 4 * 2
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
            },
            AttributeFormat::Float3x4_u16_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 2 * 4
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    4,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 2 * 4 * 2
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
            },
            AttributeFormat::Float4x2_f32 => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 4 * 2
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    2,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 4 * 2 * 2
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    2,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 4 * 2 * 3
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
                gl.enable_vertex_attrib_array(self.location + 3);
            },
            AttributeFormat::Float4x2_i8_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 1 * 2
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    2,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 1 * 2 * 2
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    2,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 1 * 2 * 3
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
                gl.enable_vertex_attrib_array(self.location + 3);
            },
            AttributeFormat::Float4x2_i8_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 1 * 2
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    2,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 1 * 2 * 2
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    2,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 1 * 2 * 3
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
                gl.enable_vertex_attrib_array(self.location + 3);
            },
            AttributeFormat::Float4x2_i16_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 2 * 2
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    2,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 2 * 2 * 2
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    2,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 2 * 2 * 3
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
                gl.enable_vertex_attrib_array(self.location + 3);
            },
            AttributeFormat::Float4x2_i16_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 2 * 2
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    2,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 2 * 2 * 2
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    2,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 2 * 2 * 3
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
                gl.enable_vertex_attrib_array(self.location + 3);
            },
            AttributeFormat::Float4x2_u8_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 1 * 2
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    2,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 1 * 2 * 2
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    2,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 1 * 2 * 3
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
                gl.enable_vertex_attrib_array(self.location + 3);
            },
            AttributeFormat::Float4x2_u8_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 1 * 2
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    2,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 1 * 2 * 2
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    2,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 1 * 2 * 3
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
                gl.enable_vertex_attrib_array(self.location + 3);
            },
            AttributeFormat::Float4x2_u16_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 2 * 2
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    2,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 2 * 2 * 2
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    2,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 2 * 2 * 3
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
                gl.enable_vertex_attrib_array(self.location + 3);
            },
            AttributeFormat::Float4x2_u16_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    2,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    2,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 2 * 2
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    2,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 2 * 2 * 2
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    2,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 2 * 2 * 3
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
                gl.enable_vertex_attrib_array(self.location + 3);
            },
            AttributeFormat::Float4x3_f32 => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 4 * 3
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    3,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 4 * 3 * 2
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    3,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 4 * 3 * 3
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
                gl.enable_vertex_attrib_array(self.location + 3);
            },
            AttributeFormat::Float4x3_i8_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 1 * 3
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    3,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 1 * 3 * 2
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    3,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 1 * 3 * 3
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
                gl.enable_vertex_attrib_array(self.location + 3);
            },
            AttributeFormat::Float4x3_i8_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 1 * 3
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    3,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 1 * 3 * 2
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    3,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 1 * 3 * 3
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
                gl.enable_vertex_attrib_array(self.location + 3);
            },
            AttributeFormat::Float4x3_i16_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 2 * 3
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    3,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 2 * 3 * 2
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    3,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 2 * 3 * 3
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
                gl.enable_vertex_attrib_array(self.location + 3);
            },
            AttributeFormat::Float4x3_i16_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 2 * 3
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    3,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 2 * 3 * 2
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    3,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 2 * 3 * 3
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
                gl.enable_vertex_attrib_array(self.location + 3);
            },
            AttributeFormat::Float4x3_u8_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 1 * 3
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    3,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 1 * 3 * 2
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    3,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 1 * 3 * 3
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
                gl.enable_vertex_attrib_array(self.location + 3);
            },
            AttributeFormat::Float4x3_u8_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 1 * 3
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    3,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 1 * 3 * 2
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    3,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 1 * 3 * 3
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
                gl.enable_vertex_attrib_array(self.location + 3);
            },
            AttributeFormat::Float4x3_u16_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 2 * 3
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    3,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 2 * 3 * 2
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    3,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 2 * 3 * 3
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
                gl.enable_vertex_attrib_array(self.location + 3);
            },
            AttributeFormat::Float4x3_u16_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    3,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    3,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 2 * 3
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    3,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 2 * 3 * 2
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    3,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 2 * 3 * 3
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
                gl.enable_vertex_attrib_array(self.location + 3);
            },
            AttributeFormat::Float4x4_f32 => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 4 * 4
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    4,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 4 * 4 * 2
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    4,
                    Gl::FLOAT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 4 * 4 * 3
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
                gl.enable_vertex_attrib_array(self.location + 3);
            },
            AttributeFormat::Float4x4_i8_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 1 * 4
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    4,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 1 * 4 * 2
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    4,
                    Gl::BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 1 * 4 * 3
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
                gl.enable_vertex_attrib_array(self.location + 3);
            },
            AttributeFormat::Float4x4_i8_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 1 * 4
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    4,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 1 * 4 * 2
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    4,
                    Gl::BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 1 * 4 * 3
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
                gl.enable_vertex_attrib_array(self.location + 3);
            },
            AttributeFormat::Float4x4_i16_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 2 * 4
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    4,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 2 * 4 * 2
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    4,
                    Gl::SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 2 * 4 * 3
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
                gl.enable_vertex_attrib_array(self.location + 3);
            },
            AttributeFormat::Float4x4_i16_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 2 * 4
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    4,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 2 * 4 * 2
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    4,
                    Gl::SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 2 * 4 * 3
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
                gl.enable_vertex_attrib_array(self.location + 3);
            },
            AttributeFormat::Float4x4_u8_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 1 * 4
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    4,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 1 * 4 * 2
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    4,
                    Gl::UNSIGNED_BYTE,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 1 * 4 * 3
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
                gl.enable_vertex_attrib_array(self.location + 3);
            },
            AttributeFormat::Float4x4_u8_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 1 * 4
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    4,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 1 * 4 * 2
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    4,
                    Gl::UNSIGNED_BYTE,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 1 * 4 * 3
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
                gl.enable_vertex_attrib_array(self.location + 3);
            },
            AttributeFormat::Float4x4_u16_fixed => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 2 * 4
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    4,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 2 * 4 * 2
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    4,
                    Gl::UNSIGNED_SHORT,
                    false,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 2 * 4 * 3
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
                gl.enable_vertex_attrib_array(self.location + 3);
            },
            AttributeFormat::Float4x4_u16_norm => {
                gl.vertex_attrib_pointer_with_i32(
                    self.location,
                    4,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 1,
                    4,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 2 * 4
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 2,
                    4,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 2 * 4 * 2
                );

                gl.vertex_attrib_pointer_with_i32(
                    self.location + 3,
                    4,
                    Gl::UNSIGNED_SHORT,
                    true,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset + 2 * 4 * 3
                );

                gl.enable_vertex_attrib_array(self.location);
                gl.enable_vertex_attrib_array(self.location + 1);
                gl.enable_vertex_attrib_array(self.location + 2);
                gl.enable_vertex_attrib_array(self.location + 3);
            },
            AttributeFormat::Integer_i8 => {
                gl.vertex_attrib_i_pointer_with_i32(
                    self.location,
                    1,
                    Gl::BYTE,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.enable_vertex_attrib_array(self.location);
            },
            AttributeFormat::Integer_u8 => {
                gl.vertex_attrib_i_pointer_with_i32(
                    self.location,
                    1,
                    Gl::UNSIGNED_BYTE,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.enable_vertex_attrib_array(self.location);
            },
            AttributeFormat::Integer_i16 => {
                gl.vertex_attrib_i_pointer_with_i32(
                    self.location,
                    1,
                    Gl::SHORT,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.enable_vertex_attrib_array(self.location);
            },
            AttributeFormat::Integer_u16 => {
                gl.vertex_attrib_i_pointer_with_i32(
                    self.location,
                    1,
                    Gl::UNSIGNED_SHORT,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.enable_vertex_attrib_array(self.location);
            },
            AttributeFormat::Integer_i32 => {
                gl.vertex_attrib_i_pointer_with_i32(
                    self.location,
                    1,
                    Gl::INT,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.enable_vertex_attrib_array(self.location);
            },
            AttributeFormat::Integer_u32 => {
                gl.vertex_attrib_i_pointer_with_i32(
                    self.location,
                    1,
                    Gl::UNSIGNED_INT,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.enable_vertex_attrib_array(self.location);
            },
            AttributeFormat::Integer2_i8 => {
                gl.vertex_attrib_i_pointer_with_i32(
                    self.location,
                    2,
                    Gl::BYTE,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.enable_vertex_attrib_array(self.location);
            },
            AttributeFormat::Integer2_u8 => {
                gl.vertex_attrib_i_pointer_with_i32(
                    self.location,
                    2,
                    Gl::UNSIGNED_BYTE,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.enable_vertex_attrib_array(self.location);
            },
            AttributeFormat::Integer2_i16 => {
                gl.vertex_attrib_i_pointer_with_i32(
                    self.location,
                    2,
                    Gl::SHORT,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.enable_vertex_attrib_array(self.location);
            },
            AttributeFormat::Integer2_u16 => {
                gl.vertex_attrib_i_pointer_with_i32(
                    self.location,
                    2,
                    Gl::UNSIGNED_SHORT,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.enable_vertex_attrib_array(self.location);
            },
            AttributeFormat::Integer2_i32 => {
                gl.vertex_attrib_i_pointer_with_i32(
                    self.location,
                    2,
                    Gl::INT,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.enable_vertex_attrib_array(self.location);
            },
            AttributeFormat::Integer2_u32 => {
                gl.vertex_attrib_i_pointer_with_i32(
                    self.location,
                    2,
                    Gl::UNSIGNED_INT,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.enable_vertex_attrib_array(self.location);
            },
            AttributeFormat::Integer3_i8 => {
                gl.vertex_attrib_i_pointer_with_i32(
                    self.location,
                    3,
                    Gl::BYTE,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.enable_vertex_attrib_array(self.location);
            },
            AttributeFormat::Integer3_u8 => {
                gl.vertex_attrib_i_pointer_with_i32(
                    self.location,
                    3,
                    Gl::UNSIGNED_BYTE,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.enable_vertex_attrib_array(self.location);
            },
            AttributeFormat::Integer3_i16 => {
                gl.vertex_attrib_i_pointer_with_i32(
                    self.location,
                    3,
                    Gl::SHORT,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.enable_vertex_attrib_array(self.location);
            },
            AttributeFormat::Integer3_u16 => {
                gl.vertex_attrib_i_pointer_with_i32(
                    self.location,
                    3,
                    Gl::UNSIGNED_SHORT,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.enable_vertex_attrib_array(self.location);
            },
            AttributeFormat::Integer3_i32 => {
                gl.vertex_attrib_i_pointer_with_i32(
                    self.location,
                    3,
                    Gl::INT,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.enable_vertex_attrib_array(self.location);
            },
            AttributeFormat::Integer3_u32 => {
                gl.vertex_attrib_i_pointer_with_i32(
                    self.location,
                    3,
                    Gl::UNSIGNED_INT,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.enable_vertex_attrib_array(self.location);
            },
            AttributeFormat::Integer4_i8 => {
                gl.vertex_attrib_i_pointer_with_i32(
                    self.location,
                    4,
                    Gl::BYTE,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.enable_vertex_attrib_array(self.location);
            },
            AttributeFormat::Integer4_u8 => {
                gl.vertex_attrib_i_pointer_with_i32(
                    self.location,
                    4,
                    Gl::UNSIGNED_BYTE,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.enable_vertex_attrib_array(self.location);
            },
            AttributeFormat::Integer4_i16 => {
                gl.vertex_attrib_i_pointer_with_i32(
                    self.location,
                    4,
                    Gl::SHORT,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.enable_vertex_attrib_array(self.location);
            },
            AttributeFormat::Integer4_u16 => {
                gl.vertex_attrib_i_pointer_with_i32(
                    self.location,
                    4,
                    Gl::UNSIGNED_SHORT,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.enable_vertex_attrib_array(self.location);
            },
            AttributeFormat::Integer4_i32 => {
                gl.vertex_attrib_i_pointer_with_i32(
                    self.location,
                    4,
                    Gl::INT,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.enable_vertex_attrib_array(self.location);
            },
            AttributeFormat::Integer4_u32 => {
                gl.vertex_attrib_i_pointer_with_i32(
                    self.location,
                    4,
                    Gl::UNSIGNED_INT,
                    stride_in_bytes,
                    base_offset_in_bytes + self.offset
                );

                gl.enable_vertex_attrib_array(self.location);
            },
        }
    }
}

#[derive(PartialEq, Debug)]
#[allow(non_camel_case_types)]
pub enum AttributeFormat {
    Float_f32,
    Float_i8_fixed,
    Float_i8_norm,
    Float_i16_fixed,
    Float_i16_norm,
    Float_u8_fixed,
    Float_u8_norm,
    Float_u16_fixed,
    Float_u16_norm,
    Float2_f32,
    Float2_i8_fixed,
    Float2_i8_norm,
    Float2_i16_fixed,
    Float2_i16_norm,
    Float2_u8_fixed,
    Float2_u8_norm,
    Float2_u16_fixed,
    Float2_u16_norm,
    Float3_f32,
    Float3_i8_fixed,
    Float3_i8_norm,
    Float3_i16_fixed,
    Float3_i16_norm,
    Float3_u8_fixed,
    Float3_u8_norm,
    Float3_u16_fixed,
    Float3_u16_norm,
    Float4_f32,
    Float4_i8_fixed,
    Float4_i8_norm,
    Float4_i16_fixed,
    Float4_i16_norm,
    Float4_u8_fixed,
    Float4_u8_norm,
    Float4_u16_fixed,
    Float4_u16_norm,
    Float2x2_f32,
    Float2x2_i8_fixed,
    Float2x2_i8_norm,
    Float2x2_i16_fixed,
    Float2x2_i16_norm,
    Float2x2_u8_fixed,
    Float2x2_u8_norm,
    Float2x2_u16_fixed,
    Float2x2_u16_norm,
    Float2x3_f32,
    Float2x3_i8_fixed,
    Float2x3_i8_norm,
    Float2x3_i16_fixed,
    Float2x3_i16_norm,
    Float2x3_u8_fixed,
    Float2x3_u8_norm,
    Float2x3_u16_fixed,
    Float2x3_u16_norm,
    Float2x4_f32,
    Float2x4_i8_fixed,
    Float2x4_i8_norm,
    Float2x4_i16_fixed,
    Float2x4_i16_norm,
    Float2x4_u8_fixed,
    Float2x4_u8_norm,
    Float2x4_u16_fixed,
    Float2x4_u16_norm,
    Float3x2_f32,
    Float3x2_i8_fixed,
    Float3x2_i8_norm,
    Float3x2_i16_fixed,
    Float3x2_i16_norm,
    Float3x2_u8_fixed,
    Float3x2_u8_norm,
    Float3x2_u16_fixed,
    Float3x2_u16_norm,
    Float3x3_f32,
    Float3x3_i8_fixed,
    Float3x3_i8_norm,
    Float3x3_i16_fixed,
    Float3x3_i16_norm,
    Float3x3_u8_fixed,
    Float3x3_u8_norm,
    Float3x3_u16_fixed,
    Float3x3_u16_norm,
    Float3x4_f32,
    Float3x4_i8_fixed,
    Float3x4_i8_norm,
    Float3x4_i16_fixed,
    Float3x4_i16_norm,
    Float3x4_u8_fixed,
    Float3x4_u8_norm,
    Float3x4_u16_fixed,
    Float3x4_u16_norm,
    Float4x2_f32,
    Float4x2_i8_fixed,
    Float4x2_i8_norm,
    Float4x2_i16_fixed,
    Float4x2_i16_norm,
    Float4x2_u8_fixed,
    Float4x2_u8_norm,
    Float4x2_u16_fixed,
    Float4x2_u16_norm,
    Float4x3_f32,
    Float4x3_i8_fixed,
    Float4x3_i8_norm,
    Float4x3_i16_fixed,
    Float4x3_i16_norm,
    Float4x3_u8_fixed,
    Float4x3_u8_norm,
    Float4x3_u16_fixed,
    Float4x3_u16_norm,
    Float4x4_f32,
    Float4x4_i8_fixed,
    Float4x4_i8_norm,
    Float4x4_i16_fixed,
    Float4x4_i16_norm,
    Float4x4_u8_fixed,
    Float4x4_u8_norm,
    Float4x4_u16_fixed,
    Float4x4_u16_norm,
    Integer_i8,
    Integer_u8,
    Integer_i16,
    Integer_u16,
    Integer_i32,
    Integer_u32,
    Integer2_i8,
    Integer2_u8,
    Integer2_i16,
    Integer2_u16,
    Integer2_i32,
    Integer2_u32,
    Integer3_i8,
    Integer3_u8,
    Integer3_i16,
    Integer3_u16,
    Integer3_i32,
    Integer3_u32,
    Integer4_i8,
    Integer4_u8,
    Integer4_i16,
    Integer4_u16,
    Integer4_i32,
    Integer4_u32,
}

impl AttributeFormat {
    pub fn is_compatible(&self, attribute_type: AttributeType) -> bool {
        match self {
            AttributeFormat::Float_f32 => attribute_type == AttributeType::Float,
            AttributeFormat::Float_i8_fixed => attribute_type == AttributeType::Float,
            AttributeFormat::Float_i8_norm => attribute_type == AttributeType::Float,
            AttributeFormat::Float_i16_fixed => attribute_type == AttributeType::Float,
            AttributeFormat::Float_i16_norm => attribute_type == AttributeType::Float,
            AttributeFormat::Float_u8_fixed => attribute_type == AttributeType::Float,
            AttributeFormat::Float_u8_norm => attribute_type == AttributeType::Float,
            AttributeFormat::Float_u16_fixed => attribute_type == AttributeType::Float,
            AttributeFormat::Float_u16_norm => attribute_type == AttributeType::Float,
            AttributeFormat::Float2_f32 => attribute_type == AttributeType::FloatVector2,
            AttributeFormat::Float2_i8_fixed => attribute_type == AttributeType::FloatVector2,
            AttributeFormat::Float2_i8_norm => attribute_type == AttributeType::FloatVector2,
            AttributeFormat::Float2_i16_fixed => attribute_type == AttributeType::FloatVector2,
            AttributeFormat::Float2_i16_norm => attribute_type == AttributeType::FloatVector2,
            AttributeFormat::Float2_u8_fixed => attribute_type == AttributeType::FloatVector2,
            AttributeFormat::Float2_u8_norm => attribute_type == AttributeType::FloatVector2,
            AttributeFormat::Float2_u16_fixed => attribute_type == AttributeType::FloatVector2,
            AttributeFormat::Float2_u16_norm => attribute_type == AttributeType::FloatVector2,
            AttributeFormat::Float3_f32 => attribute_type == AttributeType::FloatVector3,
            AttributeFormat::Float3_i8_fixed => attribute_type == AttributeType::FloatVector3,
            AttributeFormat::Float3_i8_norm => attribute_type == AttributeType::FloatVector3,
            AttributeFormat::Float3_i16_fixed => attribute_type == AttributeType::FloatVector3,
            AttributeFormat::Float3_i16_norm => attribute_type == AttributeType::FloatVector3,
            AttributeFormat::Float3_u8_fixed => attribute_type == AttributeType::FloatVector3,
            AttributeFormat::Float3_u8_norm => attribute_type == AttributeType::FloatVector3,
            AttributeFormat::Float3_u16_fixed => attribute_type == AttributeType::FloatVector3,
            AttributeFormat::Float3_u16_norm => attribute_type == AttributeType::FloatVector3,
            AttributeFormat::Float4_f32 => attribute_type == AttributeType::FloatVector4,
            AttributeFormat::Float4_i8_fixed => attribute_type == AttributeType::FloatVector4,
            AttributeFormat::Float4_i8_norm => attribute_type == AttributeType::FloatVector4,
            AttributeFormat::Float4_i16_fixed => attribute_type == AttributeType::FloatVector4,
            AttributeFormat::Float4_i16_norm => attribute_type == AttributeType::FloatVector4,
            AttributeFormat::Float4_u8_fixed => attribute_type == AttributeType::FloatVector4,
            AttributeFormat::Float4_u8_norm => attribute_type == AttributeType::FloatVector4,
            AttributeFormat::Float4_u16_fixed => attribute_type == AttributeType::FloatVector4,
            AttributeFormat::Float4_u16_norm => attribute_type == AttributeType::FloatVector4,
            AttributeFormat::Float2x2_f32 => attribute_type == AttributeType::FloatMatrix2x2,
            AttributeFormat::Float2x2_i8_fixed => attribute_type == AttributeType::FloatMatrix2x2,
            AttributeFormat::Float2x2_i8_norm => attribute_type == AttributeType::FloatMatrix2x2,
            AttributeFormat::Float2x2_i16_fixed => attribute_type == AttributeType::FloatMatrix2x2,
            AttributeFormat::Float2x2_i16_norm => attribute_type == AttributeType::FloatMatrix2x2,
            AttributeFormat::Float2x2_u8_fixed => attribute_type == AttributeType::FloatMatrix2x2,
            AttributeFormat::Float2x2_u8_norm => attribute_type == AttributeType::FloatMatrix2x2,
            AttributeFormat::Float2x2_u16_fixed => attribute_type == AttributeType::FloatMatrix2x2,
            AttributeFormat::Float2x2_u16_norm => attribute_type == AttributeType::FloatMatrix2x2,
            AttributeFormat::Float2x3_f32 => attribute_type == AttributeType::FloatMatrix2x3,
            AttributeFormat::Float2x3_i8_fixed => attribute_type == AttributeType::FloatMatrix2x3,
            AttributeFormat::Float2x3_i8_norm => attribute_type == AttributeType::FloatMatrix2x3,
            AttributeFormat::Float2x3_i16_fixed => attribute_type == AttributeType::FloatMatrix2x3,
            AttributeFormat::Float2x3_i16_norm => attribute_type == AttributeType::FloatMatrix2x3,
            AttributeFormat::Float2x3_u8_fixed => attribute_type == AttributeType::FloatMatrix2x3,
            AttributeFormat::Float2x3_u8_norm => attribute_type == AttributeType::FloatMatrix2x3,
            AttributeFormat::Float2x3_u16_fixed => attribute_type == AttributeType::FloatMatrix2x3,
            AttributeFormat::Float2x3_u16_norm => attribute_type == AttributeType::FloatMatrix2x3,
            AttributeFormat::Float2x4_f32 => attribute_type == AttributeType::FloatMatrix2x4,
            AttributeFormat::Float2x4_i8_fixed => attribute_type == AttributeType::FloatMatrix2x4,
            AttributeFormat::Float2x4_i8_norm => attribute_type == AttributeType::FloatMatrix2x4,
            AttributeFormat::Float2x4_i16_fixed => attribute_type == AttributeType::FloatMatrix2x4,
            AttributeFormat::Float2x4_i16_norm => attribute_type == AttributeType::FloatMatrix2x4,
            AttributeFormat::Float2x4_u8_fixed => attribute_type == AttributeType::FloatMatrix2x4,
            AttributeFormat::Float2x4_u8_norm => attribute_type == AttributeType::FloatMatrix2x4,
            AttributeFormat::Float2x4_u16_fixed => attribute_type == AttributeType::FloatMatrix2x4,
            AttributeFormat::Float2x4_u16_norm => attribute_type == AttributeType::FloatMatrix2x4,
            AttributeFormat::Float3x2_f32 => attribute_type == AttributeType::FloatMatrix3x2,
            AttributeFormat::Float3x2_i8_fixed => attribute_type == AttributeType::FloatMatrix3x2,
            AttributeFormat::Float3x2_i8_norm => attribute_type == AttributeType::FloatMatrix3x2,
            AttributeFormat::Float3x2_i16_fixed => attribute_type == AttributeType::FloatMatrix3x2,
            AttributeFormat::Float3x2_i16_norm => attribute_type == AttributeType::FloatMatrix3x2,
            AttributeFormat::Float3x2_u8_fixed => attribute_type == AttributeType::FloatMatrix3x2,
            AttributeFormat::Float3x2_u8_norm => attribute_type == AttributeType::FloatMatrix3x2,
            AttributeFormat::Float3x2_u16_fixed => attribute_type == AttributeType::FloatMatrix3x2,
            AttributeFormat::Float3x2_u16_norm => attribute_type == AttributeType::FloatMatrix3x2,
            AttributeFormat::Float3x3_f32 => attribute_type == AttributeType::FloatMatrix3x3,
            AttributeFormat::Float3x3_i8_fixed => attribute_type == AttributeType::FloatMatrix3x3,
            AttributeFormat::Float3x3_i8_norm => attribute_type == AttributeType::FloatMatrix3x3,
            AttributeFormat::Float3x3_i16_fixed => attribute_type == AttributeType::FloatMatrix3x3,
            AttributeFormat::Float3x3_i16_norm => attribute_type == AttributeType::FloatMatrix3x3,
            AttributeFormat::Float3x3_u8_fixed => attribute_type == AttributeType::FloatMatrix3x3,
            AttributeFormat::Float3x3_u8_norm => attribute_type == AttributeType::FloatMatrix3x3,
            AttributeFormat::Float3x3_u16_fixed => attribute_type == AttributeType::FloatMatrix3x3,
            AttributeFormat::Float3x3_u16_norm => attribute_type == AttributeType::FloatMatrix3x3,
            AttributeFormat::Float3x4_f32 => attribute_type == AttributeType::FloatMatrix3x4,
            AttributeFormat::Float3x4_i8_fixed => attribute_type == AttributeType::FloatMatrix3x4,
            AttributeFormat::Float3x4_i8_norm => attribute_type == AttributeType::FloatMatrix3x4,
            AttributeFormat::Float3x4_i16_fixed => attribute_type == AttributeType::FloatMatrix3x4,
            AttributeFormat::Float3x4_i16_norm => attribute_type == AttributeType::FloatMatrix3x4,
            AttributeFormat::Float3x4_u8_fixed => attribute_type == AttributeType::FloatMatrix3x4,
            AttributeFormat::Float3x4_u8_norm => attribute_type == AttributeType::FloatMatrix3x4,
            AttributeFormat::Float3x4_u16_fixed => attribute_type == AttributeType::FloatMatrix3x4,
            AttributeFormat::Float3x4_u16_norm => attribute_type == AttributeType::FloatMatrix3x4,
            AttributeFormat::Float4x2_f32 => attribute_type == AttributeType::FloatMatrix4x2,
            AttributeFormat::Float4x2_i8_fixed => attribute_type == AttributeType::FloatMatrix4x2,
            AttributeFormat::Float4x2_i8_norm => attribute_type == AttributeType::FloatMatrix4x2,
            AttributeFormat::Float4x2_i16_fixed => attribute_type == AttributeType::FloatMatrix4x2,
            AttributeFormat::Float4x2_i16_norm => attribute_type == AttributeType::FloatMatrix4x2,
            AttributeFormat::Float4x2_u8_fixed => attribute_type == AttributeType::FloatMatrix4x2,
            AttributeFormat::Float4x2_u8_norm => attribute_type == AttributeType::FloatMatrix4x2,
            AttributeFormat::Float4x2_u16_fixed => attribute_type == AttributeType::FloatMatrix4x2,
            AttributeFormat::Float4x2_u16_norm => attribute_type == AttributeType::FloatMatrix4x2,
            AttributeFormat::Float4x3_f32 => attribute_type == AttributeType::FloatMatrix4x3,
            AttributeFormat::Float4x3_i8_fixed => attribute_type == AttributeType::FloatMatrix4x3,
            AttributeFormat::Float4x3_i8_norm => attribute_type == AttributeType::FloatMatrix4x3,
            AttributeFormat::Float4x3_i16_fixed => attribute_type == AttributeType::FloatMatrix4x3,
            AttributeFormat::Float4x3_i16_norm => attribute_type == AttributeType::FloatMatrix4x3,
            AttributeFormat::Float4x3_u8_fixed => attribute_type == AttributeType::FloatMatrix4x3,
            AttributeFormat::Float4x3_u8_norm => attribute_type == AttributeType::FloatMatrix4x3,
            AttributeFormat::Float4x3_u16_fixed => attribute_type == AttributeType::FloatMatrix4x3,
            AttributeFormat::Float4x3_u16_norm => attribute_type == AttributeType::FloatMatrix4x3,
            AttributeFormat::Float4x4_f32 => attribute_type == AttributeType::FloatMatrix4x4,
            AttributeFormat::Float4x4_i8_fixed => attribute_type == AttributeType::FloatMatrix4x4,
            AttributeFormat::Float4x4_i8_norm => attribute_type == AttributeType::FloatMatrix4x4,
            AttributeFormat::Float4x4_i16_fixed => attribute_type == AttributeType::FloatMatrix4x4,
            AttributeFormat::Float4x4_i16_norm => attribute_type == AttributeType::FloatMatrix4x4,
            AttributeFormat::Float4x4_u8_fixed => attribute_type == AttributeType::FloatMatrix4x4,
            AttributeFormat::Float4x4_u8_norm => attribute_type == AttributeType::FloatMatrix4x4,
            AttributeFormat::Float4x4_u16_fixed => attribute_type == AttributeType::FloatMatrix4x4,
            AttributeFormat::Float4x4_u16_norm => attribute_type == AttributeType::FloatMatrix4x4,
            AttributeFormat::Integer_i8 => attribute_type == AttributeType::Integer,
            AttributeFormat::Integer_u8 => attribute_type == AttributeType::UnsignedInteger,
            AttributeFormat::Integer_i16 => attribute_type == AttributeType::Integer,
            AttributeFormat::Integer_u16 => attribute_type == AttributeType::UnsignedInteger,
            AttributeFormat::Integer_i32 => attribute_type == AttributeType::Integer,
            AttributeFormat::Integer_u32 => attribute_type == AttributeType::UnsignedInteger,
            AttributeFormat::Integer2_i8 => attribute_type == AttributeType::IntegerVector2,
            AttributeFormat::Integer2_u8 => attribute_type == AttributeType::UnsignedIntegerVector2,
            AttributeFormat::Integer2_i16 => attribute_type == AttributeType::IntegerVector2,
            AttributeFormat::Integer2_u16 => attribute_type == AttributeType::UnsignedIntegerVector2,
            AttributeFormat::Integer2_i32 => attribute_type == AttributeType::IntegerVector2,
            AttributeFormat::Integer2_u32 => attribute_type == AttributeType::UnsignedIntegerVector2,
            AttributeFormat::Integer3_i8 => attribute_type == AttributeType::IntegerVector3,
            AttributeFormat::Integer3_u8 => attribute_type == AttributeType::UnsignedIntegerVector3,
            AttributeFormat::Integer3_i16 => attribute_type == AttributeType::IntegerVector3,
            AttributeFormat::Integer3_u16 => attribute_type == AttributeType::UnsignedIntegerVector3,
            AttributeFormat::Integer3_i32 => attribute_type == AttributeType::IntegerVector3,
            AttributeFormat::Integer3_u32 => attribute_type == AttributeType::UnsignedIntegerVector3,
            AttributeFormat::Integer4_i8 => attribute_type == AttributeType::IntegerVector4,
            AttributeFormat::Integer4_u8 => attribute_type == AttributeType::UnsignedIntegerVector4,
            AttributeFormat::Integer4_i16 => attribute_type == AttributeType::IntegerVector4,
            AttributeFormat::Integer4_u16 => attribute_type == AttributeType::UnsignedIntegerVector4,
            AttributeFormat::Integer4_i32 => attribute_type == AttributeType::IntegerVector4,
            AttributeFormat::Integer4_u32 => attribute_type == AttributeType::UnsignedIntegerVector4,
        }
    }
}
