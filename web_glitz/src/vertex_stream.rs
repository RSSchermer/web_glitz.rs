use std::hash::Hash;
use std::ops::RangeBounds;
use std::ops::RangeFull;
use std::ops::Index;

pub trait VertexSource {
    fn input_attribute_description(&self, name: &str) -> &VertexInputAttributeDescription;
}

pub struct VertexInputAttributeDescription {
    pub format: AttributeFormat,
    pub offset_in_bytes: usize,
    pub stride_in_bytes: usize,
    pub divisor: usize
}

enum AttributeFormat {
    Float,
    Integer,
    UnsignedInteger,
    Vector2,
    Vector3,
    Vector4,
    IVector2,
    IVector3,
    IVector4,
    UVector2,
    UVector3,
    UVector4,
    Matrix2x2,
    Matrix2x3,
    Matrix2x4,
    Matrix3x2,
    Matrix3x3,
    Matrix3x4,
    Matrix4x2,
    Matrix4x3,
    Matrix4x4
}

enum IndexFormat {
    UnsignedByte,
    UnsignedShort,
    UnsignedInteger
}

trait AbstractVertexStreamDescription: Hash {
    fn get_input_attribute_description(&self, name: &str) -> Option<(Buffer, VertexInputAttributeDescription)>;

    fn indices(&self) -> Option<(Buffer, IndexFormat)> {
        None
    }
}

pub struct VertexStreamSlice<T, R> where T: AbstractVertexStreamDescription, R: RangeBounds<usize> {
    stream_description: T,
    range: R
}

impl <T, R> VertexStreamSlice<T, R> where T: AbstractVertexStreamDescription, R: RangeBounds<usize> {
    pub fn new(stream_description: T, range: R) -> Self {
        VertexStreamSlice {
            stream_description,
            range
        }
    }
}

impl <T> Into<VertexStreamSlice<T, RangeFull>> for T where T: AbstractVertexStreamDescription {
    fn into(self) -> VertexStreamSlice<T, RangeFull> {
        VertexStreamSlice::new(self, RangeFull)
    }
}

impl <T, R> Index<R> for T where T: VertexStreamDescription, R: RangeBounds<usize> {
    type Output = VertexStreamSlice<T, R>;

    fn index(&self, range: R) -> Self::Output {
        VertexStreamSlice {
            stream_description: self,
            range
        }
    }
}

pub struct VertexStreamDescription {
    attributes: FnvHashmap<u64, (Buffer, VertexInputAttributeDescription)>,
    indices: Option<(Buffer, IndexFormat)>
}

impl VertexStreamDescription {
    fn start() -> VertexStreamDescriptionBuilder {
        VertexStreamDescriptionBuilder::new()
    }
}

impl AbstractVertexStreamDescription for VertexStreamDescription {
    fn get_input_attribute_description(&self, name: &str) -> Option<(Buffer, VertexInputAttributeDescription)> {
        let hasher = FnvHasher::default();
        let hash = hasher.write(name).finish();

        self.attributes.get(hash)
    }

    fn indices(&self) -> Option<(Buffer, IndexFormat)> {
        self.indices
    }
}

pub struct VertexStreamDescriptionBuilder {
    attributes: FnvHashmap<u64, (Buffer, VertexInputAttributeDescription)>,
    indices: Option<(Buffer, IndexFormat)>,
    error: Option<VertexStreamDescriptionBuildError>
}

#[derive(Fail, PartialEq, Debug)]
enum VertexStreamDescriptionBuildError {
    #[fail(display = "A vertex stream description must include at least 1 attribute description")]
    NoAttributes,
    #[fail(display = "The context of the buffer on which attribute `{}` is defined does not match the context of prior attributes", _0)]
    AttributeBufferContextMismatch(String),
    #[fail(display = "The context of the index data buffer does not match the context of the attribute data buffers")]
    IndexBufferContextMismatch
}

impl VertexStreamDescriptionBuilder {
    fn new() -> Self {
        VertexStreamDescriptionBuilder {
            attributes: FnvHashmap::new(),
            indices: None,
            error: None
        }
    }

    fn input_attribute(&mut self, name: &str, buffer: Buffer, description: VertexInputAttributeDescription) {
        if self.error.is_none() {
            let hasher = FnvHasher::default();
            let hash = hasher.write(name).finish();

            match self.attributes.values().first().map(|b| b.context) {
                Some(context) if context != buffer.context => {
                    self.error = Some(VertexStreamDescriptionBuildError::AttributeBufferContextMismatch(name.to_string()))
                },
                _ => {
                    self.attributes.set(hash, (buffer, description))
                }
            }
        }
    }

    fn indices(&mut self, buffer: Buffer, format: IndexFormat) {
        self.indices = Some((buffer, format));
    }

    fn build(self) -> Result<VertexStreamDescription, VertexStreamDescriptionBuildError> {
        if let Some(error) = self.error {
            Err(error)
        } else {
            if let Some(context) = self.attributes.values().first().map(|b| b.context) {
                match self.indices {
                    Some((index_buffer, _)) if index_buffer.context != context => {
                        Err(VertexStreamDescriptionBuildError::IndexBufferContextMismatch)
                    },
                    _ => {
                        Ok(VertexStreamDescription {
                            attributes: self.attributes,
                            indices: self.indices
                        })
                    }
                }
            } else {
                Err(VertexStreamDescriptionBuildError::NoAttributes)
            }
        }
    }
}
