use std::cell::UnsafeCell;
use std::hash::{Hash, Hasher};
use std::marker;
use std::sync::Arc;

use wasm_bindgen::JsCast;
use web_sys::WebGl2RenderingContext as Gl;

use crate::image::format::{Multisamplable, Multisample, RenderbufferFormat};
use crate::runtime::state::ContextUpdate;
use crate::runtime::{Connection, RenderingContext, UnsupportedSampleCount};
use crate::task::{ContextId, GpuTask, Progress};
use crate::util::JsId;

/// Provides the information necessary for the creation of a [Renderbuffer].
///
/// See [RenderingContext::try_create_renderbuffer] and
/// [RenderingContext::try_create_multisample_renderbuffer] for details.
pub struct RenderbufferDescriptor<F> {
    /// The format type the [Renderbuffer] will use to store its image data.
    pub format: F,

    /// The width of the [Renderbuffer].
    pub width: u32,

    /// The height of the [Renderbuffer].
    pub height: u32,
}

/// Stores a single 2-dimensional image, optimized for use as a [RenderTarget] attachment.
///
/// Unlike a [Texture2D], which can also hold a single 2-dimensional image, a [Renderbuffer] cannot
/// be sampled. However, a [Renderbuffer] is optimized for use as a render target attachment,
/// whereas a [Texture2D] may not be. A [Renderbuffer] is therefor the best choice for a
/// [RenderTarget] attachment that never needs to be sampled.
///
/// See [RenderingContext::create_renderbuffer] and
/// [RenderingContext::create_multisample_renderbuffer] for details on how a [Renderbuffer] is
/// created for single-sample and multisample image data respectively.
///
/// # Example
///
/// The following example creates a [Renderbuffer] and uses it as the color attachment in a render
/// pass, which clears a central square to blue pixels:
///
/// ```rust
/// # use web_glitz::runtime::RenderingContext;
/// # fn wrapper<Rc>(context: &Rc) where Rc: RenderingContext + Clone + 'static {
/// use web_glitz::image::Region2D;
/// use web_glitz::image::format::RGB8;
/// use web_glitz::image::renderbuffer::RenderbufferDescriptor;
/// use web_glitz::rendering::{LoadOp, RenderTargetDescriptor, StoreOp};
///
/// let mut renderbuffer = context.create_renderbuffer(&RenderbufferDescriptor {
///     format: RGB8,
///     width: 256,
///     height: 256
/// });
///
/// let mut render_target = context.create_render_target(
///     RenderTargetDescriptor::new()
///         .attach_color_float(
///             &mut renderbuffer,
///             LoadOp::Load,
///             StoreOp::Store
///         )
/// );
///
/// let render_pass = render_target.create_render_pass(|framebuffer| {
///     framebuffer.color.0.clear_command([0.0, 0.0, 1.0, 0.0], Region2D::Area((64, 64), 128, 128))
/// });
///
/// context.submit(render_pass);
/// # }
/// ```
pub struct Renderbuffer<F> {
    object_id: u64,
    data: Arc<RenderbufferData>,
    _marker: marker::PhantomData<[F]>,
}

impl<F> Renderbuffer<F> {
    pub(crate) fn data(&self) -> &Arc<RenderbufferData> {
        &self.data
    }

    /// The width of this [Renderbuffer].
    pub fn width(&self) -> u32 {
        self.data.width
    }

    /// The height of this [Renderbuffer].
    pub fn height(&self) -> u32 {
        self.data.height
    }
}

impl<F> Renderbuffer<Multisample<F>>
where
    F: Multisamplable,
{
    /// Returns the number of samples this [Renderbuffer] stores for its image data.
    pub fn samples(&self) -> u8 {
        self.data.samples.unwrap()
    }
}

impl<F> Renderbuffer<F>
where
    F: RenderbufferFormat + 'static,
{
    pub(crate) fn new<Rc>(
        context: &Rc,
        object_id: u64,
        descriptor: &RenderbufferDescriptor<F>,
    ) -> Self
    where
        Rc: RenderingContext + Clone + 'static,
    {
        let data = Arc::new(RenderbufferData {
            id: UnsafeCell::new(None),
            context_id: context.id(),
            dropper: Box::new(context.clone()),
            width: descriptor.width,
            height: descriptor.height,
            samples: None,
        });

        context.submit(RenderbufferAllocateCommand::<F> {
            data: data.clone(),
            _marker: marker::PhantomData,
        });

        Renderbuffer {
            object_id,
            data,
            _marker: marker::PhantomData,
        }
    }
}

impl<F> Renderbuffer<Multisample<F>>
where
    F: RenderbufferFormat + Multisamplable + Copy + 'static,
{
    pub(crate) fn new_multisample<Rc>(
        context: &Rc,
        object_id: u64,
        descriptor: &RenderbufferDescriptor<Multisample<F>>,
    ) -> Result<Self, UnsupportedSampleCount>
    where
        Rc: RenderingContext + Clone + 'static,
    {
        let supported_samples = context.supported_samples(descriptor.format.sample_format());

        if !supported_samples
            .into_iter()
            .any(|count| count == descriptor.format.samples())
        {
            return Err(UnsupportedSampleCount {
                supported_samples,
                requested_samples: descriptor.format.samples(),
            });
        }

        let data = Arc::new(RenderbufferData {
            id: UnsafeCell::new(None),
            context_id: context.id(),
            dropper: Box::new(context.clone()),
            width: descriptor.width,
            height: descriptor.height,
            samples: Some(descriptor.format.samples()),
        });

        context.submit(MultisampleRenderbufferAllocateCommand::<F> {
            data: data.clone(),
            samples: descriptor.format.samples(),
            _marker: marker::PhantomData,
        });

        Ok(Renderbuffer {
            object_id,
            data,
            _marker: marker::PhantomData,
        })
    }
}

impl<F> PartialEq for Renderbuffer<F> {
    fn eq(&self, other: &Self) -> bool {
        self.object_id == other.object_id
    }
}

impl<F> Hash for Renderbuffer<F> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.object_id.hash(state);
    }
}

trait RenderbufferObjectDropper {
    fn drop_renderbuffer_object(&self, id: JsId);
}

impl<T> RenderbufferObjectDropper for T
where
    T: RenderingContext,
{
    fn drop_renderbuffer_object(&self, id: JsId) {
        self.submit(RenderbufferDropCommand { id });
    }
}

pub(crate) struct RenderbufferData {
    id: UnsafeCell<Option<JsId>>,
    context_id: u64,
    dropper: Box<dyn RenderbufferObjectDropper>,
    width: u32,
    height: u32,
    samples: Option<u8>,
}

impl RenderbufferData {
    pub(crate) fn id(&self) -> Option<JsId> {
        unsafe { *self.id.get() }
    }

    pub(crate) fn context_id(&self) -> u64 {
        self.context_id
    }
}

impl PartialEq for RenderbufferData {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl Hash for RenderbufferData {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.id().hash(state);
    }
}

impl Drop for RenderbufferData {
    fn drop(&mut self) {
        if let Some(id) = self.id() {
            self.dropper.drop_renderbuffer_object(id);
        }
    }
}

struct RenderbufferAllocateCommand<F> {
    data: Arc<RenderbufferData>,
    _marker: marker::PhantomData<[F]>,
}

unsafe impl<F> GpuTask<Connection> for RenderbufferAllocateCommand<F>
where
    F: RenderbufferFormat,
{
    type Output = ();

    fn context_id(&self) -> ContextId {
        ContextId::Any
    }

    fn progress(&mut self, connection: &mut Connection) -> Progress<Self::Output> {
        let (gl, state) = unsafe { connection.unpack_mut() };
        let data = &self.data;
        let object = gl.create_renderbuffer().unwrap();

        state.bind_renderbuffer(Some(&object)).apply(gl).unwrap();

        gl.renderbuffer_storage(
            Gl::RENDERBUFFER,
            F::ID,
            data.width as i32,
            data.height as i32,
        );

        unsafe {
            *data.id.get() = Some(JsId::from_value(object.into()));
        }

        Progress::Finished(())
    }
}

struct MultisampleRenderbufferAllocateCommand<F> {
    data: Arc<RenderbufferData>,
    samples: u8,
    _marker: marker::PhantomData<[F]>,
}

unsafe impl<F> GpuTask<Connection> for MultisampleRenderbufferAllocateCommand<F>
where
    F: RenderbufferFormat,
{
    type Output = ();

    fn context_id(&self) -> ContextId {
        ContextId::Any
    }

    fn progress(&mut self, connection: &mut Connection) -> Progress<Self::Output> {
        let (gl, state) = unsafe { connection.unpack_mut() };
        let data = &self.data;
        let object = gl.create_renderbuffer().unwrap();

        state.bind_renderbuffer(Some(&object)).apply(gl).unwrap();

        gl.renderbuffer_storage_multisample(
            Gl::RENDERBUFFER,
            self.samples as i32,
            F::ID,
            data.width as i32,
            data.height as i32,
        );

        unsafe {
            *data.id.get() = Some(JsId::from_value(object.into()));
        }

        Progress::Finished(())
    }
}

struct RenderbufferDropCommand {
    id: JsId,
}

unsafe impl GpuTask<Connection> for RenderbufferDropCommand {
    type Output = ();

    fn context_id(&self) -> ContextId {
        ContextId::Any
    }

    fn progress(&mut self, connection: &mut Connection) -> Progress<Self::Output> {
        let (gl, state) = unsafe { connection.unpack_mut() };
        let value = unsafe { JsId::into_value(self.id).unchecked_into() };

        state.unref_renderbuffer(&value);
        gl.delete_renderbuffer(Some(&value));

        Progress::Finished(())
    }
}
