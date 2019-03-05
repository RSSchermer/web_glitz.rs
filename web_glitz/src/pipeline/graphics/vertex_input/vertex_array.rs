use fnv::FnvHasher;

pub unsafe trait StaticInputAttributeLayout {
    fn compatibility(attribute_descriptors: &[VertexAttributeDescriptor]) -> Compatiblity;
}

pub struct AttributeIdentifier {
    name: String,
    hash_fnv64: u64
}

impl AttributeIdentifier {
    pub(crate) fn new(name: String) -> Self {
        let mut hasher = FnvHasher::default();

        name.hash(hasher);

        let hash_fnv64 = hasher.finish();

        AttributeIdentifier {
            name,
            hash_fnv64
        }
    }
}

pub enum Compatibility {
    Compatible,
    MissingAttribute {
        identifier: AttributeIdentifier
    },
    LocationMismatch {
        identifier: AttributeIdentifier,
        expected_location: u32,
        actual_location: u32
    },
    TypeMismatch {
        identifier: AttributeIdentifier,
        expected_type: AttributeType
    }
}

pub enum AttributeType {
    Float,
    FloatVector2,
    FloatVector3,
    FloatVector4,
    Integer,
    IntegerVector2,
    IntegerVector3,
    IntegerVector4,
    UnsignedInteger,
    UnsignedIntegerVector2,
    UnsignedIntegerVector3,
    UnsignedIntegerVector4,
    Bool,
    BoolVector2,
    BoolVector3,
    BoolVector4
}

pub unsafe trait IndexFormat {
    fn id() -> u32;
}

unsafe impl IndexFormat for u8 {
    fn id() -> u32 {
        Gl::UNSIGNED_BYTE
    }
}

unsafe impl IndexFormat for u16 {
    fn id() -> u32 {
        Gl::UNSIGNED_SHORT
    }
}

unsafe impl IndexFormat for u32 {
    fn id() -> u32 {
        Gl::UNSIGNED_INT
    }
}

pub trait VertexArrayDescription {
    type Encoding;

    fn encode(&self) -> VertexArrayDescriptor<Self::Encoding>;
}

pub trait AsBufferView<T> where T: ?Sized {
    fn as_buffer_view(&self) -> BufferView<T>;
}

impl<V> VertexArrayDescription for BufferView<V> where V: Vertex {
    type Encoding = ;

    fn encode(&self) -> VertexArrayDescriptor<Self::Encoding> {

    }
}

impl<V0, V1, V2> VertexArrayDescription for (V0, V1, V2)
    where
        V0: VertexArrayDescription,
        V1: VertexArrayDescription,
        V2: VertexArrayDescription
{
    type Encoding = ;

    fn encode(&self) -> VertexArrayDescriptor<Self::Encoding> {

    }
}

pub struct VertexArrayDescriptor<E> {
    encoding: E
}

pub struct VertexArray<V, I> {
    id: Arc<Option<JsId>>,
    len: usize,
    _vertex_layout_marker: marker::PhantomData<V>,
    _indices_marker: marker::PhantomData<Buffer<I>>
}

impl<V, I> VertexArray<V, I> {
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn range<R>(&self, range: R) -> Option<VertexArraySlice<V, I>> where R: VertexArrayRange {
        rante.range(VertexArraySlice {
            vertex_array: self,
            offset: 0,
            len: self.len,
        })
    }

    pub unsafe fn range_unchecked<R>(&self, range: R) -> Option<VertexArraySlice<V, I>> where R: VertexArrayRange {
        rante.range_unchecked(VertexArraySlice {
            vertex_array: self,
            offset: 0,
            len: self.len,
        })
    }

    pub fn instanced(&self, instance_count: usize) -> Instanced<VertexArraySlice<V, I>> {
        Instanced {
            vertex_array: VertexArraySlice {
                vertex_array: self,
                offset: 0,
                len: self.len,
            }
        }
    }
}

pub trait VertexArrayRange {
    fn range<'a, V, I>(self, vertex_array: VertexArraySlice<I, V>) -> Option<VertexArraySlice<V, I>>;

    unsafe fn range_unchecked<'a, V, I>(self, vertex_array: VertexArraySlice<I, V>) -> VertexArraySlice<V, I>;
}

#[derive(Clone, Copy)]
pub struct VertexArraySlice<'a, V, I> {
    vertex_array: &'a VertexArray<V, I>,
    offset: usize,
    len: usize,
}

impl<V, I> VertexArraySlice<V, I> {
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn range<R>(&self, range: R) -> Option<VertexArraySlice<V, I>> where R: VertexArrayRange {
        range.range(self)
    }

    pub unsafe fn range_unchecked<R>(&self, range: R) -> Option<VertexArraySlice<V, I>> where R: VertexArrayRange {
        range.range_unchecked(self)
    }

    pub fn instanced(&self, instance_count: usize) -> InstancedVertexArraySlice<V, I> {
        InstancedVertexArraySlice {
            vertex_array: self,
            offset: self.offset,
            len: self.len,
            instance_count: usize
        }
    }
}

#[derive(Clone, Copy)]
pub struct InstancedVertexArraySlice<'a, V, I> {
    vertex_array: &'a VertexArray<V, I>,
    offset: usize,
    len: usize,
    instance_count: usize,
}

pub trait VertexInputStreamDescription {
    type Layout: VertexInputLayout;

    type IndexFormat: IndexFormat;

    fn descriptor(&self) -> VertexInputStreamDescriptor;
}

pub struct VertexInputStreamDescriptor {
    vertex_array_id: Arc<Option<JsId>>,
    offset: usize,
    count: usize,
    instance_count: usize
}

impl<V, I> VertexInputStreamDescription for VertexArray<V, I> {
    type Layout = V;

    type IndexFormat = I;

    fn descriptor(&self) -> VertexInputStreamDescriptor {
        VertexInputStreamDescriptor {
            vertex_array_id: self.id.clone(),
            offset: 0,
            count: self.len,
            instance_count: 1
        }
    }
}

impl<'a, V, I> VertexInputStreamDescription for VertexArraySlice<'a, V, I> {
    type Layout = V;

    type IndexFormat = I;

    fn descriptor(&self) -> VertexInputStreamDescriptor {
        VertexInputStreamDescriptor {
            vertex_array_id: self.id.clone(),
            offset: self.offset,
            count: self.len,
            instance_count: 1
        }
    }
}

impl<'a, V, I> VertexInputStreamDescription for InstancedVertexArraySlice<'a, V, I> {
    type Layout = V;

    type IndexFormat = I;

    fn descriptor(&self) -> VertexInputStreamDescriptor {
        VertexInputStreamDescriptor {
            vertex_array_id: self.id.clone(),
            offset: self.offset,
            count: self.len,
            instance_count: self.instance_count
        }
    }
}
