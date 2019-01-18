use web_sys::WebGl2RenderingContext as Gl;

use image_region::Region2D;
use render_pass::RenderPassContext;
use task::GpuTask;
use task::Progress;
use util::slice_make_mut;

use crate::runtime::dynamic_state::ContextUpdate;
use image_format::ColorFloatRenderable;
use image_format::ColorIntegerRenderable;
use image_format::ColorUnsignedIntegerRenderable;
use render_pass::AttachableImage;

#[derive(Clone, Copy, PartialEq)]
pub enum BlitFilter {
    Nearest,
    Linear
}

pub struct Framebuffer<C, Ds> {
    pub color: C,
    pub depth_stencil: Ds,
    pub(crate) _private: (), // Should make it impossible to instantiate Framebuffer outside of this crate
}

impl<C, Ds> Framebuffer<C, Ds> {
    pub fn blit_color_task<I>(&mut self, source_image: I, filter: BlitFilter) -> BlitColorTask
    where
        I: BlitColorCompatible<C>,
    {
        unimplemented!()
    }
}

pub unsafe trait BlitColorCompatible<C>: AttachableImage {}

unsafe impl<T> BlitColorCompatible<ColorFloatBuffer> for T
where
    T: AttachableImage,
    T::Format: ColorFloatRenderable,
{
}
unsafe impl<T> BlitColorCompatible<ColorIntegerBuffer> for T
where
    T: AttachableImage,
    T::Format: ColorIntegerRenderable,
{
}
unsafe impl<T> BlitColorCompatible<ColorUnsignedIntegerBuffer> for T
where
    T: AttachableImage,
    T::Format: ColorUnsignedIntegerRenderable,
{
}

macro_rules! impl_blit_color_compatible {
    ($C0:ident, $($C:ident),*) => {
        unsafe impl<T, $C0, $($C),*> BlitColorCompatible<($C0, $($C),*)> for T
        where T: BlitColorCompatible<$C0> $(+ BlitColorCompatible<$C>)* {}
    }
}

impl_blit_color_compatible!(C0, C1);
impl_blit_color_compatible!(C0, C1, C2);
impl_blit_color_compatible!(C0, C1, C2, C3);
impl_blit_color_compatible!(C0, C1, C2, C3, C4);
impl_blit_color_compatible!(C0, C1, C2, C3, C4, C5);
impl_blit_color_compatible!(C0, C1, C2, C3, C4, C5, C6);
impl_blit_color_compatible!(C0, C1, C2, C3, C4, C5, C6, C7);
impl_blit_color_compatible!(C0, C1, C2, C3, C4, C5, C6, C7, C8);
impl_blit_color_compatible!(C0, C1, C2, C3, C4, C5, C6, C7, C8, C9);
impl_blit_color_compatible!(C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10);
impl_blit_color_compatible!(C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11);
impl_blit_color_compatible!(C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12);
impl_blit_color_compatible!(C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12, C13);
impl_blit_color_compatible!(C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12, C13, C14);
impl_blit_color_compatible!(C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12, C13, C14, C15);

pub struct BlitColorTask {}

pub trait Buffer {}

pub struct ColorFloatBuffer {
    index: i32,
}

impl ColorFloatBuffer {
    pub(crate) fn new(index: i32) -> Self {
        ColorFloatBuffer { index }
    }

    pub fn clear_task(&mut self, clear_value: [f32; 4], region: Region2D) -> ClearColorFloatTask {
        ClearColorFloatTask {
            buffer_index: self.index,
            clear_value,
            region,
        }
    }
}

impl Buffer for ColorFloatBuffer {}

pub struct ColorIntegerBuffer {
    index: i32,
}

impl ColorIntegerBuffer {
    pub(crate) fn new(index: i32) -> Self {
        ColorIntegerBuffer { index }
    }

    pub fn clear_task(&mut self, clear_value: [i32; 4], region: Region2D) -> ClearColorIntegerTask {
        ClearColorIntegerTask {
            buffer_index: self.index,
            clear_value,
            region,
        }
    }
}
impl Buffer for ColorIntegerBuffer {}

pub struct ColorUnsignedIntegerBuffer {
    index: i32,
}

impl ColorUnsignedIntegerBuffer {
    pub(crate) fn new(index: i32) -> Self {
        ColorUnsignedIntegerBuffer { index }
    }

    pub fn clear_task(
        &mut self,
        clear_value: [u32; 4],
        region: Region2D,
    ) -> ClearColorUnsignedIntegerTask {
        ClearColorUnsignedIntegerTask {
            buffer_index: self.index,
            clear_value,
            region,
        }
    }
}

impl Buffer for ColorUnsignedIntegerBuffer {}

pub struct DepthStencilBuffer {}

impl DepthStencilBuffer {
    pub fn clear_both_task(
        &mut self,
        depth: f32,
        stencil: i32,
        region: Region2D,
    ) -> ClearDepthStencilTask {
        ClearDepthStencilTask {
            depth,
            stencil,
            region,
        }
    }

    pub fn clear_depth_task(&mut self, depth: f32, region: Region2D) -> ClearDepthTask {
        ClearDepthTask { depth, region }
    }

    pub fn clear_stencil_task(&mut self, stencil: i32, region: Region2D) -> ClearStencilTask {
        ClearStencilTask { stencil, region }
    }
}

impl Buffer for DepthStencilBuffer {}

pub struct DepthBuffer {}

impl DepthBuffer {
    pub fn clear_task(&mut self, depth: f32, region: Region2D) -> ClearDepthTask {
        ClearDepthTask { depth, region }
    }
}

impl Buffer for DepthBuffer {}

pub struct StencilBuffer {}

impl StencilBuffer {
    pub fn clear_task(&mut self, stencil: i32, region: Region2D) -> ClearStencilTask {
        ClearStencilTask { stencil, region }
    }
}

impl Buffer for StencilBuffer {}

pub struct ClearColorFloatTask {
    buffer_index: i32,
    clear_value: [f32; 4],
    region: Region2D,
}

impl<'a> GpuTask<RenderPassContext<'a>> for ClearColorFloatTask {
    type Output = ();

    fn progress(&mut self, context: &mut RenderPassContext) -> Progress<Self::Output> {
        let (gl, state) = unsafe { context.unpack() };

        match self.region {
            Region2D::Fill => state.set_scissor_test_enabled(false).apply(gl).unwrap(),
            Region2D::Area((x, y), width, height) => {
                state.set_scissor_test_enabled(true).apply(gl).unwrap();
                state
                    .set_scissor_rect((x as i32, y as i32, width, height))
                    .apply(gl)
                    .unwrap();
            }
        }

        gl.clear_bufferfv_with_f32_array(Gl::COLOR, self.buffer_index, unsafe {
            slice_make_mut(&self.clear_value)
        });

        Progress::Finished(())
    }
}

pub struct ClearColorIntegerTask {
    buffer_index: i32,
    clear_value: [i32; 4],
    region: Region2D,
}

impl<'a> GpuTask<RenderPassContext<'a>> for ClearColorIntegerTask {
    type Output = ();

    fn progress(&mut self, context: &mut RenderPassContext) -> Progress<Self::Output> {
        let (gl, state) = unsafe { context.unpack() };

        match self.region {
            Region2D::Fill => state.set_scissor_test_enabled(false).apply(gl).unwrap(),
            Region2D::Area((x, y), width, height) => {
                state.set_scissor_test_enabled(true).apply(gl).unwrap();
                state
                    .set_scissor_rect((x as i32, y as i32, width, height))
                    .apply(gl)
                    .unwrap();
            }
        }

        gl.clear_bufferiv_with_i32_array(Gl::COLOR, self.buffer_index, unsafe {
            slice_make_mut(&self.clear_value)
        });

        Progress::Finished(())
    }
}

pub struct ClearColorUnsignedIntegerTask {
    buffer_index: i32,
    clear_value: [u32; 4],
    region: Region2D,
}

impl<'a> GpuTask<RenderPassContext<'a>> for ClearColorUnsignedIntegerTask {
    type Output = ();

    fn progress(&mut self, context: &mut RenderPassContext) -> Progress<Self::Output> {
        let (gl, state) = unsafe { context.unpack() };

        match self.region {
            Region2D::Fill => state.set_scissor_test_enabled(false).apply(gl).unwrap(),
            Region2D::Area((x, y), width, height) => {
                state.set_scissor_test_enabled(true).apply(gl).unwrap();
                state
                    .set_scissor_rect((x as i32, y as i32, width, height))
                    .apply(gl)
                    .unwrap();
            }
        }

        gl.clear_bufferuiv_with_u32_array(Gl::COLOR, self.buffer_index, unsafe {
            slice_make_mut(&self.clear_value)
        });

        Progress::Finished(())
    }
}

pub struct ClearDepthStencilTask {
    depth: f32,
    stencil: i32,
    region: Region2D,
}

impl<'a> GpuTask<RenderPassContext<'a>> for ClearDepthStencilTask {
    type Output = ();

    fn progress(&mut self, context: &mut RenderPassContext) -> Progress<Self::Output> {
        let (gl, state) = unsafe { context.unpack() };

        match self.region {
            Region2D::Fill => state.set_scissor_test_enabled(false).apply(gl).unwrap(),
            Region2D::Area((x, y), width, height) => {
                state.set_scissor_test_enabled(true).apply(gl).unwrap();
                state
                    .set_scissor_rect((x as i32, y as i32, width, height))
                    .apply(gl)
                    .unwrap();
            }
        }

        gl.clear_bufferfi(Gl::DEPTH_STENCIL, 0, self.depth, self.stencil);

        Progress::Finished(())
    }
}

pub struct ClearDepthTask {
    depth: f32,
    region: Region2D,
}

impl<'a> GpuTask<RenderPassContext<'a>> for ClearDepthTask {
    type Output = ();

    fn progress(&mut self, context: &mut RenderPassContext) -> Progress<Self::Output> {
        let (gl, state) = unsafe { context.unpack() };

        match self.region {
            Region2D::Fill => state.set_scissor_test_enabled(false).apply(gl).unwrap(),
            Region2D::Area((x, y), width, height) => {
                state.set_scissor_test_enabled(true).apply(gl).unwrap();
                state
                    .set_scissor_rect((x as i32, y as i32, width, height))
                    .apply(gl)
                    .unwrap();
            }
        }

        gl.clear_bufferfv_with_f32_array(Gl::DEPTH, 0, unsafe { slice_make_mut(&[self.depth]) });

        Progress::Finished(())
    }
}

pub struct ClearStencilTask {
    stencil: i32,
    region: Region2D,
}

impl<'a> GpuTask<RenderPassContext<'a>> for ClearStencilTask {
    type Output = ();

    fn progress(&mut self, context: &mut RenderPassContext) -> Progress<Self::Output> {
        let (gl, state) = unsafe { context.unpack() };

        match self.region {
            Region2D::Fill => state.set_scissor_test_enabled(false).apply(gl).unwrap(),
            Region2D::Area((x, y), width, height) => {
                state.set_scissor_test_enabled(true).apply(gl).unwrap();
                state
                    .set_scissor_rect((x as i32, y as i32, width, height))
                    .apply(gl)
                    .unwrap();
            }
        }

        gl.clear_bufferiv_with_i32_array(Gl::STENCIL, 0, unsafe {
            slice_make_mut(&[self.stencil])
        });

        Progress::Finished(())
    }
}
