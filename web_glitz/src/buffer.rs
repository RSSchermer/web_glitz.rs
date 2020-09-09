//! GPU-accessible memory buffers that contain typed data.
//!
//! # Initialization
//!
//! A buffer can store any type that is both [Sized] and [Copy]. We can for example store an
//! [InterfaceBlock] type (which we might later use to provide data to a uniform block in a
//! pipeline):
//!
//! ```
//! # #![feature(const_fn, const_loop, const_if_match, const_ptr_offset_from, const_transmute, ptr_offset_from)]
//! # use web_glitz::runtime::RenderingContext;
//! # fn wrapper<Rc>(context: &Rc) where Rc: RenderingContext {
//! use web_glitz::buffer::UsageHint;
//!
//! // We use the `std140` crate to ensure that the layout of our `Uniforms` type conforms to
//! // the std140 data layout.
//! #[std140::repr_std140]
//! #[derive(web_glitz::derive::InterfaceBlock, Clone, Copy)]
//! struct Uniforms {
//!     scale: std140::float,
//! }
//!
//! let uniforms = Uniforms {
//!     scale: std140::float(0.5),
//! };
//!
//! let uniform_buffer = context.create_buffer(uniforms, UsageHint::DynamicDraw);
//! # }
//! ```
//!
//! Here `context` is a [RenderingContext]. We use [UsageHint::DynamicDraw] to indicate that we
//! intend to read this buffer on the GPU and we intend to modify the contents of the buffer
//! repeatedly (see [UsageHint] for details).
//!
//! A buffer can also store an array of any type `T` that is both [Sized] and [Copy], by
//! initializing it with a type that implements `Borrow<[T]>`. We can for example store an array
//! of [Vertex] values:
//!
//! ```
//! # #![feature(const_fn, const_transmute, const_ptr_offset_from, ptr_offset_from)]
//! # use web_glitz::runtime::RenderingContext;
//! # fn wrapper<Rc>(context: &Rc) where Rc: RenderingContext {
//! use web_glitz::buffer::{Buffer, UsageHint};
//!
//! #[derive(web_glitz::derive::Vertex, Clone, Copy)]
//! struct Vertex {
//!     #[vertex_attribute(location = 0, format = "Float2_f32")]
//!     position: [f32; 2],
//!     #[vertex_attribute(location = 1, format = "Float3_u8_norm")]
//!     color: [u8; 3],
//! }
//!
//! let vertex_data = [
//!     Vertex {
//!         position: [0.0, 0.5],
//!         color: [255, 0, 0],
//!     },
//!     Vertex {
//!         position: [-0.5, -0.5],
//!         color: [0, 255, 0],
//!     },
//!     Vertex {
//!         position: [0.5, -0.5],
//!         color: [0, 0, 255],
//!     },
//! ];
//!
//! let vertex_buffer: Buffer<[Vertex]> = context.create_buffer(vertex_data, UsageHint::StaticDraw);
//! # }
//! ```
//!
//! Note that [RenderingContext::create_buffer] takes ownership of the data source (`vertex_data`
//! in the example) and that the data source must be `'static`. It is however possible to use shared
//! ownership constructs like [Rc] or [Arc]. We use a [UsageHint::StaticDraw] to once again
//! indiciate that we wish to read this data on the GPU, but this time we don't intend to modify the
//! data in the buffer later.
//!
//! [InterfaceBlock]: web_glitz::pipeline::interface_block::InterfaceBlock
//! [RenderingContext]: web_glitz::runtime::RenderingContext
//! [Vertex]: web_glitz::pipeline::graphics::Vertex
//! [Rc]: std::rc::Rc
//! [Arc]: std::sync::Arc
use std::borrow::Borrow;
use std::cell::UnsafeCell;
use std::marker;
use std::mem;
use std::ops::{Deref, Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive};
use std::slice;
use std::sync::Arc;

use wasm_bindgen::JsCast;
use web_sys::{WebGl2RenderingContext as GL, WebGlBuffer};

use crate::runtime::state::ContextUpdate;
use crate::runtime::{Connection, RenderingContext};
use crate::task::{ContextId, GpuTask, Progress};
use crate::util::JsId;
use std::hash::{Hash, Hasher};
use wasm_bindgen::__rt::core::mem::MaybeUninit;

/// A GPU-accessible memory buffer that contains typed data.
///
/// # Example
///
/// ```rust
/// # use web_glitz::runtime::RenderingContext;
/// # fn wrapper<Rc>(context: &Rc) where Rc: RenderingContext + Clone + 'static {
/// use web_glitz::buffer::{Buffer, UsageHint};
///
/// let buffer: Buffer<[f32]> = context.create_buffer([1.0, 2.0, 3.0, 4.0], UsageHint::StreamDraw);
/// # }
/// ```
///
/// A buffer can be created with any data that implements [IntoBuffer].
pub struct Buffer<T>
where
    T: ?Sized,
{
    object_id: u64,
    data: Arc<BufferData>,
    _marker: marker::PhantomData<Box<T>>,
}

impl<T> Buffer<T>
where
    T: ?Sized,
{
    pub(crate) fn data(&self) -> &Arc<BufferData> {
        &self.data
    }

    /// Returns the [UsageHint] that was specified for this [Buffer] when it was created.
    ///
    /// See [UsageHint] for details.
    pub fn usage_hint(&self) -> UsageHint {
        self.data.usage_hint
    }
}

impl<T> Buffer<MaybeUninit<T>>
where
    T: 'static,
{
    pub(crate) fn create_uninit<Rc>(
        context: &Rc,
        buffer_id: BufferId,
        usage_hint: UsageHint,
    ) -> Self
    where
        Rc: RenderingContext + Clone + 'static,
    {
        let data = Arc::new(BufferData {
            id: UnsafeCell::new(None),
            context_id: context.id(),
            dropper: Box::new(context.clone()),
            usage_hint,
            len: 1,
        });

        let marker: marker::PhantomData<T> = marker::PhantomData;

        context.submit(AllocateUninitCommand {
            data: data.clone(),
            _marker: marker,
        });

        Buffer {
            object_id: buffer_id.object_id,
            data,
            _marker: marker::PhantomData,
        }
    }
}

impl<T> Buffer<[MaybeUninit<T>]>
where
    T: 'static,
{
    pub(crate) fn create_slice_uninit<Rc>(
        context: &Rc,
        buffer_id: BufferId,
        len: usize,
        usage_hint: UsageHint,
    ) -> Self
    where
        Rc: RenderingContext + Clone + 'static,
    {
        let data = Arc::new(BufferData {
            id: UnsafeCell::new(None),
            context_id: context.id(),
            dropper: Box::new(context.clone()),
            usage_hint,
            len,
        });

        let marker: marker::PhantomData<[T]> = marker::PhantomData;

        context.submit(AllocateUninitCommand {
            data: data.clone(),
            _marker: marker,
        });

        Buffer {
            object_id: buffer_id.object_id,
            data,
            _marker: marker::PhantomData,
        }
    }
}

impl<T> Buffer<T>
where
    T: Copy,
{
    /// Returns a command which, when executed will replace the data contained in this [Buffer] with
    /// the given `data`.
    pub fn upload_command<D>(&self, data: D) -> UploadCommand<T, D>
    where
        D: Borrow<T> + Send + Sync + 'static,
    {
        UploadCommand {
            buffer_data: self.data.clone(),
            data,
            offset_in_bytes: 0,
            len: 1,
            _marker: marker::PhantomData,
        }
    }

    /// Returns a command which, when executed will copy the data contained in this [Buffer] into a
    /// [Box].
    ///
    /// When the task is finished, the [Box] containing the copied data will be output.
    pub fn download_command(&self) -> DownloadCommand<T> {
        DownloadCommand {
            data: self.data.clone(),
            state: DownloadState::Initial,
            offset_in_bytes: 0,
            len: 1,
            _marker: marker::PhantomData,
        }
    }
}

impl<T> Buffer<MaybeUninit<T>> {
    /// Converts to `Buffer<T>`.
    ///
    /// # Safety
    ///
    /// Any tasks that read from the buffer after `assume_init` was called, must only be executed
    /// after the buffer was initialized. Note that certain tasks may wait on GPU fences and allow
    /// a runtime to progress other tasks while its waiting on the fence. As such, submitting your
    /// initialization tasks as part of a task that includes fencing (these are typically tasks that
    /// include "download" commands), may not guarantee that the buffer was initialized before any
    /// tasks that are submitted later will begin executing.
    pub unsafe fn assume_init(self) -> Buffer<T> {
        mem::transmute(self)
    }
}

impl<T> Buffer<[T]> {
    /// Returns the number of elements contained in this [Buffer].
    pub fn len(&self) -> usize {
        self.data.len
    }

    /// Returns a [BufferView] on an element or a slice of the elements this [Buffer], depending
    /// on the type of `index`.
    ///
    /// - If given a position, returns a view on the element at that position or `None` if out of
    ///   bounds.
    /// - If given a range, returns a view on the slice of elements corresponding to that range, or
    ///   `None` if out of bounds.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use web_glitz::runtime::RenderingContext;
    /// # fn wrapper<Rc>(context: &Rc) where Rc: RenderingContext + Clone + 'static {
    /// use web_glitz::buffer::UsageHint;
    ///
    /// let buffer = context.create_buffer([1.0, 2.0, 3.0, 4.0], UsageHint::StreamDraw);
    ///
    /// buffer.get(1); // Some BufferView<f32> containing `2.0`
    /// buffer.get(1..3); // Some BufferView<[f32]> containing `[2.0, 3.0]`
    /// buffer.get(..2); // Some BufferView<[f32]> containing `[1.0 2.0]`
    /// buffer.get(4); // None (index out of bounds)
    /// # }
    /// ```
    pub fn get<I>(&self, index: I) -> Option<BufferView<I::Output>>
    where
        I: BufferSliceIndex<T>,
    {
        index.get(self)
    }

    /// Returns a [BufferView] on an element or a slice of the elements this [Buffer], depending
    /// on the type of `index`, without doing bounds checking.
    ///
    /// - If given a position, returns a view on the element at that position, without doing bounds
    ///   checking.
    /// - If given a range, returns a view on the slice of elements corresponding to that range,
    ///   without doing bounds checking.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use web_glitz::runtime::RenderingContext;
    /// # fn wrapper<Rc>(context: &Rc) where Rc: RenderingContext + Clone + 'static {
    /// use web_glitz::buffer::UsageHint;
    ///
    /// let buffer = context.create_buffer([1.0, 2.0, 3.0, 4.0], UsageHint::StreamDraw);
    ///
    /// unsafe { buffer.get_unchecked(1) }; // BufferView<f32> containing `2.0`
    /// # }
    /// ```
    ///
    /// # Unsafe
    ///
    /// Only safe if `index` is in bounds. See [get] for a safe alternative.
    pub unsafe fn get_unchecked<I>(&self, index: I) -> BufferView<I::Output>
    where
        I: BufferSliceIndex<T>,
    {
        index.get_unchecked(self)
    }
}

impl<T> Buffer<[T]>
where
    T: Copy,
{
    /// Returns a command which, when executed will replace the elements contained in this [Buffer]
    /// with the elements in given `data`.
    ///
    /// If the `data` contains fewer elements than this [Buffer], then only the first `N` elements
    /// will be replaced, where `N` is the number of elements in the given `data`.
    ///
    /// If the `data` contains more elements than this [Buffer], then only the first `M` elements
    /// in the `data` will be used to update this [Buffer], where `M` is the number of elements in
    /// the [Buffer].
    pub fn upload_command<D>(&self, data: D) -> UploadCommand<[T], D>
    where
        D: Borrow<[T]> + Send + Sync + 'static,
    {
        UploadCommand {
            buffer_data: self.data.clone(),
            data,
            offset_in_bytes: 0,
            len: self.data.len,
            _marker: marker::PhantomData,
        }
    }

    /// Returns a command which, when executed will copy the elements contained in this [Buffer]
    /// into a [Box] as a boxed slice.
    ///
    /// When the task is finished, the [Box] containing the copied data will be output.
    pub fn download_command(&self) -> DownloadCommand<[T]> {
        DownloadCommand {
            data: self.data.clone(),
            state: DownloadState::Initial,
            offset_in_bytes: 0,
            len: self.data.len,
            _marker: marker::PhantomData,
        }
    }
}

impl<T> Buffer<[MaybeUninit<T>]> {
    /// Converts to `Buffer<[T]>`.
    ///
    /// # Safety
    ///
    /// Any tasks that read from the buffer after `assume_init` was called, must only be executed
    /// after the buffer was initialized. Note that certain tasks may wait on GPU fences and allow
    /// a runtime to progress other tasks while its waiting on the fence. As such, submitting your
    /// initialization tasks as part of a task that includes fencing (these are typically tasks that
    /// include "download" commands), may not guarantee that the buffer was initialized before any
    /// tasks that are submitted later will begin executing.
    pub unsafe fn assume_init(self) -> Buffer<[T]> {
        mem::transmute(self)
    }
}

impl<T> PartialEq for Buffer<T>
where
    T: ?Sized,
{
    fn eq(&self, other: &Self) -> bool {
        self.object_id == other.object_id
    }
}

impl<T> Hash for Buffer<T>
where
    T: ?Sized,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.object_id.hash(state);
    }
}

impl<'a, T> From<&'a Buffer<T>> for BufferView<'a, T>
where
    T: ?Sized,
{
    fn from(buffer: &'a Buffer<T>) -> BufferView<'a, T> {
        BufferView {
            buffer,
            offset_in_bytes: 0,
            len: buffer.data.len,
        }
    }
}

impl<'a, T> From<&'a mut Buffer<T>> for BufferView<'a, T>
where
    T: ?Sized,
{
    fn from(buffer: &'a mut Buffer<T>) -> BufferView<'a, T> {
        BufferView {
            buffer,
            offset_in_bytes: 0,
            len: buffer.data.len,
        }
    }
}

// TODO: this should not be necessary if CoerceUnsized or an equivalent can be implemented for
// Buffer.
impl<'a, T, const LEN: usize> From<&'a Buffer<[T; LEN]>> for BufferView<'a, [T]> {
    fn from(buffer: &'a Buffer<[T; LEN]>) -> BufferView<'a, [T]> {
        BufferView {
            buffer: unsafe { mem::transmute(buffer) },
            offset_in_bytes: 0,
            len: buffer.data.len,
        }
    }
}

impl<'a, T, const LEN: usize> From<&'a mut Buffer<[T; LEN]>> for BufferView<'a, [T]> {
    fn from(buffer: &'a mut Buffer<[T; LEN]>) -> BufferView<'a, [T]> {
        BufferView {
            len: buffer.data.len,
            buffer: unsafe { mem::transmute(buffer) },
            offset_in_bytes: 0,
        }
    }
}

impl<'a, T> From<&'a mut Buffer<T>> for BufferViewMut<'a, T>
where
    T: ?Sized,
{
    fn from(buffer: &'a mut Buffer<T>) -> BufferViewMut<'a, T> {
        BufferViewMut {
            inner: BufferView {
                buffer,
                offset_in_bytes: 0,
                len: buffer.data.len,
            },
            _marker: marker::PhantomData,
        }
    }
}

// TODO: this should not be necessary if CoerceUnsized or an equivalent can be implemented for
// Buffer.
impl<'a, T, const LEN: usize> From<&'a mut Buffer<[T; LEN]>> for BufferViewMut<'a, [T]> {
    fn from(buffer: &'a mut Buffer<[T; LEN]>) -> BufferViewMut<'a, [T]> {
        BufferViewMut {
            inner: BufferView {
                len: buffer.data.len,
                buffer: unsafe { mem::transmute(buffer) },
                offset_in_bytes: 0,
            },
            _marker: marker::PhantomData,
        }
    }
}

// TODO: CoerceUnsized doesn't currently work with only a PhantomData field...
//impl<T, U> CoerceUnsized<Buffer<U>> for Buffer<T>
//    where
//        T: Unsize<U> + ?Sized,
//        U: ?Sized,
//{}

/// A view on a segment or the whole of a [Buffer].
#[derive(PartialEq, Hash)]
pub struct BufferView<'a, T>
where
    T: ?Sized,
{
    buffer: &'a Buffer<T>,
    pub(crate) offset_in_bytes: usize,
    len: usize,
}

impl<'a, T> BufferView<'a, T>
where
    T: ?Sized,
{
    pub(crate) fn buffer_data(&self) -> &Arc<BufferData> {
        self.buffer.data()
    }

    pub(crate) fn offset_in_bytes(&self) -> usize {
        self.offset_in_bytes
    }

    /// Returns the [UsageHint] that was specified for the [Buffer] view by this [BufferView] when
    /// it was created.
    ///
    /// See [UsageHint] for details.
    pub fn usage_hint(&self) -> UsageHint {
        self.buffer.data.usage_hint
    }
}

impl<'a, T> BufferView<'a, T> {
    /// The size in bytes of the viewed buffer region.
    pub fn size_in_bytes(&self) -> usize {
        std::mem::size_of::<T>()
    }
}

impl<'a, T> BufferView<'a, T>
where
    T: Copy,
{
    /// Returns a command which, when executed will replace the data viewed by this [BufferView]
    /// with the given `data`.
    ///
    /// This will modify the viewed [Buffer], the buffer (and any other views on the same data) will
    /// be affected by this change.
    pub fn upload_command<D>(&self, data: D) -> UploadCommand<T, D>
    where
        D: Borrow<T> + Send + Sync + 'static,
    {
        UploadCommand {
            buffer_data: self.buffer.data.clone(),
            data,
            offset_in_bytes: self.offset_in_bytes,
            len: 1,
            _marker: marker::PhantomData,
        }
    }

    /// Returns a command which, when executed will copy the data viewed by in this [BufferView]
    /// into a [Box].
    ///
    /// When the task is finished, the [Box] containing the copied data will be output.
    pub fn download_command(&self) -> DownloadCommand<T> {
        DownloadCommand {
            data: self.buffer.data.clone(),
            state: DownloadState::Initial,
            offset_in_bytes: self.offset_in_bytes,
            len: 1,
            _marker: marker::PhantomData,
        }
    }
}

impl<'a, T> BufferView<'a, MaybeUninit<T>> {
    /// Converts to `BufferView<T>`.
    ///
    /// # Safety
    ///
    /// Its up to the user to guarantee that any tasks that read buffer region viewed by this view,
    /// is only executed after the viewed region is initialized. Note that certain tasks may wait on
    /// GPU fences and allow a runtime to progress other tasks while its waiting on the fence. As
    /// such, submitting your initialization tasks as part of a task that includes fencing (these
    /// are typically tasks that include "download" commands), may not guarantee that the buffer was
    /// initialized before any tasks that are submitted later will begin executing.
    pub unsafe fn assume_init(self) -> BufferView<'a, T> {
        mem::transmute(self)
    }
}

impl<'a, T> Clone for BufferView<'a, T> {
    fn clone(&self) -> Self {
        BufferView {
            buffer: self.buffer,
            offset_in_bytes: self.offset_in_bytes,
            len: self.len,
        }
    }
}

impl<'a, T> Copy for BufferView<'a, T> {}

impl<'a, T> BufferView<'a, [T]> {
    /// Returns the number of elements contained in this [Buffer].
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns a [BufferView] on an element or a sub-slice of the elements this [Buffer], depending
    /// on the type of `index`.
    ///
    /// - If given a position, returns a view on the element at that position or `None` if out of
    ///   bounds.
    /// - If given a range, returns a view on the sub-slice of elements corresponding to that range,
    ///   or `None` if out of bounds.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use web_glitz::runtime::RenderingContext;
    /// # fn wrapper<Rc>(context: &Rc) where Rc: RenderingContext + Clone + 'static {
    /// use web_glitz::buffer::{Buffer, BufferView, UsageHint};
    ///
    /// let buffer: Buffer<[f32]> = context.create_buffer([1.0, 2.0, 3.0, 4.0], UsageHint::StreamDraw);
    /// let view = BufferView::from(&buffer);
    ///
    /// view.get(1); // Some BufferView<f32> containing `2.0`
    /// view.get(1..3); // Some BufferView<[f32]> containing `[2.0, 3.0]`
    /// view.get(..2); // Some BufferView<[f32]> containing `[1.0 2.0]`
    /// view.get(4); // None (index out of bounds)
    /// # }
    /// ```
    pub fn get<I>(&self, index: I) -> Option<BufferView<I::Output>>
    where
        I: BufferViewSliceIndex<T>,
    {
        index.get(self)
    }

    /// Returns a [BufferView] on an element or a sub-slice of the elements this [BufferView],
    /// depending on the type of `index`, without doing bounds checking.
    ///
    /// - If given a position, returns a view on the element at that position, without doing bounds
    ///   checking.
    /// - If given a range, returns a view on the slice of elements corresponding to that range,
    ///   without doing bounds checking.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use web_glitz::runtime::RenderingContext;
    /// # fn wrapper<Rc>(context: &Rc) where Rc: RenderingContext + Clone + 'static {
    /// use web_glitz::buffer::{Buffer, BufferView, UsageHint};
    ///
    /// let buffer: Buffer<[f32]> = context.create_buffer([1.0, 2.0, 3.0, 4.0], UsageHint::StreamDraw);
    /// let view = BufferView::from(&buffer);
    ///
    /// unsafe { view.get_unchecked(1) }; // BufferView<f32> containing `2.0`
    /// # }
    /// ```
    ///
    /// # Unsafe
    ///
    /// Only safe if `index` is in bounds. See [get] for a safe alternative.
    pub unsafe fn get_unchecked<I>(&self, index: I) -> BufferView<I::Output>
    where
        I: BufferViewSliceIndex<T>,
    {
        index.get_unchecked(self)
    }
}

impl<'a, T> BufferView<'a, [T]>
where
    T: Copy,
{
    /// Returns a command which, when executed will replace the elements viewed by this [BufferView]
    /// with the elements in given `data`.
    ///
    /// If the `data` contains fewer elements than the slice viewed by this [BufferView], then only
    /// the first `N` elements will be replaced, where `N` is the number of elements in the given
    /// `data`.
    ///
    /// If the `data` contains more elements than the slice viewed by this [Buffer], then only the
    /// first `M` elements in the `data` will be used to update this [Buffer], where `M` is the
    /// number of elements in the slice viewed by the [BufferView].
    ///
    /// This will modify the viewed [Buffer], the buffer (and any other views on the same data) will
    /// be affected by this change.
    pub fn upload_command<D>(&self, data: D) -> UploadCommand<[T], D>
    where
        D: Borrow<[T]> + Send + Sync + 'static,
    {
        UploadCommand {
            buffer_data: self.buffer.data.clone(),
            data,
            offset_in_bytes: self.offset_in_bytes,
            len: self.len,
            _marker: marker::PhantomData,
        }
    }

    /// Returns a command which, when executed will copy the elements viewed by in this [BufferView]
    /// into a [Box].
    ///
    /// When the task is finished, the [Box] containing the copied elements will be output.
    pub fn download_command(&self) -> DownloadCommand<[T]> {
        DownloadCommand {
            data: self.buffer.data.clone(),
            state: DownloadState::Initial,
            offset_in_bytes: self.offset_in_bytes,
            len: self.len,
            _marker: marker::PhantomData,
        }
    }
}

impl<'a, T> BufferView<'a, [MaybeUninit<T>]> {
    /// Converts to `BufferView<[T]>`.
    ///
    /// # Safety
    ///
    /// Its up to the user to guarantee that any tasks that read buffer region viewed by this view,
    /// is only executed after the viewed region is initialized. Note that certain tasks may wait on
    /// GPU fences and allow a runtime to progress other tasks while its waiting on the fence. As
    /// such, submitting your initialization tasks as part of a task that includes fencing (these
    /// are typically tasks that include "download" commands), may not guarantee that the buffer was
    /// initialized before any tasks that are submitted later will begin executing.
    pub unsafe fn assume_init(self) -> BufferView<'a, [T]> {
        mem::transmute(self)
    }
}

impl<'a, T> Clone for BufferView<'a, [T]> {
    fn clone(&self) -> Self {
        BufferView {
            buffer: self.buffer,
            offset_in_bytes: self.offset_in_bytes,
            len: self.len,
        }
    }
}

impl<'a, T> Copy for BufferView<'a, [T]> {}

pub struct BufferViewMut<'a, T>
where
    T: ?Sized,
{
    inner: BufferView<'a, T>,
    _marker: marker::PhantomData<&'a mut Buffer<T>>,
}

impl<'a, T> BufferViewMut<'a, [T]> {
    /// Returns a [BufferViewMut] on an element or a sub-slice of the elements this [BufferViewMut],
    /// depending on the type of `index`.
    ///
    /// - If given a position, returns a view on the element at that position or `None` if out of
    ///   bounds.
    /// - If given a range, returns a view on the sub-slice of elements corresponding to that range,
    ///   or `None` if out of bounds.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use web_glitz::runtime::RenderingContext;
    /// # fn wrapper<Rc>(context: &Rc) where Rc: RenderingContext + Clone + 'static {
    /// use web_glitz::buffer::{Buffer, BufferViewMut, UsageHint};
    ///
    /// let mut buffer: Buffer<[f32]> = context.create_buffer([1.0, 2.0, 3.0, 4.0], UsageHint::StreamDraw);
    /// let mut view = BufferViewMut::from(&mut buffer);
    ///
    /// view.get_mut(1); // Some BufferViewMut<f32> containing `2.0`
    /// view.get_mut(1..3); // Some BufferViewMut<[f32]> containing `[2.0, 3.0]`
    /// view.get_mut(..2); // Some BufferViewMut<[f32]> containing `[1.0 2.0]`
    /// view.get_mut(4); // None (index out of bounds)
    /// # }
    /// ```
    pub fn get_mut<I>(&mut self, index: I) -> Option<BufferViewMut<I::Output>>
    where
        I: BufferViewMutSliceIndex<T>,
    {
        index.get_mut(self)
    }

    /// Returns a [BufferViewMut] on an element or a sub-slice of the elements this [BufferViewMut],
    /// depending on the type of `index`, without doing bounds checking.
    ///
    /// - If given a position, returns a view on the element at that position, without doing bounds
    ///   checking.
    /// - If given a range, returns a view on the slice of elements corresponding to that range,
    ///   without doing bounds checking.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use web_glitz::runtime::RenderingContext;
    /// # fn wrapper<Rc>(context: &Rc) where Rc: RenderingContext + Clone + 'static {
    /// use web_glitz::buffer::{Buffer, BufferViewMut, UsageHint};
    ///
    /// let mut buffer: Buffer<[f32]> = context.create_buffer([1.0, 2.0, 3.0, 4.0], UsageHint::StreamDraw);
    /// let mut view = BufferViewMut::from(&mut buffer);
    ///
    /// unsafe { view.get_unchecked_mut(1) }; // BufferViewMut<f32> containing `2.0`
    /// # }
    /// ```
    ///
    /// # Unsafe
    ///
    /// Only safe if `index` is in bounds. See [get_mut] for a safe alternative.
    pub unsafe fn get_unchecked_mut<I>(&mut self, index: I) -> BufferViewMut<I::Output>
    where
        I: BufferViewMutSliceIndex<T>,
    {
        index.get_unchecked_mut(self)
    }
}

impl<'a, T> Deref for BufferViewMut<'a, T>
where
    T: ?Sized,
{
    type Target = BufferView<'a, T>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

// TODO: CoerceUnsized doesn't currently work with only a PhantomData field...
//impl<'a, T, U> CoerceUnsized<BufferView<'a, U>> for BufferView<'a, T>
//where
//    T: Unsize<U> + ?Sized,
//    U: ?Sized,
//{}

pub struct BufferId {
    pub(crate) object_id: u64,
}

/// Trait implemented for types that represent or contain data that may be stored in a [Buffer].
///
/// Uploading data to a buffer involves doing a bitwise copy, as does downloading data from a
/// buffer. WebGlitz relies on the semantics associated with the `Copy` trait to ensure that this
/// is safe and does not result in memory leaks.
pub trait IntoBuffer<T>
where
    T: ?Sized,
{
    /// Stores the data in a buffer belonging to the given `context`, using the given `usage_hint`.
    ///
    /// This consumes the Rust value and produces a GPU-accessible [Buffer] containing a bitwise
    /// copy of data.
    ///
    /// The usage hint may be used by the GPU driver for performance optimizations, see [UsageHint]
    /// for details.
    fn into_buffer<Rc>(self, context: &Rc, buffer_id: BufferId, usage_hint: UsageHint) -> Buffer<T>
    where
        Rc: RenderingContext + Clone + 'static;
}

impl<D, T> IntoBuffer<T> for D
where
    D: Borrow<T> + 'static,
    T: Copy + 'static,
{
    fn into_buffer<Rc>(self, context: &Rc, buffer_id: BufferId, usage_hint: UsageHint) -> Buffer<T>
    where
        Rc: RenderingContext + Clone + 'static,
    {
        let data = Arc::new(BufferData {
            id: UnsafeCell::new(None),
            context_id: context.id(),
            dropper: Box::new(context.clone()),
            usage_hint,
            len: 1,
        });

        context.submit(AllocateCommand {
            data: data.clone(),
            initial: self,
            _marker: marker::PhantomData,
        });

        Buffer {
            object_id: buffer_id.object_id,
            data,
            _marker: marker::PhantomData,
        }
    }
}

impl<D, T> IntoBuffer<[T]> for D
where
    D: Borrow<[T]> + 'static,
    T: Copy + 'static,
{
    fn into_buffer<Rc>(
        self,
        context: &Rc,
        buffer_id: BufferId,
        usage_hint: UsageHint,
    ) -> Buffer<[T]>
    where
        Rc: RenderingContext + Clone + 'static,
    {
        let len = self.borrow().len();
        let data = Arc::new(BufferData {
            id: UnsafeCell::new(None),
            context_id: context.id(),
            dropper: Box::new(context.clone()),
            usage_hint,
            len,
        });

        context.submit(AllocateCommand::<D, [T]> {
            data: data.clone(),
            initial: self,
            _marker: marker::PhantomData,
        });

        Buffer {
            object_id: buffer_id.object_id,
            data,
            _marker: marker::PhantomData,
        }
    }
}

/// Enumerates the available usage hint for [Buffer]s.
///
/// A usage hint may be used to indicate to the GPU driver how you intend to use the data in the
/// [Buffer]. The driver may use this information for performance optimizations.
///
/// Note that this is merely a performance hint: it does not affect what you can or cannot do with
/// the [Buffer].
#[derive(Clone, Copy, Debug)]
pub enum UsageHint {
    /// Hints that the data will be uploaded once and read by the GPU repeatedly.
    StaticDraw,

    /// Hints that the data will be uploaded repeatedly and read by the GPU repeatedly.
    DynamicDraw,

    /// Hints that the data will be uploaded once and read by the GPU at most a few times.
    StreamDraw,

    /// Hints that the data will be written by the GPU once and will be downloaded repeatedly.
    StaticRead,

    /// Hints that the data will be written by the GPU repeatedly and will be downloaded repeatedly.
    DynamicRead,

    /// Hints that the data will be written by the GPU once and will be downloaded at most a few
    /// times.
    StreamRead,

    /// Hints that the data will be written by the GPU once and read by the GPU repeatedly.
    StaticCopy,

    /// Hints that the data will be written by the GPU repeatedly and read by the GPU repeatedly.
    DynamicCopy,

    /// Hints that the data will be written once by the GPU and read by the GPU at most a few times.
    StreamCopy,
}

impl UsageHint {
    pub(crate) fn gl_id(&self) -> u32 {
        match self {
            UsageHint::StaticDraw => GL::STATIC_DRAW,
            UsageHint::DynamicDraw => GL::DYNAMIC_DRAW,
            UsageHint::StreamDraw => GL::STREAM_DRAW,
            UsageHint::StaticRead => GL::STATIC_READ,
            UsageHint::DynamicRead => GL::DYNAMIC_READ,
            UsageHint::StreamRead => GL::STREAM_READ,
            UsageHint::StaticCopy => GL::STATIC_COPY,
            UsageHint::DynamicCopy => GL::DYNAMIC_COPY,
            UsageHint::StreamCopy => GL::STREAM_COPY,
        }
    }
}

/// A helper trait type for indexing operations on a [Buffer] that contains a slice.
pub trait BufferSliceIndex<T>: Sized {
    /// The output type returned by the indexing operations.
    type Output: ?Sized;

    /// Returns a view on the output for this operation if in bounds, or `None` otherwise.
    fn get(self, buffer: &Buffer<[T]>) -> Option<BufferView<Self::Output>>;

    /// Returns a view on the output for this operation, without performing any bounds checking.
    unsafe fn get_unchecked(self, buffer: &Buffer<[T]>) -> BufferView<Self::Output>;

    /// Returns a mutable view on the output for this operation if in bounds, or `None` otherwise.
    fn get_mut(self, buffer: &mut Buffer<[T]>) -> Option<BufferViewMut<Self::Output>> {
        self.get(buffer).map(|view| BufferViewMut {
            inner: view,
            _marker: marker::PhantomData,
        })
    }

    /// Returns a mutable view the output for this operation, without performing any bounds
    /// checking.
    unsafe fn get_unchecked_mut(self, buffer: &mut Buffer<[T]>) -> BufferViewMut<Self::Output> {
        BufferViewMut {
            inner: self.get_unchecked(buffer),
            _marker: marker::PhantomData,
        }
    }
}

impl<T> BufferSliceIndex<T> for usize {
    type Output = T;

    fn get(self, buffer: &Buffer<[T]>) -> Option<BufferView<Self::Output>> {
        if self < buffer.data.len {
            Some(BufferView {
                buffer: unsafe { mem::transmute(buffer) },
                offset_in_bytes: self * mem::size_of::<T>(),
                len: 1,
            })
        } else {
            None
        }
    }

    unsafe fn get_unchecked(self, buffer: &Buffer<[T]>) -> BufferView<Self::Output> {
        BufferView {
            buffer: mem::transmute(buffer),
            offset_in_bytes: self * mem::size_of::<T>(),
            len: 1,
        }
    }
}

impl<T> BufferSliceIndex<T> for RangeFull {
    type Output = [T];

    fn get(self, buffer: &Buffer<[T]>) -> Option<BufferView<Self::Output>> {
        Some(BufferView {
            buffer,
            offset_in_bytes: 0,
            len: buffer.data.len,
        })
    }

    unsafe fn get_unchecked(self, buffer: &Buffer<[T]>) -> BufferView<Self::Output> {
        BufferView {
            buffer,
            offset_in_bytes: 0,
            len: buffer.data.len,
        }
    }
}

impl<T> BufferSliceIndex<T> for Range<usize> {
    type Output = [T];

    fn get(self, buffer: &Buffer<[T]>) -> Option<BufferView<Self::Output>> {
        let Range { start, end } = self;

        if start > end || end > buffer.data.len {
            None
        } else {
            Some(BufferView {
                buffer,
                offset_in_bytes: start * mem::size_of::<T>(),
                len: end - start,
            })
        }
    }

    unsafe fn get_unchecked(self, buffer: &Buffer<[T]>) -> BufferView<Self::Output> {
        BufferView {
            buffer,
            offset_in_bytes: self.start * mem::size_of::<T>(),
            len: self.end - self.start,
        }
    }
}

impl<T> BufferSliceIndex<T> for RangeInclusive<usize> {
    type Output = [T];

    fn get(self, buffer: &Buffer<[T]>) -> Option<BufferView<Self::Output>> {
        if *self.end() == usize::max_value() {
            None
        } else {
            buffer.get(*self.start()..self.end() + 1)
        }
    }

    unsafe fn get_unchecked(self, buffer: &Buffer<[T]>) -> BufferView<Self::Output> {
        buffer.get_unchecked(*self.start()..self.end() + 1)
    }
}

impl<T> BufferSliceIndex<T> for RangeFrom<usize> {
    type Output = [T];

    fn get(self, buffer: &Buffer<[T]>) -> Option<BufferView<Self::Output>> {
        buffer.get(self.start..buffer.data.len)
    }

    unsafe fn get_unchecked(self, buffer: &Buffer<[T]>) -> BufferView<Self::Output> {
        buffer.get_unchecked(self.start..buffer.data.len)
    }
}

impl<T> BufferSliceIndex<T> for RangeTo<usize> {
    type Output = [T];

    fn get(self, buffer: &Buffer<[T]>) -> Option<BufferView<Self::Output>> {
        buffer.get(0..self.end)
    }

    unsafe fn get_unchecked(self, buffer: &Buffer<[T]>) -> BufferView<Self::Output> {
        buffer.get_unchecked(0..self.end)
    }
}

impl<T> BufferSliceIndex<T> for RangeToInclusive<usize> {
    type Output = [T];

    fn get(self, buffer: &Buffer<[T]>) -> Option<BufferView<Self::Output>> {
        buffer.get(0..=self.end)
    }

    unsafe fn get_unchecked(self, buffer: &Buffer<[T]>) -> BufferView<Self::Output> {
        buffer.get_unchecked(0..=self.end)
    }
}

/// A helper trait type for indexing operations on a [BufferView] that contains a slice.
pub trait BufferViewSliceIndex<T>: Sized {
    /// The output type returned by the indexing operations.
    type Output: ?Sized;

    /// Returns a view on the output for this operation if in bounds, or `None` otherwise.
    fn get<'a>(self, view: &'a BufferView<[T]>) -> Option<BufferView<'a, Self::Output>>;

    /// Returns a view on the output for this operation, without performing any bounds checking.
    unsafe fn get_unchecked<'a>(self, view: &'a BufferView<[T]>) -> BufferView<'a, Self::Output>;
}

impl<T> BufferViewSliceIndex<T> for usize {
    type Output = T;

    fn get<'a>(self, view: &'a BufferView<[T]>) -> Option<BufferView<'a, Self::Output>> {
        if self < view.len {
            Some(BufferView {
                buffer: unsafe { mem::transmute(view.buffer) },
                offset_in_bytes: view.offset_in_bytes + self * mem::size_of::<T>(),
                len: 1,
            })
        } else {
            None
        }
    }

    unsafe fn get_unchecked<'a>(self, view: &'a BufferView<[T]>) -> BufferView<'a, Self::Output> {
        BufferView {
            buffer: mem::transmute(view.buffer),
            offset_in_bytes: view.offset_in_bytes + self * mem::size_of::<T>(),
            len: 1,
        }
    }
}

impl<T> BufferViewSliceIndex<T> for RangeFull {
    type Output = [T];

    fn get<'a>(self, view: &'a BufferView<[T]>) -> Option<BufferView<'a, Self::Output>> {
        Some(BufferView {
            buffer: view.buffer,
            offset_in_bytes: view.offset_in_bytes,
            len: view.len,
        })
    }

    unsafe fn get_unchecked<'a>(self, view: &'a BufferView<[T]>) -> BufferView<'a, Self::Output> {
        BufferView {
            buffer: view.buffer,
            offset_in_bytes: view.offset_in_bytes,
            len: view.len,
        }
    }
}

impl<T> BufferViewSliceIndex<T> for Range<usize> {
    type Output = [T];

    fn get<'a>(self, view: &'a BufferView<[T]>) -> Option<BufferView<'a, Self::Output>> {
        let Range { start, end } = self;

        if start > end || end > view.len {
            None
        } else {
            Some(BufferView {
                buffer: view.buffer,
                offset_in_bytes: view.offset_in_bytes + start * mem::size_of::<T>(),
                len: end - start,
            })
        }
    }

    unsafe fn get_unchecked<'a>(self, view: &'a BufferView<[T]>) -> BufferView<'a, Self::Output> {
        BufferView {
            buffer: view.buffer,
            offset_in_bytes: view.offset_in_bytes + self.start * mem::size_of::<T>(),
            len: self.end - self.start,
        }
    }
}

impl<T> BufferViewSliceIndex<T> for RangeInclusive<usize> {
    type Output = [T];

    fn get<'a>(self, view: &'a BufferView<[T]>) -> Option<BufferView<'a, Self::Output>> {
        if *self.end() == usize::max_value() {
            None
        } else {
            view.get(*self.start()..self.end() + 1)
        }
    }

    unsafe fn get_unchecked<'a>(self, view: &'a BufferView<[T]>) -> BufferView<'a, Self::Output> {
        view.get_unchecked(*self.start()..self.end() + 1)
    }
}

impl<T> BufferViewSliceIndex<T> for RangeFrom<usize> {
    type Output = [T];

    fn get<'a>(self, view: &'a BufferView<[T]>) -> Option<BufferView<'a, Self::Output>> {
        view.get(self.start..view.len)
    }

    unsafe fn get_unchecked<'a>(self, view: &'a BufferView<[T]>) -> BufferView<'a, Self::Output> {
        view.get_unchecked(self.start..view.len)
    }
}

impl<T> BufferViewSliceIndex<T> for RangeTo<usize> {
    type Output = [T];

    fn get<'a>(self, view: &'a BufferView<[T]>) -> Option<BufferView<'a, Self::Output>> {
        view.get(0..self.end)
    }

    unsafe fn get_unchecked<'a>(self, view: &'a BufferView<[T]>) -> BufferView<'a, Self::Output> {
        view.get_unchecked(0..self.end)
    }
}

impl<T> BufferViewSliceIndex<T> for RangeToInclusive<usize> {
    type Output = [T];

    fn get<'a>(self, view: &'a BufferView<[T]>) -> Option<BufferView<'a, Self::Output>> {
        view.get(0..=self.end)
    }

    unsafe fn get_unchecked<'a>(self, view: &'a BufferView<[T]>) -> BufferView<'a, Self::Output> {
        view.get_unchecked(0..=self.end)
    }
}

/// A helper trait type for indexing operations on a [BufferViewMut] that contains a slice.
pub trait BufferViewMutSliceIndex<T>: BufferViewSliceIndex<T> {
    /// Returns a mutable view on the output for this operation if in bounds, or `None` otherwise.
    fn get_mut<'a>(
        self,
        view: &'a mut BufferViewMut<[T]>,
    ) -> Option<BufferViewMut<'a, Self::Output>> {
        self.get(view).map(|view| BufferViewMut {
            inner: view,
            _marker: marker::PhantomData,
        })
    }

    /// Returns a mutable view the output for this operation, without performing any bounds
    /// checking.
    unsafe fn get_unchecked_mut<'a>(
        self,
        view: &'a mut BufferViewMut<[T]>,
    ) -> BufferViewMut<'a, Self::Output> {
        BufferViewMut {
            inner: self.get_unchecked(view),
            _marker: marker::PhantomData,
        }
    }
}

impl<T> BufferViewMutSliceIndex<T> for usize {}

impl<T> BufferViewMutSliceIndex<T> for RangeFull {}

impl<T> BufferViewMutSliceIndex<T> for Range<usize> {}

impl<T> BufferViewMutSliceIndex<T> for RangeInclusive<usize> {}

impl<T> BufferViewMutSliceIndex<T> for RangeFrom<usize> {}

impl<T> BufferViewMutSliceIndex<T> for RangeTo<usize> {}

impl<T> BufferViewMutSliceIndex<T> for RangeToInclusive<usize> {}

/// Command for uploading data to a [Buffer] or a sub-section of a buffer as viewed by a
/// [BufferView].
///
/// See [Buffer::upload_command] and [BufferView::upload_command] for details.
pub struct UploadCommand<T, D>
where
    T: ?Sized,
{
    buffer_data: Arc<BufferData>,
    data: D,
    offset_in_bytes: usize,
    len: usize,
    _marker: marker::PhantomData<T>,
}

unsafe impl<T, D> GpuTask<Connection> for UploadCommand<T, D>
where
    D: Borrow<T>,
{
    type Output = ();

    fn context_id(&self) -> ContextId {
        ContextId::Id(self.buffer_data.context_id)
    }

    fn progress(&mut self, connection: &mut Connection) -> Progress<Self::Output> {
        let (gl, state) = unsafe { connection.unpack_mut() };

        unsafe {
            self.buffer_data
                .id()
                .unwrap()
                .with_value_unchecked(|buffer_object| {
                    state
                        .bind_copy_write_buffer(Some(&buffer_object))
                        .apply(gl)
                        .unwrap();
                });
        }

        unsafe {
            let data = slice::from_raw_parts(
                self.data.borrow() as *const _ as *const u8,
                mem::size_of::<T>(),
            );

            gl.buffer_sub_data_with_i32_and_u8_array(
                GL::COPY_WRITE_BUFFER,
                self.offset_in_bytes as i32,
                data,
            );
        };

        Progress::Finished(())
    }
}

unsafe impl<T, D> GpuTask<Connection> for UploadCommand<[T], D>
where
    D: Borrow<[T]>,
{
    type Output = ();

    fn context_id(&self) -> ContextId {
        ContextId::Id(self.buffer_data.context_id)
    }

    fn progress(&mut self, connection: &mut Connection) -> Progress<Self::Output> {
        let (gl, state) = unsafe { connection.unpack_mut() };

        unsafe {
            self.buffer_data
                .id()
                .unwrap()
                .with_value_unchecked(|buffer_object| {
                    state
                        .bind_copy_write_buffer(Some(&buffer_object))
                        .apply(gl)
                        .unwrap();
                });
        }

        let data = self.data.borrow();
        let size = data.len() * mem::size_of::<T>();
        let max_size = self.len * mem::size_of::<T>();

        unsafe {
            let mut data = slice::from_raw_parts(self.data.borrow() as *const _ as *const u8, size);

            if max_size < size {
                data = &data[0..max_size];
            }

            gl.buffer_sub_data_with_i32_and_u8_array(
                GL::COPY_WRITE_BUFFER,
                self.offset_in_bytes as i32,
                data,
            );
        };

        Progress::Finished(())
    }
}

/// Command for downloading data from a [Buffer] or a sub-section of a buffer as viewed by a
/// [BufferView].
///
/// See [Buffer::download_command] and [BufferView::download_command] for details.
pub struct DownloadCommand<T>
where
    T: ?Sized,
{
    data: Arc<BufferData>,
    state: DownloadState,
    offset_in_bytes: usize,
    len: usize,
    _marker: marker::PhantomData<T>,
}

enum DownloadState {
    Initial,
    Copied(Option<WebGlBuffer>),
}

unsafe impl<T> GpuTask<Connection> for DownloadCommand<T> {
    type Output = Box<T>;

    fn context_id(&self) -> ContextId {
        ContextId::Id(self.data.context_id)
    }

    fn progress(&mut self, connection: &mut Connection) -> Progress<Self::Output> {
        match self.state {
            DownloadState::Initial => {
                let (gl, state) = unsafe { connection.unpack_mut() };
                let read_buffer = GL::create_buffer(&gl).unwrap();
                let size_in_bytes = mem::size_of::<T>();

                state
                    .bind_copy_write_buffer(Some(&read_buffer))
                    .apply(gl)
                    .unwrap();

                gl.buffer_data_with_i32(
                    GL::COPY_WRITE_BUFFER,
                    size_in_bytes as i32,
                    GL::STREAM_READ,
                );

                unsafe {
                    self.data
                        .id()
                        .unwrap()
                        .with_value_unchecked(|buffer_object| {
                            state
                                .bind_copy_read_buffer(Some(&buffer_object))
                                .apply(gl)
                                .unwrap();
                        });
                }

                gl.copy_buffer_sub_data_with_i32_and_i32_and_i32(
                    GL::COPY_READ_BUFFER,
                    GL::COPY_WRITE_BUFFER,
                    self.offset_in_bytes as i32,
                    0,
                    size_in_bytes as i32,
                );

                self.state = DownloadState::Copied(Some(read_buffer));

                Progress::ContinueFenced
            }
            DownloadState::Copied(ref mut read_buffer) => {
                let read_buffer = read_buffer
                    .take()
                    .expect("Cannot make progress on a BufferDownload task after it has finished");
                let (gl, state) = unsafe { connection.unpack_mut() };

                state
                    .bind_copy_read_buffer(Some(&read_buffer))
                    .apply(gl)
                    .unwrap();

                let size_in_bytes = self.len * mem::size_of::<T>();
                let mut data = vec![0; size_in_bytes];

                gl.get_buffer_sub_data_with_i32_and_u8_array(GL::COPY_READ_BUFFER, 0, &mut data);

                gl.delete_buffer(Some(&read_buffer));

                let value = unsafe { Box::from_raw(mem::transmute(data.as_mut_ptr())) };

                mem::forget(data);

                Progress::Finished(value)
            }
        }
    }
}

unsafe impl<T> GpuTask<Connection> for DownloadCommand<[T]> {
    type Output = Box<[T]>;

    fn context_id(&self) -> ContextId {
        ContextId::Id(self.data.context_id)
    }

    fn progress(&mut self, connection: &mut Connection) -> Progress<Self::Output> {
        match self.state {
            DownloadState::Initial => {
                let (gl, state) = unsafe { connection.unpack_mut() };
                let read_buffer = GL::create_buffer(&gl).unwrap();
                let size_in_bytes = self.len * mem::size_of::<T>();

                state
                    .bind_copy_write_buffer(Some(&read_buffer))
                    .apply(gl)
                    .unwrap();

                gl.buffer_data_with_i32(
                    GL::COPY_WRITE_BUFFER,
                    size_in_bytes as i32,
                    GL::STREAM_READ,
                );

                unsafe {
                    self.data
                        .id()
                        .unwrap()
                        .with_value_unchecked(|buffer_object| {
                            state
                                .bind_copy_read_buffer(Some(&buffer_object))
                                .apply(gl)
                                .unwrap();
                        });
                }

                gl.copy_buffer_sub_data_with_i32_and_i32_and_i32(
                    GL::COPY_READ_BUFFER,
                    GL::COPY_WRITE_BUFFER,
                    self.offset_in_bytes as i32,
                    0,
                    size_in_bytes as i32,
                );

                self.state = DownloadState::Copied(Some(read_buffer));

                Progress::ContinueFenced
            }
            DownloadState::Copied(ref mut read_buffer) => {
                let read_buffer = read_buffer
                    .take()
                    .expect("Cannot make progress on a BufferDownload task after it has finished");
                let (gl, state) = unsafe { connection.unpack_mut() };

                state
                    .bind_copy_read_buffer(Some(&read_buffer))
                    .apply(gl)
                    .unwrap();

                let size_in_bytes = self.len * mem::size_of::<T>();
                let mut data = vec![0; size_in_bytes];

                gl.get_buffer_sub_data_with_i32_and_u8_array(GL::COPY_READ_BUFFER, 0, &mut data);

                gl.delete_buffer(Some(&read_buffer));

                unsafe {
                    let ptr = mem::transmute(data.as_mut_ptr());
                    let slice = slice::from_raw_parts_mut(ptr, self.len);
                    let boxed = Box::from_raw(slice);

                    mem::forget(data);

                    Progress::Finished(boxed)
                }
            }
        }
    }
}

trait BufferObjectDropper {
    fn drop_buffer_object(&self, id: JsId);
}

impl<T> BufferObjectDropper for T
where
    T: RenderingContext,
{
    fn drop_buffer_object(&self, id: JsId) {
        self.submit(DropCommand { id });
    }
}

pub(crate) struct BufferData {
    id: UnsafeCell<Option<JsId>>,
    context_id: u64,
    dropper: Box<dyn BufferObjectDropper>,
    len: usize,
    usage_hint: UsageHint,
}

impl BufferData {
    pub(crate) fn id(&self) -> Option<JsId> {
        unsafe { *self.id.get() }
    }

    pub(crate) fn context_id(&self) -> u64 {
        self.context_id
    }
}

impl Drop for BufferData {
    fn drop(&mut self) {
        if let Some(id) = self.id() {
            self.dropper.drop_buffer_object(id);
        }
    }
}

struct AllocateUninitCommand<T>
where
    T: ?Sized,
{
    data: Arc<BufferData>,
    _marker: marker::PhantomData<T>,
}

unsafe impl<T> GpuTask<Connection> for AllocateUninitCommand<T> {
    type Output = ();

    fn context_id(&self) -> ContextId {
        ContextId::Any
    }

    fn progress(&mut self, connection: &mut Connection) -> Progress<Self::Output> {
        let (gl, state) = unsafe { connection.unpack_mut() };
        let data = &self.data;

        let buffer_object = GL::create_buffer(&gl).unwrap();

        state
            .bind_copy_write_buffer(Some(&buffer_object))
            .apply(gl)
            .unwrap();

        let size = mem::size_of::<T>();

        gl.buffer_data_with_i32(GL::COPY_WRITE_BUFFER, size as i32, data.usage_hint.gl_id());

        unsafe {
            *data.id.get() = Some(JsId::from_value(buffer_object.into()));
        }

        Progress::Finished(())
    }
}

unsafe impl<T> GpuTask<Connection> for AllocateUninitCommand<[T]> {
    type Output = ();

    fn context_id(&self) -> ContextId {
        ContextId::Any
    }

    fn progress(&mut self, connection: &mut Connection) -> Progress<Self::Output> {
        let (gl, state) = unsafe { connection.unpack_mut() };
        let data = &self.data;

        let buffer_object = GL::create_buffer(&gl).unwrap();

        state
            .bind_copy_write_buffer(Some(&buffer_object))
            .apply(gl)
            .unwrap();

        let size = mem::size_of::<T>() * data.len;

        gl.buffer_data_with_i32(GL::COPY_WRITE_BUFFER, size as i32, data.usage_hint.gl_id());

        unsafe {
            *data.id.get() = Some(JsId::from_value(buffer_object.into()));
        }

        Progress::Finished(())
    }
}

struct AllocateCommand<D, T>
where
    T: ?Sized,
{
    data: Arc<BufferData>,
    initial: D,
    _marker: marker::PhantomData<T>,
}

unsafe impl<D, T> GpuTask<Connection> for AllocateCommand<D, T>
where
    D: Borrow<T>,
{
    type Output = ();

    fn context_id(&self) -> ContextId {
        ContextId::Any
    }

    fn progress(&mut self, connection: &mut Connection) -> Progress<Self::Output> {
        let (gl, state) = unsafe { connection.unpack_mut() };
        let data = &self.data;

        let buffer_object = GL::create_buffer(&gl).unwrap();

        state
            .bind_copy_write_buffer(Some(&buffer_object))
            .apply(gl)
            .unwrap();

        unsafe {
            let initial = slice::from_raw_parts(
                self.initial.borrow() as *const _ as *const u8,
                mem::size_of::<T>(),
            );

            gl.buffer_data_with_u8_array(GL::COPY_WRITE_BUFFER, initial, data.usage_hint.gl_id());
        }

        unsafe {
            *data.id.get() = Some(JsId::from_value(buffer_object.into()));
        }

        Progress::Finished(())
    }
}

unsafe impl<D, T> GpuTask<Connection> for AllocateCommand<D, [T]>
where
    D: Borrow<[T]>,
{
    type Output = ();

    fn context_id(&self) -> ContextId {
        ContextId::Any
    }

    fn progress(&mut self, connection: &mut Connection) -> Progress<Self::Output> {
        let (gl, state) = unsafe { connection.unpack_mut() };
        let data = &self.data;

        let buffer_object = GL::create_buffer(&gl).unwrap();

        state
            .bind_copy_write_buffer(Some(&buffer_object))
            .apply(gl)
            .unwrap();

        unsafe {
            let initial = self.initial.borrow();
            let size = initial.len() * mem::size_of::<T>();
            let initial = slice::from_raw_parts(initial as *const _ as *const u8, size);

            gl.buffer_data_with_u8_array(GL::COPY_WRITE_BUFFER, initial, data.usage_hint.gl_id());
        }

        unsafe {
            *data.id.get() = Some(JsId::from_value(buffer_object.into()));
        }

        Progress::Finished(())
    }
}

struct DropCommand {
    id: JsId,
}

unsafe impl GpuTask<Connection> for DropCommand {
    type Output = ();

    fn context_id(&self) -> ContextId {
        ContextId::Any
    }

    fn progress(&mut self, connection: &mut Connection) -> Progress<Self::Output> {
        let (gl, state) = unsafe { connection.unpack_mut() };

        state
            .vertex_array_cache_mut()
            .remove_buffer_dependents(self.id, gl);

        let value = unsafe { JsId::into_value(self.id).unchecked_into() };

        state.unref_buffer(&value);
        gl.delete_buffer(Some(&value));

        Progress::Finished(())
    }
}
