use std::borrow::Borrow;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use web_sys::WebGl2RenderingContext as Gl;

use crate::buffer::{Buffer, BufferUsage, IntoBuffer};
use crate::image::format::{Filterable, TextureFormat, RenderbufferFormat};
use crate::image::renderbuffer::RenderbufferHandle;
use crate::image::texture_2d::Texture2D;
use crate::image::texture_2d_array::Texture2DArray;
use crate::image::texture_3d::Texture3D;
use crate::image::texture_cube::TextureCubeHandle;
use crate::runtime::dynamic_state::DynamicState;
use crate::runtime::executor_job::job;
use crate::runtime::fenced::JsTimeoutFencedTaskRunner;
use crate::runtime::{Connection, Execution, RenderingContext};
use crate::task::{GpuTask, Progress};

thread_local!(static ID_GEN: IdGen = IdGen::new());

struct IdGen {
    next: Cell<usize>
}

impl IdGen {
    const fn new() -> Self {
        IdGen {
            next: Cell::new(0)
        }
    }

    fn next(&self) -> usize {
        let next = self.next.get();

        self.next.set(next + 1);

        next
    }
}

#[derive(Clone)]
pub struct SingleThreadedContext {
    executor: Rc<RefCell<SingleThreadedExecutor>>,
    id: usize
}

impl RenderingContext for SingleThreadedContext {
    fn id(&self) -> usize {
        self.id
    }

    fn create_buffer<D, T>(&self, data: D, usage_hint: BufferUsage) -> Buffer<T>
    where
        D: IntoBuffer<T>,
    {
        data.into_buffer(self, usage_hint)
    }

    fn create_renderbuffer<F>(&self, width: u32, height: u32) -> RenderbufferHandle<F>
    where
        F: RenderbufferFormat + 'static,
    {
        RenderbufferHandle::new(
            self,
            width,
            height,
        )
    }

    fn create_texture_2d<F>(&self, width: u32, height: u32) -> Texture2D<F>
    where
        F: TextureFormat + 'static,
    {
        Texture2D::new(
            self,
            width,
            height,
        )
    }

    fn create_texture_2d_mipmapped<F>(
        &self,
        width: u32,
        height: u32,
        levels: usize,
    ) -> Texture2D<F>
    where
        F: TextureFormat + Filterable + 'static,
    {
        Texture2D::new_mipmapped(
            self,
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
    ) -> Texture2DArray<F>
    where
        F: TextureFormat + 'static,
    {
        Texture2DArray::new(
            self,
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
    ) -> Texture3D<F>
    where
        F: TextureFormat + 'static,
    {
        Texture3D::new(
            self,
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
        let id = ID_GEN.with(|id_gen| id_gen.next());

        SingleThreadedContext {
            executor: RefCell::new(SingleThreadedExecutor::new(Connection::new(id, gl, state))).into(),
            id
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
