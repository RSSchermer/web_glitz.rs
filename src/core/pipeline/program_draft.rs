use super::super::webgl_rendering_context::{WebGL2RenderingContext as gl, WebGLProgram,
                                            WebGLShader, WebGLUniformLocation};
use super::super::vertex_stream::AttributeFormat;

static ATTRIBUTE_FORMAT_MAPPING: phf::Map<(GLenum, GLint), AttributeFormat> = phf_map! {
    (gl.FLOAT, 1) => AttributeFormat::Float,
    (gl.FLOAT_VEC2, 1) => AttributeFormat::Vector2,
    (gl.FLOAT_VEC3, 1) => AttributeFormat::Vector3,
    (gl.FLOAT_VEC4, 1) => AttributeFormat::Vector4,
    (gl.INT, 1) => AttributeFormat::Integer,
    (gl.INT_VEC2, 1) => AttributeFormat::IVector2,
    (gl.INT_VEC3, 1) => AttributeFormat::IVector3,
    (gl.INT_VEC4, 1) => AttributeFormat::IVector4,
    (gl.UNSIGNED_INT, 1) => AttributeFormat::UnsignedInteger,
    (gl.UNSIGNED_INT_VEC2, 1) => AttributeFormat::UVector2,
    (gl.UNSIGNED_INT_VEC3, 1) => AttributeFormat::UVector3,
    (gl.UNSIGNED_INT_VEC4, 1) => AttributeFormat::UVector4,
    (gl.FLOAT_VEC2, 2) => AttributeFormat::Matrix2x2,
    (gl.FLOAT_VEC3, 2) => AttributeFormat::Matrix2x3,
    (gl.FLOAT_VEC4, 2) => AttributeFormat::Matrix2x4,
    (gl.FLOAT_VEC2, 3) => AttributeFormat::Matrix3x2,
    (gl.FLOAT_VEC3, 3) => AttributeFormat::Matrix3x3,
    (gl.FLOAT_VEC4, 3) => AttributeFormat::Matrix3x4,
    (gl.FLOAT_VEC2, 4) => AttributeFormat::Matrix4x2,
    (gl.FLOAT_VEC3, 4) => AttributeFormat::Matrix4x3,
    (gl.FLOAT_VEC4, 4) => AttributeFormat::Matrix4x4
};

static TRANSFORM_FEEDBACK_BUFFER_MODE_MAPPING: phf::Map<TransformFeedbackBufferMode, GLenum> = phf_map! {
    TransformFeedbackBufferMode::Separate => gl.SEPARATE_ATTRIBS,
    TransformFeedbackBufferMode::Interleaved => gl.INTERLEAVED_ATTRIBS
};

struct Program {
    vertex_shader: Shader,
    fragment_shader: Shader,
    transform_feedback: TransformFeedbackVaryings,
    gl_program: WebGLProgram,
    attributes: Vec<AttributeInfo>,
    uniforms: Vec<UniformInfo>,
    uniform_blocks: Vec<UniformBlockInfo>
}

#[derive(Fail, PartialEq, Debug)]
enum ProgramCreationError {
    #[fail(display = "Program failed to link (message: `{}`)", _0)]
    LinkingFailure(Option<String>),
    #[fail(display = "Tried to create a program with an unsupported attribute (type: `{}`, size: {})", _0, _1)]
    UnsupportedAttribute(GLenum, GLint)
}

impl Program {
    pub fn link(context: Context, vertex_shader: Shader, fragment_shader: Shader, transform_feedback: Option<TransformFeedbackVaryings>) -> Result<Program, ProgramCreationError> {
        let gl_context = context.gl_context;
        let gl_program = glContext.create_program();

        gl_context.attach_shader(&gl_program, &vertex_shader.gl_shader);
        gl_context.attach_shader(&gl_program, &fragment_shader.gl_shader);

        if let Some(transform_feedback) = transform_feedback {
            gl_context.transform_feedback_varyings(&gl_program, &transform_feedback.output_names(), TRANSFORM_FEEDBACK_BUFFER_MODE_MAPPING.get(transform_feedback.mode()));
        }

        gl_context.link_program(&gl_program);

        if !gl_context.get_program_parameter(&gl_program, gl.LINK_STATUS).try_into().unwrap() {
            return ProgramCreationError::LinkingFailure(gl_context.get_program_info_log(&gl_program));
        }

        let mut texture_slot_usage_count = 0;

        let active_attributes_count: i32 = gl_context.get_program_parameter(gl_program, gl_context.ACTIVE_ATTRIBUTES).try_into().unwrap();
        let active_attributes = Vec::with_capacity(active_attributes_count);

        for i in 0..active_attributes_count {
            if let Some(info) = glContext.get_active_attrib(glProgramObject, i) {
                let name = info.name;
                let location = glContext.getAttribLocation(glProgramObject, name);
                let format = ATTRIBUTE_FORMAT_MAPPING.get((info.type_, info.size))
                    .ok_or(ProgramCreationError::UnsupportedAttribute(info.type_, info.size))?;

                if location != -1 {
                    active_attributes.push(AttributeInfo {
                        name,
                        format,
                        location
                    });
                }
            }
        }

        let active_uniforms_count: i32 = gl_context.get_program_parameter(gl_program, gl_context.ACTIVE_UNIFORMS).try_into().unwrap();
        let active_uniforms = Vec::with_capacity(active_uniforms_count);


    }

    fn bind_uniform_values(&mut self, uniform_values: Uniforms) -> Result<(), UniformBindingError>  {
        for uniform in self.uniforms {
            uniform.bind_value_from(uniform_values)?;
        }
    }
}

enum UniformBindingError {
    MissingUniformValue(String),
    ValueTypeMismatch(String, UniformValue, BasicUniformType)
}

struct AttributeInfo {
    name: String,
    format: AttributeFormat,
    location: u32
}

struct UniformInfo {
    name: String,
    description: UniformDescription
}

impl UniformInfo {
    fn bind_value_from(&mut self, uniform_values: Uniforms) -> Result<(), UniformBindingError> {
        let name = self.name;
        let value = uniform_values.get(name).ok_or(UniformBindingError::MissingUniformValue(name))?;

        description.bind_value(value)
    }
}

enum UniformDescription {
    Primitive(PrimitiveUniformDescription),
    PrimitiveArray(PrimitiveArrayUniformDescription),
    Struct(StructUniformDescription),
    Array(ArrayUniformDescription)
}

impl UniformDescription {
    fn bind_value(&mut self, value: UniformValue) -> Result<(), UniformBindingError> {
        match self {
            UniformDescription::Primitive(primitive_uniform_description) => {
                primitive_uniform_description.bind_value(value)
            },
            UniformDescription::Struct(struct_uniform_description) => {
                struct_uniform_description.bind_value(value)
            },
            UniformDescription::Array(array_uniform_description) => {
                array_uniform_description.bind_value(value)
            }
        }
    }
}

struct Uniform1fBindPoint {
    location: WebGLUniformLocation,
    current_value: Option<f32>
}

impl Uniform1fBindPoint {
    fn bind_value(&mut self, value: f32) -> Result<(), UniformBindingError> {
        if Some(value) != current_value {

        }
    }
}

enum UniformBinding {
    Uniform1f {
        location: WebGLUniformLocation,
        current_value: Option<f32>
    },
    Uniform1fv {
        location: WebGLUniformLocation,
        len: u32
    },
    Uniform2f {
        location: WebGLUniformLocation,
        current_value: Option<f32>
    },
    Uniform2fv {
        location: WebGLUniformLocation,
        len: u32
    },
}

enum ScalarValueBinding {
    Uniform1f(Option<f32>),
    Uniform2f(Option<(f32, f32)>),
    Uniform3f(Option<(f32, f32, f32)>),
    Uniform4f(Option<(f32, f32, f32, f32)>),
    Uniform1i(Option<i32>),
    Uniform2i(Option<(i32, i32)>),
    Uniform3i(Option<(i32, i32, i32)>),
    Uniform4i(Option<(i32, i32, i32, i32)>),
    Uniform1ui(Option<u32>),
    Uniform2ui(Option<(u32, u32)>),
    Uniform3ui(Option<(u32, u32, u32)>),
    Uniform4ui(Option<(u32, u32, u32, u32)>),
    UniformMatrix2x2(Option<[f32;4]>),
    UniformMatrix2x3(Option<[f32;6]>),
    UniformMatrix2x4(Option<[f32;8]>),
    UniformMatrix3x2(Option<[f32;6]>),
    UniformMatrix3x3(Option<[f32;9]>),
    UniformMatrix3x4(Option<[f32;12]>),
    UniformMatrix4x2(Option<[f32;8]>),
    UniformMatrix4x3(Option<[f32;12]>),
    UniformMatrix4x4(Option<[f32;16]>),
}

trait UniformValueBinder {
    fn bind_value(&mut self, context: &mut RenderingContext, value: &T) -> Result<(), UniformBindingError> where T: AsRef<UniformValue> + Display;
}

struct BooleanBinding {
    context: WebGL2RenderingContext,
    location: WebGLUniformLocation,
    current_value: Option<bool>,
}

impl BooleanBinding {
    fn new(context: WebGL2RenderingContext, location: WebGLUniformLocation) -> BooleanBinding {
        BooleanBinding {
            context,
            location,
            current_value: None
        }
    }
}

impl UniformValueBinder for BooleanBinding {
    fn bind_value(&mut self, context: &mut RenderingContext, value: &T) -> Result<(), UniformBindingError> where T: AsRef<UniformValue> + Display {
        if let UniformValue::Boolean(value) = value.as_ref() {
            if Some(value) != self.current_value {
                self.context.internal.uniform1i(Some(location), value);
                self.current_value = Some(value)
            }
        } else {
            UniformBindingError
        }
    }
}

enum PrimitiveUniformBinding {
    Boolean(BooleanBinding),
    BooleanVec2(BooleanVec2Binding),
    BooleanVec3(BooleanVec3Binding),
    BooleanVec4(BooleanVec4Binding),
    Integer(IntegerBinding),
    IntegerVec2(IntegerVec2Binding),
    IntegerVec3(IntegerVec3Binding),
    IntegerVec4(IntegerVec4Binding),
    UnsignedInteger(UnsignedIntegerBinding),
    UnsignedIntegerVec2(UnsignedIntegerVec2Binding),
    UnsignedIntegerVec3(UnsignedIntegerVec3Binding),
    UnsignedIntegerVec4(UnsignedIntegerVec4Binding),
    Float(FloatBinding),
    Vec2(Vec2Binding),
    Vec3(Vec3Binding),
    Vec4(Vec4Binding),
    Matrix2x2(Matrix2x2Binding),
    Matrix2x3(Matrix2x3Binding),
    Matrix2x4(Matrix2x4Binding),
    Matrix3x2(Matrix3x2Binding),
    Matrix3x3(Matrix3x3Binding),
    Matrix3x4(Matrix3x4Binding),
    Matrix4x2(Matrix4x2Binding),
    Matrix4x3(Matrix4x3Binding),
    Matrix4x4(Matrix4x4Binding),
    Sampler2D(SamplerBinding),
    Sampler2DArray(SamplerBinding),
    Sampler3D(SamplerBinding),
    SamplerCube(SamplerBinding),
    IntegerSampler2D(SamplerBinding),
    IntegerSampler2DArray(SamplerBinding),
    IntegerSampler3D(SamplerBinding),
    IntegerSamplerCube(SamplerBinding),
    UnsignedIntegerSampled2D(SamplerBinding),
    UnsignedIntegerSampled2DArray(SamplerBinding),
    UnsignedIntegerSampler3D(SamplerBinding),
    UnsignedIntegerSamplerCube(SamplerBinding),
    Sampler2DShadow(SamplerBinding),
    Sampler2DArrayShadow(SamplerBinding),
    SamplerCubeShadow(SamplerBinding),
}

enum PrimitiveArrayUniformBinding {
    Boolean(BooleanArrayBinding),
    BooleanVec2(BooleanVec2ArrayBinding),
    BooleanVec3(BooleanVec3ArrayBinding),
    BooleanVec4(BooleanVec4ArrayBinding),
    Integer(IntegerArrayBinding),
    IntegerVec2(IntegerVec2ArrayBinding),
    IntegerVec3(IntegerVec3ArrayBinding),
    IntegerVec4(IntegerVec4ArrayBinding),
    UnsignedInteger(UnsignedIntegerArrayBinding),
    UnsignedIntegerVec2(UnsignedIntegerVec2ArrayBinding),
    UnsignedIntegerVec3(UnsignedIntegerVec3ArrayBinding),
    UnsignedIntegerVec4(UnsignedIntegerVec4ArrayBinding),
    Float(FloatArrayBinding),
    Vec2(Vec2ArrayBinding),
    Vec3(Vec3ArrayBinding),
    Vec4(Vec4ArrayBinding),
    Matrix2x2(Matrix2x2ArrayBinding),
    Matrix2x3(Matrix2x3ArrayBinding),
    Matrix2x4(Matrix2x4ArrayBinding),
    Matrix3x2(Matrix3x2ArrayBinding),
    Matrix3x3(Matrix3x3ArrayBinding),
    Matrix3x4(Matrix3x4ArrayBinding),
    Matrix4x2(Matrix4x2ArrayBinding),
    Matrix4x3(Matrix4x3ArrayBinding),
    Matrix4x4(Matrix4x4ArrayBinding),
    Sampler2D(SamplerArrayBinding),
    Sampler2DArray(SamplerArrayBinding),
    Sampler3D(SamplerArrayBinding),
    SamplerCube(SamplerArrayBinding),
    IntegerSampler2D(SamplerArrayBinding),
    IntegerSampler2DArray(SamplerArrayBinding),
    IntegerSampler3D(SamplerArrayBinding),
    IntegerSamplerCube(SamplerArrayBinding),
    UnsignedIntegerSampled2D(SamplerArrayBinding),
    UnsignedIntegerSampled2DArray(SamplerArrayBinding),
    UnsignedIntegerSampler3D(SamplerArrayBinding),
    UnsignedIntegerSamplerCube(SamplerArrayBinding),
    Sampler2DShadow(SamplerArrayBinding),
    Sampler2DArrayShadow(SamplerArrayBinding),
    SamplerCubeShadow(SamplerArrayBinding),
}

enum UniformBinding {
    Primitive(PrimitiveUniformBinding),
    Struct(StructUniformBinding),
    PrimitiveArray(PrimitiveArrayUniformBinding),
    ComplexArray(ArrayUniformBinding),
}

enum ComplexArrayElement {
    PrimitiveArray(PrimitiveArrayUniformBinding),
    ComplexArray(ArrayUniformBinding),
    Struct(StructUniformBinding),
}

struct PrimitiveScalarValueUniform {
    location: WebGLUniformLocation,
    ty: BasicUniformType,
    current_value: ScalarValueBinding
}

struct PrimitiveUniformDescription {
    location: WebGLUniformLocation,
    binding: PrimitiveUniformBinding
}

struct PrimitiveArrayUniformBinding {
    location: WebGLUniformLocation,
    element_type: BasicUniformType,
    len: u32
}

struct ArrayUniformDescription {
    element_description: UniformDescription,
    len: u32
}

enum BasicUniformType {
    Boolean,
    BooleanVec2,
    BooleanVec3,
    BooleanVec4,
    Integer,
    IntegerVec2,
    IntegerVec3,
    IntegerVec4,
    UnsignedInteger,
    UnsignedIntegerVec2,
    UnsignedIntegerVec3,
    UnsignedIntegerVec4,
    Float,
    Vec2,
    Vec3,
    Vec4,
    Matrix2x2,
    Matrix2x3,
    Matrix2x4,
    Matrix3x2,
    Matrix3x3,
    Matrix3x4,
    Matrix4x2,
    Matrix4x3,
    Matrix4x4,
    Sampler2D,
    Sampler2DArray,
    Sampler3D,
    SamplerCube,
    IntegerSampler2D,
    IntegerSampler2DArray,
    IntegerSampler3D,
    IntegerSamplerCube,
    UnsignedIntegerSampled2D,
    UnsignedIntegerSampled2DArray,
    UnsignedIntegerSampler3D,
    UnsignedIntegerSamplerCube,
    Sampler2DShadow,
    Sampler2DArrayShadow,
    SamplerCubeShadow,
}

enum PrimitiveUniformBinding {
    Boolean(Option<bool>),
    BooleanVec2(Option<(bool, bool)>),
    BooleanVec3(Option<(bool, bool, bool)>),
    BooleanVec4(Option<(bool, bool, bool, bool)>),
    Integer(Option<i32>),
    IntegerVec2(Option<(i32, i32)>),
    IntegerVec3(Option<(i32, i32, i32)>),
    IntegerVec4(Option<(i32, i32, i32, i32)>),
    UnsignedInteger(Option<u32>),
    UnsignedIntegerVec2(Option<(u32, u32)>),
    UnsignedIntegerVec3(Option<(u32, u32, u32)>),
    UnsignedIntegerVec4(Option<(u32, u32, u32, u32)>),
    Float(Option<f32>),
    Vec2(Option<(f32, f32)>),
    Vec3(Option<(f32, f32, f32)>),
    Vec4(Option<(f32, f32, f32, f32)>),
    Matrix2x2(Option<[f32;4]>),
    Matrix2x3(Option<[f32;6]>),
    Matrix2x4(Option<[f32;8]>),
    Matrix3x2(Option<[f32;6]>),
    Matrix3x3(Option<[f32;9]>),
    Matrix3x4(Option<[f32;12]>),
    Matrix4x2(Option<[f32;8]>),
    Matrix4x3(Option<[f32;12]>),
    Matrix4x4(Option<[f32;16]>),
    Sampler2D(Option<i32>),
    Sampler2DArray(Option<i32>),
    Sampler3D(Option<i32>),
    SamplerCube(Option<i32>),
    IntegerSampler2D(Option<i32>),
    IntegerSampler2DArray(Option<i32>),
    IntegerSampler3D(Option<i32>),
    IntegerSamplerCube(Option<i32>),
    UnsignedIntegerSampled2D(Option<i32>),
    UnsignedIntegerSampled2DArray(Option<i32>),
    UnsignedIntegerSampler3D(Option<i32>),
    UnsignedIntegerSamplerCube(Option<i32>),
    Sampler2DShadow(Option<i32>),
    Sampler2DArrayShadow(Option<i32>),
    SamplerCubeShadow(Option<i32>),
}

struct StructMember {
    name: String,
    tpe: UniformType
}

struct BasicUniformDescription {
    name: String,
    format
}

struct StructUniformDescription {
    name: String,
    member_descriptions: Vec<UniformDescription>
}

enum UniformValue {
    Boolean(Bool),
    Integer(i32),
    UnsignedInteger(u32),
    Float(f32),

    Struct
}

impl UniformInfo {
    fn bind(&self, value: &UniformValue) -> Result<(), UniformBindingError> {
        match self {
            UniformInfo::Basic(basic_uniform_info) => basic_uniform_info.bind(value),
            UniformInfo::Struct(struct_uniform_info) => struct_uniform_info.bind(value),
            UniformInfo::Array(array_uniform_info) => array_uniform_info.bind(value)
        }
    }
}
