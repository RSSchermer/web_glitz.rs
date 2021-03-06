use js_sys::Uint32Array;
use web_sys::WebGl2RenderingContext as Gl;

use crate::rendering::render_target::RenderTargetData;
use crate::rendering::StoreOp;
use crate::runtime::state::{ContextUpdate, DepthStencilAttachmentDescriptor, DynamicState};
use crate::runtime::Connection;
use crate::task::{ContextId, GpuTask, Progress};

/// Encapsulates a render pass.
///
/// A render pass task consists of a render target (see [RenderTarget] and
/// [MultisampleRenderTarget]) and a render pass task (a series of commands). The images attached to
/// the render target may be loaded into the framebuffer. The commands in render pass task may then
/// modify the contents of the framebuffer. When the task is complete, the contents of the
/// framebuffer may be stored back into the images attached to the render target. If and how
/// the image data is to be loaded and stored is declared as part of the render target, see
/// [RenderTargetDescriptor] and [MultisampleRenderTargetDescriptor] for details.
///
/// For details on how a [RenderPass] is created, see [RenderTarget::create_render_pass] and
/// [MultisampleRenderTarget::create_render_pass].
#[derive(Clone)]
pub struct RenderPass<T> {
    pub(crate) id: u64,
    pub(crate) context_id: u64,
    pub(crate) render_target: RenderTargetData,
    pub(crate) task: T,
}

/// An execution context associated with a [RenderPass].
pub struct RenderPassContext {
    connection: *mut Connection,
    render_pass_id: u64,
}

impl RenderPassContext {
    /// The ID of the [RenderPass] this [RenderPassContext] is associated with.
    pub fn render_pass_id(&self) -> u64 {
        self.render_pass_id
    }

    pub(crate) fn connection_mut(&mut self) -> &mut Connection {
        unsafe { &mut *self.connection }
    }

    /// Unpacks this context into a reference to the raw [web_sys::WebGl2RenderingContext] and a
    /// reference to the WebGlitz state cache for this context.
    ///
    /// # Unsafe
    ///
    /// If state is changed on the [web_sys::WebGl2RenderingContext], than the cache must be updated
    /// accordingly.
    pub unsafe fn unpack(&self) -> (&Gl, &DynamicState) {
        (*self.connection).unpack()
    }

    /// Unpacks this context into a mutable reference to the raw [web_sys::WebGl2RenderingContext]
    /// and a mutable reference to the WebGlitz state cache for this context.
    ///
    /// # Unsafe
    ///
    /// If state is changed on the [web_sys::WebGl2RenderingContext], than the cache must be updated
    /// accordingly.
    pub unsafe fn unpack_mut(&mut self) -> (&mut Gl, &mut DynamicState) {
        (*self.connection).unpack_mut()
    }
}

unsafe impl<T, O> GpuTask<Connection> for RenderPass<T>
where
    T: GpuTask<RenderPassContext, Output = O>,
{
    type Output = O;

    fn context_id(&self) -> ContextId {
        ContextId::Id(self.context_id)
    }

    fn progress(&mut self, connection: &mut Connection) -> Progress<Self::Output> {
        let (gl, state) = unsafe { connection.unpack_mut() };

        match &self.render_target {
            RenderTargetData::Default => {
                state.bind_draw_framebuffer(None).apply(gl).unwrap();

                self.task.progress(&mut RenderPassContext {
                    connection,
                    render_pass_id: self.id,
                })
            }
            RenderTargetData::Custom(data) => {
                state
                    .framebuffer_cache_mut()
                    .bind_or_create(data, gl)
                    .set_draw_buffers(data.draw_buffers());

                for i in 0..data.color_count {
                    data.load_ops[i].perform(gl);
                }

                if &data.depth_stencil_attachment != &DepthStencilAttachmentDescriptor::None {
                    data.load_ops[16].perform(gl);
                }

                let output = self.task.progress(&mut RenderPassContext {
                    connection,
                    render_pass_id: self.id,
                });

                let mut invalidate_buffers = [0; 17];
                let mut invalidate_counter = 0;

                for i in 0..data.color_count {
                    if data.store_ops[i] == StoreOp::DontCare {
                        invalidate_buffers[invalidate_counter] = Gl::COLOR_ATTACHMENT0 + i as u32;

                        invalidate_counter += 1;
                    }
                }

                if let Some(buffer_id) = match &data.depth_stencil_attachment {
                    DepthStencilAttachmentDescriptor::DepthStencil(_) => {
                        Some(Gl::DEPTH_STENCIL_ATTACHMENT)
                    }
                    DepthStencilAttachmentDescriptor::Depth(_) => Some(Gl::DEPTH_ATTACHMENT),
                    DepthStencilAttachmentDescriptor::Stencil(_) => Some(Gl::STENCIL_ATTACHMENT),
                    DepthStencilAttachmentDescriptor::None => None,
                } {
                    if data.store_ops[16] == StoreOp::DontCare {
                        invalidate_buffers[invalidate_counter] = buffer_id;

                        invalidate_counter += 1;
                    }
                }

                if invalidate_counter > 0 {
                    let (gl, _) = unsafe { connection.unpack() };
                    let array =
                        unsafe { Uint32Array::view(&invalidate_buffers[0..invalidate_counter]) };

                    gl.invalidate_framebuffer(Gl::DRAW_FRAMEBUFFER, array.as_ref())
                        .unwrap();
                }

                output
            }
        }
    }
}
