use std::ops::{Deref, DerefMut};
use std::sync::Arc;

use crate::framebuffer::framebuffer_handle::FramebufferData;
use crate::rendering_context::{Connection, ContextUpdate};
use crate::task::{GpuTask, Progress};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum DrawBuffer {
    None,
    ColorAttachment0,
    ColorAttachment1,
    ColorAttachment2,
    ColorAttachment3,
    ColorAttachment4,
    ColorAttachment5,
    ColorAttachment6,
    ColorAttachment7,
    ColorAttachment8,
    ColorAttachment9,
    ColorAttachment10,
    ColorAttachment11,
    ColorAttachment12,
    ColorAttachment13,
    ColorAttachment14,
    ColorAttachment15,
}

pub struct RenderPass<T> {
    task: T,
    framebuffer_data: Arc<FramebufferData>,
}

pub struct RenderPassContext {
    connection: *mut Connection,
}

impl Deref for RenderPassContext {
    type Target = Connection;

    fn deref(&self) -> &Connection {
        unsafe { &*self.connection }
    }
}

impl DerefMut for RenderPassContext {
    fn deref_mut(&mut self) -> &mut Connection {
        unsafe { &mut *self.connection }
    }
}

impl<T> GpuTask<Connection> for RenderPass<T>
where
    T: GpuTask<RenderPassContext>,
{
    type Output = T::Output;

    fn progress(&mut self, connection: &mut Connection) -> Progress<Self::Output> {
        let Connection(gl, state) = connection;

        unsafe {
            self.framebuffer_data
                .id
                .unwrap()
                .with_value_unchecked(|fbo| {
                    state
                        .set_bound_draw_framebuffer(Some(&fbo))
                        .apply(gl)
                        .unwrap();
                });
        }

        self.task.progress(&mut RenderPassContext {
            connection: connection as *mut _,
        })
    }
}

pub struct SubPass<T> {
    draw_buffers: [DrawBuffer; 16],
    task: T,
}

pub fn sub_pass<B, T>(draw_buffers: B, task: T) -> SubPass<T>
where
    B: IntoIterator<Item = DrawBuffer>,
    T: GpuTask<SubPassContext>,
{
    let mut draw_buffer_array = [DrawBuffer::None; 16];

    for (i, buffer) in draw_buffers.into_iter().enumerate() {
        draw_buffer_array[i] = buffer;
    }

    SubPass {
        draw_buffers: draw_buffer_array,
        task,
    }
}

pub struct SubPassContext {
    connection: *mut Connection,
}

impl Deref for SubPassContext {
    type Target = Connection;

    fn deref(&self) -> &Connection {
        unsafe { &*self.connection }
    }
}

impl DerefMut for SubPassContext {
    fn deref_mut(&mut self) -> &mut Connection {
        unsafe { &mut *self.connection }
    }
}

impl<T> GpuTask<RenderPassContext> for SubPass<T>
where
    T: GpuTask<SubPassContext>,
{
    type Output = T::Output;

    fn progress(&mut self, context: &mut RenderPassContext) -> Progress<Self::Output> {
        // TODO: set draw_buffers (not supported yet by web_sys)

        self.task.progress(&mut SubPassContext {
            connection: context.connection,
        })
    }
}
