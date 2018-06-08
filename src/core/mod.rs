use super::webgl_rendering_context::{WebGL2RenderingContext as gl, WebGLBuffer, WebGLProgram,
                                     WebGLShader, WebGLUniformLocation};

use std::hash::{Hash, Hasher};
use std::rc::{Rc, Weak};
use stdweb::web::html_element::CanvasElement;

pub struct RenderingContext {
    internal: Rc<RenderingContextInternal>,
}

impl RenderingContext {
    pub fn for_canvas(canvas: &CanvasElement) -> RenderingContext {}

    pub fn for_canvas_with_options(canvas: &CanvasElement, options: ContextOptions) {}

    pub fn default_frame(&self) -> DefaultFrame {}

    pub fn transform_feedback_session(&mut self, program: &Program) -> TransformFeedbackSession {}
}

pub struct ContextOptions {
    alpha: bool,
    depth: bool,
    stencil: bool,
    antialias: bool,
    premultiplied_alpha: bool,
    preserve_drawing_buffer: bool,
    fail_if_major_performance_caveat: bool,
}

impl Default for ContextOptions {
    fn default() -> ContextOptions {
        ContextOptions {
            alpha: true,
            depth: true,
            stencil: false,
            antialias: true,
            premultiplied_alpha: true,
            preserve_drawing_buffer: false,
            fail_if_major_performance_caveat: false,
        }
    }
}

struct TransformFeedbackSession<'a> {
    context: &'a mut RenderingContext,
}

pub trait Frame {
    fn painter<'a, 'b: 'a, C, P>(&'a mut self, context: &'b mut C) -> &'b C::Painter
    where
        C: PaintingContext;

    fn clear_color(&mut self, color: &[f32; 4]);

    fn clear_depth(&mut self, depth: f32);

    fn clear_stencil(&mut self, stencil: u8);

    fn clear_color_and_depth(&mut self);

    fn clear_color_and_stencil(&mut self);

    fn clear_depth_and_stencil(&mut self);

    fn clear_all(&mut self, color: &[f32; 4], depth: f32, stencil: u8);
}

pub trait PaintingContextMut<'a, F>
where
    F: Frame,
{
    type Painter;

    fn painter_for_frame(self, frame: &'a mut F) -> Self::Painter;
}

impl<'a, F> PaitingContextMut for &'a mut RenderingContext
where
    F: Frame,
{
    type Painter = StandardPainter<'a, F>;

    fn painter_for_frame(self, frame: &'a mut F) -> Self::Painter {
        StandardPainter {
            context: self,
            frame,
        }
    }
}

pub struct StandardPainter<'a, F>
where
    F: Frame,
{
    context: &'a mut RenderingContext,
    frame: &'a mut F,
}

impl<'a, F> StandardPainter<'a, F>
where
    F: Frame,
{
    pub fn draw(
        &mut self,
        geometry: G,
        program: Program,
        uniforms: Uniforms,
        options: DrawOptions,
    ) -> Result<(), DrawError> {

    }
}

pub struct TransformFeedbackPainter<'a, 'b: 'a, F>
where
    F: Frame,
{
    transform_feedback_session: &'a mut TransformFeedbackSession<'b>,
    frame: &'a mut F,
}

impl<'a, 'b: 'a, F> TransformFeedbackPainter<'a, 'b, F>
where
    F: Frame,
{
    pub fn draw(
        &mut self,
        geometry: G,
        uniforms: Uniforms,
        options: DrawOptions,
    ) -> Result<(), DrawError> {

    }
}

struct RenderingContextInternal {
    wrapped: gl::RenderingContext,
}

struct BufferBindings {
    gl_context: gl::RenderingContext,
    array_buffer_binding: Option<GLUint>,
    element_array_buffer_binding: Option<GLUint>,
    pixel_pack_buffer_binding: Option<GLUint>,
    pixel_unpack_buffer_binding: Option<GLUint>,
    copy_read_buffer_binding: Option<GLUint>,
    copy_write_buffer_binding: Option<GLUint>,
}

impl BufferBindings {}

pub struct Buffer {
    context: Weak<RenderingContextInternal>,
    id: GLUint,
    usage_hint: UsageHint,
}

impl Buffer {
    pub fn upload<D>(
        context: &RenderingContext,
        initial_data: D,
        usage_hint: UsageHint,
    ) -> Result<Buffer, BufferCreationError>
    where
        D: Into<&[u8]>,
    {
        let context_internal = context.internal.clone();

        let id = context_internal.create_buffer();

        context_internal.bind_buffer(gl::COPY_WRITE_BUFFER, id);
        context_internal.buffer_data_1(target, initial_data, usage_hint);

        DrawBuffer {
            context: context.internal.clone().downgrade(),
            id,
            usage_hint,
        }
    }

    pub fn copy(from: Buffer, usage_hint: UsageHint) -> Result<Buffer, BufferCreationError> {
        // TODO: call create_buffer and copy_buffer_sub_data
    }

    pub fn empty(
        context: &RenderingContext,
        size_in_bytes: usize,
        usage_hint: UsageHint,
    ) -> Result<Buffer, BufferCreationError> {
        // TODO: call create_buffer and buffer_data
    }

    pub fn usage_hint(&self) -> UsageHint {
        self.usage_hint
    }

    pub fn upload_sub_data(&self, offset: usize, data: D) -> Result<(), BufferError>
    where
        D: Into<&[u8]>,
    {
        // TODO: call buffer_sub_data on context
    }

    pub fn copy_sub_data(&self, offset: usize, buffer: Buffer) -> Result<(), CopyError> {
        // TODO: call copy_buffer_sub_data on context
    }

    pub fn download(&self) -> impl Future<Item = Vec<u8>, Error = DownloadError> {
        // TODO: use fencing and PromiseFuture? Possibly WEBGL_get_buffer_sub_data_async extension?
    }

    pub fn download_sync(&self) -> Result<Vec<u8>, DownloadError> {}
}

impl Drop for Buffer {
    fn drop(&mut self) {
        // TODO: call delete_buffer on context if context is not dropped
    }
}

impl Hash for Buffer {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.context.hash(state);
    }
}

impl PartialEq for Buffer {
    fn eq(&self, other: Self) -> bool {
        self.context == other.context && self.id == other.id
    }
}

enum AttributeType {
    Byte,
    Short,
    UnsignedByte,
    UnsignedShort,
    Float,
}

enum IndexType {
    UnsignedByte,
    UnsignedShort,
    UnsignedInt,
}

enum UsageHint {
    StaticDraw,
    StaticCopy,
    StaticRead,
    DynamicDraw,
    DynamicCopy,
    DynamicRead,
    StreamDraw,
    StreamCopy,
    StreamRead,
}

enum Topology<'a> {
    Points {
        first: u32,
        count: u32,
    },
    IndexedPoints {
        indices: &'a IndexData,
        first: u32,
        count: u32,
    },
    Lines {
        first: u32,
        count: u32,
    },
    IndexedLines {
        indices: &'a IndexData,
        first: u32,
        count: u32,
    },
    LineStrip {
        first: u32,
        count: u32,
    },
    IndexedLineStrip {
        indices: &'a IndexData,
        first: u32,
        count: u32,
    },
    LineLoop {
        first: u32,
        count: u32,
    },
    IndexedLineLoop {
        indices: &'a IndexData,
        first: u32,
        count: u32,
    },
    Triangles {
        first: u32,
        count: u32,
    },
    IndexedTriangles {
        indices: &'a IndexData,
        first: u32,
        count: u32,
    },
    TriangleStrip {
        first: u32,
        count: u32,
    },
    IndexedTriangleStrip {
        indices: &'a IndexData,
        first: u32,
        count: u32,
    },
    TriangleFan {
        first: u32,
        count: u32,
    },
    IndexedTriangleFan {
        indices: &'a IndexData,
        first: u32,
        count: u32,
    },
}

pub trait AttributeFormat: Hash {
    fn get_pointer(&self, name: &str) -> Option<AttributePointer>;
}

impl AttributeFormat for &[(&str, AttributePointer)] {
    fn get_pointer(&self, name: &str) -> Option<AttributePointer> {
        self.iter().find_map(|(attribute_name, pointer)| {
            if attribute_name == name {
                Some(pointer)
            } else {
                None
            }
        })
    }
}

trait IndexSource {
    type Data: BufferableData;

    fn data(&self) -> &Self::Data;

    fn data_type(&self) -> IndexDataType;
}

pub struct AttributePointer {
    data_type: AttributeType,
    offset_in_bytes: u32,
    stride_in_bytes: u32,
    normalize: bool,
}

struct Painter {}

impl Painter {
    fn draw<F, G>(
        &mut self,
        frame: &F,
        geometry: G,
        program: Program,
        uniforms: Uniforms,
        config: DrawConfig,
    ) where
        F: Frame,
        G: Borrow<Geometry>,
    {
        let geometry = geometry.borrow();
    }

    fn transform_feedback_session<F>(
        &mut self,
        program: TransformFeedbackProgram,
        block: F,
    ) -> PausedTransformFeedbackSession
    where
        F: FnOnce(TransformFeedbackPainter) -> (),
    {

    }

    fn samples_passed_session<F>(&mut self, block: F) -> SamplesPassedResult
    where
        F: FnOnce() -> (),
    {

    }

    fn samples_passed_conservative_session<F>(
        &mut self,
        block: F,
    ) -> SamplesPassedConservativeResult
    where
        F: FnOnce() -> (),
    {

    }
}

#[derive(Debug, Clone)]
struct Shader {
    context: Weak<RenderingContextInternal>,
    gl_shader: WebGLShader,
}

#[derive(Debug)]
struct Program {
    context: Weak<RenderingContextInternal>,
    gl_program: WebGLProgram,
    vertex_shader: Shader,
    fragment_shader: Shader,
}

impl Program {
    fn new(
        context: &RenderingContext,
        vertex_shader: Shader,
        fragment_shader: Shader,
    ) -> Result<Program, ProgramCreationError> {
        Program {
            vertex_shader,
            fragment_shader,
        }
    }
}

struct TransformFeedbackProgram {}

struct Uniforms {}

struct PausedTransformFeedbackSession<'a> {}

impl PausedTransformFeedbackSession {
    fn resume<F>(self) -> PausedTransformFeedbackSession
    where
        F: FnOnce(TransformFeedbackContext) -> (),
    {

    }
}

struct TransformFeedbackContext {}

impl TransformFeedbackContext {
    fn draw<G>(&mut self, geometry: G, uniforms: Uniforms, config: DrawConfig)
    where
        G: Borrow<Geometry>,
    {
        let geometry = geometry.borrow();
    }
}
