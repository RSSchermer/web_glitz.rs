use std::borrow::Borrow;

use futures::future::Future;
use futures::sync::oneshot::{Canceled, Receiver};
use futures::{Async, Poll};
use web_sys::WebGl2RenderingContext as Gl;

use crate::buffer::{Buffer, BufferUsage, IntoBuffer};
use crate::image::format::{Filterable, RenderbufferFormat, TextureFormat};
use crate::image::renderbuffer::Renderbuffer;
use crate::image::texture_2d::{Texture2D, Texture2DDescriptor};
use crate::image::texture_2d_array::{Texture2DArray, Texture2DArrayDescriptor};
use crate::image::texture_3d::{Texture3D, Texture3DDescriptor};
use crate::image::texture_cube::{TextureCube, TextureCubeDescriptor};
use crate::image::{MaxMipmapLevelsExceeded, MipmapLevels};
use crate::pipeline::graphics::vertex_input::{
    Incompatible as IncompatibleInputAttributeLayout, InputAttributeLayout,
};
use crate::pipeline::resources::{Incompatible as IncompatibleResourceLayout, Resources};
use crate::runtime::state::DynamicState;
use crate::sampler::{Sampler, SamplerDescriptor, ShadowSampler, ShadowSamplerDescriptor};
use crate::task::GpuTask;
use crate::pipeline::graphics::{GraphicsPipelineDescriptor, GraphicsPipeline};

pub trait RenderingContext {
    fn id(&self) -> usize;

    fn create_buffer<D, T>(&self, data: D, usage_hint: BufferUsage) -> Buffer<T>
    where
        D: IntoBuffer<T>;

    fn create_renderbuffer<F>(&self, width: u32, height: u32) -> Renderbuffer<F>
    where
        F: RenderbufferFormat + 'static;

    fn create_graphics_pipeline<Il, R, Tf>(
        &self,
        descriptor: &GraphicsPipelineDescriptor<Il, R, Tf>,
    ) -> Result<GraphicsPipeline<Il, R, Tf>, CreateGraphicsPipelineError>
    where
        Il: InputAttributeLayout,
        R: Resources,
        Tf: TransformFeedbackVaryings;

    fn create_texture_2d<F>(
        &self,
        descriptor: &Texture2DDescriptor<F>,
    ) -> Result<Texture2D<F>, MaxMipmapLevelsExceeded>
    where
        F: TextureFormat + 'static;

    fn create_texture_2d_array<F>(
        &self,
        descriptor: &Texture2DArrayDescriptor<F>,
    ) -> Result<Texture2DArray<F>, MaxMipmapLevelsExceeded>
    where
        F: TextureFormat + 'static;

    fn create_texture_3d<F>(
        &self,
        descriptor: &Texture3DDescriptor<F>,
    ) -> Result<Texture3D<F>, MaxMipmapLevelsExceeded>
    where
        F: TextureFormat + 'static;

    fn create_texture_cube<F>(
        &self,
        descriptor: &TextureCubeDescriptor<F>,
    ) -> Result<TextureCube<F>, MaxMipmapLevelsExceeded>
    where
        F: TextureFormat + 'static;

    fn create_sampler(&self, descriptor: &SamplerDescriptor) -> Sampler;

    fn create_shadow_sampler(&self, descriptor: &ShadowSamplerDescriptor) -> ShadowSampler;

    fn submit<T>(&self, task: T) -> Execution<T::Output>
    where
        T: GpuTask<Connection> + 'static;
}

pub enum CreateGraphicsPipelineError {
    LinkingError(String),
    IncompatibleInputAttributeLayout(IncompatibleInputAttributeLayout),
    IncompatibleResourceLayout(IncompatibleResourceLayout),
    IncompatibleTransformFeedbackVaryings(IncompatibleTransformFeedbackVaryings),
}

pub enum SubmitError {
    Cancelled,
}

impl From<Canceled> for SubmitError {
    fn from(_: Canceled) -> Self {
        SubmitError::Cancelled
    }
}

pub enum Execution<O> {
    Ready(Option<O>),
    Pending(Receiver<O>),
}

impl<O> Future for Execution<O> {
    type Item = O;

    type Error = SubmitError;

    fn poll(&mut self) -> Poll<O, SubmitError> {
        match self {
            Execution::Ready(ref mut output) => {
                let output = output
                    .take()
                    .expect("Cannot poll Execution more than once after its ready");

                Ok(Async::Ready(output))
            }
            Execution::Pending(ref mut recv) => match recv.poll() {
                Ok(Async::Ready(output)) => Ok(Async::Ready(output)),
                Ok(Async::NotReady) => Ok(Async::NotReady),
                Err(Canceled) => Err(SubmitError::Cancelled),
            },
        }
    }
}

impl<T> From<T> for Execution<T> {
    fn from(value: T) -> Self {
        Execution::Ready(Some(value))
    }
}

impl<T> From<Receiver<T>> for Execution<T> {
    fn from(recv: Receiver<T>) -> Self {
        Execution::Pending(recv)
    }
}

pub struct Connection {
    context_id: usize,
    gl: Gl,
    state: DynamicState,
}

impl Connection {
    pub fn new(context_id: usize, gl: Gl, state: DynamicState) -> Self {
        Connection {
            context_id,
            gl,
            state,
        }
    }

    pub fn context_id(&self) -> usize {
        self.context_id
    }

    pub unsafe fn unpack(&self) -> (&Gl, &DynamicState) {
        (&self.gl, &self.state)
    }

    pub unsafe fn unpack_mut(&mut self) -> (&mut Gl, &mut DynamicState) {
        (&mut self.gl, &mut self.state)
    }
}

pub struct TaskContextMismatch;
