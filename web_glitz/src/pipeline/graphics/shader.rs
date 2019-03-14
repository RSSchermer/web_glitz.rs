use crate::runtime::{Connection, RenderingContext};
use crate::task::{GpuTask, Progress};
use crate::util::{arc_get_mut_unchecked, JsId};
use std::borrow::Borrow;
use std::sync::Arc;

pub struct VertexShader {
    data: Arc<ShaderData>,
}

impl VertexShader {
    pub(crate) fn new<S, Rc>(context: &Rc, source: S) -> Self
    where
        S: Borrow<str> + 'static,
        Rc: RenderingContext,
    {
        let data = Arc::new(ShaderData {
            id: None,
            context_id: context.id(),
            dropper: Box::new(context.clone()),
        });

        context.submit(ShaderAllocateCommand {
            data: data.clone(),
            tpe: Gl::VERTEX_SHADER,
            source,
        });

        VertexShader { data }
    }
}

pub struct FragmentShader {
    data: Arc<ShaderData>,
}

impl FragmentShader {
    pub(crate) fn new<S, Rc>(context: &Rc, source: S) -> Self
    where
        S: Borrow<str> + 'static,
        Rc: RenderingContext,
    {
        let data = Arc::new(ShaderData {
            id: None,
            context_id: context.id(),
            dropper: Box::new(context.clone()),
        });

        context.submit(ShaderAllocateCommand {
            data: data.clone(),
            tpe: Gl::VERTEX_SHADER,
            source,
        });

        FragmentShader { data }
    }
}

pub(crate) struct ShaderData {
    id: Option<JsId>,
    context_id: usize,
    dropper: Box<ShaderObjectDropper>,
}

impl ShaderData {
    pub(crate) fn id(&self) -> Option<JsId> {
        self.id
    }

    pub(crate) fn context_id(&self) -> usize {
        self.context_id
    }
}

trait ShaderObjectDropper {
    fn drop_shader_object(&self, id: JsId);
}

impl<T> ShaderObjectDropper for T
where
    T: RenderingContext,
{
    fn drop_shader_object(&self, id: JsId) {
        self.submit(ShaderDropCommand { id });
    }
}

impl Drop for ShaderData {
    fn drop(&mut self) {
        if let Some(id) = self.id {
            self.dropper.drop_shader_object(id);
        }
    }
}

struct ShaderAllocateCommand<S> {
    data: Arc<ShaderData>,
    tpe: u32,
    source: S,
}

impl<S> GpuTask<Connection> for ShaderAllocateCommand<S>
where
    S: Borrow<str>,
{
    type Output = ();

    fn progress(&mut self, connection: &mut Connection) -> Progress<Self::Output> {
        let (gl, _) = unsafe { connection.unpack_mut() };
        let mut data = unsafe { arc_get_mut_unchecked(&mut self.data) };

        let shader_object = gl.create_shader(self.tpe).unwrap();

        gl.shader_source(&shader_object, self.source.borrow());
        gl.compile_shader(&shader_object);

        data.id = Some(JsId::from_value(shader_object.into()));

        Progress::Finished(())
    }
}

struct ShaderDropCommand {
    id: JsId,
}

impl GpuTask<Connection> for ShaderDropCommand {
    type Output = ();

    fn progress(&mut self, connection: &mut Connection) -> Progress<Self::Output> {
        let (gl, _) = unsafe { connection.unpack() };
        let value = unsafe { JsId::into_value(self.id) };

        gl.delete_shader(Some(&value.unchecked_into()));

        Progress::Finished(())
    }
}
