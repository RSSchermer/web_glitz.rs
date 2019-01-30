use std::marker;
use std::sync::Arc;

use wasm_bindgen::JsCast;
use web_sys::WebGl2RenderingContext as Gl;

use crate::image_format::InternalFormat;
use crate::runtime::dynamic_state::ContextUpdate;
use crate::runtime::{Connection, RenderingContext};
use crate::task::{GpuTask, Progress};
use crate::util::{arc_get_mut_unchecked, JsId};

pub unsafe trait RenderbufferFormat: InternalFormat {}

pub struct RenderbufferHandle<F> {
    data: Arc<RenderbufferData>,
    _marker: marker::PhantomData<[F]>,
}

impl<F> RenderbufferHandle<F>
where
    F: RenderbufferFormat + 'static,
{
    pub(crate) fn new<Rc>(context: &Rc, width: u32, height: u32) -> Self
    where
        Rc: RenderingContext + Clone + 'static,
    {
        let data = Arc::new(RenderbufferData {
            id: None,
            dropper: Box::new(context.clone()),
            width,
            height,
        });

        context.submit(RenderbufferAllocateTask::<F> {
            data: data.clone(),
            _marker: marker::PhantomData,
        });

        RenderbufferHandle {
            data,
            _marker: marker::PhantomData,
        }
    }

    pub(crate) fn id(&self) -> Option<JsId> {
        self.data.id
    }

    pub fn width(&self) -> u32 {
        self.data.width
    }

    pub fn height(&self) -> u32 {
        self.data.height
    }
}

trait RenderbufferObjectDropper {
    fn drop_renderbuffer_object(&self, id: JsId);
}

impl<T> RenderbufferObjectDropper for T
    where
        T: RenderingContext,
{
    fn drop_renderbuffer_object(&self, id: JsId) {
        self.submit(RenderbufferDropCommand { id });
    }
}

pub(crate) struct RenderbufferData {
    id: Option<JsId>,
    dropper: Box<RenderbufferObjectDropper>,
    width: u32,
    height: u32,
}

impl Drop for RenderbufferData {
    fn drop(&mut self) {
        if let Some(id) = self.id {
            self.dropper.drop_renderbuffer_object(id);
        }
    }
}

struct RenderbufferAllocateTask<F> {
    data: Arc<RenderbufferData>,
    _marker: marker::PhantomData<[F]>,
}

impl<F> GpuTask<Connection> for RenderbufferAllocateTask<F>
where
    F: RenderbufferFormat,
{
    type Output = ();

    fn progress(&mut self, connection: &mut Connection) -> Progress<Self::Output> {
        let (gl, state) = unsafe { connection.unpack_mut() };
        let mut data = unsafe { arc_get_mut_unchecked(&mut self.data) };
        let object = gl.create_renderbuffer().unwrap();

        state
            .set_bound_renderbuffer(Some(&object))
            .apply(gl)
            .unwrap();

        gl.renderbuffer_storage(
            Gl::RENDERBUFFER,
            F::id(),
            data.width as i32,
            data.height as i32,
        );

        data.id = Some(JsId::from_value(object.into()));

        Progress::Finished(())
    }
}

struct RenderbufferDropCommand {
    id: JsId,
}

impl GpuTask<Connection> for RenderbufferDropCommand {
    type Output = ();

    fn progress(&mut self, connection: &mut Connection) -> Progress<Self::Output> {
        let (gl, _) = unsafe { connection.unpack() };
        let value = unsafe { JsId::into_value(self.id) };

        gl.delete_renderbuffer(Some(&value.unchecked_into()));

        Progress::Finished(())
    }
}
