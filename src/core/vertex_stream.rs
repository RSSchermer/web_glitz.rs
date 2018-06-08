//enum VertexStream {
//    Array(VertexSource),
//    Indexed(VertexSource, IndexSource),
//    Instanced(VertexSource, VertexSource)
//}

pub trait VertexSource {
    fn input_attribute_description(&self, name: &str) -> &VertexAttributeInputDescription;
}

struct VertexAttributeInputDescription {

}

trait VertexStreamDescription {
    fn input_attribute_description(&self, name: &str) -> &VertexAttributeInputDescription;

    fn indices(&self) -> Option<Buffer>;

    fn instance_count(&self) -> Option<usize>;
}

pub struct VertexStream {
    attributes: Hashmap<&str, VertexAttributeInputDescription>,
    indices: Option<Buffer>,
    skip: u32,
    count: u32,
    instance_count: u32
}

impl VertexStream {

}

impl VertexStreamDescription for VertexStream {

}
