use rendering_context::RenderingContext;
use util::JsId;
use task::GpuTask;
use rendering_context::Connection;
use task::Progress;
use wasm_bindgen::JsCast;
use std::sync::Arc;
use rendering_context::ContextUpdate;
use image_format::InternalFormat;
use std::marker;
use web_sys::WebGl2RenderingContext as Gl;

pub unsafe trait RenderbufferFormat: InternalFormat {}

pub trait Renderbuffer<F> where F: RenderbufferFormat {
    fn width(&self) -> u32;

    fn height(&self) -> u32;
}

pub struct RenderbufferHandle<F, C> where C: RenderingContext {
    data: Arc<RenderbufferData<C>>,
    _marker: marker::PhantomData<[F]>
}

pub(crate) struct RenderbufferData<C> where C: RenderingContext {
    gl_object_id: Option<JsId>,
    context: C,
    width: u32,
    height: u32
}

impl<F, C> Renderbuffer<F> for RenderbufferHandle<F, C> where F: RenderbufferFormat, C: RenderingContext {
    fn width(&self) -> u32 {
        self.data.width
    }

    fn height(&self) -> u32 {
        self.data.height
    }
}

impl<C> Drop for RenderbufferData<C> where C: RenderingContext {
    fn drop(&mut self) {
        if let Some(id) = self.gl_object_id {
            self.context.submit(RenderbufferDropTask {
                id
            });
        }
    }
}

struct RenderbufferAllocateTask<F, C> where C: RenderingContext {
    data: Arc<RenderbufferData<C>>,
    _marker: marker::PhantomData<[F]>
}

impl<F, C> GpuTask<Connection> for RenderbufferAllocateTask<F, C> where F: RenderbufferFormat, C: RenderingContext {
    type Output = ();

    type Error = ();

    fn progress(&mut self, connection: &mut Connection) -> Progress<Self::Output, Self::Error> {
        let Connection(gl, state) = connection;

        let object = gl.create_renderbuffer().unwrap();

        state.set_bound_renderbuffer(Some(&object)).apply(gl).unwrap();

        let data = &self.data;

        gl.renderbuffer_storage(Gl::RENDERBUFFER, F::id(), data.width as i32, data.height as i32);

        unsafe {
            let ptr = &data.gl_object_id as *const _ as *mut Option<JsId>;

            *ptr = Some(JsId::from_value(object.into()));
        }

        Progress::Finished(Ok(()))
    }
}

struct RenderbufferDropTask {
    id: JsId
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