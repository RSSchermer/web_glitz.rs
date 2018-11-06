use super::buffer::BufferHandle;





pub struct VertexAttributeDescriptor {
    location: u32,
    offset: u8,
    format: AttributeFormat
}

pub struct VertexBufferDescriptor<T, C> where C: RenderingContext {
    buffer: BufferHandle<[T], C>,
    offset: u32,
    stride: u8,
    divisor: u32,
    attribute_descriptors: Vec<VertexAttributeDescriptor>
}

impl<T, C> VertexBufferDescriptor<T, C> {
    pub fn new(buffer: &BufferHandle<T, C>, offset: u32, stride: u8, divisor: u32, attributes: A) -> Self where A: IntoIterator<Item> {

    }

    pub fn buffer(&self) -> &BufferHandle<[T], C> {
        self.buffer
    }

    pub fn offset(&self) -> u32 {
        self.offset
    }

    pub fn stride(&self) -> u8 {
        self.stride
    }

    pub fn divisor(&self) -> u32 {
        self.divisor
    }

    pub fn attribute_descriptors(&self) -> &[VertexAttributeDescriptor] {
        self.attribute_descriptors
    }
}

pub struct VertexBufferDescriptorBuilder<T, C> where C: RenderingContext {
    vertex_buffer_descriptor: VertexBufferDescriptor<T, C>
}

pub struct VertexInputAttributesDescriptor<C> {
    bindings: Vec<VertexAttributeDescriptor<C>>
}

impl<C> VertexInputAttributesDescriptor<C> {
    pub fn begin() -> VertexInputAttributesDescriptorBuilder<C> {
        VertexInputAttributesDescriptorBuilder {

        }
    }
}

struct VertexInputAttributesDescriptorBuilder<C> {

}

impl<C> VertexInputAttributesDescriptorBuilder<C> {
    pub fn begin_vertex_buffer(self, input: &BufferHandle<C>, offset: u32, stride: Stride) -> VertexBufferBuilder<C> {

    }

    pub fn finish(self) -> VertexInputAttributesDescriptor<C> {

    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Stride {
    Packed,
    Manual(u8)
}
