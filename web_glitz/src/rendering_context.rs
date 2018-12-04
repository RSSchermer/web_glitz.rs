use std::borrow::Borrow;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

use futures::future::Future;
use futures::sync::oneshot::{channel, Canceled, Receiver, Sender};
use futures::{Async, Poll};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{
    window, WebGl2RenderingContext as GL, WebGlBuffer, WebGlFramebuffer, WebGlProgram,
    WebGlRenderbuffer, WebGlSampler, WebGlSync, WebGlTexture, WebGlVertexArrayObject,
};

use super::buffer::{BufferHandle, BufferUsage};
use super::task::{GpuTask, Progress};
use framebuffer::FramebufferDescriptor;
use framebuffer::FramebufferHandle;

const TEXTURE_UNIT_CONSTANTS: [u32; 32] = [
    GL::TEXTURE0,
    GL::TEXTURE1,
    GL::TEXTURE2,
    GL::TEXTURE3,
    GL::TEXTURE4,
    GL::TEXTURE5,
    GL::TEXTURE6,
    GL::TEXTURE7,
    GL::TEXTURE8,
    GL::TEXTURE9,
    GL::TEXTURE10,
    GL::TEXTURE11,
    GL::TEXTURE12,
    GL::TEXTURE13,
    GL::TEXTURE14,
    GL::TEXTURE15,
    GL::TEXTURE16,
    GL::TEXTURE17,
    GL::TEXTURE18,
    GL::TEXTURE19,
    GL::TEXTURE20,
    GL::TEXTURE21,
    GL::TEXTURE22,
    GL::TEXTURE23,
    GL::TEXTURE24,
    GL::TEXTURE25,
    GL::TEXTURE26,
    GL::TEXTURE27,
    GL::TEXTURE28,
    GL::TEXTURE29,
    GL::TEXTURE30,
    GL::TEXTURE31,
];

pub trait Submitter: Clone {
    fn accept<T>(&self, task: T) -> Execution<T::Output, T::Error>
    where
        T: GpuTask<Connection> + 'static;
}

pub enum Execution<O, E> {
    Ready(Option<Result<O, E>>),
    Pending(Receiver<Result<O, E>>),
}

impl<O, E> Future for Execution<O, E> {
    type Item = O;

    type Error = SubmitError<E>;

    fn poll(&mut self) -> Poll<O, SubmitError<E>> {
        match self {
            Execution::Ready(ref mut res) => res
                .take()
                .expect("Cannot poll Execution more than once after its ready")
                .map(|o| Async::Ready(o))
                .map_err(|e| SubmitError::TaskError(e)),
            Execution::Pending(ref mut recv) => match recv.poll() {
                Ok(Async::Ready(Ok(output))) => Ok(Async::Ready(output)),
                Ok(Async::Ready(Err(err))) => Err(SubmitError::TaskError(err)),
                Ok(Async::NotReady) => Ok(Async::NotReady),
                Err(Canceled) => Err(SubmitError::Cancelled),
            },
        }
    }
}

impl<O, E> From<Result<O, E>> for Execution<O, E> {
    fn from(result: Result<O, E>) -> Self {
        Execution::Ready(Some(result))
    }
}

impl<O, E> From<Receiver<Result<O, E>>> for Execution<O, E> {
    fn from(recv: Receiver<Result<O, E>>) -> Self {
        Execution::Pending(recv)
    }
}

trait ExecutorJob {
    fn progress(&mut self, connection: &mut Connection) -> JobState;
}

#[derive(PartialEq)]
enum JobState {
    Finished,
    ContinueFenced,
}

struct Job<T>
where
    T: GpuTask<Connection>,
{
    task: T,
    result_tx: Option<Sender<Result<T::Output, T::Error>>>,
}

impl<T> ExecutorJob for Job<T>
where
    T: GpuTask<Connection>,
{
    fn progress(&mut self, connection: &mut Connection) -> JobState {
        match self.task.progress(connection) {
            Progress::Finished(res) => {
                self.result_tx
                    .take()
                    .expect("Cannot progress a Job after it finished")
                    .send(res)
                    .map_err(|_| SendFailed)
                    .unwrap();

                JobState::Finished
            }
            Progress::ContinueFenced => JobState::ContinueFenced,
        }
    }
}

#[derive(Debug)]
struct SendFailed;

fn job<T>(task: T) -> (Job<T>, Execution<T::Output, T::Error>)
where
    T: GpuTask<Connection>,
{
    let (tx, rx) = channel();
    let job = Job {
        task,
        result_tx: Some(tx),
    };

    (job, Execution::Pending(rx))
}

#[derive(Clone)]
pub struct SingleThreadedSubmitter {
    executor: Rc<RefCell<SingleThreadedExecutor>>,
}

impl SingleThreadedSubmitter {
    pub fn new(connection: Connection) -> Self {
        SingleThreadedSubmitter {
            executor: RefCell::new(SingleThreadedExecutor::new(connection)).into(),
        }
    }
}

impl Submitter for SingleThreadedSubmitter {
    fn accept<T>(&self, task: T) -> Execution<T::Output, T::Error>
    where
        T: GpuTask<Connection> + 'static,
    {
        self.executor.borrow_mut().accept(task)
    }
}

struct SingleThreadedExecutor {
    connection: Rc<RefCell<Connection>>,
    fenced_task_queue: Rc<RefCell<FencedTaskQueue>>,
    animation_frame_handle: i32,
}

impl SingleThreadedExecutor {
    fn new(connection: Connection) -> Self {
        let connection = Rc::new(RefCell::new(connection));
        let connection_clone = connection.clone();
        let fenced_task_queue = Rc::new(RefCell::new(FencedTaskQueue::new()));
        let fenced_task_queue_clone = fenced_task_queue.clone();

        let closure: Closure<FnMut()> = Closure::wrap(Box::new(move || {
            fenced_task_queue_clone
                .borrow_mut()
                .run(&mut connection_clone.borrow_mut());
        }) as Box<FnMut()>);

        let animation_frame_handle = window()
            .unwrap()
            .request_animation_frame(closure.as_ref().unchecked_ref())
            .unwrap();

        closure.forget();

        SingleThreadedExecutor {
            connection,
            fenced_task_queue,
            animation_frame_handle,
        }
    }

    fn accept<T>(&mut self, mut task: T) -> Execution<T::Output, T::Error>
    where
        T: GpuTask<Connection> + 'static,
    {
        match task.progress(&mut self.connection.borrow_mut()) {
            Progress::Finished(res) => res.into(),
            Progress::ContinueFenced => {
                let (job, execution) = job(task);

                self.fenced_task_queue
                    .borrow_mut()
                    .push(job, &mut self.connection.borrow_mut());

                execution
            }
        }
    }
}

impl Drop for SingleThreadedExecutor {
    fn drop(&mut self) {
        window()
            .unwrap()
            .cancel_animation_frame(self.animation_frame_handle)
            .unwrap();
    }
}

struct FencedTaskQueue {
    queue: VecDeque<(WebGlSync, Box<ExecutorJob>)>,
}

impl FencedTaskQueue {
    fn new() -> Self {
        FencedTaskQueue {
            queue: VecDeque::new(),
        }
    }

    fn push<T>(&mut self, job: T, connection: &mut Connection)
    where
        T: ExecutorJob + 'static,
    {
        let fence = connection
            .0
            .fence_sync(GL::SYNC_GPU_COMMANDS_COMPLETE, 0)
            .unwrap();

        self.queue.push_back((fence, Box::new(job)));
    }

    fn run(&mut self, connection: &mut Connection) {
        let Connection(gl, _) = connection;

        for _ in 0..self.queue.len() {
            if gl
                .get_sync_parameter(&self.queue[0].0, GL::SYNC_STATUS)
                .as_f64()
                .unwrap() as u32
                == GL::SIGNALED
            {
                let (_, job) = self.queue.pop_front().unwrap();

                let fence = gl.fence_sync(GL::SYNC_GPU_COMMANDS_COMPLETE, 0).unwrap();

                self.queue.push_back((fence, job));
            } else {
                break;
            }
        }
    }
}

pub struct Connection(pub GL, pub DynamicState);

pub trait RenderingContext: Clone {
    fn create_value_buffer<T>(&self, usage_hint: BufferUsage) -> BufferHandle<T, Self>;

    fn create_array_buffer<T>(
        &self,
        len: usize,
        usage_hint: BufferUsage,
    ) -> BufferHandle<[T], Self>;

    fn create_framebuffer<D>(&self, descriptor: &D) -> FramebufferHandle<Self>
    where
        D: FramebufferDescriptor<Self>;

    fn submit<T>(&self, task: T) -> Execution<T::Output, T::Error>
    where
        T: GpuTask<Connection> + 'static;
}

#[derive(Clone)]
pub struct SingleThreadedContext {
    submitter: SingleThreadedSubmitter,
}

impl RenderingContext for SingleThreadedContext {
    fn create_value_buffer<T>(&self, usage_hint: BufferUsage) -> BufferHandle<T, Self> {
        BufferHandle::value(self.clone(), usage_hint)
    }

    fn create_array_buffer<T>(
        &self,
        len: usize,
        usage_hint: BufferUsage,
    ) -> BufferHandle<[T], Self> {
        BufferHandle::array(self.clone(), len, usage_hint)
    }

    fn create_framebuffer<D>(&self, descriptor: &D) -> FramebufferHandle<Self>
    where
        D: FramebufferDescriptor<Self>,
    {
        FramebufferHandle::new(self, descriptor)
    }

    fn submit<T>(&self, task: T) -> Execution<T::Output, T::Error>
    where
        T: GpuTask<Connection> + 'static,
    {
        self.submitter.accept(task)
    }
}

impl SingleThreadedContext {
    pub fn from_webgl2_context(gl: GL, state: DynamicState) -> Self {
        SingleThreadedContext {
            submitter: SingleThreadedSubmitter::new(Connection(gl, state)),
        }
    }
}

pub enum SubmitError<E> {
    Cancelled,
    TaskError(E),
}

impl<E> From<Canceled> for SubmitError<E> {
    fn from(_: Canceled) -> Self {
        SubmitError::Cancelled
    }
}

pub struct DynamicState {
    active_program: Option<WebGlProgram>,
    bound_array_buffer: Option<WebGlBuffer>,
    bound_element_array_buffer: Option<WebGlBuffer>,
    bound_copy_read_buffer: Option<WebGlBuffer>,
    bound_copy_write_buffer: Option<WebGlBuffer>,
    bound_pixel_pack_buffer: Option<WebGlBuffer>,
    bound_pixel_unpack_buffer: Option<WebGlBuffer>,
    bound_transform_feedback_buffers: Vec<BufferRange<WebGlBuffer>>,
    bound_uniform_buffers: Vec<BufferRange<WebGlBuffer>>,
    bound_draw_framebuffer: Option<WebGlFramebuffer>,
    bound_read_framebuffer: Option<WebGlFramebuffer>,
    bound_renderbuffer: Option<WebGlRenderbuffer>,
    bound_texture_2d: Option<WebGlTexture>,
    bound_texture_cube_map: Option<WebGlTexture>,
    bound_texture_3d: Option<WebGlTexture>,
    bound_texture_2d_array: Option<WebGlTexture>,
    bound_samplers: Vec<Option<WebGlSampler>>,
    texture_units_lru: TextureUnitLRU,
    texture_units_textures: Vec<Option<WebGlTexture>>,
    bound_vertex_array: Option<WebGlVertexArrayObject>,
    active_texture: u32,
    clear_color: [f32; 4],
    clear_depth: f32,
    clear_stencil: i32,
    depth_test_enabled: bool,
    stencil_test_enabled: bool,
    scissor_test_enabled: bool,
    blend_enabled: bool,
    cull_face_enabled: bool,
    dither_enabled: bool,
    polygon_offset_fill_enabled: bool,
    sample_aplha_to_coverage_enabled: bool,
    sample_coverage_enabled: bool,
    rasterizer_discard_enabled: bool,
    //    read_buffer: ReadBuffer,
    //    blend_color: [f32;4],
    //    blend_equation_rgb: BlendEquation,
    //    blend_equation_alpha: BlendEquation,
    //    blend_func_rgb: BlendFunc,
    //    blend_func_alpha: BlendFunc,
    //    color_mask: [bool;4],
    //    cull_face: CullFace,
    //    front_face: FrontFace,
    //    line_width: f32,
    //    pixel_pack_alignment: u32,
    pixel_unpack_alignment: i32,
    //    pixel_unpack_flip_y: bool,
    //    pixel_unpack_premultiply_alpha: bool,
    //    pixel_unpack_colorspace_conversion: ColorspaceConversion,
    //    pixel_pack_row_length: u32,
    //    pixel_pack_skip_pixels: u32,
    //    pixel_pack_skip_rows: u32,
    pixel_unpack_row_length: i32,
    pixel_unpack_image_height: i32,
    //    pixel_unpack_skip_pixels: u32,
    //    pixel_unpack_skip_rows: u32,
    //    pixel_unpack_skip_images: u32,
    //    sample_coverage: SampleCoverage,
    //    scissor: Region,
    //    viewport: Region,
    //    stencil_func_rgb: StencilFunc,
    //    stencil_func_alpha: StencilFunc,
    //    stencil_mask_rgb: u32,
    //    stencil_mask_alpha: u32,
    //    stencil_op_rgb: StencilOp,
    //    stencil_op_alpha: StencilOp,
}

impl DynamicState {
    pub fn active_program(&self) -> Option<&WebGlProgram> {
        self.active_program.as_ref()
    }

    pub fn set_active_program<'a>(
        &mut self,
        program: Option<&'a WebGlProgram>,
    ) -> impl ContextUpdate<'a, ()> {
        if !identical(program, self.active_program.as_ref()) {
            self.active_program = program.map(|p| p.clone());

            Some(move |context: &GL| {
                context.use_program(program);

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn bound_array_buffer(&self) -> Option<&WebGlBuffer> {
        self.bound_array_buffer.as_ref()
    }

    pub fn set_bound_array_buffer<'a>(
        &mut self,
        buffer: Option<&'a WebGlBuffer>,
    ) -> impl ContextUpdate<'a, ()> {
        if !identical(buffer, self.bound_array_buffer.as_ref()) {
            self.bound_array_buffer = buffer.map(|b| b.clone());

            Some(move |context: &GL| {
                context.bind_buffer(GL::ARRAY_BUFFER, buffer);

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn bound_element_array_buffer(&self) -> Option<&WebGlBuffer> {
        self.bound_element_array_buffer.as_ref()
    }

    pub fn set_bound_element_array_buffer<'a>(
        &mut self,
        buffer: Option<&'a WebGlBuffer>,
    ) -> impl ContextUpdate<'a, ()> {
        if !identical(buffer, self.bound_element_array_buffer.as_ref()) {
            self.bound_element_array_buffer = buffer.map(|b| b.clone());

            Some(move |context: &GL| {
                context.bind_buffer(GL::ELEMENT_ARRAY_BUFFER, buffer);

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn bound_copy_read_buffer(&self) -> Option<&WebGlBuffer> {
        self.bound_copy_read_buffer.as_ref()
    }

    pub fn set_bound_copy_read_buffer<'a>(
        &mut self,
        buffer: Option<&'a WebGlBuffer>,
    ) -> impl ContextUpdate<'a, ()> {
        if !identical(buffer, self.bound_copy_read_buffer.as_ref()) {
            self.bound_copy_read_buffer = buffer.map(|b| b.clone());

            Some(move |context: &GL| {
                context.bind_buffer(GL::COPY_READ_BUFFER, buffer);

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn bound_copy_write_buffer(&self) -> Option<&WebGlBuffer> {
        self.bound_copy_write_buffer.as_ref()
    }

    pub fn set_bound_copy_write_buffer<'a>(
        &mut self,
        buffer: Option<&'a WebGlBuffer>,
    ) -> impl ContextUpdate<'a, ()> {
        if !identical(buffer, self.bound_copy_write_buffer.as_ref()) {
            self.bound_copy_write_buffer = buffer.map(|b| b.clone());

            Some(move |context: &GL| {
                context.bind_buffer(GL::COPY_WRITE_BUFFER, buffer);

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn bound_pixel_pack_buffer(&self) -> Option<&WebGlBuffer> {
        self.bound_pixel_pack_buffer.as_ref()
    }

    pub fn set_bound_pixel_pack_buffer<'a>(
        &mut self,
        buffer: Option<&'a WebGlBuffer>,
    ) -> impl ContextUpdate<'a, ()> {
        if !identical(buffer, self.bound_pixel_pack_buffer.as_ref()) {
            self.bound_pixel_pack_buffer = buffer.map(|b| b.clone());

            Some(move |context: &GL| {
                context.bind_buffer(GL::PIXEL_PACK_BUFFER, buffer);

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn bound_pixel_unpack_buffer(&self) -> Option<&WebGlBuffer> {
        self.bound_pixel_unpack_buffer.as_ref()
    }

    pub fn set_bound_pixel_unpack_buffer<'a>(
        &mut self,
        buffer: Option<&'a WebGlBuffer>,
    ) -> impl ContextUpdate<'a, ()> {
        if !identical(buffer, self.bound_pixel_unpack_buffer.as_ref()) {
            self.bound_pixel_unpack_buffer = buffer.map(|b| b.clone());

            Some(move |context: &GL| {
                context.bind_buffer(GL::PIXEL_UNPACK_BUFFER, buffer);

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn bound_transform_feedback_buffer_range(&self, index: u32) -> BufferRange<&WebGlBuffer> {
        self.bound_transform_feedback_buffers[index as usize].as_ref()
    }

    pub fn set_bound_transform_feedback_buffer_range<'a>(
        &mut self,
        index: u32,
        buffer_range: BufferRange<&'a WebGlBuffer>,
    ) -> impl ContextUpdate<'a, ()> {
        if buffer_range != self.bound_transform_feedback_buffers[index as usize].as_ref() {
            self.bound_transform_feedback_buffers[index as usize] = buffer_range.to_owned_buffer();

            Some(move |context: &GL| {
                match buffer_range {
                    BufferRange::None => {
                        context.bind_buffer_base(GL::TRANSFORM_FEEDBACK_BUFFER, index, None)
                    }
                    BufferRange::Full(buffer) => {
                        context.bind_buffer_base(GL::TRANSFORM_FEEDBACK_BUFFER, index, Some(buffer))
                    }
                    BufferRange::OffsetSize(buffer, offset, size) => context
                        .bind_buffer_range_with_i32_and_i32(
                            GL::TRANSFORM_FEEDBACK_BUFFER,
                            index,
                            Some(buffer),
                            offset as i32,
                            size as i32,
                        ),
                };

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn bound_uniform_buffer_range(&self, index: u32) -> BufferRange<&WebGlBuffer> {
        self.bound_uniform_buffers[index as usize].as_ref()
    }

    pub fn set_bound_uniform_buffer_range<'a>(
        &mut self,
        index: u32,
        buffer_range: BufferRange<&'a WebGlBuffer>,
    ) -> impl ContextUpdate<'a, ()> {
        if buffer_range != self.bound_uniform_buffers[index as usize].as_ref() {
            self.bound_uniform_buffers[index as usize] = buffer_range.to_owned_buffer();

            Some(move |context: &GL| {
                match buffer_range {
                    BufferRange::None => context.bind_buffer_base(GL::UNIFORM_BUFFER, index, None),
                    BufferRange::Full(buffer) => {
                        context.bind_buffer_base(GL::UNIFORM_BUFFER, index, Some(buffer))
                    }
                    BufferRange::OffsetSize(buffer, offset, size) => context
                        .bind_buffer_range_with_i32_and_i32(
                            GL::UNIFORM_BUFFER,
                            index,
                            Some(buffer),
                            offset as i32,
                            size as i32,
                        ),
                };

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn bound_draw_framebuffer(&self) -> Option<&WebGlFramebuffer> {
        self.bound_draw_framebuffer.as_ref()
    }

    pub fn set_bound_draw_framebuffer<'a>(
        &mut self,
        framebuffer: Option<&'a WebGlFramebuffer>,
    ) -> impl ContextUpdate<'a, ()> {
        if !identical(framebuffer, self.bound_draw_framebuffer.as_ref()) {
            self.bound_draw_framebuffer = framebuffer.map(|f| f.clone());

            Some(move |context: &GL| {
                context.bind_framebuffer(GL::DRAW_FRAMEBUFFER, framebuffer);

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn bound_read_framebuffer(&self) -> Option<&WebGlFramebuffer> {
        self.bound_read_framebuffer.as_ref()
    }

    pub fn set_bound_read_framebuffer<'a>(
        &mut self,
        framebuffer: Option<&'a WebGlFramebuffer>,
    ) -> impl ContextUpdate<'a, ()> {
        if !identical(framebuffer, self.bound_read_framebuffer.as_ref()) {
            self.bound_read_framebuffer = framebuffer.map(|f| f.clone());

            Some(move |context: &GL| {
                context.bind_framebuffer(GL::READ_FRAMEBUFFER, framebuffer);

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn bound_renderbuffer(&self) -> Option<&WebGlRenderbuffer> {
        self.bound_renderbuffer.as_ref()
    }

    pub fn set_bound_renderbuffer<'a>(
        &mut self,
        renderbuffer: Option<&'a WebGlRenderbuffer>,
    ) -> impl ContextUpdate<'a, ()> {
        if !identical(renderbuffer, self.bound_renderbuffer.as_ref()) {
            self.bound_renderbuffer = renderbuffer.map(|r| r.clone());

            Some(move |context: &GL| {
                context.bind_renderbuffer(GL::RENDERBUFFER, renderbuffer);

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn bound_texture_2d(&self) -> Option<&WebGlTexture> {
        self.bound_texture_2d.as_ref()
    }

    pub fn set_bound_texture_2d<'a>(
        &mut self,
        texture: Option<&'a WebGlTexture>,
    ) -> impl ContextUpdate<'a, ()> {
        if !identical(texture, self.bound_texture_2d.as_ref()) {
            self.bound_texture_2d = texture.map(|t| t.clone());
            self.texture_units_textures[self.active_texture as usize] = texture.map(|t| t.clone());

            Some(move |context: &GL| {
                context.bind_texture(GL::TEXTURE_2D, texture);

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn bound_texture_2d_array(&self) -> Option<&WebGlTexture> {
        self.bound_texture_2d_array.as_ref()
    }

    pub fn set_bound_texture_2d_array<'a>(
        &mut self,
        texture: Option<&'a WebGlTexture>,
    ) -> impl ContextUpdate<'a, ()> {
        if !identical(texture, self.bound_texture_2d_array.as_ref()) {
            self.bound_texture_2d_array = texture.map(|t| t.clone());
            self.texture_units_textures[self.active_texture as usize] = texture.map(|t| t.clone());

            Some(move |context: &GL| {
                context.bind_texture(GL::TEXTURE_2D_ARRAY, texture);

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn bound_texture_3d(&self) -> Option<&WebGlTexture> {
        self.bound_texture_3d.as_ref()
    }

    pub fn set_bound_texture_3d<'a>(
        &mut self,
        texture: Option<&'a WebGlTexture>,
    ) -> impl ContextUpdate<'a, ()> {
        if !identical(texture, self.bound_texture_3d.as_ref()) {
            self.bound_texture_3d = texture.map(|t| t.clone());
            self.texture_units_textures[self.active_texture as usize] = texture.map(|t| t.clone());

            Some(move |context: &GL| {
                context.bind_texture(GL::TEXTURE_3D, texture);

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn bound_texture_cube_map(&self) -> Option<&WebGlTexture> {
        self.bound_texture_cube_map.as_ref()
    }

    pub fn set_bound_texture_cube_map<'a>(
        &mut self,
        texture: Option<&'a WebGlTexture>,
    ) -> impl ContextUpdate<'a, ()> {
        if !identical(texture, self.bound_texture_cube_map.as_ref()) {
            self.bound_texture_cube_map = texture.map(|t| t.clone());
            self.texture_units_textures[self.active_texture as usize] = texture.map(|t| t.clone());

            Some(move |context: &GL| {
                context.bind_texture(GL::TEXTURE_CUBE_MAP, texture);

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn texture_units_textures(&self) -> &[Option<WebGlTexture>] {
        &self.texture_units_textures
    }

    pub fn texture_units_textures_mut(&mut self) -> &mut [Option<WebGlTexture>] {
        &mut self.texture_units_textures
    }

    pub fn bound_sampler(&self, texture_unit: u32) -> Option<&WebGlSampler> {
        self.bound_samplers[texture_unit as usize].as_ref()
    }

    pub fn set_bound_sampler<'a>(
        &mut self,
        texture_unit: u32,
        sampler: Option<&'a WebGlSampler>,
    ) -> impl ContextUpdate<'a, ()> {
        if !identical(sampler, self.bound_samplers[texture_unit as usize].as_ref()) {
            self.bound_samplers[texture_unit as usize] = sampler.map(|v| v.clone());

            Some(move |context: &GL| {
                context.bind_sampler(texture_unit, sampler);

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn bound_vertex_array(&self) -> Option<&WebGlVertexArrayObject> {
        self.bound_vertex_array.as_ref()
    }

    pub fn set_bound_vertex_array<'a>(
        &mut self,
        vertex_array: Option<&'a WebGlVertexArrayObject>,
    ) -> impl ContextUpdate<'a, ()> {
        if !identical(vertex_array, self.bound_vertex_array.as_ref()) {
            self.bound_vertex_array = vertex_array.map(|v| v.clone());

            Some(move |context: &GL| {
                context.bind_vertex_array(vertex_array);

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn active_texture(&self) -> u32 {
        self.active_texture
    }

    pub fn set_active_texture(&mut self, texture_unit: u32) -> impl ContextUpdate<'static, ()> {
        if texture_unit != self.active_texture {
            self.active_texture = texture_unit;
            self.texture_units_lru.use_unit(texture_unit as usize);

            Some(move |context: &GL| {
                context.active_texture(TEXTURE_UNIT_CONSTANTS[texture_unit as usize]);

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn set_active_texture_lru(&mut self) -> impl ContextUpdate<'static, ()> {
        let texture_unit = self.texture_units_lru.use_lru_unit();
        self.active_texture = texture_unit as u32;

        Some(move |context: &GL| {
            context.active_texture(TEXTURE_UNIT_CONSTANTS[texture_unit]);

            Ok(())
        })
    }

    pub fn clear_color(&self) -> [f32; 4] {
        self.clear_color
    }

    pub fn set_clear_color(&mut self, color: [f32; 4]) -> impl ContextUpdate<'static, ()> {
        if color != self.clear_color {
            self.clear_color = color;

            Some(move |context: &GL| {
                context.clear_color(color[0], color[1], color[2], color[3]);

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn clear_depth(&self) -> f32 {
        self.clear_depth
    }

    pub fn set_clear_depth(&mut self, depth: f32) -> impl ContextUpdate<'static, ()> {
        if depth != self.clear_depth {
            self.clear_depth = depth;

            Some(move |context: &GL| {
                context.clear_depth(depth);

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn clear_stencil(&self) -> i32 {
        self.clear_stencil
    }

    pub fn set_clear_stencil(&mut self, stencil: i32) -> impl ContextUpdate<'static, ()> {
        if stencil != self.clear_stencil {
            self.clear_stencil = stencil;

            Some(move |context: &GL| {
                context.clear_stencil(stencil);

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn pixel_unpack_alignment(&self) -> i32 {
        self.pixel_unpack_alignment
    }

    pub fn set_pixel_unpack_alignment(
        &mut self,
        pixel_unpack_alignment: i32,
    ) -> impl ContextUpdate<'static, ()> {
        if pixel_unpack_alignment != self.pixel_unpack_alignment {
            self.pixel_unpack_alignment = pixel_unpack_alignment;

            Some(move |context: &GL| {
                context.pixel_storei(GL::UNPACK_ALIGNMENT, pixel_unpack_alignment);

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn pixel_unpack_row_length(&self) -> i32 {
        self.pixel_unpack_row_length
    }

    pub fn set_pixel_unpack_row_length(
        &mut self,
        pixel_unpack_row_length: i32,
    ) -> impl ContextUpdate<'static, ()> {
        if pixel_unpack_row_length != self.pixel_unpack_row_length {
            self.pixel_unpack_row_length = pixel_unpack_row_length;

            Some(move |context: &GL| {
                context.pixel_storei(GL::UNPACK_ROW_LENGTH, pixel_unpack_row_length);

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn pixel_unpack_image_height(&self) -> i32 {
        self.pixel_unpack_image_height
    }

    pub fn set_pixel_unpack_image_height(
        &mut self,
        pixel_unpack_image_height: i32,
    ) -> impl ContextUpdate<'static, ()> {
        if pixel_unpack_image_height != self.pixel_unpack_image_height {
            self.pixel_unpack_image_height = pixel_unpack_image_height;

            Some(move |context: &GL| {
                context.pixel_storei(GL::UNPACK_IMAGE_HEIGHT, pixel_unpack_image_height);

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn depth_test_enabled(&self) -> bool {
        self.depth_test_enabled
    }

    pub fn set_depth_test_enabled(
        &mut self,
        depth_test_enabled: bool,
    ) -> impl ContextUpdate<'static, ()> {
        if depth_test_enabled != self.depth_test_enabled {
            self.depth_test_enabled = depth_test_enabled;

            Some(move |context: &GL| {
                context.enable(GL::DEPTH_TEST);

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn stencil_test_enabled(&self) -> bool {
        self.stencil_test_enabled
    }

    pub fn set_stencil_test_enabled(
        &mut self,
        stencil_test_enabled: bool,
    ) -> impl ContextUpdate<'static, ()> {
        if stencil_test_enabled != self.stencil_test_enabled {
            self.stencil_test_enabled = stencil_test_enabled;

            Some(move |context: &GL| {
                context.enable(GL::STENCIL_TEST);

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn scissor_test_enabled(&self) -> bool {
        self.scissor_test_enabled
    }

    pub fn set_scissor_test_enabled(
        &mut self,
        scissor_test_enabled: bool,
    ) -> impl ContextUpdate<'static, ()> {
        if scissor_test_enabled != self.scissor_test_enabled {
            self.scissor_test_enabled = scissor_test_enabled;

            Some(move |context: &GL| {
                context.enable(GL::SCISSOR_TEST);

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn blend_enabled(&self) -> bool {
        self.blend_enabled
    }

    pub fn set_blend_enabled(&mut self, blend_enabled: bool) -> impl ContextUpdate<'static, ()> {
        if blend_enabled != self.blend_enabled {
            self.blend_enabled = blend_enabled;

            Some(move |context: &GL| {
                context.enable(GL::BLEND);

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn cull_face_enabled(&self) -> bool {
        self.cull_face_enabled
    }

    pub fn set_cull_face_enabled(
        &mut self,
        cull_face_enabled: bool,
    ) -> impl ContextUpdate<'static, ()> {
        if cull_face_enabled != self.cull_face_enabled {
            self.cull_face_enabled = cull_face_enabled;

            Some(move |context: &GL| {
                context.enable(GL::CULL_FACE);

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn dither_enabled(&self) -> bool {
        self.dither_enabled
    }

    pub fn set_dither_enabled(&mut self, dither_enabled: bool) -> impl ContextUpdate<'static, ()> {
        if dither_enabled != self.dither_enabled {
            self.dither_enabled = dither_enabled;

            Some(move |context: &GL| {
                context.enable(GL::DITHER);

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn polygon_offset_fill_enabled(&self) -> bool {
        self.polygon_offset_fill_enabled
    }

    pub fn set_polygon_offset_fill_enabled(
        &mut self,
        polygon_offset_fill_enabled: bool,
    ) -> impl ContextUpdate<'static, ()> {
        if polygon_offset_fill_enabled != self.polygon_offset_fill_enabled {
            self.polygon_offset_fill_enabled = polygon_offset_fill_enabled;

            Some(move |context: &GL| {
                context.enable(GL::POLYGON_OFFSET_FILL);

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn sample_aplha_to_coverage_enabled(&self) -> bool {
        self.sample_aplha_to_coverage_enabled
    }

    pub fn set_sample_aplha_to_coverage_enabled(
        &mut self,
        sample_aplha_to_coverage_enabled: bool,
    ) -> impl ContextUpdate<'static, ()> {
        if sample_aplha_to_coverage_enabled != self.sample_aplha_to_coverage_enabled {
            self.sample_aplha_to_coverage_enabled = sample_aplha_to_coverage_enabled;

            Some(move |context: &GL| {
                context.enable(GL::SAMPLE_ALPHA_TO_COVERAGE);

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn sample_coverage_enabled(&self) -> bool {
        self.sample_coverage_enabled
    }

    pub fn set_sample_coverage_enabled(
        &mut self,
        sample_coverage_enabled: bool,
    ) -> impl ContextUpdate<'static, ()> {
        if sample_coverage_enabled != self.sample_coverage_enabled {
            self.sample_coverage_enabled = sample_coverage_enabled;

            Some(move |context: &GL| {
                context.enable(GL::SAMPLE_COVERAGE);

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn rasterizer_discard_enabled(&self) -> bool {
        self.rasterizer_discard_enabled
    }

    pub fn set_rasterizer_discard_enabled(
        &mut self,
        rasterizer_discard_enabled: bool,
    ) -> impl ContextUpdate<'static, ()> {
        if rasterizer_discard_enabled != self.rasterizer_discard_enabled {
            self.rasterizer_discard_enabled = rasterizer_discard_enabled;

            Some(move |context: &GL| {
                context.enable(GL::RASTERIZER_DISCARD);

                Ok(())
            })
        } else {
            None
        }
    }
}

impl DynamicState {
    pub fn initial(context: &GL) -> Self {
        let max_combined_texture_image_units = context
            .get_parameter(GL::MAX_COMBINED_TEXTURE_IMAGE_UNITS)
            .unwrap()
            .as_f64()
            .unwrap() as usize;

        DynamicState {
            active_program: None,
            bound_array_buffer: None,
            bound_element_array_buffer: None,
            bound_copy_read_buffer: None,
            bound_copy_write_buffer: None,
            bound_pixel_pack_buffer: None,
            bound_pixel_unpack_buffer: None,
            bound_transform_feedback_buffers: vec![
                BufferRange::None;
                context
                    .get_parameter(GL::MAX_TRANSFORM_FEEDBACK_SEPARATE_ATTRIBS)
                    .unwrap()
                    .as_f64()
                    .unwrap() as usize
            ],
            bound_uniform_buffers: vec![
                BufferRange::None;
                context
                    .get_parameter(GL::MAX_UNIFORM_BUFFER_BINDINGS)
                    .unwrap()
                    .as_f64()
                    .unwrap() as usize
            ],
            bound_draw_framebuffer: None,
            bound_read_framebuffer: None,
            bound_renderbuffer: None,
            bound_texture_2d: None,
            bound_texture_cube_map: None,
            bound_texture_3d: None,
            bound_texture_2d_array: None,
            bound_samplers: vec![None; max_combined_texture_image_units],
            texture_units_lru: TextureUnitLRU::new(max_combined_texture_image_units),
            texture_units_textures: vec![None; max_combined_texture_image_units],
            bound_vertex_array: None,
            active_texture: 0,
            clear_color: [0.0, 0.0, 0.0, 0.0],
            clear_depth: 1.0,
            clear_stencil: 0,
            pixel_unpack_alignment: 4,
            pixel_unpack_row_length: 0,
            pixel_unpack_image_height: 0,
            depth_test_enabled: false,
            stencil_test_enabled: false,
            scissor_test_enabled: false,
            blend_enabled: false,
            cull_face_enabled: false,
            dither_enabled: true,
            polygon_offset_fill_enabled: false,
            sample_aplha_to_coverage_enabled: false,
            sample_coverage_enabled: false,
            rasterizer_discard_enabled: false,
        }
    }
}

fn identical<T>(a: Option<&T>, b: Option<&T>) -> bool
where
    T: AsRef<JsValue>,
{
    a.map(|t| t.as_ref()) == b.map(|t| t.as_ref())
}

pub trait ContextUpdate<'a, E> {
    fn apply(self, context: &GL) -> Result<(), E>;
}

impl<'a, F, E> ContextUpdate<'a, E> for Option<F>
where
    F: FnOnce(&GL) -> Result<(), E> + 'a,
{
    fn apply(self, context: &GL) -> Result<(), E> {
        self.map(|f| f(context)).unwrap_or(Ok(()))
    }
}

#[derive(Clone)]
pub enum BufferRange<T> {
    None,
    Full(T),
    OffsetSize(T, u32, u32),
}

impl<T> BufferRange<T> {
    pub fn as_ref(&self) -> BufferRange<&T> {
        match *self {
            BufferRange::None => BufferRange::None,
            BufferRange::Full(ref buffer) => BufferRange::Full(buffer),
            BufferRange::OffsetSize(ref buffer, offset, size) => {
                BufferRange::OffsetSize(buffer, offset, size)
            }
        }
    }
}

impl<'a> BufferRange<&'a WebGlBuffer> {
    pub fn to_owned_buffer(&self) -> BufferRange<WebGlBuffer> {
        match *self {
            BufferRange::None => BufferRange::None,
            BufferRange::Full(buffer) => BufferRange::Full(buffer.clone()),
            BufferRange::OffsetSize(buffer, offset, size) => {
                BufferRange::OffsetSize(buffer.clone(), offset, size)
            }
        }
    }
}

impl<T> PartialEq for BufferRange<T>
where
    T: Borrow<WebGlBuffer>,
{
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (BufferRange::None, BufferRange::None) => true,
            (BufferRange::Full(a), BufferRange::Full(b)) => {
                AsRef::<JsValue>::as_ref(a.borrow()) == AsRef::<JsValue>::as_ref(b.borrow())
            }
            (
                BufferRange::OffsetSize(a, offset_a, size_a),
                BufferRange::OffsetSize(b, offset_b, size_b),
            ) => {
                offset_a == offset_b
                    && size_a == size_b
                    && AsRef::<JsValue>::as_ref(a.borrow()) == AsRef::<JsValue>::as_ref(b.borrow())
            }
            _ => false,
        }
    }
}

struct TextureUnitLRU {
    linkage: Vec<(usize, usize)>,
    lru_index: usize,
    mru_index: usize,
}

impl TextureUnitLRU {
    fn new(texture_units: usize) -> Self {
        let mut linkage = Vec::with_capacity(texture_units);
        let texture_units = texture_units as i32;

        for i in 0..texture_units {
            linkage.push((
                ((i - 1) % texture_units) as usize,
                ((i + 1) % texture_units) as usize,
            ));
        }

        TextureUnitLRU {
            linkage,
            lru_index: 0,
            mru_index: (texture_units - 1) as usize,
        }
    }

    fn use_unit(&mut self, unit: usize) {
        if unit != self.mru_index {
            if unit == self.lru_index {
                self.use_lru_unit();
            } else {
                let (previous, next) = self.linkage[unit];

                self.linkage[previous].1 = next;
                self.linkage[next].0 = previous;
                self.linkage[self.lru_index].0 = unit;
                self.linkage[self.mru_index].1 = unit;
                self.linkage[unit].0 = self.mru_index;
                self.linkage[unit].1 = self.lru_index;
                self.mru_index = unit;
            }
        }
    }

    fn use_lru_unit(&mut self) -> usize {
        let old_lru_index = self.lru_index;

        self.lru_index = self.linkage[old_lru_index].1;
        self.mru_index = old_lru_index;

        old_lru_index
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_texture_unit_lru() {
        let mut lru = TextureUnitLRU::new(4);

        assert_eq!(lru.use_lru_unit(), 0);
        assert_eq!(lru.use_lru_unit(), 1);
        assert_eq!(lru.use_lru_unit(), 2);
        assert_eq!(lru.use_lru_unit(), 3);
        assert_eq!(lru.use_lru_unit(), 0);

        lru.use_unit(0);

        assert_eq!(lru.use_lru_unit(), 1);

        lru.use_unit(3);

        assert_eq!(lru.use_lru_unit(), 2);
        assert_eq!(lru.use_lru_unit(), 0);
        assert_eq!(lru.use_lru_unit(), 1);
        assert_eq!(lru.use_lru_unit(), 3);
        assert_eq!(lru.use_lru_unit(), 2);
        assert_eq!(lru.use_lru_unit(), 0);
        assert_eq!(lru.use_lru_unit(), 1);
    }
}
