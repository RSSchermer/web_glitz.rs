use rendering_context::RenderingContext;
use util::JsId;
use std::sync::Arc;
use std::borrow::Borrow;
use task::GpuTask;
use rendering_context::Connection;
use task::Progress;
use uniform::UniformIdentifier;

use wasm_bindgen::JsCast;
use web_sys::{WebGl2RenderingContext as Gl, WebGlProgram, WebGlUniformLocation};
use std::marker;
use std::mem;
use rendering_context::ContextUpdate;

pub struct VertexShaderHandle<Rc> where Rc: RenderingContext {
    data: Arc<ShaderData<Rc>>
}

impl<Rc> VertexShaderHandle<Rc> where Rc: RenderingContext + 'static {
    pub(crate) fn new<S>(context: Rc, source: S) -> Self where S: Borrow<str> + 'static{
        let data = Arc::new(ShaderData {
            id: None,
            context: context.clone()
        });

        context.submit(ShaderAllocateTask {
            data: data.clone(),
            tpe: Gl::VERTEX_SHADER,
            source
        });

        VertexShaderHandle {
            data
        }
    }
}

pub struct FragmentShaderHandle<Rc> where Rc: RenderingContext {
    data: Arc<ShaderData<Rc>>
}

impl<Rc> FragmentShaderHandle<Rc> where Rc: RenderingContext + 'static {
    pub(crate) fn new<S>(context: Rc, source: S) -> Self where S: Borrow<str> + 'static{
        let data = Arc::new(ShaderData {
            id: None,
            context: context.clone()
        });

        context.submit(ShaderAllocateTask {
            data: data.clone(),
            tpe: Gl::VERTEX_SHADER,
            source
        });

        FragmentShaderHandle {
            data
        }
    }
}

struct ShaderData<Rc> where Rc: RenderingContext {
    id: Option<JsId>,
    context: Rc
}

impl<Rc> Drop for ShaderData<Rc> where Rc: RenderingContext {
    fn drop(&mut self) {
        if let Some(id) = self.id {
            self.context.submit(ShaderDropTask {
                id
            });
        }
    }
}

struct ShaderAllocateTask<S, Rc> where Rc: RenderingContext {
    data: Arc<ShaderData<Rc>>,
    tpe: u32,
    source: S
}

impl<S, Rc> GpuTask<Connection> for ShaderAllocateTask<S, Rc> where S: Borrow<str>, Rc: RenderingContext {
    type Output = ();

    fn progress(&mut self, connection: &mut Connection) -> Progress<Self::Output> {
        let Connection(gl, state) = connection;
        let mut data = Arc::get_mut(&mut self.data).unwrap();

        let shader_object = gl.create_shader(self.tpe).unwrap();

        gl.shader_source(&shader_object, self.source.borrow());
        gl.compile_shader(&shader_object);

        data.id = Some(JsId::from_value(shader_object.into()));

        Progress::Finished(())
    }
}

struct ShaderDropTask {
    id: JsId
}

impl GpuTask<Connection> for ShaderDropTask {
    type Output = ();

    fn progress(&mut self, connection: &mut Connection) -> Progress<Self::Output> {
        let Connection(gl, _) = connection;
        let shader_object = unsafe { JsId::into_value(self.id).unchecked_into() };

        gl.delete_shader(Some(&shader_object));

        Progress::Finished(())
    }
}

pub struct ProgramHandle<Fs, Tf, Rc> where Rc: RenderingContext {
    data: Arc<ProgramData<Fs, Tf, Rc>>
}

impl<Rc> ProgramHandle<FragmentShader, (), Rc> where Rc: RenderingContext + 'static {
    pub(crate) fn new(context: &Rc, descriptor: &ProgramDescriptor<FragmentShader, (), Rc>) -> Self {
        let data = Arc::new(ProgramData {
            context: context.clone(),
            id: None,
            vertex_shader: descriptor.vertex_shader.clone(),
            fragment_shader: descriptor.fragment_shader.clone(),
            active_uniforms: Vec::new(),
            _fragment_shader_marker: marker::PhantomData,
            _transform_feedback_marker: marker::PhantomData,
        });

        context.submit(ProgramAllocateTask {
            data: data.clone()
        });

        ProgramHandle {
            data
        }
    }
}

struct ProgramData<Fs, Tf, Rc> where Rc: RenderingContext {
    context: Rc,
    id: Option<JsId>,
    vertex_shader: Option<Arc<ShaderData<Rc>>>,
    fragment_shader: Option<Arc<ShaderData<Rc>>>,
    active_uniforms: Vec<ActiveUniform>,
    _fragment_shader_marker: marker::PhantomData<Fs>,
    _transform_feedback_marker: marker::PhantomData<Tf>,
}

impl<Fs, Tf, Rc> Drop for ProgramData<Fs, Tf, Rc> where Rc: RenderingContext {
    fn drop(&mut self) {
        if let Some(id) = self.id {
            self.context.submit(ProgramDropTask {
                id
            });
        }
    }
}

pub struct ProgramDescriptor<Fs, Tf, Rc> where Rc: RenderingContext {
    vertex_shader: Option<Arc<ShaderData<Rc>>>,
    fragment_shader: Option<Arc<ShaderData<Rc>>>,
    transform_feedback_varyings: Option<TransformFeedbackVaryings>,
    _fragment_shader_marker: marker::PhantomData<Fs>,
    _transform_feedback_marker: marker::PhantomData<Tf>,
}

impl<Fs, Tf, Rc> ProgramDescriptor<Fs, Tf, Rc> where Rc: RenderingContext {
    pub fn begin() -> ProgramDescriptorBuilder<(), (), (), Rc> {
        ProgramDescriptorBuilder {
            vertex_shader: None,
            fragment_shader: None,
            transform_feedback_varyings: None,
            _vertex_shader_marker: marker::PhantomData,
            _fragment_shader_marker: marker::PhantomData,
            _transform_feedback_marker: marker::PhantomData
        }
    }
}

pub struct ProgramDescriptorBuilder<Vs, Fs, Tf, Rc> where Rc: RenderingContext {
    vertex_shader: Option<Arc<ShaderData<Rc>>>,
    fragment_shader: Option<Arc<ShaderData<Rc>>>,
    transform_feedback_varyings: Option<TransformFeedbackVaryings>,
    _vertex_shader_marker: marker::PhantomData<Vs>,
    _fragment_shader_marker: marker::PhantomData<Fs>,
    _transform_feedback_marker: marker::PhantomData<Tf>,
}

impl<Vs, Fs, Tf, Rc> ProgramDescriptorBuilder<Vs, Fs, Tf, Rc> where Rc: RenderingContext {
    pub fn vertex_shader(self, vertex_shader: &VertexShaderHandle<Rc>) -> ProgramDescriptorBuilder<VertexShader, Fs, Tf, Rc> {
        ProgramDescriptorBuilder {
            vertex_shader: Some(vertex_shader.data.clone()),
            fragment_shader: self.fragment_shader,
            transform_feedback_varyings:  self.transform_feedback_varyings,
            _vertex_shader_marker: marker::PhantomData,
            _fragment_shader_marker: marker::PhantomData,
            _transform_feedback_marker: marker::PhantomData,
        }
    }

    pub fn fragment_shader(self, fragment_shader: &FragmentShaderHandle<Rc>) -> ProgramDescriptorBuilder<Vs, FragmentShader, Tf, Rc> {
        ProgramDescriptorBuilder {
            vertex_shader: self.vertex_shader,
            fragment_shader: Some(fragment_shader.data.clone()),
            transform_feedback_varyings:  self.transform_feedback_varyings,
            _vertex_shader_marker: marker::PhantomData,
            _fragment_shader_marker: marker::PhantomData,
            _transform_feedback_marker: marker::PhantomData,
        }
    }

    pub fn transform_feedback<V>(self, varyings: V) -> ProgramDescriptorBuilder<Vs, FragmentShader, TransformFeedback, Rc> where V: Into<TransformFeedbackVaryings> {
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

impl<Rc> ProgramDescriptorBuilder<VertexShader, FragmentShader, (), Rc> where Rc: RenderingContext {
    pub fn finish(self) -> ProgramDescriptor<FragmentShader, (), Rc> {
        ProgramDescriptor {
            vertex_shader: self.vertex_shader,
            fragment_shader: self.fragment_shader,
            transform_feedback_varyings: self.transform_feedback_varyings,
            _fragment_shader_marker: marker::PhantomData,
            _transform_feedback_marker: marker::PhantomData,
        }
    }
}

impl<Rc> ProgramDescriptorBuilder<VertexShader, (), TransformFeedback, Rc> where Rc: RenderingContext {
    pub fn finish(self) -> ProgramDescriptor<(), TransformFeedback, Rc> {
        ProgramDescriptor {
            vertex_shader: self.vertex_shader,
            fragment_shader: self.fragment_shader,
            transform_feedback_varyings: self.transform_feedback_varyings,
            _fragment_shader_marker: marker::PhantomData,
            _transform_feedback_marker: marker::PhantomData,
        }
    }
}

impl<Rc> ProgramDescriptorBuilder<VertexShader, FragmentShader, TransformFeedback, Rc> where Rc: RenderingContext {
    pub fn finish(self) -> ProgramDescriptor<FragmentShader, TransformFeedback, Rc> {
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
    names: Vec<String>
}

impl TransformFeedbackVaryings {
    pub fn new<I, S>(names: I) -> Self where I: IntoIterator<Item=S>, S: Borrow<str> {
        let names = names.into_iter().map(|n| n.borrow().to_string()).collect();

        TransformFeedbackVaryings {
            names
        }
    }
}

impl<I, S> From<I> for TransformFeedbackVaryings where I: IntoIterator<Item=S>, S: Borrow<str> {
    fn from(names: I) -> TransformFeedbackVaryings {
        TransformFeedbackVaryings::new(names)
    }
}

pub struct VertexShader;
pub struct FragmentShader;
pub struct TransformFeedback;

struct ActiveUniform {
    identifier: UniformIdentifier,
    location: JsId
}

struct ProgramAllocateTask<Fs, Tf, Rc> where Rc: RenderingContext {
    data: Arc<ProgramData<Fs, Tf, Rc>>
}

impl<Rc> GpuTask<Connection> for ProgramAllocateTask<FragmentShader, (), Rc> where Rc: RenderingContext {
    type Output = ();

    fn progress(&mut self, connection: &mut Connection) -> Progress<Self::Output> {
        let Connection(gl, state) = connection;
        let data = Arc::get_mut(&mut self.data).unwrap();

        let program_object = gl.create_program().unwrap();

        state.set_active_program(Some(&program_object)).apply(gl).unwrap();

        unsafe {
            if let Some(ref shader_data) = data.vertex_shader {
                shader_data.id.unwrap().with_value_unchecked(|shader_object| {
                    gl.attach_shader(&program_object, &shader_object);
                });
            }

            if let Some(ref shader_data) = data.fragment_shader {
                shader_data.id.unwrap().with_value_unchecked(|shader_object| {
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

struct ProgramDropTask {
    id: JsId
}

impl GpuTask<Connection> for ProgramDropTask {
    type Output = ();

    fn progress(&mut self, connection: &mut Connection) -> Progress<Self::Output> {
        let Connection(gl, _) = connection;

        unsafe {
            let program_object = JsId::into_value(self.id).unchecked_into();

            gl.delete_program(Some(&program_object));
        }

        Progress::Finished(())
    }
}

fn active_uniforms(gl: &Gl, program: &WebGlProgram) -> Vec<ActiveUniform> {
    let active_uniform_count = gl.get_program_parameter(program, Gl::ACTIVE_UNIFORMS).as_f64().unwrap() as u32;
    let mut result = Vec::with_capacity(active_uniform_count as usize);

    for i in 0..active_uniform_count {
        let info = gl.get_active_uniform(program, i).unwrap();
        let name = info.name();
        let identifier = UniformIdentifier::from_string(&name);
        let location = JsId::from_value(gl.get_uniform_location(&program, &name).unwrap().into());

        result.push(ActiveUniform {
            identifier,
            location
        });
    }

    result
}
