use std::borrow::Borrow;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use serde::Serialize;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext as Gl};

use crate::buffer::{Buffer, BufferUsage, IntoBuffer};
use crate::image::format::{Filterable, RenderbufferFormat, TextureFormat};
use crate::image::renderbuffer::Renderbuffer;
use crate::image::texture_2d::Texture2D;
use crate::image::texture_2d_array::Texture2DArray;
use crate::image::texture_3d::Texture3D;
use crate::image::texture_cube::TextureCube;
use crate::image::{MaxMipmapLevelsExceeded, MipmapLevels};
use crate::render_pass::{
    DefaultDepthBuffer, DefaultDepthStencilBuffer, DefaultFramebufferRef, DefaultRGBABuffer,
    DefaultRGBBuffer, DefaultStencilBuffer,
};
use crate::runtime::executor_job::job;
use crate::runtime::fenced::JsTimeoutFencedTaskRunner;
use crate::runtime::state::DynamicState;
use crate::runtime::{Connection, ContextOptions, Execution, PowerPreference, RenderingContext};
use crate::task::{GpuTask, Progress};

thread_local!(static ID_GEN: IdGen = IdGen::new());

struct IdGen {
    next: Cell<usize>,
}

impl IdGen {
    const fn new() -> Self {
        IdGen { next: Cell::new(0) }
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
    id: usize,
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

    fn create_renderbuffer<F>(&self, width: u32, height: u32) -> Renderbuffer<F>
    where
        F: RenderbufferFormat + 'static,
    {
        Renderbuffer::new(self, width, height)
    }

    fn create_texture_2d<F>(&self, width: u32, height: u32) -> Texture2D<F>
    where
        F: TextureFormat + 'static,
    {
        Texture2D::new(self, width, height)
    }

    fn create_texture_2d_mipmapped<F>(
        &self,
        width: u32,
        height: u32,
        levels: MipmapLevels,
    ) -> Result<Texture2D<F>, MaxMipmapLevelsExceeded>
    where
        F: TextureFormat + Filterable + 'static,
    {
        Texture2D::new_mipmapped(self, width, height, levels)
    }

    fn create_texture_2d_array<F>(&self, width: u32, height: u32, depth: u32) -> Texture2DArray<F>
    where
        F: TextureFormat + 'static,
    {
        Texture2DArray::new(self, width, height, depth)
    }

    fn create_texture_2d_array_mipmapped<F>(
        &self,
        width: u32,
        height: u32,
        depth: u32,
        levels: MipmapLevels,
    ) -> Result<Texture2DArray<F>, MaxMipmapLevelsExceeded>
    where
        F: TextureFormat + Filterable + 'static,
    {
        Texture2DArray::new_mipmapped(self, width, height, depth, levels)
    }

    fn create_texture_3d<F>(&self, width: u32, height: u32, depth: u32) -> Texture3D<F>
    where
        F: TextureFormat + 'static,
    {
        Texture3D::new(self, width, height, depth)
    }

    fn create_texture_3d_mipmapped<F>(
        &self,
        width: u32,
        height: u32,
        depth: u32,
        levels: MipmapLevels,
    ) -> Result<Texture3D<F>, MaxMipmapLevelsExceeded>
    where
        F: TextureFormat + Filterable + 'static,
    {
        Texture3D::new_mipmapped(self, width, height, depth, levels)
    }

    fn create_texture_cube<F>(&self, width: u32, height: u32) -> TextureCube<F>
    where
        F: TextureFormat + 'static,
    {
        TextureCube::new(self, width, height)
    }

    fn create_texture_cube_mipmapped<F>(
        &self,
        width: u32,
        height: u32,
        levels: MipmapLevels,
    ) -> Result<TextureCube<F>, MaxMipmapLevelsExceeded>
    where
        F: TextureFormat + Filterable + 'static,
    {
        TextureCube::new_mipmapped(self, width, height, levels)
    }

    fn submit<T>(&self, task: T) -> Execution<T::Output>
    where
        T: GpuTask<Connection> + 'static,
    {
        self.executor.borrow_mut().accept(task)
    }
}

impl SingleThreadedContext {
    pub unsafe fn from_webgl2_context(gl: Gl, state: DynamicState) -> Self {
        let id = ID_GEN.with(|id_gen| id_gen.next());

        SingleThreadedContext {
            executor: RefCell::new(SingleThreadedExecutor::new(Connection::new(id, gl, state)))
                .into(),
            id,
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

pub unsafe fn single_threaded<O>(canvas: &HtmlCanvasElement, options: &O) -> O::Output
where
    O: Options,
{
    options.get_context(canvas)
}

pub trait Options {
    type Output;

    unsafe fn get_context(&self, canvas: &HtmlCanvasElement) -> Self::Output;
}

impl Options for ContextOptions<DefaultRGBABuffer, ()> {
    type Output = Result<
        (
            SingleThreadedContext,
            DefaultFramebufferRef<DefaultRGBBuffer, ()>,
        ),
        String,
    >;

    unsafe fn get_context(&self, canvas: &HtmlCanvasElement) -> Self::Output {
        let options = JsValue::from_serde(&OptionsJson {
            alpha: true,
            antialias: self.antialias(),
            depth: false,
            fail_if_major_performance_caveat: self.fail_if_major_performance_caveat(),
            power_preference: self.power_preference(),
            premultiplied_alpha: self.premultiplied_alpha(),
            preserve_drawing_buffer: self.preserve_drawing_buffer(),
            stencil: false,
        })
        .unwrap();

        let gl = canvas
            .get_context_with_context_options("webgl2", &options)
            .map_err(|e| e.as_string().unwrap())?
            .unwrap()
            .unchecked_into();
        let state = DynamicState::initial(&gl);
        let context = SingleThreadedContext::from_webgl2_context(gl, state);
        let default_framebuffer_ref = DefaultFramebufferRef::new(context.id());

        Ok((context, default_framebuffer_ref))
    }
}

impl Options for ContextOptions<DefaultRGBABuffer, DefaultDepthStencilBuffer> {
    type Output = Result<
        (
            SingleThreadedContext,
            DefaultFramebufferRef<DefaultRGBBuffer, ()>,
        ),
        String,
    >;

    unsafe fn get_context(&self, canvas: &HtmlCanvasElement) -> Self::Output {
        let options = JsValue::from_serde(&OptionsJson {
            alpha: true,
            antialias: self.antialias(),
            depth: true,
            fail_if_major_performance_caveat: self.fail_if_major_performance_caveat(),
            power_preference: self.power_preference(),
            premultiplied_alpha: self.premultiplied_alpha(),
            preserve_drawing_buffer: self.preserve_drawing_buffer(),
            stencil: true,
        })
        .unwrap();

        let gl = canvas
            .get_context_with_context_options("webgl2", &options)
            .map_err(|e| e.as_string().unwrap())?
            .unwrap()
            .unchecked_into();
        let state = DynamicState::initial(&gl);
        let context = SingleThreadedContext::from_webgl2_context(gl, state);
        let default_framebuffer_ref = DefaultFramebufferRef::new(context.id());

        Ok((context, default_framebuffer_ref))
    }
}

impl Options for ContextOptions<DefaultRGBABuffer, DefaultDepthBuffer> {
    type Output = Result<
        (
            SingleThreadedContext,
            DefaultFramebufferRef<DefaultRGBBuffer, ()>,
        ),
        String,
    >;

    unsafe fn get_context(&self, canvas: &HtmlCanvasElement) -> Self::Output {
        let options = JsValue::from_serde(&OptionsJson {
            alpha: true,
            antialias: self.antialias(),
            depth: true,
            fail_if_major_performance_caveat: self.fail_if_major_performance_caveat(),
            power_preference: self.power_preference(),
            premultiplied_alpha: self.premultiplied_alpha(),
            preserve_drawing_buffer: self.preserve_drawing_buffer(),
            stencil: false,
        })
        .unwrap();

        let gl = canvas
            .get_context_with_context_options("webgl2", &options)
            .map_err(|e| e.as_string().unwrap())?
            .unwrap()
            .unchecked_into();
        let state = DynamicState::initial(&gl);
        let context = SingleThreadedContext::from_webgl2_context(gl, state);
        let default_framebuffer_ref = DefaultFramebufferRef::new(context.id());

        Ok((context, default_framebuffer_ref))
    }
}

impl Options for ContextOptions<DefaultRGBABuffer, DefaultStencilBuffer> {
    type Output = Result<
        (
            SingleThreadedContext,
            DefaultFramebufferRef<DefaultRGBBuffer, ()>,
        ),
        String,
    >;

    unsafe fn get_context(&self, canvas: &HtmlCanvasElement) -> Self::Output {
        let options = JsValue::from_serde(&OptionsJson {
            alpha: true,
            antialias: self.antialias(),
            depth: false,
            fail_if_major_performance_caveat: self.fail_if_major_performance_caveat(),
            power_preference: self.power_preference(),
            premultiplied_alpha: self.premultiplied_alpha(),
            preserve_drawing_buffer: self.preserve_drawing_buffer(),
            stencil: true,
        })
        .unwrap();

        let gl = canvas
            .get_context_with_context_options("webgl2", &options)
            .map_err(|e| e.as_string().unwrap())?
            .unwrap()
            .unchecked_into();
        let state = DynamicState::initial(&gl);
        let context = SingleThreadedContext::from_webgl2_context(gl, state);
        let default_framebuffer_ref = DefaultFramebufferRef::new(context.id());

        Ok((context, default_framebuffer_ref))
    }
}

impl Options for ContextOptions<DefaultRGBBuffer, ()> {
    type Output = Result<
        (
            SingleThreadedContext,
            DefaultFramebufferRef<DefaultRGBBuffer, ()>,
        ),
        String,
    >;

    unsafe fn get_context(&self, canvas: &HtmlCanvasElement) -> Self::Output {
        let options = JsValue::from_serde(&OptionsJson {
            alpha: true,
            antialias: self.antialias(),
            depth: false,
            fail_if_major_performance_caveat: self.fail_if_major_performance_caveat(),
            power_preference: self.power_preference(),
            premultiplied_alpha: self.premultiplied_alpha(),
            preserve_drawing_buffer: self.preserve_drawing_buffer(),
            stencil: false,
        })
        .unwrap();

        let gl = canvas
            .get_context_with_context_options("webgl2", &options)
            .map_err(|e| e.as_string().unwrap())?
            .unwrap()
            .unchecked_into();
        let state = DynamicState::initial(&gl);
        let context = SingleThreadedContext::from_webgl2_context(gl, state);
        let default_framebuffer_ref = DefaultFramebufferRef::new(context.id());

        Ok((context, default_framebuffer_ref))
    }
}

impl Options for ContextOptions<DefaultRGBBuffer, DefaultDepthStencilBuffer> {
    type Output = Result<
        (
            SingleThreadedContext,
            DefaultFramebufferRef<DefaultRGBBuffer, ()>,
        ),
        String,
    >;

    unsafe fn get_context(&self, canvas: &HtmlCanvasElement) -> Self::Output {
        let options = JsValue::from_serde(&OptionsJson {
            alpha: true,
            antialias: self.antialias(),
            depth: true,
            fail_if_major_performance_caveat: self.fail_if_major_performance_caveat(),
            power_preference: self.power_preference(),
            premultiplied_alpha: self.premultiplied_alpha(),
            preserve_drawing_buffer: self.preserve_drawing_buffer(),
            stencil: true,
        })
        .unwrap();

        let gl = canvas
            .get_context_with_context_options("webgl2", &options)
            .map_err(|e| e.as_string().unwrap())?
            .unwrap()
            .unchecked_into();
        let state = DynamicState::initial(&gl);
        let context = SingleThreadedContext::from_webgl2_context(gl, state);
        let default_framebuffer_ref = DefaultFramebufferRef::new(context.id());

        Ok((context, default_framebuffer_ref))
    }
}

impl Options for ContextOptions<DefaultRGBBuffer, DefaultDepthBuffer> {
    type Output = Result<
        (
            SingleThreadedContext,
            DefaultFramebufferRef<DefaultRGBBuffer, ()>,
        ),
        String,
    >;

    unsafe fn get_context(&self, canvas: &HtmlCanvasElement) -> Self::Output {
        let options = JsValue::from_serde(&OptionsJson {
            alpha: true,
            antialias: self.antialias(),
            depth: true,
            fail_if_major_performance_caveat: self.fail_if_major_performance_caveat(),
            power_preference: self.power_preference(),
            premultiplied_alpha: self.premultiplied_alpha(),
            preserve_drawing_buffer: self.preserve_drawing_buffer(),
            stencil: false,
        })
        .unwrap();

        let gl = canvas
            .get_context_with_context_options("webgl2", &options)
            .map_err(|e| e.as_string().unwrap())?
            .unwrap()
            .unchecked_into();
        let state = DynamicState::initial(&gl);
        let context = SingleThreadedContext::from_webgl2_context(gl, state);
        let default_framebuffer_ref = DefaultFramebufferRef::new(context.id());

        Ok((context, default_framebuffer_ref))
    }
}

impl Options for ContextOptions<DefaultRGBBuffer, DefaultStencilBuffer> {
    type Output = Result<
        (
            SingleThreadedContext,
            DefaultFramebufferRef<DefaultRGBBuffer, ()>,
        ),
        String,
    >;

    unsafe fn get_context(&self, canvas: &HtmlCanvasElement) -> Self::Output {
        let options = JsValue::from_serde(&OptionsJson {
            alpha: true,
            antialias: self.antialias(),
            depth: false,
            fail_if_major_performance_caveat: self.fail_if_major_performance_caveat(),
            power_preference: self.power_preference(),
            premultiplied_alpha: self.premultiplied_alpha(),
            preserve_drawing_buffer: self.preserve_drawing_buffer(),
            stencil: true,
        })
        .unwrap();

        let gl = canvas
            .get_context_with_context_options("webgl2", &options)
            .map_err(|e| e.as_string().unwrap())?
            .unwrap()
            .unchecked_into();
        let state = DynamicState::initial(&gl);
        let context = SingleThreadedContext::from_webgl2_context(gl, state);
        let default_framebuffer_ref = DefaultFramebufferRef::new(context.id());

        Ok((context, default_framebuffer_ref))
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct OptionsJson {
    alpha: bool,
    antialias: bool,
    depth: bool,
    fail_if_major_performance_caveat: bool,
    power_preference: PowerPreference,
    premultiplied_alpha: bool,
    preserve_drawing_buffer: bool,
    stencil: bool,
}
