use futures::future::Future;
use futures::sync::oneshot::{Canceled, Receiver};
use futures::{Async, Poll};
use web_sys::WebGl2RenderingContext as Gl;

use crate::buffer::{BufferHandle, BufferUsage};
use crate::framebuffer::{FramebufferDescriptor, FramebufferHandle};
use crate::image_format::Filterable;
use crate::renderbuffer::{RenderbufferFormat, RenderbufferHandle};
use crate::runtime::dynamic_state::DynamicState;
use crate::task::GpuTask;
use crate::texture::{
    Texture2DArrayHandle, Texture2DHandle, Texture3DHandle, TextureCubeHandle, TextureFormat,
};

pub trait RenderingContext: Clone {
    fn create_value_buffer<T>(&self, usage_hint: BufferUsage) -> BufferHandle<T>;

    fn create_array_buffer<T>(&self, len: usize, usage_hint: BufferUsage) -> BufferHandle<[T]>;

    fn create_framebuffer<D>(&self, descriptor: &D) -> FramebufferHandle
    where
        D: FramebufferDescriptor;

    fn create_renderbuffer<F>(&self, width: u32, height: u32) -> RenderbufferHandle<F>
    where
        F: RenderbufferFormat + 'static;

    fn create_texture_2d<F>(&self, width: u32, height: u32) -> Texture2DHandle<F>
    where
        F: TextureFormat + 'static;

    fn create_texture_2d_mipmapped<F>(
        &self,
        width: u32,
        height: u32,
        levels: usize,
    ) -> Texture2DHandle<F>
    where
        F: TextureFormat + Filterable + 'static;

    fn create_texture_2d_array<F>(
        &self,
        width: u32,
        height: u32,
        depth: u32,
        levels: usize,
    ) -> Texture2DArrayHandle<F>
    where
        F: TextureFormat + 'static;

    fn create_texture_3d<F>(
        &self,
        width: u32,
        height: u32,
        depth: u32,
        levels: usize,
    ) -> Texture3DHandle<F>
    where
        F: TextureFormat + 'static;

    fn create_texture_cube<F>(
        &self,
        width: u32,
        height: u32,
        levels: usize,
    ) -> TextureCubeHandle<F>
    where
        F: TextureFormat + 'static;

    fn submit<T>(&self, task: T) -> Execution<T::Output>
    where
        T: GpuTask<Connection> + 'static;
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

pub struct Connection(pub Gl, pub DynamicState);
