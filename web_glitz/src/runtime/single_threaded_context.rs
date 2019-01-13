use std::cell::RefCell;
use std::rc::Rc;

use web_sys::WebGl2RenderingContext as Gl;

use crate::buffer::{BufferHandle, BufferUsage};
use crate::framebuffer::{FramebufferDescriptor, FramebufferHandle};
use crate::image_format::Filterable;
use crate::renderbuffer::{RenderbufferFormat, RenderbufferHandle};
use crate::runtime::dropper::{DropObject, DropTask, Dropper, RefCountedDropper};
use crate::runtime::dynamic_state::DynamicState;
use crate::runtime::executor_job::job;
use crate::runtime::fenced::JsTimeoutFencedTaskRunner;
use crate::runtime::{Connection, Execution, RenderingContext};
use crate::task::{GpuTask, Progress};
use crate::texture::{
    Texture2DArrayHandle, Texture2DHandle, Texture3DHandle, TextureCubeHandle, TextureFormat,
};
use std::borrow::Borrow;
use buffer::IntoBuffer;

#[derive(Clone)]
pub struct SingleThreadedContext {
    executor: Rc<RefCell<SingleThreadedExecutor>>,
}

impl RenderingContext for SingleThreadedContext {
    fn create_buffer<D, T>(&self, data: D, usage_hint: BufferUsage) -> BufferHandle<T> where D: IntoBuffer<T> {
        data.into_buffer(self, usage_hint)
    }

    fn create_framebuffer<D>(&self, descriptor: &D) -> FramebufferHandle
    where
        D: FramebufferDescriptor,
    {
        FramebufferHandle::new(
            self,
            RefCountedDropper::Rc(self.executor.clone()),
            descriptor,
        )
    }

    fn create_renderbuffer<F>(&self, width: u32, height: u32) -> RenderbufferHandle<F>
    where
        F: RenderbufferFormat + 'static,
    {
        RenderbufferHandle::new(
            self,
            RefCountedDropper::Rc(self.executor.clone()),
            width,
            height,
        )
    }

    fn create_texture_2d<F>(&self, width: u32, height: u32) -> Texture2DHandle<F>
    where
        F: TextureFormat + 'static,
    {
        Texture2DHandle::new(
            self,
            RefCountedDropper::Rc(self.executor.clone()),
            width,
            height,
        )
    }

    fn create_texture_2d_mipmapped<F>(
        &self,
        width: u32,
        height: u32,
        levels: usize,
    ) -> Texture2DHandle<F>
    where
        F: TextureFormat + Filterable + 'static,
    {
        Texture2DHandle::new_mipmapped(
            self,
            RefCountedDropper::Rc(self.executor.clone()),
            width,
            height,
            levels,
        )
    }

    fn create_texture_2d_array<F>(
        &self,
        width: u32,
        height: u32,
        depth: u32,
        levels: usize,
    ) -> Texture2DArrayHandle<F>
    where
        F: TextureFormat + 'static,
    {
        Texture2DArrayHandle::new(
            self,
            RefCountedDropper::Rc(self.executor.clone()),
            width,
            height,
            depth,
            levels,
        )
    }

    fn create_texture_3d<F>(
        &self,
        width: u32,
        height: u32,
        depth: u32,
        levels: usize,
    ) -> Texture3DHandle<F>
    where
        F: TextureFormat + 'static,
    {
        Texture3DHandle::new(
            self,
            RefCountedDropper::Rc(self.executor.clone()),
            width,
            height,
            depth,
            levels,
        )
    }

    fn create_texture_cube<F>(&self, width: u32, height: u32, levels: usize) -> TextureCubeHandle<F>
    where
        F: TextureFormat + 'static,
    {
        TextureCubeHandle::new(
            self,
            RefCountedDropper::Rc(self.executor.clone()),
            width,
            height,
            levels,
        )
    }

    fn submit<T>(&self, task: T) -> Execution<T::Output>
    where
        T: GpuTask<Connection> + 'static,
    {
        self.executor.borrow_mut().accept(task)
    }
}

impl SingleThreadedContext {
    pub fn from_webgl2_context(gl: Gl, state: DynamicState) -> Self {
        SingleThreadedContext {
            executor: RefCell::new(SingleThreadedExecutor::new(Connection(gl, state))).into(),
        }
    }
}

struct SingleThreadedExecutor {
    connection: Rc<RefCell<Connection>>,
    fenced_task_queue_runner: JsTimeoutFencedTaskRunner,
}

impl SingleThreadedExecutor {
    fn new(connection: Connection) -> Self {
        let connection = Rc::new(RefCell::new(connection));
        let fenced_task_queue_runner = JsTimeoutFencedTaskRunner::new(connection.clone());

        SingleThreadedExecutor {
            connection,
            fenced_task_queue_runner,
        }
    }

    fn accept<T>(&mut self, mut task: T) -> Execution<T::Output>
    where
        T: GpuTask<Connection> + 'static,
    {
        match task.progress(&mut self.connection.borrow_mut()) {
            Progress::Finished(res) => res.into(),
            Progress::ContinueFenced => {
                let (job, execution) = job(task);

                self.fenced_task_queue_runner.schedule(job);

                execution
            }
        }
    }
}

impl Dropper for RefCell<SingleThreadedExecutor> {
    fn drop_gl_object(&self, object: DropObject) {
        self.borrow_mut().accept(DropTask::new(object));
    }
}
