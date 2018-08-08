trait Uniforms {
    fn get(&self, identifier: &UniformIdentifier) -> &AsUniformValue;
}

#[derive(PartialEq, Hash)]
struct UniformIdentifier {
    segments: Vec<UniformIdentifierSegment>
}

impl UniformIdentifier {
    fn from_string(string: &str) -> Self {
        UniformIdentifier {
            segments: string.split(".").map(|s| UniformIdentifierSegment::from_string(s)).collect()
        }
    }

    fn is_array_identifier(&self) -> bool {
        if let Some(segment) = self.segments.last() {
            segment.is_array_identifier()
        } else {
            false
        }
    }
}

impl Display for UniformIdentifier {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, segments.map(|s| s.to_string()).join("."))
    }
}

#[derive(Clone, PartialEq, Hash)]
enum UniformIdentifierSegment {
    Simple(String),
    ArrayElement(String, u32)
}

impl UniformIdentifierSegment {
    fn from_string(string: &str) -> Self {
        let parts = string.split("[");

        if parts.len() == 1 {
            UniformIdentifierSegment::Simple(parts[0])
        } else {
            let index = parts[1].trim_right_matches("]").parse::<u32>().unwrap();

            UniformIdentifierSegment::ArrayElement(parts[0], index);
        }
    }

    fn is_array_identifier(&self) -> bool {
        if let UniformIdentifierSegment::ArrayElement(_, _) = self {
            true
        } else {
            false
        }
    }
}

impl Into<UniformIdentifier> for UniformIdentifierSegment {
    fn into(self) -> UniformIdentifier {
        UniformIdentifier {
            segments: vec![self]
        }
    }
}

impl Display for UniformIdentifierSegment {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            UniformIdentifierSegment::Simple(name) => write!(f, name),
            UniformIdentifierSegment::ArrayElement(array_name, index) => write!(f, "{}[{}]", array_name, index)
        }
    }
}

enum UniformValue<'a> {
    Float(f32),
    Vector2((f32, f32)),
    Vector3((f32, f32)),
    Vector4((f32, f32)),
    Matrix2x2([f32;4]),
    Matrix2x3([f32;6]),
    Matrix2x4([f32;8]),
    Matrix3x2([f32;6]),
    Matrix3x3([f32;9]),
    Matrix3x4([f32;12]),
    Matrix4x2([f32;8]),
    Matrix4x3([f32;12]),
    Matrix4x4([f32;16]),
    Boolean(bool),
    BooleanVector2((bool, bool)),
    BooleanVector3((bool, bool, bool)),
    BooleanVector4((bool, bool, bool, bool)),
    Integer(i32),
    IntegerVector2((i32, i32)),
    IntegerVector3((i32, i32, i32)),
    IntegerVector4((i32, i32, i32, i32)),
    UnsignedInteger(u32),
    UnsignedIntegerVector2((u32, u32)),
    UnsignedIntegerVector3((u32, u32, u32)),
    UnsignedIntegerVector4((u32, u32, u32, u32)),
    FloatArray(ArrayValue<'a, f32>),
    Vector2Array(ArrayValue<'a, (f32, f32)>),
    Vector3Array(ArrayValue<'a, (f32, f32, f32)>),
    Vector4Array(ArrayValue<'a, (f32, f32, f32, f32)>),
    Matrix2x2Array(ArrayValue<'a, [f32;4]>),
    Matrix2x3Array(ArrayValue<'a, [f32;6]>),
    Matrix2x4Array(ArrayValue<'a, [f32;8]>),
    Matrix3x2Array(ArrayValue<'a, [f32;6]>),
    Matrix3x3Array(ArrayValue<'a, [f32;9]>),
    Matrix3x4Array(ArrayValue<'a, [f32;12]>),
    Matrix4x2Array(ArrayValue<'a, [f32;8]>),
    Matrix4x3Array(ArrayValue<'a, [f32;12]>),
    Matrix4x4Array(ArrayValue<'a, [f32;16]>),
    BooleanArray(ArrayValue<'a, bool>),
    BooleanVector2Array(ArrayValue<'a, (bool, bool)>),
    BooleanVector3Array(ArrayValue<'a, (bool, bool, bool)>),
    BooleanVector4Array(ArrayValue<'a, (bool, bool, bool, bool)>),
    IntegerArray(ArrayValue<'a, i32>),
    IntegerVector2Array(ArrayValue<'a, (i32, i32)>),
    IntegerVector3Array(ArrayValue<'a, (i32, i32, i32)>),
    IntegerVector4Array(ArrayValue<'a, (i32, i32, i32, i32)>),
    UnsignedIntegerArray(ArrayValue<'a, u32>),
    UnsignedIntegerVector2Array(ArrayValue<'a, (u32, u32)>),
    UnsignedIntegerVector3Array(ArrayValue<'a, (u32, u32, u32)>),
    UnsignedIntegerVector4Array(ArrayValue<'a, (u32, u32, u32, u32)>),
}

enum ArrayValue<'a, T> {
    Slice(&'a [T]),
    BoxedSlice(Box<[T]>)
}

impl<'a, T> From<&'a [T]> for ArrayValue<'a, T> {
    fn from(slice: &[T]) -> ArrayValue<T> {
        ArrayValue::Slice(slice)
    }
}

impl<T> From<Box<[T]>> for ArrayValue<'static, T> {
    fn from(boxed_slice: Box<[T]>) -> ArrayValue<'static, T> {
        ArrayValue::BoxedSlice(boxed_slice)
    }
}

impl<'a, T> Borrow<[T]> for ArrayValue<'a, T> {
    fn borrow(&self) -> &[T] {
        match *self {
            ArrayValue::Slice(value) => value,
            ArrayValue::BoxedSlice(value) => value.borrow()
        }
    }
}

pub trait AsUniformValue: Display {
    fn as_uniform_value(&self) -> UniformValue;
}

impl AsUniformValue for f32 {
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::Float(*self)
    }
}

impl AsUniformValue for (f32, f32) {
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::Vector2(*self)
    }
}

impl AsUniformValue for [f32; 2] {
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::Vector2((self[0], self[1]))
    }
}

impl AsUniformValue for (f32, f32, f32) {
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::Vector3(*self)
    }
}

impl AsUniformValue for [f32; 3] {
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::Vector3((self[0], self[1], self[2]))
    }
}

impl AsUniformValue for (f32, f32, f32, f32) {
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::Vector4(*self)
    }
}

impl AsUniformValue for [f32; 4] {
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::Vector4((self[0], self[1], self[2], self[3]))
    }
}
