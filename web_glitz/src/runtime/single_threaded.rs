use std::borrow::Borrow;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use serde_derive::Serialize;

use wasm_bindgen::{JsCast, JsValue};

use web_sys::{HtmlCanvasElement, WebGl2RenderingContext as Gl};

use crate::buffer::{Buffer, IntoBuffer, UsageHint};
use crate::image::format::{RenderbufferFormat, TextureFormat};
use crate::image::renderbuffer::Renderbuffer;
use crate::image::texture_2d::{Texture2D, Texture2DDescriptor};
use crate::image::texture_2d_array::{Texture2DArray, Texture2DArrayDescriptor};
use crate::image::texture_3d::{Texture3D, Texture3DDescriptor};
use crate::image::texture_cube::{TextureCube, TextureCubeDescriptor};
use crate::image::MaxMipmapLevelsExceeded;
use crate::pipeline::graphics::shader::{
    FragmentShaderAllocateCommand, VertexShaderAllocateCommand,
};
use crate::pipeline::graphics::vertex_input::{
    IndexBufferDescription, InputAttributeLayout, VertexArray, VertexArrayDescriptor,
    VertexBuffersDescription,
};
use crate::pipeline::graphics::{
    FragmentShader, GraphicsPipeline, GraphicsPipelineDescriptor, ShaderCompilationError,
    VertexShader,
};
use crate::pipeline::resources::Resources;
use crate::render_pass::{
    DefaultDepthBuffer, DefaultDepthStencilBuffer, DefaultRGBABuffer, DefaultRGBBuffer,
    DefaultRenderTarget, DefaultStencilBuffer, RenderPass, RenderPassContext,
    RenderTargetDescription,
};
use crate::runtime::executor_job::job;
use crate::runtime::fenced::JsTimeoutFencedTaskRunner;
use crate::runtime::rendering_context::{
    CreateGraphicsPipelineError, Extensions, TransformFeedbackVaryings,
};
use crate::runtime::state::DynamicState;
use crate::runtime::{Connection, ContextOptions, Execution, PowerPreference, RenderingContext};
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

#[derive(Clone)]
pub struct SingleThreadedContext {
    executor: Rc<RefCell<SingleThreadedExecutor>>,
    id: usize,
    extensions: Extensions,
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

    fn create_renderbuffer<F>(&self, width: u32, height: u32) -> Renderbuffer<F>
    where
        F: RenderbufferFormat + 'static,
    {
        Renderbuffer::new(self, width, height)
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

    fn create_graphics_pipeline<Il, R, Tf>(
        &self,
        descriptor: &GraphicsPipelineDescriptor<Il, R, Tf>,
    ) -> Result<GraphicsPipeline<Il, R, Tf>, CreateGraphicsPipelineError>
    where
        Il: InputAttributeLayout,
        R: Resources + 'static,
        Tf: TransformFeedbackVaryings,
    {
        let executor = self.executor.borrow_mut();
        let mut connection = executor.connection.borrow_mut();

        GraphicsPipeline::create(self, &mut connection, descriptor)
    }

    fn create_render_pass<R, F, T>(&self, render_target: R, f: F) -> RenderPass<T>
    where
        R: RenderTargetDescription,
        F: FnOnce(&mut R::Framebuffer) -> T,
        for<'a> T: GpuTask<RenderPassContext<'a>>,
    {
        let id = ID_GEN.with(|id_gen| id_gen.next());

        RenderPass::new(id, self, render_target, f)
    }

    fn create_sampler(&self, descriptor: &SamplerDescriptor) -> Sampler {
        Sampler::new(self, descriptor)
    }

    fn create_shadow_sampler(&self, descriptor: &ShadowSamplerDescriptor) -> ShadowSampler {
        ShadowSampler::new(self, descriptor)
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

    fn create_vertex_array<V, I>(
        &self,
        descriptor: &VertexArrayDescriptor<V, I>,
    ) -> VertexArray<V::Layout>
    where
        V: VertexBuffersDescription,
        I: IndexBufferDescription,
    {
        VertexArray::new(self, descriptor)
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
            extensions: Extensions::default(),
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

pub unsafe fn context<O>(canvas: &HtmlCanvasElement, options: &O) -> O::Output
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
