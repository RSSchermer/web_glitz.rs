use std::marker;

use web_sys::WebGl2RenderingContext as Gl;

use crate::image::format::{
    ColorFloatRenderable, ColorIntegerRenderable, ColorUnsignedIntegerRenderable, DepthRenderable,
    DepthStencilRenderable, Filterable, InternalFormat, RenderbufferFormat, StencilRenderable,
};
use crate::image::renderbuffer::Renderbuffer;
use crate::image::Region2D;
use crate::render_pass::{
    AttachableImage, AttachableImageDescriptor, RenderPassContext, RenderPassMismatch,
};
use crate::runtime::dynamic_state::ContextUpdate;
use crate::task::{GpuTask, Progress};
use crate::util::slice_make_mut;

pub struct Framebuffer<C, Ds> {
    pub color: C,
    pub depth_stencil: Ds,
    pub(crate) render_pass_id: usize,
}

impl<C, Ds> Framebuffer<C, Ds>
where
    C: BlitColorTarget,
{
    pub fn blit_color_command<S>(&self, region: Region2D, source: S) -> BlitCommand
    where
        S: BlitColorCompatible<C>,
    {
        let region = match region {
            Region2D::Area(origin, width, height) => (origin, width, height),
            Region2D::Fill => {
                let BlitTargetDescriptor { width, height } = self.color.descriptor();

                ((0, 0), width, height)
            }
        };

        BlitCommand {
            render_pass_id: self.render_pass_id,
            read_slot: Gl::COLOR_ATTACHMENT0,
            bitmask: Gl::COLOR_BUFFER_BIT,
            filter: Gl::NEAREST,
            region,
            source: source.descriptor(),
        }
    }

    pub fn blit_color_linear_command<S>(&self, region: Region2D, source: S) -> BlitCommand
    where
        S: BlitColorCompatible<C>,
        S::Format: Filterable,
    {
        let region = match region {
            Region2D::Area(origin, width, height) => (origin, width, height),
            Region2D::Fill => {
                let BlitTargetDescriptor { width, height } = self.color.descriptor();

                ((0, 0), width, height)
            }
        };

        BlitCommand {
            render_pass_id: self.render_pass_id,
            read_slot: Gl::COLOR_ATTACHMENT0,
            bitmask: Gl::COLOR_BUFFER_BIT,
            filter: Gl::LINEAR,
            region,
            source: source.descriptor(),
        }
    }
}

impl<C, F> Framebuffer<C, DepthStencilBuffer<F>>
where
    F: DepthStencilRenderable,
{
    pub fn blit_depth_stencil_command<S>(&self, region: Region2D, source: S) -> BlitCommand
    where
        S: BlitSource<Format = F>,
    {
        let region = match region {
            Region2D::Area(origin, width, height) => (origin, width, height),
            Region2D::Fill => (
                (0, 0),
                self.depth_stencil.width(),
                self.depth_stencil.height(),
            ),
        };

        BlitCommand {
            render_pass_id: self.render_pass_id,
            read_slot: Gl::DEPTH_STENCIL_ATTACHMENT,
            bitmask: Gl::DEPTH_BUFFER_BIT & Gl::STENCIL_BUFFER_BIT,
            filter: Gl::NEAREST,
            region,
            source: source.descriptor(),
        }
    }

    pub fn blit_depth_command<S>(&self, region: Region2D, source: S) -> BlitCommand
    where
        S: BlitSource<Format = F>,
    {
        let region = match region {
            Region2D::Area(origin, width, height) => (origin, width, height),
            Region2D::Fill => (
                (0, 0),
                self.depth_stencil.width(),
                self.depth_stencil.height(),
            ),
        };

        BlitCommand {
            render_pass_id: self.render_pass_id,
            read_slot: Gl::DEPTH_STENCIL_ATTACHMENT,
            bitmask: Gl::DEPTH_BUFFER_BIT,
            filter: Gl::NEAREST,
            region,
            source: source.descriptor(),
        }
    }

    pub fn blit_stencil_command<S>(&self, region: Region2D, source: S) -> BlitCommand
    where
        S: BlitSource<Format = F>,
    {
        let region = match region {
            Region2D::Area(origin, width, height) => (origin, width, height),
            Region2D::Fill => (
                (0, 0),
                self.depth_stencil.width(),
                self.depth_stencil.height(),
            ),
        };

        BlitCommand {
            render_pass_id: self.render_pass_id,
            read_slot: Gl::DEPTH_STENCIL_ATTACHMENT,
            bitmask: Gl::STENCIL_BUFFER_BIT,
            filter: Gl::NEAREST,
            region,
            source: source.descriptor(),
        }
    }
}

impl<C, F> Framebuffer<C, DepthBuffer<F>>
where
    F: DepthRenderable,
{
    pub fn blit_depth_command<S>(&self, region: Region2D, source: S) -> BlitCommand
    where
        S: BlitSource<Format = F>,
    {
        let region = match region {
            Region2D::Area(origin, width, height) => (origin, width, height),
            Region2D::Fill => (
                (0, 0),
                self.depth_stencil.width(),
                self.depth_stencil.height(),
            ),
        };

        BlitCommand {
            render_pass_id: self.render_pass_id,
            read_slot: Gl::DEPTH_ATTACHMENT,
            bitmask: Gl::DEPTH_BUFFER_BIT,
            filter: Gl::NEAREST,
            region,
            source: source.descriptor(),
        }
    }
}

impl<C, F> Framebuffer<C, StencilBuffer<F>>
where
    F: StencilRenderable,
{
    pub fn blit_stencil_command<S>(&self, region: Region2D, source: S) -> BlitCommand
    where
        S: BlitSource<Format = F>,
    {
        let region = match region {
            Region2D::Area(origin, width, height) => (origin, width, height),
            Region2D::Fill => (
                (0, 0),
                self.depth_stencil.width(),
                self.depth_stencil.height(),
            ),
        };

        BlitCommand {
            render_pass_id: self.render_pass_id,
            read_slot: Gl::STENCIL_ATTACHMENT,
            bitmask: Gl::STENCIL_BUFFER_BIT,
            filter: Gl::NEAREST,
            region,
            source: source.descriptor(),
        }
    }
}

pub trait BlitColorTarget {
    fn descriptor(&self) -> BlitTargetDescriptor;
}

impl<C> BlitColorTarget for C
where
    C: Buffer,
{
    fn descriptor(&self) -> BlitTargetDescriptor {
        BlitTargetDescriptor {
            width: self.width(),
            height: self.height(),
        }
    }
}

macro_rules! impl_blit_color_target {
    ($C0:ident, $($C:ident),*) => {
        impl<$C0, $($C),*> BlitColorTarget for ($C0, $($C),*)
        where
            $C0: Buffer,
            $($C: Buffer),*
        {
            fn descriptor(&self) -> BlitTargetDescriptor {
                #[allow(non_snake_case)]
                let ($C0, $($C),*) = self;

                let mut width = $C0.width();

                $(
                    if $C.width() < width {
                        width = $C.width();
                    }
                )*


                let mut height = $C0.height();

                $(
                    if $C.height() < height {
                        height = $C.height();
                    }
                )*

                BlitTargetDescriptor {
                    width,
                    height
                }
            }
        }
    }
}

impl_blit_color_target!(C0, C1);
impl_blit_color_target!(C0, C1, C2);
impl_blit_color_target!(C0, C1, C2, C3);
impl_blit_color_target!(C0, C1, C2, C3, C4);
impl_blit_color_target!(C0, C1, C2, C3, C4, C5);
impl_blit_color_target!(C0, C1, C2, C3, C4, C5, C6);
impl_blit_color_target!(C0, C1, C2, C3, C4, C5, C6, C7);
impl_blit_color_target!(C0, C1, C2, C3, C4, C5, C6, C7, C8);
impl_blit_color_target!(C0, C1, C2, C3, C4, C5, C6, C7, C8, C9);
impl_blit_color_target!(C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10);
impl_blit_color_target!(C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11);
impl_blit_color_target!(C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12);
impl_blit_color_target!(C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12, C13);
impl_blit_color_target!(C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12, C13, C14);
impl_blit_color_target!(C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12, C13, C14, C15);

pub struct BlitTargetDescriptor {
    width: u32,
    height: u32,
}

pub trait BlitSource {
    type Format: InternalFormat;

    fn descriptor(&self) -> BlitSourceDescriptor;
}

pub struct BlitSourceDescriptor {
    image_descriptor: AttachableImageDescriptor,
    region: ((u32, u32), u32, u32),
}

impl<F> BlitSource for Renderbuffer<F>
where
    F: RenderbufferFormat + 'static,
{
    type Format = F;

    fn descriptor(&self) -> BlitSourceDescriptor {
        BlitSourceDescriptor {
            image_descriptor: AttachableImage::descriptor(self),
            region: ((0, 0), self.width(), self.height()),
        }
    }
}

pub unsafe trait BlitColorCompatible<C>: BlitSource {}

unsafe impl<T> BlitColorCompatible<ColorFloatBuffer<T::Format>> for T
where
    T: BlitSource,
    T::Format: ColorFloatRenderable,
{
}
unsafe impl<T> BlitColorCompatible<ColorIntegerBuffer<T::Format>> for T
where
    T: BlitSource,
    T::Format: ColorIntegerRenderable,
{
}
unsafe impl<T> BlitColorCompatible<ColorUnsignedIntegerBuffer<T::Format>> for T
where
    T: BlitSource,
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

pub struct BlitColorCommand {
    region: ((u32, u32), u32, u32),
    source: BlitSourceDescriptor,
}

pub struct BlitCommand {
    render_pass_id: usize,
    read_slot: u32,
    bitmask: u32,
    filter: u32,
    region: ((u32, u32), u32, u32),
    source: BlitSourceDescriptor,
}

impl<'a> GpuTask<RenderPassContext<'a>> for BlitCommand {
    type Output = Result<(), RenderPassMismatch>;

    fn progress(&mut self, context: &mut RenderPassContext) -> Progress<Self::Output> {
        if self.render_pass_id != context.render_pass_id() {
            return Progress::Finished(Err(RenderPassMismatch));
        }

        let (gl, state) = unsafe { context.unpack_mut() };

        state.bind_read_framebuffer(gl);

        self.source
            .image_descriptor
            .attach(gl, Gl::READ_FRAMEBUFFER, self.read_slot);

        let ((src_x0, src_y0), src_width, src_height) = self.source.region;
        let src_x1 = src_x0 + src_width;
        let src_y1 = src_y0 + src_height;

        let ((dst_x0, dst_y0), dst_width, dst_height) = self.region;
        let dst_x1 = dst_x0 + dst_width;
        let dst_y1 = dst_y0 + dst_height;

        gl.blit_framebuffer(
            src_x0 as i32,
            src_y0 as i32,
            src_x1 as i32,
            src_y1 as i32,
            dst_x0 as i32,
            dst_y0 as i32,
            dst_x1 as i32,
            dst_y1 as i32,
            self.bitmask,
            self.filter,
        );

        Progress::Finished(Ok(()))
    }
}

pub trait Buffer {
    type Format: InternalFormat;

    fn width(&self) -> u32;

    fn height(&self) -> u32;
}

pub struct ColorFloatBuffer<F>
where
    F: ColorFloatRenderable,
{
    render_pass_id: usize,
    index: i32,
    width: u32,
    height: u32,
    _marker: marker::PhantomData<Box<F>>,
}

impl<F> ColorFloatBuffer<F>
where
    F: ColorFloatRenderable,
{
    pub(crate) fn new(render_pass_id: usize, index: i32, width: u32, height: u32) -> Self {
        ColorFloatBuffer {
            render_pass_id,
            index,
            width,
            height,
            _marker: marker::PhantomData,
        }
    }

    pub fn clear_command(
        &mut self,
        clear_value: [f32; 4],
        region: Region2D,
    ) -> ClearColorFloatCommand {
        ClearColorFloatCommand {
            render_pass_id: self.render_pass_id,
            buffer_index: self.index,
            clear_value,
            region,
        }
    }
}

impl<F> Buffer for ColorFloatBuffer<F>
where
    F: ColorFloatRenderable,
{
    type Format = F;

    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }
}

pub struct ColorIntegerBuffer<F>
where
    F: ColorIntegerRenderable,
{
    render_pass_id: usize,
    index: i32,
    width: u32,
    height: u32,
    _marker: marker::PhantomData<Box<F>>,
}

impl<F> ColorIntegerBuffer<F>
where
    F: ColorIntegerRenderable,
{
    pub(crate) fn new(render_pass_id: usize, index: i32, width: u32, height: u32) -> Self {
        ColorIntegerBuffer {
            render_pass_id,
            index,
            width,
            height,
            _marker: marker::PhantomData,
        }
    }

    pub fn clear_command(
        &mut self,
        clear_value: [i32; 4],
        region: Region2D,
    ) -> ClearColorIntegerCommand {
        ClearColorIntegerCommand {
            render_pass_id: self.render_pass_id,
            buffer_index: self.index,
            clear_value,
            region,
        }
    }
}
impl<F> Buffer for ColorIntegerBuffer<F>
where
    F: ColorIntegerRenderable,
{
    type Format = F;

    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }
}

pub struct ColorUnsignedIntegerBuffer<F>
where
    F: ColorUnsignedIntegerRenderable,
{
    render_pass_id: usize,
    index: i32,
    width: u32,
    height: u32,
    _marker: marker::PhantomData<Box<F>>,
}

impl<F> ColorUnsignedIntegerBuffer<F>
where
    F: ColorUnsignedIntegerRenderable,
{
    pub(crate) fn new(render_pass_id: usize, index: i32, width: u32, height: u32) -> Self {
        ColorUnsignedIntegerBuffer {
            render_pass_id,
            index,
            width,
            height,
            _marker: marker::PhantomData,
        }
    }

    pub fn clear_command(
        &mut self,
        clear_value: [u32; 4],
        region: Region2D,
    ) -> ClearColorUnsignedIntegerCommand {
        ClearColorUnsignedIntegerCommand {
            render_pass_id: self.render_pass_id,
            buffer_index: self.index,
            clear_value,
            region,
        }
    }
}

impl<F> Buffer for ColorUnsignedIntegerBuffer<F>
where
    F: ColorUnsignedIntegerRenderable,
{
    type Format = F;

    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }
}

pub struct DepthStencilBuffer<F>
where
    F: DepthStencilRenderable,
{
    render_pass_id: usize,
    width: u32,
    height: u32,
    _marker: marker::PhantomData<Box<F>>,
}

impl<F> DepthStencilBuffer<F>
where
    F: DepthStencilRenderable,
{
    pub(crate) fn new(render_pass_id: usize, width: u32, height: u32) -> Self {
        DepthStencilBuffer {
            render_pass_id,
            width,
            height,
            _marker: marker::PhantomData,
        }
    }

    pub fn clear_command(
        &mut self,
        depth: f32,
        stencil: i32,
        region: Region2D,
    ) -> ClearDepthStencilCommand {
        ClearDepthStencilCommand {
            render_pass_id: self.render_pass_id,
            depth,
            stencil,
            region,
        }
    }

    pub fn clear_depth_command(&mut self, depth: f32, region: Region2D) -> ClearDepthCommand {
        ClearDepthCommand {
            render_pass_id: self.render_pass_id,
            depth,
            region,
        }
    }

    pub fn clear_stencil_command(&mut self, stencil: i32, region: Region2D) -> ClearStencilCommand {
        ClearStencilCommand {
            render_pass_id: self.render_pass_id,
            stencil,
            region,
        }
    }
}

impl<F> Buffer for DepthStencilBuffer<F>
where
    F: DepthStencilRenderable,
{
    type Format = F;

    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }
}

pub struct DepthBuffer<F>
where
    F: DepthRenderable,
{
    render_pass_id: usize,
    width: u32,
    height: u32,
    _marker: marker::PhantomData<Box<F>>,
}

impl<F> DepthBuffer<F>
where
    F: DepthRenderable,
{
    pub(crate) fn new(render_pass_id: usize, width: u32, height: u32) -> Self {
        DepthBuffer {
            render_pass_id,
            width,
            height,
            _marker: marker::PhantomData,
        }
    }

    pub fn clear_command(&mut self, depth: f32, region: Region2D) -> ClearDepthCommand {
        ClearDepthCommand {
            render_pass_id: self.render_pass_id,
            depth,
            region,
        }
    }
}

impl<F> Buffer for DepthBuffer<F>
where
    F: DepthRenderable,
{
    type Format = F;

    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }
}

pub struct StencilBuffer<F>
where
    F: StencilRenderable,
{
    render_pass_id: usize,
    width: u32,
    height: u32,
    _marker: marker::PhantomData<Box<F>>,
}

impl<F> StencilBuffer<F>
where
    F: StencilRenderable,
{
    pub(crate) fn new(render_pass_id: usize, width: u32, height: u32) -> Self {
        StencilBuffer {
            render_pass_id,
            width,
            height,
            _marker: marker::PhantomData,
        }
    }

    pub fn clear_command(&mut self, stencil: i32, region: Region2D) -> ClearStencilCommand {
        ClearStencilCommand {
            render_pass_id: self.render_pass_id,
            stencil,
            region,
        }
    }
}

impl<F> Buffer for StencilBuffer<F>
where
    F: StencilRenderable,
{
    type Format = F;

    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }
}

pub struct ClearColorFloatCommand {
    render_pass_id: usize,
    buffer_index: i32,
    clear_value: [f32; 4],
    region: Region2D,
}

impl<'a> GpuTask<RenderPassContext<'a>> for ClearColorFloatCommand {
    type Output = Result<(), RenderPassMismatch>;

    fn progress(&mut self, context: &mut RenderPassContext) -> Progress<Self::Output> {
        if self.render_pass_id != context.render_pass_id() {
            return Progress::Finished(Err(RenderPassMismatch));
        }

        let (gl, state) = unsafe { context.unpack_mut() };

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

        Progress::Finished(Ok(()))
    }
}

pub struct ClearColorIntegerCommand {
    render_pass_id: usize,
    buffer_index: i32,
    clear_value: [i32; 4],
    region: Region2D,
}

impl<'a> GpuTask<RenderPassContext<'a>> for ClearColorIntegerCommand {
    type Output = Result<(), RenderPassMismatch>;

    fn progress(&mut self, context: &mut RenderPassContext) -> Progress<Self::Output> {
        if self.render_pass_id != context.render_pass_id() {
            return Progress::Finished(Err(RenderPassMismatch));
        }

        let (gl, state) = unsafe { context.unpack_mut() };

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

        Progress::Finished(Ok(()))
    }
}

pub struct ClearColorUnsignedIntegerCommand {
    render_pass_id: usize,
    buffer_index: i32,
    clear_value: [u32; 4],
    region: Region2D,
}

impl<'a> GpuTask<RenderPassContext<'a>> for ClearColorUnsignedIntegerCommand {
    type Output = Result<(), RenderPassMismatch>;

    fn progress(&mut self, context: &mut RenderPassContext) -> Progress<Self::Output> {
        if self.render_pass_id != context.render_pass_id() {
            return Progress::Finished(Err(RenderPassMismatch));
        }

        let (gl, state) = unsafe { context.unpack_mut() };

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

        Progress::Finished(Ok(()))
    }
}

pub struct ClearDepthStencilCommand {
    render_pass_id: usize,
    depth: f32,
    stencil: i32,
    region: Region2D,
}

impl<'a> GpuTask<RenderPassContext<'a>> for ClearDepthStencilCommand {
    type Output = Result<(), RenderPassMismatch>;

    fn progress(&mut self, context: &mut RenderPassContext) -> Progress<Self::Output> {
        if self.render_pass_id != context.render_pass_id() {
            return Progress::Finished(Err(RenderPassMismatch));
        }

        let (gl, state) = unsafe { context.unpack_mut() };

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

        Progress::Finished(Ok(()))
    }
}

pub struct ClearDepthCommand {
    render_pass_id: usize,
    depth: f32,
    region: Region2D,
}

impl<'a> GpuTask<RenderPassContext<'a>> for ClearDepthCommand {
    type Output = Result<(), RenderPassMismatch>;

    fn progress(&mut self, context: &mut RenderPassContext) -> Progress<Self::Output> {
        if self.render_pass_id != context.render_pass_id() {
            return Progress::Finished(Err(RenderPassMismatch));
        }

        let (gl, state) = unsafe { context.unpack_mut() };

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

        Progress::Finished(Ok(()))
    }
}

pub struct ClearStencilCommand {
    render_pass_id: usize,
    stencil: i32,
    region: Region2D,
}

impl<'a> GpuTask<RenderPassContext<'a>> for ClearStencilCommand {
    type Output = Result<(), RenderPassMismatch>;

    fn progress(&mut self, context: &mut RenderPassContext) -> Progress<Self::Output> {
        if self.render_pass_id != context.render_pass_id() {
            return Progress::Finished(Err(RenderPassMismatch));
        }

        let (gl, state) = unsafe { context.unpack_mut() };

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

        Progress::Finished(Ok(()))
    }
}
