//! This module implements a runtime designed to run on a single thread (the main thread). This
//! runtime's [RenderingContext] implementation, [SingleThreadedContext], may be [Clone]d (which
//! results in another handle to the same context), but neither it nor any of its clones may be
//! send to or shared with other threads/workers.
//!
//! Tasks submitted to a single threaded context (see [RenderingContext::submit]) are executed
//! immediately and typically don't involve dynamic dispatch (unless the task becomes fenced, in
//! which case it will be "boxed" and stored in the "fenced-task queue" from where it will be
//! invoked dynamically once its fence becomes signalled, see below).
//!
//! # Example
//!
//! A new single threaded runtime may be initialized by calling [init] with a
//! [web_sys::HtmlCanvasElement] and [ContextOptions]:
//!
//! ```no_run
//! use wasm_bindgen::JsCast;
//! use web_glitz::runtime::{single_threaded, ContextOptions};
//! use web_sys::{window, HtmlCanvasElement};
//!
//! let canvas: HtmlCanvasElement = window()
//!     .unwrap()
//!     .document()
//!     .unwrap()
//!     .get_element_by_id("canvas")
//!     .unwrap()
//!     .dyn_into()
//!     .unwrap();
//!
//! let options = ContextOptions::default();
//!
//! let (context, render_target) = unsafe { single_threaded::init(&canvas, &options).unwrap() };
//! ```
//!
//! This returns a tuple of the [SingleThreadedContext] and the [DefaultRenderTarget] for the canvas
//! or an error if the requested [ContextOptions] could not be supported. For more details on the
//! options available when initializing a single threaded runtime, see [ContextOptions].
//!
//! # Unsafe
//!
//! Note that the [init] function is marked `unsafe`: the canvas's WebGL2 context must be in its
//! original state when [init] was called. Additionally, for the lifetime of the
//! [SingleThreadedContext] or any of its clones, the state of the context should not be modified
//! through another handle to the canvas's raw WebGL2 context; the [SingleThreadedContext] tracks
//! the changes it makes to the state of its associated WebGL2 context in a state cache and if at
//! any point during the execution of a task the actual state of the WebGL2b context and the cached
//! state don't match, unexpected results may ocurr. In short: if you only initialize one WebGlitz
//! [RenderingContext] or raw WebGL2 context per canvas, then calling [init] is safe.
//!
//! If you do wish to access or use the raw [web_sys::WebGl2RenderingContext], rather than obtaining
//! a seperate WebGL2 context directly from the canvas, instead consider implementing your own
//! [GpuTask]:
//!
//! ```
//! use web_glitz::task::{GpuTask, Progress, ContextId};
//! use web_glitz::runtime::Connection;
//!
//! struct MyTask {
//!     // ...
//! }
//!
//! unsafe impl GpuTask<Connection> for MyTask {
//!     type Output = ();
//!
//!     fn context_id(&self) -> ContextId {
//!         ContextId::Any
//!     }
//!
//!     fn progress(&mut self, connection: &mut Connection) -> Progress<Self::Output> {
//!         let (raw_context, state_cache) = unsafe { connection.unpack_mut() };
//!
//!         // Do something using the raw context...
//!
//!         Progress::Finished(())
//!     }
//! }
//! ```
//!
//! You may unpack the `connection` into a reference to the raw WebGL2 context and the state cache
//! by calling [Connection::unpack_mut]. This is unsafe: you must ensure that the state cache
//! reflects the actual state of the WebGL2 context when your implementation of [GpuTask::progress]
//! returns (by updating the state cache when necessary, see [DynamicState] for details).
//!
//! # Multi-part Tasks and Fencing
//!
//! A [GpuTask] may consists of multiple stages, where in between stages the task has to wait for a
//! GPU fence to become signalled. This mostly concerns tasks that contain "read" or "download"
//! commands (commands with non-void outputs), where the first part of the command sets up the
//! command, then a fence is inserted, and then the actual read/download occurs once the fence is
//! reached; this may avoid stalls on the CPU and/or GPU. This runtime handles such tasks by
//! maintaining a "fenced-task" queue for tasks where [GpuTask::progress] returns
//! [Progress::ContinueFenced]. If this queue is not empty, then a 1ms timeout is scheduled with the
//! JavaScript event queue. After this timeout expires it will try to again make progress on the
//! tasks in the fenced-task queue (this shortcircuits on the first fence that has not yet become
//! signalled, as WebGL/OpenGL fences cannot become signalled out of order). If the fenced-task
//! queue is not emptied (either because not all fences became signalled, or because one of the
//! tasks again returned [Progress::ContinueFenced]), then a new 1ms timeout is scheduled on the
//! JavaScript event loop.
//!
//! Note that such repeated scheduling of timeout events may result in throttling (to ~4ms) in most
//! browsers after a certain number of iterations (5 in Chrome and FireFox, 6 in Safari and 3 in
//! Edge, at the time of this writing). Note also that timeouts indicate a minimum timeout: if the
//! JavaScript main thread is already busy, or higher priority events exists in the event queue
//! (micro-tasks or macro-tasks that were scheduled earlier), then the JavaScript/WASM runtime will
//! finish this work before checking the fenced-task queue again.

use std::any::TypeId;
use std::borrow::Borrow;
use std::cell::{Cell, RefCell};
use std::hash::{Hash, Hasher};
use std::rc::Rc;

use fnv::FnvHasher;
use serde_derive::Serialize;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext as Gl};

use crate::buffer::{Buffer, IntoBuffer, UsageHint};
use crate::image::format::{RenderbufferFormat, TextureFormat};
use crate::image::renderbuffer::{Renderbuffer, RenderbufferDescriptor};
use crate::image::texture_2d::{Texture2D, Texture2DDescriptor};
use crate::image::texture_2d_array::{Texture2DArray, Texture2DArrayDescriptor};
use crate::image::texture_3d::{Texture3D, Texture3DDescriptor};
use crate::image::texture_cube::{TextureCube, TextureCubeDescriptor};
use crate::image::MaxMipmapLevelsExceeded;
use crate::pipeline::graphics::shader::{
    FragmentShaderAllocateCommand, VertexShaderAllocateCommand,
};
use crate::pipeline::graphics::{
    FragmentShader, GraphicsPipeline, GraphicsPipelineDescriptor, VertexShader,
};
use crate::render_pass::{
    DefaultDepthBuffer, DefaultDepthStencilBuffer, DefaultRGBABuffer, DefaultRGBBuffer,
    DefaultStencilBuffer, RenderPass, RenderPassContext, RenderPassId,
};
use crate::render_target::{DefaultRenderTarget, RenderTargetDescription};
use crate::runtime::executor_job::job;
use crate::runtime::fenced::JsTimeoutFencedTaskRunner;
use crate::runtime::rendering_context::{CreateGraphicsPipelineError, Extensions};
use crate::runtime::state::DynamicState;
use crate::runtime::{
    Connection, ContextOptions, Execution, PowerPreference, RenderingContext,
    ShaderCompilationError,
};
use crate::sampler::{Sampler, SamplerDescriptor, ShadowSampler, ShadowSamplerDescriptor};
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

/// A handle to a single-threaded WebGlitz rendering context.
///
/// See the module documentation for [web_glitz::runtime::single_threaded] for details.
#[derive(Clone)]
pub struct SingleThreadedContext {
    executor: Rc<RefCell<SingleThreadedExecutor>>,
    id: usize,
    extensions: Extensions,
    last_render_pass_id: Cell<usize>,
}

impl RenderingContext for SingleThreadedContext {
    fn id(&self) -> usize {
        self.id
    }

    fn extensions(&self) -> &Extensions {
        &self.extensions
    }

    fn create_buffer<D, T>(&self, data: D, usage_hint: UsageHint) -> Buffer<T>
    where
        D: IntoBuffer<T>,
        T: ?Sized,
    {
        data.into_buffer(self, usage_hint)
    }

    fn create_renderbuffer<F>(&self, descriptor: &RenderbufferDescriptor<F>) -> Renderbuffer<F>
    where
        F: RenderbufferFormat + 'static,
    {
        Renderbuffer::new(self, descriptor)
    }

    fn create_vertex_shader<S>(&self, source: S) -> Result<VertexShader, ShaderCompilationError>
    where
        S: Borrow<str> + 'static,
    {
        let allocate_command = VertexShaderAllocateCommand::new(self, source);

        match self.submit(allocate_command) {
            Execution::Ready(res) => res.unwrap(),
            _ => unreachable!(),
        }
    }

    fn create_fragment_shader<S>(&self, source: S) -> Result<FragmentShader, ShaderCompilationError>
    where
        S: Borrow<str> + 'static,
    {
        let allocate_command = FragmentShaderAllocateCommand::new(self, source);

        match self.submit(allocate_command) {
            Execution::Ready(res) => res.unwrap(),
            _ => unreachable!(),
        }
    }

    fn create_graphics_pipeline<V, R, Tf>(
        &self,
        descriptor: &GraphicsPipelineDescriptor<V, R, Tf>,
    ) -> Result<GraphicsPipeline<V, R, Tf>, CreateGraphicsPipelineError>
    {
        let executor = self.executor.borrow_mut();
        let mut connection = executor.connection.borrow_mut();

        GraphicsPipeline::create(self, &mut connection, descriptor)
    }

    fn create_render_pass<R, F, T>(&self, mut render_target: R, f: F) -> RenderPass<T>
    where
        R: RenderTargetDescription,
        F: FnOnce(&R::Framebuffer) -> T,
        T: GpuTask<RenderPassContext>,
    {
        let id = self.last_render_pass_id.get();

        self.last_render_pass_id.set(id + 1);

        let mut hasher = FnvHasher::default();

        (self.id, id).hash(&mut hasher);

        let id = hasher.finish() as usize;

        render_target.create_render_pass(
            RenderPassId {
                id,
                context_id: self.id(),
            },
            f,
        )
    }

    fn create_texture_2d<F>(
        &self,
        descriptor: &Texture2DDescriptor<F>,
    ) -> Result<Texture2D<F>, MaxMipmapLevelsExceeded>
    where
        F: TextureFormat + 'static,
    {
        Texture2D::new(self, descriptor)
    }

    fn create_texture_2d_array<F>(
        &self,
        descriptor: &Texture2DArrayDescriptor<F>,
    ) -> Result<Texture2DArray<F>, MaxMipmapLevelsExceeded>
    where
        F: TextureFormat + 'static,
    {
        Texture2DArray::new(self, descriptor)
    }

    fn create_texture_3d<F>(
        &self,
        descriptor: &Texture3DDescriptor<F>,
    ) -> Result<Texture3D<F>, MaxMipmapLevelsExceeded>
    where
        F: TextureFormat + 'static,
    {
        Texture3D::new(self, descriptor)
    }

    fn create_texture_cube<F>(
        &self,
        descriptor: &TextureCubeDescriptor<F>,
    ) -> Result<TextureCube<F>, MaxMipmapLevelsExceeded>
    where
        F: TextureFormat + 'static,
    {
        TextureCube::new(self, descriptor)
    }

    fn create_sampler(&self, descriptor: &SamplerDescriptor) -> Sampler {
        Sampler::new(self, descriptor)
    }

    fn create_shadow_sampler(&self, descriptor: &ShadowSamplerDescriptor) -> ShadowSampler {
        ShadowSampler::new(self, descriptor)
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

        let mut hasher = FnvHasher::default();

        (TypeId::of::<Self>(), id).hash(&mut hasher);

        let id = hasher.finish() as usize;

        SingleThreadedContext {
            executor: RefCell::new(SingleThreadedExecutor::new(Connection::new(id, gl, state)))
                .into(),
            id,
            extensions: Extensions::default(),
            last_render_pass_id: Cell::new(0),
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

/// Initializes a single threaded WebGlitz runtime for the `canvas` using the `options` and returns
/// a tuple of the WebGlitz [RenderingContext] and the [DefaultRenderTarget] associated with the
/// canvas.
///
/// See the module documentation for [web_glitz::runtime::single_threaded] for details.
pub unsafe fn init<O>(canvas: &HtmlCanvasElement, options: &O) -> O::Output
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
            DefaultRenderTarget<DefaultRGBABuffer, ()>,
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
        let default_framebuffer_ref = DefaultRenderTarget::new(context.id());

        Ok((context, default_framebuffer_ref))
    }
}

impl Options for ContextOptions<DefaultRGBABuffer, DefaultDepthStencilBuffer> {
    type Output = Result<
        (
            SingleThreadedContext,
            DefaultRenderTarget<DefaultRGBBuffer, ()>,
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
        let default_framebuffer_ref = DefaultRenderTarget::new(context.id());

        Ok((context, default_framebuffer_ref))
    }
}

impl Options for ContextOptions<DefaultRGBABuffer, DefaultDepthBuffer> {
    type Output = Result<
        (
            SingleThreadedContext,
            DefaultRenderTarget<DefaultRGBABuffer, DefaultDepthBuffer>,
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
        let default_framebuffer_ref = DefaultRenderTarget::new(context.id());

        Ok((context, default_framebuffer_ref))
    }
}

impl Options for ContextOptions<DefaultRGBABuffer, DefaultStencilBuffer> {
    type Output = Result<
        (
            SingleThreadedContext,
            DefaultRenderTarget<DefaultRGBABuffer, DefaultStencilBuffer>,
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
        let default_framebuffer_ref = DefaultRenderTarget::new(context.id());

        Ok((context, default_framebuffer_ref))
    }
}

impl Options for ContextOptions<DefaultRGBBuffer, ()> {
    type Output = Result<
        (
            SingleThreadedContext,
            DefaultRenderTarget<DefaultRGBBuffer, ()>,
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
        let default_framebuffer_ref = DefaultRenderTarget::new(context.id());

        Ok((context, default_framebuffer_ref))
    }
}

impl Options for ContextOptions<DefaultRGBBuffer, DefaultDepthStencilBuffer> {
    type Output = Result<
        (
            SingleThreadedContext,
            DefaultRenderTarget<DefaultRGBBuffer, DefaultDepthStencilBuffer>,
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
        let default_framebuffer_ref = DefaultRenderTarget::new(context.id());

        Ok((context, default_framebuffer_ref))
    }
}

impl Options for ContextOptions<DefaultRGBBuffer, DefaultDepthBuffer> {
    type Output = Result<
        (
            SingleThreadedContext,
            DefaultRenderTarget<DefaultRGBBuffer, DefaultDepthBuffer>,
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
        let default_framebuffer_ref = DefaultRenderTarget::new(context.id());

        Ok((context, default_framebuffer_ref))
    }
}

impl Options for ContextOptions<DefaultRGBBuffer, DefaultStencilBuffer> {
    type Output = Result<
        (
            SingleThreadedContext,
            DefaultRenderTarget<DefaultRGBBuffer, DefaultStencilBuffer>,
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
        let default_framebuffer_ref = DefaultRenderTarget::new(context.id());

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
