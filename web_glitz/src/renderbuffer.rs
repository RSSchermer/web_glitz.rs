use image_format::InternalFormat;
use rendering_context::Connection;
use rendering_context::ContextUpdate;
use rendering_context::RenderingContext;
use std::marker;
use std::sync::Arc;
use task::GpuTask;
use task::Progress;
use util::JsId;
use wasm_bindgen::JsCast;
use web_sys::WebGl2RenderingContext as Gl;

pub unsafe trait RenderbufferFormat: InternalFormat {}

pub trait Renderbuffer<F>
where
    F: RenderbufferFormat,
{
    fn width(&self) -> u32;

    fn height(&self) -> u32;
}

pub struct RenderbufferHandle<F, Rc>
where
    Rc: RenderingContext,
{
    pub(crate) data: Arc<RenderbufferData<Rc>>,
    _marker: marker::PhantomData<[F]>,
}

pub(crate) struct RenderbufferData<Rc>
where
    Rc: RenderingContext,
{
    pub(crate) id: Option<JsId>,
    context: Rc,
    width: u32,
    height: u32,
}

impl<F, Rc> Renderbuffer<F> for RenderbufferHandle<F, Rc>
where
    F: RenderbufferFormat,
    Rc: RenderingContext,
{
    fn width(&self) -> u32 {
        self.data.width
    }

    fn height(&self) -> u32 {
        self.data.height
    }
}

impl<Rc> Drop for RenderbufferData<Rc>
where
    Rc: RenderingContext,
{
    fn drop(&mut self) {
        if let Some(id) = self.id {
            self.context.submit(RenderbufferDropTask { id });
        }
    }
}

struct RenderbufferAllocateTask<F, Rc>
where
    Rc: RenderingContext,
{
    data: Arc<RenderbufferData<Rc>>,
    _marker: marker::PhantomData<[F]>,
}

impl<F, Rc> GpuTask<Connection> for RenderbufferAllocateTask<F, Rc>
where
    F: RenderbufferFormat,
    Rc: RenderingContext,
{
    type Output = ();

    type Error = ();

    fn progress(&mut self, connection: &mut Connection) -> Progress<Self::Output, Self::Error> {
        let Connection(gl, state) = connection;
        let mut data = Arc::get_mut(&mut self.data).unwrap();
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

        Progress::Finished(Ok(()))
    }
}

struct RenderbufferDropTask {
    id: JsId,
}

impl GpuTask<Connection> for RenderbufferDropTask {
    type Output = ();

    type Error = ();

    fn progress(&mut self, connection: &mut Connection) -> Progress<Self::Output, Self::Error> {
        let Connection(gl, _) = connection;

        unsafe {
            gl.delete_renderbuffer(Some(&JsId::into_value(self.id).unchecked_into()));
        }

        Progress::Finished(Ok(()))
    }
}
