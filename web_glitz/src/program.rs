use rendering_context::Connection;
use rendering_context::RenderingContext;
use std::borrow::Borrow;
use std::sync::Arc;
use task::GpuTask;
use task::Progress;
use uniform::UniformValueIdentifier;
use util::JsId;

use buffer::BufferData;
use rendering_context::ContextUpdate;
use rendering_context::DropObject;
use rendering_context::Dropper;
use rendering_context::RefCountedDropper;
use std::marker;
use std::mem;
use util::arc_get_mut_unchecked;
use wasm_bindgen::JsCast;
use web_sys::{WebGl2RenderingContext as Gl, WebGlProgram, WebGlUniformLocation, WebGlActiveInfo};

pub struct VertexShaderHandle {
    data: Arc<ShaderData>,
}

impl VertexShaderHandle {
    pub(crate) fn new<S, Rc>(context: &Rc, dropper: RefCountedDropper, source: S) -> Self
    where
        S: Borrow<str> + 'static,
        Rc: RenderingContext,
    {
        let data = Arc::new(ShaderData { id: None, dropper });

        context.submit(ShaderAllocateTask {
            data: data.clone(),
            tpe: Gl::VERTEX_SHADER,
            source,
        });

        VertexShaderHandle { data }
    }
}

pub struct FragmentShaderHandle {
    data: Arc<ShaderData>,
}

impl FragmentShaderHandle {
    pub(crate) fn new<S, Rc>(context: &Rc, dropper: RefCountedDropper, source: S) -> Self
    where
        S: Borrow<str> + 'static,
        Rc: RenderingContext,
    {
        let data = Arc::new(ShaderData { id: None, dropper });

        context.submit(ShaderAllocateTask {
            data: data.clone(),
            tpe: Gl::VERTEX_SHADER,
            source,
        });

        FragmentShaderHandle { data }
    }
}

struct ShaderData {
    id: Option<JsId>,
    dropper: RefCountedDropper,
}

impl Drop for ShaderData {
    fn drop(&mut self) {
        if let Some(id) = self.id {
            self.dropper.drop_gl_object(DropObject::Shader(id));
        }
    }
}

struct ShaderAllocateTask<S> {
    data: Arc<ShaderData>,
    tpe: u32,
    source: S,
}

impl<S> GpuTask<Connection> for ShaderAllocateTask<S>
where
    S: Borrow<str>,
{
    type Output = ();

    fn progress(&mut self, connection: &mut Connection) -> Progress<Self::Output> {
        let Connection(gl, state) = connection;
        let mut data = unsafe { arc_get_mut_unchecked(&mut self.data) };

        let shader_object = gl.create_shader(self.tpe).unwrap();

        gl.shader_source(&shader_object, self.source.borrow());
        gl.compile_shader(&shader_object);

        data.id = Some(JsId::from_value(shader_object.into()));

        Progress::Finished(())
    }
}

pub struct ProgramHandle<Fs, Tf> {
    data: Arc<ProgramData<Fs, Tf>>,
}

impl ProgramHandle<FragmentShader, ()> {
    pub(crate) fn new<Rc>(
        context: &Rc,
        dropper: RefCountedDropper,
        descriptor: &ProgramDescriptor<FragmentShader, ()>,
    ) -> Self
    where
        Rc: RenderingContext,
    {
        let data = Arc::new(ProgramData {
            id: None,
            dropper,
            vertex_shader: descriptor.vertex_shader.clone(),
            fragment_shader: descriptor.fragment_shader.clone(),
            active_uniforms: Vec::new(),
            _fragment_shader_marker: marker::PhantomData,
            _transform_feedback_marker: marker::PhantomData,
        });

        context.submit(ProgramAllocateTask { data: data.clone() });

        ProgramHandle { data }
    }
}

struct ProgramData<Fs, Tf> {
    id: Option<JsId>,
    dropper: RefCountedDropper,
    vertex_shader: Option<Arc<ShaderData>>,
    fragment_shader: Option<Arc<ShaderData>>,
    active_uniforms: Vec<(UniformValueIdentifier, UniformInfo)>,
    _fragment_shader_marker: marker::PhantomData<Fs>,
    _transform_feedback_marker: marker::PhantomData<Tf>,
}

impl<Fs, Tf> Drop for ProgramData<Fs, Tf> {
    fn drop(&mut self) {
        if let Some(id) = self.id {
            self.dropper.drop_gl_object(DropObject::Program(id));
        }
    }
}

pub struct ProgramDescriptor<Fs, Tf> {
    vertex_shader: Option<Arc<ShaderData>>,
    fragment_shader: Option<Arc<ShaderData>>,
    transform_feedback_varyings: Option<TransformFeedbackVaryings>,
    _fragment_shader_marker: marker::PhantomData<Fs>,
    _transform_feedback_marker: marker::PhantomData<Tf>,
}

impl<Fs, Tf> ProgramDescriptor<Fs, Tf> {
    pub fn begin() -> ProgramDescriptorBuilder<(), (), ()> {
        ProgramDescriptorBuilder {
            vertex_shader: None,
            fragment_shader: None,
            transform_feedback_varyings: None,
            _vertex_shader_marker: marker::PhantomData,
            _fragment_shader_marker: marker::PhantomData,
            _transform_feedback_marker: marker::PhantomData,
        }
    }
}

pub struct ProgramDescriptorBuilder<Vs, Fs, Tf> {
    vertex_shader: Option<Arc<ShaderData>>,
    fragment_shader: Option<Arc<ShaderData>>,
    transform_feedback_varyings: Option<TransformFeedbackVaryings>,
    _vertex_shader_marker: marker::PhantomData<Vs>,
    _fragment_shader_marker: marker::PhantomData<Fs>,
    _transform_feedback_marker: marker::PhantomData<Tf>,
}

impl<Vs, Fs, Tf> ProgramDescriptorBuilder<Vs, Fs, Tf> {
    pub fn vertex_shader(
        self,
        vertex_shader: &VertexShaderHandle,
    ) -> ProgramDescriptorBuilder<VertexShader, Fs, Tf> {
        ProgramDescriptorBuilder {
            vertex_shader: Some(vertex_shader.data.clone()),
            fragment_shader: self.fragment_shader,
            transform_feedback_varyings: self.transform_feedback_varyings,
            _vertex_shader_marker: marker::PhantomData,
            _fragment_shader_marker: marker::PhantomData,
            _transform_feedback_marker: marker::PhantomData,
        }
    }

    pub fn fragment_shader(
        self,
        fragment_shader: &FragmentShaderHandle,
    ) -> ProgramDescriptorBuilder<Vs, FragmentShader, Tf> {
        ProgramDescriptorBuilder {
            vertex_shader: self.vertex_shader,
            fragment_shader: Some(fragment_shader.data.clone()),
            transform_feedback_varyings: self.transform_feedback_varyings,
            _vertex_shader_marker: marker::PhantomData,
            _fragment_shader_marker: marker::PhantomData,
            _transform_feedback_marker: marker::PhantomData,
        }
    }

    pub fn transform_feedback<V>(
        self,
        varyings: V,
    ) -> ProgramDescriptorBuilder<Vs, FragmentShader, TransformFeedback>
    where
        V: Into<TransformFeedbackVaryings>,
    {
        ProgramDescriptorBuilder {
            vertex_shader: self.vertex_shader,
            fragment_shader: self.fragment_shader,
            transform_feedback_varyings: Some(varyings.into()),
            _vertex_shader_marker: marker::PhantomData,
            _fragment_shader_marker: marker::PhantomData,
            _transform_feedback_marker: marker::PhantomData,
        }
    }
}

impl ProgramDescriptorBuilder<VertexShader, FragmentShader, ()> {
    pub fn finish(self) -> ProgramDescriptor<FragmentShader, ()> {
        ProgramDescriptor {
            vertex_shader: self.vertex_shader,
            fragment_shader: self.fragment_shader,
            transform_feedback_varyings: self.transform_feedback_varyings,
            _fragment_shader_marker: marker::PhantomData,
            _transform_feedback_marker: marker::PhantomData,
        }
    }
}

impl ProgramDescriptorBuilder<VertexShader, (), TransformFeedback> {
    pub fn finish(self) -> ProgramDescriptor<(), TransformFeedback> {
        ProgramDescriptor {
            vertex_shader: self.vertex_shader,
            fragment_shader: self.fragment_shader,
            transform_feedback_varyings: self.transform_feedback_varyings,
            _fragment_shader_marker: marker::PhantomData,
            _transform_feedback_marker: marker::PhantomData,
        }
    }
}

impl ProgramDescriptorBuilder<VertexShader, FragmentShader, TransformFeedback> {
    pub fn finish(self) -> ProgramDescriptor<FragmentShader, TransformFeedback> {
        ProgramDescriptor {
            vertex_shader: self.vertex_shader,
            fragment_shader: self.fragment_shader,
            transform_feedback_varyings: self.transform_feedback_varyings,
            _fragment_shader_marker: marker::PhantomData,
            _transform_feedback_marker: marker::PhantomData,
        }
    }
}

pub struct TransformFeedbackVaryings {
    names: Vec<String>,
}

impl TransformFeedbackVaryings {
    pub fn new<I, S>(names: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Borrow<str>,
    {
        let names = names.into_iter().map(|n| n.borrow().to_string()).collect();

        TransformFeedbackVaryings { names }
    }
}

impl<I, S> From<I> for TransformFeedbackVaryings
where
    I: IntoIterator<Item = S>,
    S: Borrow<str>,
{
    fn from(names: I) -> TransformFeedbackVaryings {
        TransformFeedbackVaryings::new(names)
    }
}

pub struct VertexShader;
pub struct FragmentShader;
pub struct TransformFeedback;

pub(crate) struct UniformInfo {
    pub(crate) location: JsId,
    pub(crate) value_type: UniformType,
    pub(crate) size: usize,
    pub(crate) current_value: Option<UniformValue>,
}

#[derive(PartialEq)]
pub enum UniformValue {
    Float(f32),
    FloatVector2((f32, f32)),
    FloatVector3((f32, f32, f32)),
    FloatVector4((f32, f32, f32, f32)),
    FloatMatrix2x2([f32; 4]),
    FloatMatrix2x3([f32; 6]),
    FloatMatrix2x4([f32; 8]),
    FloatMatrix3x2([f32; 6]),
    FloatMatrix3x3([f32; 9]),
    FloatMatrix3x4([f32; 12]),
    FloatMatrix4x2([f32; 8]),
    FloatMatrix4x3([f32; 12]),
    FloatMatrix4x4([f32; 16]),
    Integer(i32),
    IntegerVector2((i32, i32)),
    IntegerVector3((i32, i32, i32)),
    IntegerVector4((i32, i32, i32, i32)),
    UnsignedInteger(u32),
    UnsignedIntegerVector2((u32, u32)),
    UnsignedIntegerVector3((u32, u32, u32)),
    UnsignedIntegerVector4((u32, u32, u32, u32)),
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum UniformType {
    Float,
    FloatVector2,
    FloatVector3,
    FloatVector4,
    FloatMatrix2,
    FloatMatrix3,
    FloatMatrix4,
    FloatMatrix2x3,
    FloatMatrix2x4,
    FloatMatrix3x2,
    FloatMatrix3x4,
    FloatMatrix4x2,
    FloatMatrix4x3,
    Integer,
    IntegerVector2,
    IntegerVector3,
    IntegerVector4,
    UnsignedInteger,
    UnsignedIntegerVector2,
    UnsignedIntegerVector3,
    UnsignedIntegerVector4,
    Boolean,
    BooleanVector2,
    BooleanVector3,
    BooleanVector4,
    Sampler2D,
    SamplerCUBE,
    Sampler3D,
    Sampler2DShadow,
    Sampler2DArray,
    Sampler2DArrayShadow,
    SamplerCubeShadow,
    IntegerSampler2D,
    IntegerSampler3D,
    IntegerSamplerCube,
    IntegerSampler2DArray,
    UnsignedIntegerSampler2D,
    UnsignedIntegerSampler3D,
    UnsignedIntegerSamplerCube,
    UnsignedIntegerSampler2DArray,
    ArrayOfFloat(usize),
    ArrayOfFloatVector2(usize),
    ArrayOfFloatVector3(usize),
    ArrayOfFloatVector4(usize),
    ArrayOfFloatMatrix2(usize),
    ArrayOfFloatMatrix3(usize),
    ArrayOfFloatMatrix4(usize),
    ArrayOfFloatMatrix2x3(usize),
    ArrayOfFloatMatrix2x4(usize),
    ArrayOfFloatMatrix3x2(usize),
    ArrayOfFloatMatrix3x4(usize),
    ArrayOfFloatMatrix4x2(usize),
    ArrayOfFloatMatrix4x3(usize),
    ArrayOfInteger(usize),
    ArrayOfIntegerVector2(usize),
    ArrayOfIntegerVector3(usize),
    ArrayOfIntegerVector4(usize),
    ArrayOfUnsignedInteger(usize),
    ArrayOfUnsignedIntegerVector2(usize),
    ArrayOfUnsignedIntegerVector3(usize),
    ArrayOfUnsignedIntegerVector4(usize),
    ArrayOfBoolean(usize),
    ArrayOfBooleanVector2(usize),
    ArrayOfBooleanVector3(usize),
    ArrayOfBooleanVector4(usize),
    ArrayOfSampler2D(usize),
    ArrayOfSamplerCUBE(usize),
    ArrayOfSampler3D(usize),
    ArrayOfSampler2DShadow(usize),
    ArrayOfSampler2DArray(usize),
    ArrayOfSampler2DArrayShadow(usize),
    ArrayOfSamplerCubeShadow(usize),
    ArrayOfIntegerSampler2D(usize),
    ArrayOfIntegerSampler3D(usize),
    ArrayOfIntegerSamplerCube(usize),
    ArrayOfIntegerSampler2DArray(usize),
    ArrayOfUnsignedIntegerSampler2D(usize),
    ArrayOfUnsignedIntegerSampler3D(usize),
    ArrayOfUnsignedIntegerSamplerCube(usize),
    ArrayOfUnsignedIntegerSampler2DArray(usize),
}

impl UniformType {
    fn from_info(info: &WebGlActiveInfo) -> Self {
        let size = info.size() as usize;
        let is_array = info.name().ends_with("[0]");

        if is_array {
            match info.type_() {
                Gl::FLOAT => UniformType::ArrayOfFloat(size),
                Gl::FLOAT_VEC2 => UniformType::ArrayOfFloatVector2(size),
                Gl::FLOAT_VEC3 => UniformType::ArrayOfFloatVector3(size),
                Gl::FLOAT_VEC4 => UniformType::ArrayOfFloatVector4(size),
                Gl::FLOAT_MAT2 => UniformType::ArrayOfFloatMatrix2(size),
                Gl::FLOAT_MAT3 => UniformType::ArrayOfFloatMatrix3(size),
                Gl::FLOAT_MAT4 => UniformType::ArrayOfFloatMatrix4(size),
                Gl::FLOAT_MAT2X3 => UniformType::ArrayOfFloatMatrix2x3(size),
                Gl::FLOAT_MAT2X4 => UniformType::ArrayOfFloatMatrix2x4(size),
                Gl::FLOAT_MAT3X2 => UniformType::ArrayOfFloatMatrix3x2(size),
                Gl::FLOAT_MAT3X4 => UniformType::ArrayOfFloatMatrix3x4(size),
                Gl::FLOAT_MAT4X2 => UniformType::ArrayOfFloatMatrix4x2(size),
                Gl::FLOAT_MAT4X3 => UniformType::ArrayOfFloatMatrix4x3(size),
                Gl::INT => UniformType::ArrayOfInteger(size),
                Gl::INT_VEC2 => UniformType::ArrayOfIntegerVector2(size),
                Gl::INT_VEC3 => UniformType::ArrayOfIntegerVector3(size),
                Gl::INT_VEC4 => UniformType::ArrayOfIntegerVector4(size),
                Gl::BOOL => UniformType::ArrayOfBoolean(size),
                Gl::BOOL_VEC2 => UniformType::ArrayOfBooleanVector2(size),
                Gl::BOOL_VEC3 => UniformType::ArrayOfBooleanVector3(size),
                Gl::BOOL_VEC4 => UniformType::ArrayOfBooleanVector4(size),
                Gl::UNSIGNED_INT => UniformType::ArrayOfUnsignedInteger(size),
                Gl::UNSIGNED_INT_VEC2 => UniformType::ArrayOfUnsignedIntegerVector2(size),
                Gl::UNSIGNED_INT_VEC3 => UniformType::ArrayOfUnsignedIntegerVector3(size),
                Gl::UNSIGNED_INT_VEC4 => UniformType::ArrayOfUnsignedIntegerVector4(size),
                Gl::SAMPLER_2D => UniformType::ArrayOfSampler2D(size),
                Gl::SAMPLER_CUBE => UniformType::ArrayOfSamplerCUBE(size),
                Gl::SAMPLER_3D => UniformType::ArrayOfSampler3D(size),
                Gl::SAMPLER_2D_SHADOW => UniformType::ArrayOfSampler2DShadow(size),
                Gl::SAMPLER_2D_ARRAY => UniformType::ArrayOfSampler2DArray(size),
                Gl::SAMPLER_2D_ARRAY_SHADOW => UniformType::ArrayOfSampler2DArrayShadow(size),
                Gl::SAMPLER_CUBE_SHADOW => UniformType::ArrayOfSamplerCubeShadow(size),
                Gl::INT_SAMPLER_2D => UniformType::ArrayOfIntegerSampler2D(size),
                Gl::INT_SAMPLER_3D => UniformType::ArrayOfIntegerSampler3D(size),
                Gl::INT_SAMPLER_CUBE => UniformType::ArrayOfIntegerSamplerCube(size),
                Gl::INT_SAMPLER_2D_ARRAY => UniformType::ArrayOfIntegerSampler2DArray(size),
                Gl::UNSIGNED_INT_SAMPLER_2D => UniformType::ArrayOfUnsignedIntegerSampler2D(size),
                Gl::UNSIGNED_INT_SAMPLER_3D => UniformType::ArrayOfUnsignedIntegerSampler3D(size),
                Gl::UNSIGNED_INT_SAMPLER_CUBE => UniformType::ArrayOfUnsignedIntegerSamplerCube(size),
                Gl::UNSIGNED_INT_SAMPLER_2D_ARRAY => UniformType::ArrayOfUnsignedIntegerSampler2DArray(size),
                _ => panic!("Invalid uniform type ID")
            }
        } else {
            match info.type_() {
                Gl::FLOAT => UniformType::Float,
                Gl::FLOAT_VEC2 => UniformType::FloatVector2,
                Gl::FLOAT_VEC3 => UniformType::FloatVector3,
                Gl::FLOAT_VEC4 => UniformType::FloatVector4,
                Gl::FLOAT_MAT2 => UniformType::FloatMatrix2,
                Gl::FLOAT_MAT3 => UniformType::FloatMatrix3,
                Gl::FLOAT_MAT4 => UniformType::FloatMatrix4,
                Gl::FLOAT_MAT2X3 => UniformType::FloatMatrix2x3,
                Gl::FLOAT_MAT2X4 => UniformType::FloatMatrix2x4,
                Gl::FLOAT_MAT3X2 => UniformType::FloatMatrix3x2,
                Gl::FLOAT_MAT3X4 => UniformType::FloatMatrix3x4,
                Gl::FLOAT_MAT4X2 => UniformType::FloatMatrix4x2,
                Gl::FLOAT_MAT4X3 => UniformType::FloatMatrix4x3,
                Gl::INT => UniformType::Integer,
                Gl::INT_VEC2 => UniformType::IntegerVector2,
                Gl::INT_VEC3 => UniformType::IntegerVector3,
                Gl::INT_VEC4 => UniformType::IntegerVector4,
                Gl::BOOL => UniformType::Boolean,
                Gl::BOOL_VEC2 => UniformType::BooleanVector2,
                Gl::BOOL_VEC3 => UniformType::BooleanVector3,
                Gl::BOOL_VEC4 => UniformType::BooleanVector4,
                Gl::UNSIGNED_INT => UniformType::UnsignedInteger,
                Gl::UNSIGNED_INT_VEC2 => UniformType::UnsignedIntegerVector2,
                Gl::UNSIGNED_INT_VEC3 => UniformType::UnsignedIntegerVector3,
                Gl::UNSIGNED_INT_VEC4 => UniformType::UnsignedIntegerVector4,
                Gl::SAMPLER_2D => UniformType::Sampler2D,
                Gl::SAMPLER_CUBE => UniformType::SamplerCUBE,
                Gl::SAMPLER_3D => UniformType::Sampler3D,
                Gl::SAMPLER_2D_SHADOW => UniformType::Sampler2DShadow,
                Gl::SAMPLER_2D_ARRAY => UniformType::Sampler2DArray,
                Gl::SAMPLER_2D_ARRAY_SHADOW => UniformType::Sampler2DArrayShadow,
                Gl::SAMPLER_CUBE_SHADOW => UniformType::SamplerCubeShadow,
                Gl::INT_SAMPLER_2D => UniformType::IntegerSampler2D,
                Gl::INT_SAMPLER_3D => UniformType::IntegerSampler3D,
                Gl::INT_SAMPLER_CUBE => UniformType::IntegerSamplerCube,
                Gl::INT_SAMPLER_2D_ARRAY => UniformType::IntegerSampler2DArray,
                Gl::UNSIGNED_INT_SAMPLER_2D => UniformType::UnsignedIntegerSampler2D,
                Gl::UNSIGNED_INT_SAMPLER_3D => UniformType::UnsignedIntegerSampler3D,
                Gl::UNSIGNED_INT_SAMPLER_CUBE => UniformType::UnsignedIntegerSamplerCube,
                Gl::UNSIGNED_INT_SAMPLER_2D_ARRAY => UniformType::UnsignedIntegerSampler2DArray,
                _ => panic!("Invalid uniform type ID")
            }
        }
    }
}

struct ProgramAllocateTask<Fs, Tf> {
    data: Arc<ProgramData<Fs, Tf>>,
}

impl GpuTask<Connection> for ProgramAllocateTask<FragmentShader, ()> {
    type Output = ();

    fn progress(&mut self, connection: &mut Connection) -> Progress<Self::Output> {
        let Connection(gl, state) = connection;
        let data = unsafe { arc_get_mut_unchecked(&mut self.data) };

        let program_object = gl.create_program().unwrap();

        state
            .set_active_program(Some(&program_object))
            .apply(gl)
            .unwrap();

        unsafe {
            if let Some(ref shader_data) = data.vertex_shader {
                shader_data
                    .id
                    .unwrap()
                    .with_value_unchecked(|shader_object| {
                        gl.attach_shader(&program_object, &shader_object);
                    });
            }

            if let Some(ref shader_data) = data.fragment_shader {
                shader_data
                    .id
                    .unwrap()
                    .with_value_unchecked(|shader_object| {
                        gl.attach_shader(&program_object, &shader_object);
                    });
            }
        }

        gl.link_program(&program_object);

        data.active_uniforms = active_uniforms(&gl, &program_object);
        data.id = Some(JsId::from_value(program_object.into()));

        Progress::Finished(())
    }
}

// TODO: implement GpuTask for the other 2 combinations once web_sys supports transform_feedback_varyings

fn active_uniforms(gl: &Gl, program: &WebGlProgram) -> Vec<(UniformValueIdentifier, UniformInfo)> {
    let active_uniform_count = gl
        .get_program_parameter(program, Gl::ACTIVE_UNIFORMS)
        .as_f64()
        .unwrap() as u32;
    let mut result = Vec::with_capacity(active_uniform_count as usize);

    for i in 0..active_uniform_count {
        let info = gl.get_active_uniform(program, i).unwrap();
        let name = info.name();
        let value_type = UniformType::from_info(&info);
        let identifier = UniformValueIdentifier::from_string(&name);
        let location = JsId::from_value(gl.get_uniform_location(&program, &name).unwrap().into());

        result.push((identifier, UniformInfo {
            location,
            value_type,
            size: info.size() as usize,
            current_value: None,
        }));
    }

    result
}
