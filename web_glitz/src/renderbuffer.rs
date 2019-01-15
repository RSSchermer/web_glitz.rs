use std::marker;
use std::sync::Arc;

use wasm_bindgen::JsCast;
use web_sys::WebGl2RenderingContext as Gl;

use crate::image_format::InternalFormat;
use crate::runtime::dropper::{DropObject, Dropper, RefCountedDropper};
use crate::runtime::dynamic_state::ContextUpdate;
use crate::runtime::{Connection, RenderingContext};
use crate::task::{GpuTask, Progress};
use crate::util::{arc_get_mut_unchecked, JsId};

pub unsafe trait RenderbufferFormat: InternalFormat {}

pub struct RenderbufferHandle<F> {
    pub(crate) data: Arc<RenderbufferData>,
    _marker: marker::PhantomData<[F]>,
}

impl<F> RenderbufferHandle<F>
where
    F: RenderbufferFormat + 'static,
{
    pub(crate) fn new<Rc>(context: &Rc, dropper: RefCountedDropper, width: u32, height: u32) -> Self
    where
        Rc: RenderingContext,
    {
        let data = Arc::new(RenderbufferData {
            id: None,
            dropper,
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

    fn width(&self) -> u32 {
        self.data.width
    }

    fn height(&self) -> u32 {
        self.data.height
    }
}

pub(crate) struct RenderbufferData {
    pub(crate) id: Option<JsId>,
    dropper: RefCountedDropper,
    width: u32,
    height: u32,
}

impl Drop for RenderbufferData {
    fn drop(&mut self) {
        if let Some(id) = self.id {
            self.dropper.drop_gl_object(DropObject::Renderbuffer(id));
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
        let Connection(gl, state) = connection;
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
