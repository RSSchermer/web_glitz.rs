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
use std::collections::{HashMap, VecDeque};
use std::hash::{Hash, Hasher};
use std::mem;
use std::ops::Deref;
use std::rc::Rc;

use fnv::FnvHasher;
use js_sys::{Int32Array, Promise};
use serde_derive::Serialize;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext as Gl};

use crate::buffer::{Buffer, BufferId, IntoBuffer, UsageHint};
use crate::extensions::Extension;
use crate::image::format::{
    InternalFormat, Multisamplable, Multisample, RenderbufferFormat, TextureFormat,
};
use crate::image::renderbuffer::{Renderbuffer, RenderbufferDescriptor};
use crate::image::sampler::{
    MagnificationFilter, MinificationFilter, Sampler, SamplerDescriptor, ShadowSampler,
    ShadowSamplerDescriptor,
};
use crate::image::texture_2d::{Texture2D, Texture2DDescriptor};
use crate::image::texture_2d_array::{Texture2DArray, Texture2DArrayDescriptor};
use crate::image::texture_3d::{Texture3D, Texture3DDescriptor};
use crate::image::texture_cube::{TextureCube, TextureCubeDescriptor};
use crate::image::MaxMipmapLevelsExceeded;
use crate::pipeline::graphics::shader::{
    FragmentShaderAllocateCommand, VertexShaderAllocateCommand,
};
use crate::pipeline::graphics::{
    FragmentShader, GraphicsPipeline, GraphicsPipelineDescriptor, IndexBuffer, IndexFormat,
    VertexShader,
};
use crate::pipeline::resources::{BindGroup, EncodeBindableResourceGroup};
use crate::rendering::{
    DefaultDepthBuffer, DefaultDepthStencilBuffer, DefaultMultisampleRenderTarget,
    DefaultRGBABuffer, DefaultRGBBuffer, DefaultRenderTarget, DefaultStencilBuffer,
    MultisampleRenderTarget, MultisampleRenderTargetDescriptor, RenderTarget,
    RenderTargetDescriptor,
};
use crate::runtime::executor_job::{job, ExecutorJob, JobState};
use crate::runtime::fenced::JsTimeoutFencedTaskRunner;
use crate::runtime::rendering_context::{
    CreateGraphicsPipelineError, MaxColorBuffersExceeded, UnsupportedSampleCount,
};
use crate::runtime::state::DynamicState;
use crate::runtime::{
    Connection, ContextOptions, Execution, PowerPreference, RenderingContext,
    ShaderCompilationError, SupportedSamples,
};
use crate::task::{GpuTask, Progress};
use wasm_bindgen::__rt::core::mem::MaybeUninit;

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
pub(crate) struct ObjectIdGen {
    context_id: u64,
    object_id: Rc<Cell<u64>>,
}

impl ObjectIdGen {
    fn new(context_id: u64) -> Self {
        ObjectIdGen {
            context_id,
            object_id: Rc::new(Cell::new(0)),
        }
    }

    pub(crate) fn next(&self) -> u64 {
        let id = self.object_id.get();

        self.object_id.set(id + 1);

        let mut hasher = FnvHasher::default();

        (self.context_id, id).hash(&mut hasher);

        hasher.finish()
    }
}

/// A handle to a single-threaded WebGlitz rendering context.
///
/// See the module documentation for [web_glitz::runtime::single_threaded] for details.
#[derive(Clone)]
pub struct SingleThreadedContext {
    executor: Rc<SingleThreadedExecutor>,
    id: u64,
    object_id_gen: ObjectIdGen,
    max_color_attachments: u8,
    supported_samples_cache: Rc<RefCell<HashMap<u32, SupportedSamples>>>,
}

impl RenderingContext for SingleThreadedContext {
    fn id(&self) -> u64 {
        self.id
    }

    fn get_extension<T>(&self) -> Option<T>
    where
        T: Extension,
    {
        let executor = self.executor.deref().borrow();
        let mut connection = executor.connection.deref().borrow_mut();

        Extension::try_init(&mut connection, self.id)
    }

    fn supported_samples<F>(&self, _format: F) -> SupportedSamples
    where
        F: InternalFormat + Multisamplable,
    {
        let mut cache = self.supported_samples_cache.borrow_mut();

        if let Some(supported_samples) = cache.get(&F::ID) {
            *supported_samples
        } else {
            let executor = self.executor.deref().borrow();
            let connection = executor.connection.deref().borrow();

            let (gl, _) = unsafe { connection.unpack() };

            let array: Int32Array = gl
                .get_internalformat_parameter(Gl::RENDERBUFFER, F::ID, Gl::SAMPLES)
                .unwrap()
                .into();

            let len = array.length();
            let mut supported_samples = SupportedSamples::NONE;

            for i in 0..len {
                let entry = array.get_index(i) as u8;

                if entry == 16 {
                    supported_samples |= SupportedSamples::SAMPLES_16;
                } else if entry == 8 {
                    supported_samples |= SupportedSamples::SAMPLES_8;
                } else if entry == 4 {
                    supported_samples |= SupportedSamples::SAMPLES_4;
                } else if entry == 2 {
                    supported_samples |= SupportedSamples::SAMPLES_2;
                }
            }

            cache.insert(F::ID, supported_samples);

            supported_samples
        }
    }

    fn create_bind_group<T>(&self, resources: T) -> BindGroup<T>
    where
        T: EncodeBindableResourceGroup,
    {
        let object_id = self.object_id_gen.next();

        BindGroup::new(object_id, self.id, resources)
    }

    fn create_buffer<D, T>(&self, data: D, usage_hint: UsageHint) -> Buffer<T>
    where
        D: IntoBuffer<T>,
        T: ?Sized,
    {
        let object_id = self.object_id_gen.next();
        let buffer_id = BufferId { object_id };

        data.into_buffer(self, buffer_id, usage_hint)
    }

    fn create_buffer_uninit<T>(&self, usage_hint: UsageHint) -> Buffer<MaybeUninit<T>>
    where
        T: 'static,
    {
        let object_id = self.object_id_gen.next();
        let buffer_id = BufferId { object_id };

        Buffer::create_uninit(self, buffer_id, usage_hint)
    }

    fn create_buffer_slice_uninit<T>(
        &self,
        len: usize,
        usage_hint: UsageHint,
    ) -> Buffer<[MaybeUninit<T>]>
    where
        T: 'static,
    {
        let object_id = self.object_id_gen.next();
        let buffer_id = BufferId { object_id };

        Buffer::create_slice_uninit(self, buffer_id, len, usage_hint)
    }

    fn create_index_buffer<D, T>(&self, data: D, usage_hint: UsageHint) -> IndexBuffer<T>
    where
        D: Borrow<[T]> + 'static,
        T: IndexFormat + 'static,
    {
        let object_id = self.object_id_gen.next();

        IndexBuffer::new(self, object_id, data, usage_hint)
    }

    fn create_index_buffer_uninit<T>(&self, len: usize, usage_hint: UsageHint) -> IndexBuffer<MaybeUninit<T>>
        where
            T: IndexFormat + 'static,
    {
        let object_id = self.object_id_gen.next();

        IndexBuffer::new_uninit(self, object_id, len, usage_hint)
    }

    fn create_renderbuffer<F>(&self, descriptor: &RenderbufferDescriptor<F>) -> Renderbuffer<F>
    where
        F: RenderbufferFormat + 'static,
    {
        let object_id = self.object_id_gen.next();

        Renderbuffer::new(self, object_id, descriptor)
    }

    fn try_create_multisample_renderbuffer<F>(
        &self,
        descriptor: &RenderbufferDescriptor<Multisample<F>>,
    ) -> Result<Renderbuffer<Multisample<F>>, UnsupportedSampleCount>
    where
        F: RenderbufferFormat + Multisamplable + Copy + 'static,
    {
        let object_id = self.object_id_gen.next();

        Renderbuffer::new_multisample(self, object_id, descriptor)
    }

    fn try_create_vertex_shader<S>(&self, source: S) -> Result<VertexShader, ShaderCompilationError>
    where
        S: Borrow<str> + 'static,
    {
        let object_id = self.object_id_gen.next();
        let allocate_command = VertexShaderAllocateCommand::new(self, object_id, source);

        match self.submit(allocate_command) {
            Execution::Ready(res) => res.unwrap(),
            _ => unreachable!(),
        }
    }

    fn try_create_fragment_shader<S>(
        &self,
        source: S,
    ) -> Result<FragmentShader, ShaderCompilationError>
    where
        S: Borrow<str> + 'static,
    {
        let object_id = self.object_id_gen.next();
        let allocate_command = FragmentShaderAllocateCommand::new(self, object_id, source);

        match self.submit(allocate_command) {
            Execution::Ready(res) => res.unwrap(),
            _ => unreachable!(),
        }
    }

    fn try_create_graphics_pipeline<V, R, Tf>(
        &self,
        descriptor: &GraphicsPipelineDescriptor<V, R, Tf>,
    ) -> Result<GraphicsPipeline<V, R, Tf>, CreateGraphicsPipelineError> {
        let mut connection = self.executor.connection.borrow_mut();
        let object_id = self.object_id_gen.next();

        GraphicsPipeline::create(self, object_id, &mut connection, descriptor)
    }

    fn create_render_target<C, Ds>(
        &self,
        descriptor: RenderTargetDescriptor<(C,), Ds>,
    ) -> RenderTarget<(C,), Ds> {
        let RenderTargetDescriptor {
            color_attachments,
            depth_stencil_attachment,
            context_id,
            ..
        } = descriptor;
        let object_id = self.object_id_gen.next();

        context_id.verify(self.id);

        RenderTarget {
            color_attachments,
            depth_stencil_attachment,
            object_id,
            context_id: self.id,
            render_pass_id_gen: self.object_id_gen.clone(),
        }
    }

    fn try_create_render_target<C, Ds>(
        &self,
        descriptor: RenderTargetDescriptor<C, Ds>,
    ) -> Result<RenderTarget<C, Ds>, MaxColorBuffersExceeded> {
        let RenderTargetDescriptor {
            color_attachments,
            depth_stencil_attachment,
            color_attachment_count,
            context_id,
        } = descriptor;

        context_id.verify(self.id);

        if color_attachment_count > self.max_color_attachments {
            Err(MaxColorBuffersExceeded {
                max_supported_color_buffers: self.max_color_attachments,
                requested_color_buffers: color_attachment_count,
            })
        } else {
            let object_id = self.object_id_gen.next();

            Ok(RenderTarget {
                color_attachments,
                depth_stencil_attachment,
                object_id,
                context_id: self.id,
                render_pass_id_gen: self.object_id_gen.clone(),
            })
        }
    }

    fn create_multisample_render_target<C, Ds>(
        &self,
        descriptor: MultisampleRenderTargetDescriptor<(C,), Ds>,
    ) -> MultisampleRenderTarget<(C,), Ds> {
        let MultisampleRenderTargetDescriptor {
            color_attachments,
            depth_stencil_attachment,
            samples,
            context_id,
            ..
        } = descriptor;
        let object_id = self.object_id_gen.next();

        context_id.verify(self.id);

        MultisampleRenderTarget {
            color_attachments,
            depth_stencil_attachment,
            samples,
            object_id,
            context_id: self.id,
            render_pass_id_gen: self.object_id_gen.clone(),
        }
    }

    fn try_create_multisample_render_target<C, Ds>(
        &self,
        descriptor: MultisampleRenderTargetDescriptor<C, Ds>,
    ) -> Result<MultisampleRenderTarget<C, Ds>, MaxColorBuffersExceeded> {
        let MultisampleRenderTargetDescriptor {
            color_attachments,
            depth_stencil_attachment,
            samples,
            color_attachment_count,
            context_id,
        } = descriptor;

        context_id.verify(self.id);

        if color_attachment_count > self.max_color_attachments {
            Err(MaxColorBuffersExceeded {
                max_supported_color_buffers: self.max_color_attachments,
                requested_color_buffers: color_attachment_count,
            })
        } else {
            let object_id = self.object_id_gen.next();

            Ok(MultisampleRenderTarget {
                color_attachments,
                depth_stencil_attachment,
                samples,
                object_id,
                context_id: self.id,
                render_pass_id_gen: self.object_id_gen.clone(),
            })
        }
    }

    fn try_create_texture_2d<F>(
        &self,
        descriptor: &Texture2DDescriptor<F>,
    ) -> Result<Texture2D<F>, MaxMipmapLevelsExceeded>
    where
        F: TextureFormat + 'static,
    {
        let object_id = self.object_id_gen.next();

        Texture2D::new(self, object_id, descriptor)
    }

    fn try_create_texture_2d_array<F>(
        &self,
        descriptor: &Texture2DArrayDescriptor<F>,
    ) -> Result<Texture2DArray<F>, MaxMipmapLevelsExceeded>
    where
        F: TextureFormat + 'static,
    {
        let object_id = self.object_id_gen.next();

        Texture2DArray::new(self, object_id, descriptor)
    }

    fn try_create_texture_3d<F>(
        &self,
        descriptor: &Texture3DDescriptor<F>,
    ) -> Result<Texture3D<F>, MaxMipmapLevelsExceeded>
    where
        F: TextureFormat + 'static,
    {
        let object_id = self.object_id_gen.next();

        Texture3D::new(self, object_id, descriptor)
    }

    fn try_create_texture_cube<F>(
        &self,
        descriptor: &TextureCubeDescriptor<F>,
    ) -> Result<TextureCube<F>, MaxMipmapLevelsExceeded>
    where
        F: TextureFormat + 'static,
    {
        let object_id = self.object_id_gen.next();

        TextureCube::new(self, object_id, descriptor)
    }

    fn create_sampler<Min, Mag>(
        &self,
        descriptor: &SamplerDescriptor<Min, Mag>,
    ) -> Sampler<Min, Mag>
    where
        Min: MinificationFilter + Copy + 'static,
        Mag: MagnificationFilter + Copy + 'static,
    {
        let object_id = self.object_id_gen.next();

        Sampler::new(self, object_id, descriptor)
    }

    fn create_shadow_sampler(&self, descriptor: &ShadowSamplerDescriptor) -> ShadowSampler {
        let object_id = self.object_id_gen.next();

        ShadowSampler::new(self, object_id, descriptor)
    }

    fn submit<T>(&self, task: T) -> Execution<T::Output>
    where
        T: GpuTask<Connection> + 'static,
    {
        self.executor.accept(task)
    }
}

impl SingleThreadedContext {
    pub unsafe fn from_webgl2_context(gl: Gl, state: DynamicState) -> Self {
        let id = ID_GEN.with(|id_gen| id_gen.next());

        let mut hasher = FnvHasher::default();

        (TypeId::of::<Self>(), id).hash(&mut hasher);

        let id = hasher.finish();
        let max_color_attachments = gl
            .get_parameter(Gl::MAX_COLOR_ATTACHMENTS)
            .unwrap()
            .as_f64()
            .unwrap() as u8;

        SingleThreadedContext {
            executor: SingleThreadedExecutor::new(Connection::new(id, gl, state)).into(),
            id,
            object_id_gen: ObjectIdGen::new(id),
            max_color_attachments,
            supported_samples_cache: Rc::new(RefCell::new(HashMap::new())),
        }
    }
}

struct SingleThreadedExecutor {
    connection: Rc<RefCell<Connection>>,
    fenced_task_queue_runner: Rc<RefCell<JsTimeoutFencedTaskRunner>>,
    buffer: Rc<RefCell<VecDeque<Box<dyn ExecutorJob>>>>,
    process_buffer_closure: Rc<RefCell<Option<Closure<dyn FnMut(JsValue)>>>>,
    process_buffer_promise: Promise,
}

impl SingleThreadedExecutor {
    fn new(connection: Connection) -> Self {
        let connection = Rc::new(RefCell::new(connection));
        let fenced_task_queue_runner = Rc::new(RefCell::new(JsTimeoutFencedTaskRunner::new(
            connection.clone(),
        )));
        let buffer: Rc<RefCell<VecDeque<Box<dyn ExecutorJob>>>> =
            Rc::new(RefCell::new(VecDeque::new()));

        // Initialize a closure that will process any buffered tasks in a micro-task. We'll have to
        // make sure the closure lives long enough for the JS callback to succeed. This is
        // potentially longer than the lifetime of the executor itself (the executor may already
        // have been dropped while a micro-task is still queued). We create 2 copies of an Rc, one
        // we give to the executor and one to the closure itself. Whenever the closure runs, check
        // the reference count. If the count has dropped to 1, then drop the closure. We add a Drop
        // implementation to the executor that schedules the closure callback one last time to
        // ensure that this check occurs.
        let rc = Rc::new(RefCell::new(None));
        let rc_clone = rc.clone();
        let connection_clone = connection.clone();
        let fenced_task_queue_runner_clone = fenced_task_queue_runner.clone();
        let buffer_clone = buffer.clone();

        let callback = Closure::wrap(Box::new(move |_| {
            while let Some(mut job) = buffer_clone.borrow_mut().pop_front() {
                if let JobState::ContinueFenced = job.progress(&mut connection_clone.borrow_mut()) {
                    fenced_task_queue_runner_clone.borrow_mut().schedule(job);
                }
            }

            if Rc::strong_count(&rc_clone) == 1 {
                // The executor was dropped, clean up after ourselves
                let callback = rc_clone.borrow_mut().take();

                mem::drop(callback);
            }
        }) as Box<dyn FnMut(JsValue)>);

        *rc.borrow_mut() = Some(callback);

        SingleThreadedExecutor {
            connection,
            fenced_task_queue_runner,
            buffer,
            process_buffer_closure: rc,
            process_buffer_promise: Promise::resolve(&JsValue::null()),
        }
    }

    fn accept<T>(&self, mut task: T) -> Execution<T::Output>
    where
        T: GpuTask<Connection> + 'static,
    {
        if let Ok(mut connection) = self.connection.try_borrow_mut() {
            let output = task.progress(&mut connection);

            // Explicitly drop the connection reference, otherwise it lives until the end of the
            // scope while the task queue runner may want to use it below, causing a panic.
            mem::drop(connection);

            match output {
                Progress::Finished(res) => res.into(),
                Progress::ContinueFenced => {
                    let (job, execution) = job(task);

                    self.fenced_task_queue_runner
                        .borrow_mut()
                        .schedule(Box::new(job));

                    execution
                }
            }
        } else {
            // We're already executing a task, probably means that this new task was submitted
            // during task progression. Jobify and buffer it in a queue so we can handle this task
            // after the current task is done.

            let (job, execution) = job(task);
            let mut buffer = self.buffer.borrow_mut();

            buffer.push_back(Box::new(job));

            // Only queue a new micro task if this if the first job to be buffered, otherwise a
            // micro task will have already been queued.
            if buffer.len() == 1 {
                let ref_cell: &RefCell<_> = self.process_buffer_closure.borrow();
                let callback_ref = ref_cell.borrow();

                let promise = self
                    .process_buffer_promise
                    .then(callback_ref.as_ref().unwrap());

                // Explicitly drop promise to get around "unused_must_use" warning; we really don't
                // need to use this promise...
                mem::drop(promise);
            }

            execution
        }
    }
}

impl Drop for SingleThreadedExecutor {
    fn drop(&mut self) {
        // Only schedule the callback if the buffer is empty, otherwise a callback is already
        // queued.
        if self.buffer.deref().borrow().len() == 0 {
            let ref_cell: &RefCell<_> = self.process_buffer_closure.borrow();
            let callback_ref = ref_cell.borrow();

            let promise = self
                .process_buffer_promise
                .then(callback_ref.as_ref().unwrap());

            // Explicitly drop promise to get around "unused_must_use" warning; we really don't need
            // to use this promise...
            mem::drop(promise);
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

impl Options for ContextOptions<DefaultMultisampleRenderTarget<DefaultRGBABuffer, ()>> {
    type Output = Result<
        (
            SingleThreadedContext,
            DefaultMultisampleRenderTarget<DefaultRGBABuffer, ()>,
        ),
        String,
    >;

    unsafe fn get_context(&self, canvas: &HtmlCanvasElement) -> Self::Output {
        let options = JsValue::from_serde(&OptionsJson {
            alpha: true,
            antialias: true,
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
        let samples = gl.get_parameter(Gl::SAMPLES).unwrap().as_f64().unwrap() as u8;
        let context = SingleThreadedContext::from_webgl2_context(gl, state);
        let render_target = DefaultMultisampleRenderTarget::new(
            context.id(),
            samples,
            context.object_id_gen.clone(),
        );

        Ok((context, render_target))
    }
}

impl Options
    for ContextOptions<DefaultMultisampleRenderTarget<DefaultRGBABuffer, DefaultDepthStencilBuffer>>
{
    type Output = Result<
        (
            SingleThreadedContext,
            DefaultMultisampleRenderTarget<DefaultRGBABuffer, DefaultDepthStencilBuffer>,
        ),
        String,
    >;

    unsafe fn get_context(&self, canvas: &HtmlCanvasElement) -> Self::Output {
        let options = JsValue::from_serde(&OptionsJson {
            alpha: true,
            antialias: true,
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
        let samples = gl.get_parameter(Gl::SAMPLES).unwrap().as_f64().unwrap() as u8;
        let context = SingleThreadedContext::from_webgl2_context(gl, state);
        let render_target = DefaultMultisampleRenderTarget::new(
            context.id(),
            samples,
            context.object_id_gen.clone(),
        );

        Ok((context, render_target))
    }
}

impl Options
    for ContextOptions<DefaultMultisampleRenderTarget<DefaultRGBABuffer, DefaultDepthBuffer>>
{
    type Output = Result<
        (
            SingleThreadedContext,
            DefaultMultisampleRenderTarget<DefaultRGBABuffer, DefaultDepthBuffer>,
        ),
        String,
    >;

    unsafe fn get_context(&self, canvas: &HtmlCanvasElement) -> Self::Output {
        let options = JsValue::from_serde(&OptionsJson {
            alpha: true,
            antialias: true,
            depth: true,
            fail_if_major_performance_caveat: self.fail_if_major_performance_caveat(),
            power_preference: self.power_preference(),
            premultiplied_alpha: self.premultiplied_alpha(),
            preserve_drawing_buffer: self.preserve_drawing_buffer(),
            stencil: false,
        })
        .unwrap();

        let gl: web_sys::WebGl2RenderingContext = canvas
            .get_context_with_context_options("webgl2", &options)
            .map_err(|e| e.as_string().unwrap())?
            .unwrap()
            .unchecked_into();

        let state = DynamicState::initial(&gl);
        let samples = gl.get_parameter(Gl::SAMPLES).unwrap().as_f64().unwrap() as u8;
        let context = SingleThreadedContext::from_webgl2_context(gl, state);
        let render_target = DefaultMultisampleRenderTarget::new(
            context.id(),
            samples,
            context.object_id_gen.clone(),
        );

        Ok((context, render_target))
    }
}

impl Options
    for ContextOptions<DefaultMultisampleRenderTarget<DefaultRGBABuffer, DefaultStencilBuffer>>
{
    type Output = Result<
        (
            SingleThreadedContext,
            DefaultMultisampleRenderTarget<DefaultRGBABuffer, DefaultStencilBuffer>,
        ),
        String,
    >;

    unsafe fn get_context(&self, canvas: &HtmlCanvasElement) -> Self::Output {
        let options = JsValue::from_serde(&OptionsJson {
            alpha: true,
            antialias: true,
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
        let samples = gl.get_parameter(Gl::SAMPLES).unwrap().as_f64().unwrap() as u8;
        let context = SingleThreadedContext::from_webgl2_context(gl, state);
        let render_target = DefaultMultisampleRenderTarget::new(
            context.id(),
            samples,
            context.object_id_gen.clone(),
        );

        Ok((context, render_target))
    }
}

impl Options for ContextOptions<DefaultMultisampleRenderTarget<DefaultRGBBuffer, ()>> {
    type Output = Result<
        (
            SingleThreadedContext,
            DefaultMultisampleRenderTarget<DefaultRGBBuffer, ()>,
        ),
        String,
    >;

    unsafe fn get_context(&self, canvas: &HtmlCanvasElement) -> Self::Output {
        let options = JsValue::from_serde(&OptionsJson {
            alpha: false,
            antialias: true,
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
        let samples = gl.get_parameter(Gl::SAMPLES).unwrap().as_f64().unwrap() as u8;
        let context = SingleThreadedContext::from_webgl2_context(gl, state);
        let render_target = DefaultMultisampleRenderTarget::new(
            context.id(),
            samples,
            context.object_id_gen.clone(),
        );

        Ok((context, render_target))
    }
}

impl Options
    for ContextOptions<DefaultMultisampleRenderTarget<DefaultRGBBuffer, DefaultDepthStencilBuffer>>
{
    type Output = Result<
        (
            SingleThreadedContext,
            DefaultMultisampleRenderTarget<DefaultRGBBuffer, DefaultDepthStencilBuffer>,
        ),
        String,
    >;

    unsafe fn get_context(&self, canvas: &HtmlCanvasElement) -> Self::Output {
        let options = JsValue::from_serde(&OptionsJson {
            alpha: false,
            antialias: true,
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
        let samples = gl.get_parameter(Gl::SAMPLES).unwrap().as_f64().unwrap() as u8;
        let context = SingleThreadedContext::from_webgl2_context(gl, state);
        let render_target = DefaultMultisampleRenderTarget::new(
            context.id(),
            samples,
            context.object_id_gen.clone(),
        );

        Ok((context, render_target))
    }
}

impl Options
    for ContextOptions<DefaultMultisampleRenderTarget<DefaultRGBBuffer, DefaultDepthBuffer>>
{
    type Output = Result<
        (
            SingleThreadedContext,
            DefaultMultisampleRenderTarget<DefaultRGBBuffer, DefaultDepthBuffer>,
        ),
        String,
    >;

    unsafe fn get_context(&self, canvas: &HtmlCanvasElement) -> Self::Output {
        let options = JsValue::from_serde(&OptionsJson {
            alpha: false,
            antialias: true,
            depth: true,
            fail_if_major_performance_caveat: self.fail_if_major_performance_caveat(),
            power_preference: self.power_preference(),
            premultiplied_alpha: self.premultiplied_alpha(),
            preserve_drawing_buffer: self.preserve_drawing_buffer(),
            stencil: false,
        })
        .unwrap();

        let gl: web_sys::WebGl2RenderingContext = canvas
            .get_context_with_context_options("webgl2", &options)
            .map_err(|e| e.as_string().unwrap())?
            .unwrap()
            .unchecked_into();

        let state = DynamicState::initial(&gl);
        let samples = gl.get_parameter(Gl::SAMPLES).unwrap().as_f64().unwrap() as u8;
        let context = SingleThreadedContext::from_webgl2_context(gl, state);
        let render_target = DefaultMultisampleRenderTarget::new(
            context.id(),
            samples,
            context.object_id_gen.clone(),
        );

        Ok((context, render_target))
    }
}

impl Options
    for ContextOptions<DefaultMultisampleRenderTarget<DefaultRGBBuffer, DefaultStencilBuffer>>
{
    type Output = Result<
        (
            SingleThreadedContext,
            DefaultMultisampleRenderTarget<DefaultRGBBuffer, DefaultStencilBuffer>,
        ),
        String,
    >;

    unsafe fn get_context(&self, canvas: &HtmlCanvasElement) -> Self::Output {
        let options = JsValue::from_serde(&OptionsJson {
            alpha: false,
            antialias: true,
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
        let samples = gl.get_parameter(Gl::SAMPLES).unwrap().as_f64().unwrap() as u8;
        let context = SingleThreadedContext::from_webgl2_context(gl, state);
        let render_target = DefaultMultisampleRenderTarget::new(
            context.id(),
            samples,
            context.object_id_gen.clone(),
        );

        Ok((context, render_target))
    }
}

impl Options for ContextOptions<DefaultRenderTarget<DefaultRGBABuffer, ()>> {
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
            antialias: false,
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
        let render_target = DefaultRenderTarget::new(context.id(), context.object_id_gen.clone());

        Ok((context, render_target))
    }
}

impl Options for ContextOptions<DefaultRenderTarget<DefaultRGBABuffer, DefaultDepthStencilBuffer>> {
    type Output = Result<
        (
            SingleThreadedContext,
            DefaultRenderTarget<DefaultRGBABuffer, DefaultDepthStencilBuffer>,
        ),
        String,
    >;

    unsafe fn get_context(&self, canvas: &HtmlCanvasElement) -> Self::Output {
        let options = JsValue::from_serde(&OptionsJson {
            alpha: true,
            antialias: false,
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
        let render_target = DefaultRenderTarget::new(context.id(), context.object_id_gen.clone());

        Ok((context, render_target))
    }
}

impl Options for ContextOptions<DefaultRenderTarget<DefaultRGBABuffer, DefaultDepthBuffer>> {
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
            antialias: false,
            depth: true,
            fail_if_major_performance_caveat: self.fail_if_major_performance_caveat(),
            power_preference: self.power_preference(),
            premultiplied_alpha: self.premultiplied_alpha(),
            preserve_drawing_buffer: self.preserve_drawing_buffer(),
            stencil: false,
        })
        .unwrap();

        let gl: web_sys::WebGl2RenderingContext = canvas
            .get_context_with_context_options("webgl2", &options)
            .map_err(|e| e.as_string().unwrap())?
            .unwrap()
            .unchecked_into();

        let state = DynamicState::initial(&gl);
        let context = SingleThreadedContext::from_webgl2_context(gl, state);
        let render_target = DefaultRenderTarget::new(context.id(), context.object_id_gen.clone());

        Ok((context, render_target))
    }
}

impl Options for ContextOptions<DefaultRenderTarget<DefaultRGBABuffer, DefaultStencilBuffer>> {
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
            antialias: false,
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
        let render_target = DefaultRenderTarget::new(context.id(), context.object_id_gen.clone());

        Ok((context, render_target))
    }
}

impl Options for ContextOptions<DefaultRenderTarget<DefaultRGBBuffer, ()>> {
    type Output = Result<
        (
            SingleThreadedContext,
            DefaultRenderTarget<DefaultRGBBuffer, ()>,
        ),
        String,
    >;

    unsafe fn get_context(&self, canvas: &HtmlCanvasElement) -> Self::Output {
        let options = JsValue::from_serde(&OptionsJson {
            alpha: false,
            antialias: false,
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
        let render_target = DefaultRenderTarget::new(context.id(), context.object_id_gen.clone());

        Ok((context, render_target))
    }
}

impl Options for ContextOptions<DefaultRenderTarget<DefaultRGBBuffer, DefaultDepthStencilBuffer>> {
    type Output = Result<
        (
            SingleThreadedContext,
            DefaultRenderTarget<DefaultRGBBuffer, DefaultDepthStencilBuffer>,
        ),
        String,
    >;

    unsafe fn get_context(&self, canvas: &HtmlCanvasElement) -> Self::Output {
        let options = JsValue::from_serde(&OptionsJson {
            alpha: false,
            antialias: false,
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
        let render_target = DefaultRenderTarget::new(context.id(), context.object_id_gen.clone());

        Ok((context, render_target))
    }
}

impl Options for ContextOptions<DefaultRenderTarget<DefaultRGBBuffer, DefaultDepthBuffer>> {
    type Output = Result<
        (
            SingleThreadedContext,
            DefaultRenderTarget<DefaultRGBBuffer, DefaultDepthBuffer>,
        ),
        String,
    >;

    unsafe fn get_context(&self, canvas: &HtmlCanvasElement) -> Self::Output {
        let options = JsValue::from_serde(&OptionsJson {
            alpha: false,
            antialias: false,
            depth: true,
            fail_if_major_performance_caveat: self.fail_if_major_performance_caveat(),
            power_preference: self.power_preference(),
            premultiplied_alpha: self.premultiplied_alpha(),
            preserve_drawing_buffer: self.preserve_drawing_buffer(),
            stencil: false,
        })
        .unwrap();

        let gl: web_sys::WebGl2RenderingContext = canvas
            .get_context_with_context_options("webgl2", &options)
            .map_err(|e| e.as_string().unwrap())?
            .unwrap()
            .unchecked_into();

        let state = DynamicState::initial(&gl);
        let context = SingleThreadedContext::from_webgl2_context(gl, state);
        let render_target = DefaultRenderTarget::new(context.id(), context.object_id_gen.clone());

        Ok((context, render_target))
    }
}

impl Options for ContextOptions<DefaultRenderTarget<DefaultRGBBuffer, DefaultStencilBuffer>> {
    type Output = Result<
        (
            SingleThreadedContext,
            DefaultRenderTarget<DefaultRGBBuffer, DefaultStencilBuffer>,
        ),
        String,
    >;

    unsafe fn get_context(&self, canvas: &HtmlCanvasElement) -> Self::Output {
        let options = JsValue::from_serde(&OptionsJson {
            alpha: false,
            antialias: false,
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
        let render_target = DefaultRenderTarget::new(context.id(), context.object_id_gen.clone());

        Ok((context, render_target))
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
