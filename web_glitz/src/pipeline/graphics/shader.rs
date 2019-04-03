use crate::runtime::{Connection, RenderingContext};
use crate::task::{ContextId, GpuTask, Progress};
use crate::util::{arc_get_mut_unchecked, JsId};
use std::borrow::Borrow;
use std::sync::Arc;

use wasm_bindgen::JsCast;
use web_sys::WebGl2RenderingContext as Gl;

#[derive(PartialEq, Debug)]
pub struct ShaderCompilationError(String);

pub struct VertexShader {
    data: Arc<VertexShaderData>,
}

impl VertexShader {
    pub(crate) fn data(&self) -> &Arc<VertexShaderData> {
        &self.data
    }
}

pub struct FragmentShader {
    data: Arc<FragmentShaderData>,
}

impl FragmentShader {
    pub(crate) fn data(&self) -> &Arc<FragmentShaderData> {
        &self.data
    }
}

pub(crate) struct VertexShaderData {
    id: Option<JsId>,
    context_id: usize,
    dropper: Box<VertexShaderObjectDropper>,
}

impl VertexShaderData {
    pub(crate) fn id(&self) -> Option<JsId> {
        self.id
    }

    pub(crate) fn context_id(&self) -> usize {
        self.context_id
    }
}

pub(crate) struct FragmentShaderData {
    id: Option<JsId>,
    context_id: usize,
    dropper: Box<FragmentShaderObjectDropper>,
}

impl FragmentShaderData {
    pub(crate) fn id(&self) -> Option<JsId> {
        self.id
    }

    pub(crate) fn context_id(&self) -> usize {
        self.context_id
    }
}

trait VertexShaderObjectDropper {
    fn drop_shader_object(&self, id: JsId);
}

impl<T> VertexShaderObjectDropper for T
where
    T: RenderingContext,
{
    fn drop_shader_object(&self, id: JsId) {
        self.submit(VertexShaderDropCommand { id });
    }
}

impl Drop for VertexShaderData {
    fn drop(&mut self) {
        if let Some(id) = self.id {
            self.dropper.drop_shader_object(id);
        }
    }
}

trait FragmentShaderObjectDropper {
    fn drop_shader_object(&self, id: JsId);
}

impl<T> FragmentShaderObjectDropper for T
where
    T: RenderingContext,
{
    fn drop_shader_object(&self, id: JsId) {
        self.submit(FragmentShaderDropCommand { id });
    }
}

impl Drop for FragmentShaderData {
    fn drop(&mut self) {
        if let Some(id) = self.id {
            self.dropper.drop_shader_object(id);
        }
    }
}

pub(crate) struct VertexShaderAllocateCommand<S> {
    data: Arc<VertexShaderData>,
    source: S,
}

impl<S> VertexShaderAllocateCommand<S> where S: Borrow<str> + 'static {
    pub(crate) fn new<Rc>(context: &Rc, source: S) -> Self
        where
            Rc: RenderingContext + Clone + 'static
    {
        let data = Arc::new(VertexShaderData {
            id: None,
            context_id: context.id(),
            dropper: Box::new(context.clone()),
        });

        VertexShaderAllocateCommand {
            data,
            source,
        }
    }
}

unsafe impl<S> GpuTask<Connection> for VertexShaderAllocateCommand<S>
    where
        S: Borrow<str>,
{
    type Output = Result<VertexShader, ShaderCompilationError>;

    fn context_id(&self) -> ContextId {
        ContextId::Any
    }

    fn progress(&mut self, connection: &mut Connection) -> Progress<Self::Output> {
        let (gl, _) = unsafe { connection.unpack_mut() };
        let data = unsafe { arc_get_mut_unchecked(&mut self.data) };

        let shader_object = gl.create_shader(Gl::VERTEX_SHADER).unwrap();

        gl.shader_source(&shader_object, self.source.borrow());
        gl.compile_shader(&shader_object);

        if !gl.get_shader_parameter(&shader_object, Gl::COMPILE_STATUS).as_bool().unwrap() {
            let error = gl.get_shader_info_log(&shader_object).unwrap();

            Progress::Finished(Err(ShaderCompilationError(error)))
        } else {
            data.id = Some(JsId::from_value(shader_object.into()));

            Progress::Finished(Ok(VertexShader {
                data: self.data.clone()
            }))
        }
    }
}

pub(crate) struct FragmentShaderAllocateCommand<S> {
    data: Arc<FragmentShaderData>,
    source: S,
}

impl<S> FragmentShaderAllocateCommand<S> where S: Borrow<str> + 'static {
    pub(crate) fn new<Rc>(context: &Rc, source: S) -> Self
        where
            Rc: RenderingContext + Clone + 'static
    {
        let data = Arc::new(FragmentShaderData {
            id: None,
            context_id: context.id(),
            dropper: Box::new(context.clone()),
        });

        FragmentShaderAllocateCommand {
            data,
            source,
        }
    }
}

unsafe impl<S> GpuTask<Connection> for FragmentShaderAllocateCommand<S>
where
    S: Borrow<str>,
{
    type Output = Result<FragmentShader, ShaderCompilationError>;

    fn context_id(&self) -> ContextId {
        ContextId::Any
    }

    fn progress(&mut self, connection: &mut Connection) -> Progress<Self::Output> {
        let (gl, _) = unsafe { connection.unpack_mut() };
        let data = unsafe { arc_get_mut_unchecked(&mut self.data) };

        let shader_object = gl.create_shader(Gl::FRAGMENT_SHADER).unwrap();

        gl.shader_source(&shader_object, self.source.borrow());
        gl.compile_shader(&shader_object);

        if !gl.get_shader_parameter(&shader_object, Gl::COMPILE_STATUS).as_bool().unwrap() {
            let error = gl.get_shader_info_log(&shader_object).unwrap();

            Progress::Finished(Err(ShaderCompilationError(error)))
        } else {
            data.id = Some(JsId::from_value(shader_object.into()));

            Progress::Finished(Ok(FragmentShader {
                data: self.data.clone()
            }))
        }
    }
}

struct VertexShaderDropCommand {
    id: JsId,
}

unsafe impl GpuTask<Connection> for VertexShaderDropCommand {
    type Output = ();

    fn context_id(&self) -> ContextId {
        ContextId::Any
    }

    fn progress(&mut self, connection: &mut Connection) -> Progress<Self::Output> {
        let (gl, state) = unsafe { connection.unpack_mut() };
        let value = unsafe { JsId::into_value(self.id) };

        state
            .program_cache_mut()
            .remove_vertex_shader_dependent(self.id);
        gl.delete_shader(Some(&value.unchecked_into()));

        Progress::Finished(())
    }
}

struct FragmentShaderDropCommand {
    id: JsId,
}

unsafe impl GpuTask<Connection> for FragmentShaderDropCommand {
    type Output = ();

    fn context_id(&self) -> ContextId {
        ContextId::Any
    }

    fn progress(&mut self, connection: &mut Connection) -> Progress<Self::Output> {
        let (gl, state) = unsafe { connection.unpack_mut() };
        let value = unsafe { JsId::into_value(self.id) };

        state
            .program_cache_mut()
            .remove_fragment_shader_dependent(self.id);
        gl.delete_shader(Some(&value.unchecked_into()));

        Progress::Finished(())
    }
}
