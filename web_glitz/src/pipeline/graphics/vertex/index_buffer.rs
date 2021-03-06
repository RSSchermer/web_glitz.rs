use std::borrow::Borrow;
use std::cell::UnsafeCell;
use std::hash::{Hash, Hasher};
use std::marker;
use std::mem;
use std::ops::{Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive};
use std::slice;
use std::sync::Arc;

use wasm_bindgen::JsCast;
use web_sys::WebGl2RenderingContext as Gl;

use crate::buffer::UsageHint;
use crate::runtime::state::ContextUpdate;
use crate::runtime::{Connection, RenderingContext};
use crate::task::{ContextId, GpuTask, Progress};
use crate::util::JsId;
use std::mem::MaybeUninit;

/// Describes a data source that can be used to provide indexing data to a draw command.
pub trait IndexData {
    /// Returns a descriptor of the index data.
    fn descriptor(&self) -> IndexDataDescriptor;
}

impl<'a, T> IndexData for &'a IndexBuffer<T>
where
    T: IndexFormat,
{
    fn descriptor(&self) -> IndexDataDescriptor {
        IndexDataDescriptor {
            buffer_data: self.data.clone(),
            index_type: T::TYPE,
            offset: 0,
            len: self.len() as u32,
        }
    }
}

impl<'a, T> IndexData for IndexBufferView<'a, T>
where
    T: IndexFormat,
{
    fn descriptor(&self) -> IndexDataDescriptor {
        IndexDataDescriptor {
            buffer_data: self.buffer_data().clone(),
            index_type: T::TYPE,
            offset: self.offset_in_bytes() as u32,
            len: self.len() as u32,
        }
    }
}

/// Trait implemented for types that can be used as indices for a [VertexArray] encoded in the
/// associated [IndexType].
pub unsafe trait IndexFormat: Copy {
    /// The [IndexType] associated with this [IndexFormat].
    const TYPE: IndexType;
}

unsafe impl IndexFormat for u8 {
    const TYPE: IndexType = IndexType::UnsignedByte;
}

unsafe impl IndexFormat for u16 {
    const TYPE: IndexType = IndexType::UnsignedShort;
}

unsafe impl IndexFormat for u32 {
    const TYPE: IndexType = IndexType::UnsignedInt;
}

/// Describes an [IndexBuffer] region that contains data that may be used to index a [VertexArray].
///
/// See also [IndexData].
#[derive(Clone)]
pub struct IndexDataDescriptor {
    pub(crate) buffer_data: Arc<IndexBufferData>,
    pub(crate) index_type: IndexType,
    pub(crate) offset: u32,
    pub(crate) len: u32,
}

impl Hash for IndexDataDescriptor {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.buffer_data.id().unwrap().hash(state);
        self.index_type.hash(state);
        self.offset.hash(state);
        self.len.hash(state);
    }
}

/// Enumerates the available type encodings for [VertexArray] indices.
#[derive(Clone, Copy, PartialEq, Hash, Debug)]
pub enum IndexType {
    UnsignedByte,
    UnsignedShort,
    UnsignedInt,
}

impl IndexType {
    pub(crate) fn id(&self) -> u32 {
        match self {
            IndexType::UnsignedByte => Gl::UNSIGNED_BYTE,
            IndexType::UnsignedShort => Gl::UNSIGNED_SHORT,
            IndexType::UnsignedInt => Gl::UNSIGNED_INT,
        }
    }
}

/// A GPU-accessible memory buffer that contains an indexed list for indexed drawing.
///
/// Very similar to a [Buffer] but exclusively for data used for indexed drawing. For security
/// reasons, WebGL implementations require that index buffer never reference out of bounds vertices.
/// To ensure this, the browser will perform additional checks on the indices in index buffers. To
/// ensure that these checks needs only be done for index buffers (and not all buffers), a dichotomy
/// was introduced that treats index buffers differently from buffers intended for different uses
/// (e.g. a vertex data buffer, a uniform buffer, or a transform feedback buffer), where an index
/// buffers may only ever be bound as an index buffer, never as a different kind of buffer, whereas
/// other buffer kinds may be used for multiple ends. Additionally, index buffers are not allowed to
/// perform GPU copy commands; only upload commands are permitted, where after each upload the
/// browser will have to reevaluate the range of indices contained in the buffer (or buffer
/// sub-region) when it is next used to draw. In short, for index data use an [IndexBuffer]; for all
/// other ends use an ordinary [Buffer].
///
/// # Example
///
/// ```rust
/// # use web_glitz::runtime::RenderingContext;
/// # fn wrapper<Rc>(context: &Rc) where Rc: RenderingContext + Clone + 'static {
/// use web_glitz::pipeline::graphics::IndexBuffer;
/// use web_glitz::buffer::UsageHint;
///
/// let index_buffer: IndexBuffer<u16> = context.create_index_buffer([1, 2, 3, 4], UsageHint::StreamDraw);
/// # }
/// ```
pub struct IndexBuffer<T> {
    object_id: u64,
    data: Arc<IndexBufferData>,
    _marker: marker::PhantomData<Box<T>>,
}

impl<T> IndexBuffer<T>
where
    T: IndexFormat + 'static,
{
    pub(crate) fn new<Rc, D>(
        context: &Rc,
        object_id: u64,
        data: D,
        usage_hint: UsageHint,
    ) -> IndexBuffer<T>
    where
        Rc: RenderingContext + Clone + 'static,
        D: Borrow<[T]> + 'static,
    {
        let buffer_data = Arc::new(IndexBufferData {
            id: UnsafeCell::new(None),
            context_id: context.id(),
            dropper: Box::new(context.clone()),
            usage_hint,
            len: data.borrow().len(),
        });

        context.submit(AllocateCommand {
            data: buffer_data.clone(),
            initial: data,
            _marker: marker::PhantomData,
        });

        IndexBuffer {
            object_id,
            data: buffer_data,
            _marker: marker::PhantomData,
        }
    }
}

impl<T> IndexBuffer<MaybeUninit<T>>
where
    T: IndexFormat + 'static,
{
    pub(crate) fn new_uninit<Rc>(
        context: &Rc,
        object_id: u64,
        len: usize,
        usage_hint: UsageHint,
    ) -> IndexBuffer<MaybeUninit<T>>
    where
        Rc: RenderingContext + Clone + 'static,
    {
        let buffer_data = Arc::new(IndexBufferData {
            id: UnsafeCell::new(None),
            context_id: context.id(),
            dropper: Box::new(context.clone()),
            usage_hint,
            len,
        });

        let marker: marker::PhantomData<T> = marker::PhantomData;

        context.submit(AllocateUninitCommand {
            data: buffer_data.clone(),
            _marker: marker,
        });

        IndexBuffer {
            object_id,
            data: buffer_data,
            _marker: marker::PhantomData,
        }
    }
}

impl<T> IndexBuffer<T> {
    pub(crate) fn data(&self) -> &Arc<IndexBufferData> {
        &self.data
    }

    /// Returns the [UsageHint] that was specified for this [IndexBuffer] when it was created.
    ///
    /// See [UsageHint] for details.
    pub fn usage_hint(&self) -> UsageHint {
        self.data.usage_hint
    }

    /// Returns the number of indices contained in this [IndexBuffer].
    pub fn len(&self) -> usize {
        self.data.len
    }

    /// Returns an [IndexBufferView] on a slice of the indices this [IndexBuffer] based on the given
    /// `range` or `None` if the range is out of bounds
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use web_glitz::runtime::RenderingContext;
    /// # fn wrapper<Rc>(context: &Rc) where Rc: RenderingContext + Clone + 'static {
    /// use web_glitz::pipeline::graphics::IndexBuffer;
    /// use web_glitz::buffer::UsageHint;
    ///
    /// let index_buffer: IndexBuffer<u16> = context.create_index_buffer([1, 2, 3, 4], UsageHint::StreamDraw);
    ///
    /// index_buffer.get(1..3); // Some IndexBufferView<[f32]> containing `[2.0, 3.0]`
    /// index_buffer.get(..2); // Some IndexBufferView<[f32]> containing `[1.0 2.0]`
    /// # }
    /// ```
    pub fn get<R>(&self, range: R) -> Option<IndexBufferView<T>>
    where
        R: IndexBufferSliceRange<T>,
    {
        range.get(self)
    }

    /// Returns an [IndexBufferView] on a slice of the indices this [IndexBuffer] based on the given
    /// `range`, without performing any bounds checks
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use web_glitz::runtime::RenderingContext;
    /// # fn wrapper<Rc>(context: &Rc) where Rc: RenderingContext + Clone + 'static {
    /// use web_glitz::pipeline::graphics::IndexBuffer;
    /// use web_glitz::buffer::UsageHint;
    ///
    /// let index_buffer: IndexBuffer<u16> = context.create_index_buffer([1, 2, 3, 4], UsageHint::StreamDraw);
    ///
    /// unsafe { index_buffer.get_unchecked(1..3) }; // IndexBufferView<[f32]> containing `[2.0, 3.0]`
    /// # }
    /// ```
    ///
    /// # Unsafe
    ///
    /// Only safe if `range` is in bounds. See [get] for a safe alternative.
    pub unsafe fn get_unchecked<R>(&self, index: R) -> IndexBufferView<T>
    where
        R: IndexBufferSliceRange<T>,
    {
        index.get_unchecked(self)
    }
}

impl<T> IndexBuffer<T>
where
    T: IndexFormat,
{
    /// Returns a command which, when executed will replace the indices contained in this
    /// [IndexBuffer] with the indices in given `index_data`.
    ///
    /// If the `index_data` contains fewer elements than this [IndexBuffer], then only the first `N`
    /// elements will be replaced, where `N` is the number of elements in the given `index_data`.
    ///
    /// If the `index_data` contains more elements than this [IndexBuffer], then only the first `M`
    /// elements in the `index_data` will be used to update this [IndexBuffer], where `M` is the
    /// number of elements in the [IndexBuffer].
    pub fn upload_command<D>(&self, index_data: D) -> IndexBufferUploadCommand<T, D>
    where
        D: Borrow<[T]> + Send + Sync + 'static,
    {
        IndexBufferUploadCommand {
            buffer_data: self.data.clone(),
            index_data,
            offset_in_bytes: 0,
            len: self.data.len,
            _marker: marker::PhantomData,
        }
    }
}

impl<T> IndexBuffer<MaybeUninit<T>>
where
    T: IndexFormat,
{
    /// Returns a command which, when executed will replace the indices contained in this
    /// [IndexBuffer] with the indices in given `index_data`.
    ///
    /// If the `index_data` contains fewer elements than this [IndexBuffer], then only the first `N`
    /// elements will be replaced, where `N` is the number of elements in the given `index_data`.
    ///
    /// If the `index_data` contains more elements than this [IndexBuffer], then only the first `M`
    /// elements in the `index_data` will be used to update this [IndexBuffer], where `M` is the
    /// number of elements in the [IndexBuffer].
    pub fn upload_command<D>(&self, index_data: D) -> IndexBufferUploadCommand<MaybeUninit<T>, D>
    where
        D: Borrow<[MaybeUninit<T>]> + Send + Sync + 'static,
    {
        IndexBufferUploadCommand {
            buffer_data: self.data.clone(),
            index_data,
            offset_in_bytes: 0,
            len: self.data.len,
            _marker: marker::PhantomData,
        }
    }
}

impl<T> IndexBuffer<MaybeUninit<T>>
where
    T: IndexFormat,
{
    /// Converts to `IndexBuffer<T>`.
    ///
    /// # Safety
    ///
    /// Any tasks that read from the buffer after `assume_init` was called, must only be executed
    /// after the buffer was initialized. Note that certain tasks may wait on GPU fences and allow
    /// a runtime to progress other tasks while its waiting on the fence. As such, submitting your
    /// initialization tasks as part of a task that includes fencing (these are typically tasks that
    /// include "download" commands), may not guarantee that the buffer was initialized before any
    /// tasks that are submitted later will begin executing.
    pub unsafe fn assume_init(self) -> IndexBuffer<T> {
        mem::transmute(self)
    }
}

impl<T> PartialEq for IndexBuffer<T> {
    fn eq(&self, other: &Self) -> bool {
        self.object_id == other.object_id
    }
}

impl<T> Hash for IndexBuffer<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.object_id.hash(state);
    }
}

impl<'a, T> From<&'a IndexBuffer<T>> for IndexBufferView<'a, T> {
    fn from(buffer: &'a IndexBuffer<T>) -> IndexBufferView<'a, T> {
        IndexBufferView {
            buffer,
            offset_in_bytes: 0,
            len: buffer.data.len,
        }
    }
}

/// A view on a segment or the whole of an [IndexBuffer].
#[derive(PartialEq, Hash)]
pub struct IndexBufferView<'a, T> {
    buffer: &'a IndexBuffer<T>,
    pub(crate) offset_in_bytes: usize,
    len: usize,
}

impl<'a, T> IndexBufferView<'a, T> {
    pub(crate) fn buffer_data(&self) -> &Arc<IndexBufferData> {
        self.buffer.data()
    }

    pub(crate) fn offset_in_bytes(&self) -> usize {
        self.offset_in_bytes
    }

    /// The size in bytes of the viewed index buffer region.
    pub fn size_in_bytes(&self) -> usize {
        std::mem::size_of::<T>()
    }

    /// Returns the [UsageHint] that was specified for the [IndexBuffer] viewed by this
    /// [IndexBufferView] when it was created.
    ///
    /// See [UsageHint] for details.
    pub fn usage_hint(&self) -> UsageHint {
        self.buffer.data.usage_hint
    }

    /// Returns the number of elements viewed by this [IndexBufferView].
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns an [IndexBufferView] on a slice of the indices this [IndexBuffer] based on the given
    /// `range` or `None` if the range is out of bounds
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use web_glitz::runtime::RenderingContext;
    /// # fn wrapper<Rc>(context: &Rc) where Rc: RenderingContext + Clone + 'static {
    /// use web_glitz::pipeline::graphics::{IndexBuffer, IndexBufferView};
    /// use web_glitz::buffer::UsageHint;
    ///
    /// let index_buffer: IndexBuffer<u16> = context.create_index_buffer([1, 2, 3, 4], UsageHint::StreamDraw);
    /// let view = IndexBufferView::from(&index_buffer);
    ///
    /// view.get(1..3); // Some IndexBufferView<[f32]> containing `[2.0, 3.0]`
    /// view.get(..2); // Some IndexBufferView<[f32]> containing `[1.0 2.0]`
    /// # }
    /// ```
    pub fn get<R>(&self, range: R) -> Option<IndexBufferView<T>>
    where
        R: IndexBufferViewSliceIndex<T>,
    {
        range.get(self)
    }

    /// Returns an [IndexBufferView] on a slice of the indices this [IndexBuffer] based on the given
    /// `range`, without performing any bounds checks
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use web_glitz::runtime::RenderingContext;
    /// # fn wrapper<Rc>(context: &Rc) where Rc: RenderingContext + Clone + 'static {
    /// use web_glitz::pipeline::graphics::{IndexBuffer, IndexBufferView};
    /// use web_glitz::buffer::UsageHint;
    ///
    /// let index_buffer: IndexBuffer<u16> = context.create_index_buffer([1, 2, 3, 4], UsageHint::StreamDraw);
    /// let view = IndexBufferView::from(&index_buffer);
    ///
    /// unsafe { view.get_unchecked(1..3) }; // IndexBufferView<[f32]> containing `[2.0, 3.0]`
    /// # }
    /// ```
    ///
    /// # Unsafe
    ///
    /// Only safe if `range` is in bounds. See [get] for a safe alternative.
    pub unsafe fn get_unchecked<R>(&self, range: R) -> IndexBufferView<T>
    where
        R: IndexBufferViewSliceIndex<T>,
    {
        range.get_unchecked(self)
    }
}

impl<'a, T> IndexBufferView<'a, T>
where
    T: IndexFormat,
{
    /// Returns a command which, when executed will replace the indices viewed contained by this
    /// [IndexBufferView] with the indices in given `index_data`.
    ///
    /// If the `index_data` contains fewer elements than the slice viewed by this [IndexBufferView],
    /// then only the first `N` elements will be replaced, where `N` is the number of elements in
    /// the given `index_data`.
    ///
    /// If the `index_data` contains more elements than the slice viewed this [IndexBufferView],
    /// then only the first `M` elements in the `index_data` will be used to update the index
    /// buffer, where `M` is the number of elements viewed by the [IndexBufferView].
    ///
    /// This will modify the viewed [IndexBuffer], the buffer (and any other views on the same data)
    /// will be affected by this change.
    pub fn upload_command<D>(&self, data: D) -> IndexBufferUploadCommand<T, D>
    where
        D: Borrow<[T]> + Send + Sync + 'static,
    {
        IndexBufferUploadCommand {
            buffer_data: self.buffer.data.clone(),
            index_data: data,
            offset_in_bytes: self.offset_in_bytes,
            len: self.len,
            _marker: marker::PhantomData,
        }
    }
}

impl<'a, T> IndexBufferView<'a, MaybeUninit<T>>
where
    T: IndexFormat,
{
    /// Returns a command which, when executed will replace the indices viewed contained by this
    /// [IndexBufferView] with the indices in given `data`.
    ///
    /// If the `data` contains fewer elements than the slice viewed by this [IndexBufferView],
    /// then only the first `N` elements will be replaced, where `N` is the number of elements in
    /// the given `index_data`.
    ///
    /// If the `index_data` contains more elements than the slice viewed this [IndexBufferView],
    /// then only the first `M` elements in the `index_data` will be used to update the index
    /// buffer, where `M` is the number of elements viewed by the [IndexBufferView].
    ///
    /// This will modify the viewed [IndexBuffer], the buffer (and any other views on the same data)
    /// will be affected by this change.
    pub fn upload_command<D>(&self, data: D) -> IndexBufferUploadCommand<MaybeUninit<T>, D>
    where
        D: Borrow<[MaybeUninit<T>]> + Send + Sync + 'static,
    {
        IndexBufferUploadCommand {
            buffer_data: self.buffer.data.clone(),
            index_data: data,
            offset_in_bytes: self.offset_in_bytes,
            len: self.len,
            _marker: marker::PhantomData,
        }
    }
}

impl<'a, T> IndexBufferView<'a, MaybeUninit<T>>
where
    T: IndexFormat,
{
    /// Converts to `IndexBufferView<T>`.
    ///
    /// # Safety
    ///
    /// Its up to the user to guarantee that any tasks that read buffer region viewed by this view,
    /// is only executed after the viewed region is initialized. Note that certain tasks may wait on
    /// GPU fences and allow a runtime to progress other tasks while its waiting on the fence. As
    /// such, submitting your initialization tasks as part of a task that includes fencing (these
    /// are typically tasks that include "download" commands), may not guarantee that the buffer was
    /// initialized before any tasks that are submitted later will begin executing.
    pub unsafe fn assume_init(self) -> IndexBufferView<'a, T> {
        mem::transmute(self)
    }
}

impl<'a, T> Clone for IndexBufferView<'a, T> {
    fn clone(&self) -> Self {
        IndexBufferView {
            buffer: self.buffer,
            offset_in_bytes: self.offset_in_bytes,
            len: self.len,
        }
    }
}

impl<'a, T> Copy for IndexBufferView<'a, T> {}

/// A helper trait type for indexing operations on a [IndexBuffer].
pub trait IndexBufferSliceRange<T>: Sized {
    /// Returns a view on the index buffer if in bounds, or `None` otherwise.
    fn get(self, index_buffer: &IndexBuffer<T>) -> Option<IndexBufferView<T>>;

    /// Returns a view on the index buffer, without performing any bounds checking.
    unsafe fn get_unchecked(self, index_buffer: &IndexBuffer<T>) -> IndexBufferView<T>;
}

impl<T> IndexBufferSliceRange<T> for RangeFull {
    fn get(self, index_buffer: &IndexBuffer<T>) -> Option<IndexBufferView<T>> {
        Some(IndexBufferView {
            buffer: index_buffer,
            offset_in_bytes: 0,
            len: index_buffer.data.len,
        })
    }

    unsafe fn get_unchecked(self, index_buffer: &IndexBuffer<T>) -> IndexBufferView<T> {
        IndexBufferView {
            buffer: index_buffer,
            offset_in_bytes: 0,
            len: index_buffer.data.len,
        }
    }
}

impl<T> IndexBufferSliceRange<T> for Range<usize> {
    fn get(self, index_buffer: &IndexBuffer<T>) -> Option<IndexBufferView<T>> {
        let Range { start, end } = self;

        if start > end || end > index_buffer.data.len {
            None
        } else {
            Some(IndexBufferView {
                buffer: index_buffer,
                offset_in_bytes: start * mem::size_of::<T>(),
                len: end - start,
            })
        }
    }

    unsafe fn get_unchecked(self, index_buffer: &IndexBuffer<T>) -> IndexBufferView<T> {
        IndexBufferView {
            buffer: index_buffer,
            offset_in_bytes: self.start * mem::size_of::<T>(),
            len: self.end - self.start,
        }
    }
}

impl<T> IndexBufferSliceRange<T> for RangeInclusive<usize> {
    fn get(self, index_buffer: &IndexBuffer<T>) -> Option<IndexBufferView<T>> {
        if *self.end() == usize::max_value() {
            None
        } else {
            index_buffer.get(*self.start()..self.end() + 1)
        }
    }

    unsafe fn get_unchecked(self, index_buffer: &IndexBuffer<T>) -> IndexBufferView<T> {
        index_buffer.get_unchecked(*self.start()..self.end() + 1)
    }
}

impl<T> IndexBufferSliceRange<T> for RangeFrom<usize> {
    fn get(self, index_buffer: &IndexBuffer<T>) -> Option<IndexBufferView<T>> {
        index_buffer.get(self.start..index_buffer.data.len)
    }

    unsafe fn get_unchecked(self, index_buffer: &IndexBuffer<T>) -> IndexBufferView<T> {
        index_buffer.get_unchecked(self.start..index_buffer.data.len)
    }
}

impl<T> IndexBufferSliceRange<T> for RangeTo<usize> {
    fn get(self, index_buffer: &IndexBuffer<T>) -> Option<IndexBufferView<T>> {
        index_buffer.get(0..self.end)
    }

    unsafe fn get_unchecked(self, index_buffer: &IndexBuffer<T>) -> IndexBufferView<T> {
        index_buffer.get_unchecked(0..self.end)
    }
}

impl<T> IndexBufferSliceRange<T> for RangeToInclusive<usize> {
    fn get(self, index_buffer: &IndexBuffer<T>) -> Option<IndexBufferView<T>> {
        index_buffer.get(0..=self.end)
    }

    unsafe fn get_unchecked(self, index_buffer: &IndexBuffer<T>) -> IndexBufferView<T> {
        index_buffer.get_unchecked(0..=self.end)
    }
}

/// A helper trait type for indexing operations on an [IndexBufferView].
pub trait IndexBufferViewSliceIndex<T>: Sized {
    /// Returns a view on the [IndexBufferView] if in bounds, or `None` otherwise.
    fn get<'a>(self, view: &'a IndexBufferView<T>) -> Option<IndexBufferView<'a, T>>;

    /// Returns a view on the [IndexBufferView], without performing any bounds checking.
    unsafe fn get_unchecked<'a>(self, view: &'a IndexBufferView<T>) -> IndexBufferView<'a, T>;
}

impl<T> IndexBufferViewSliceIndex<T> for RangeFull {
    fn get<'a>(self, view: &'a IndexBufferView<T>) -> Option<IndexBufferView<'a, T>> {
        Some(IndexBufferView {
            buffer: view.buffer,
            offset_in_bytes: view.offset_in_bytes,
            len: view.len,
        })
    }

    unsafe fn get_unchecked<'a>(self, view: &'a IndexBufferView<T>) -> IndexBufferView<'a, T> {
        IndexBufferView {
            buffer: view.buffer,
            offset_in_bytes: view.offset_in_bytes,
            len: view.len,
        }
    }
}

impl<T> IndexBufferViewSliceIndex<T> for Range<usize> {
    fn get<'a>(self, view: &'a IndexBufferView<T>) -> Option<IndexBufferView<'a, T>> {
        let Range { start, end } = self;

        if start > end || end > view.len {
            None
        } else {
            Some(IndexBufferView {
                buffer: view.buffer,
                offset_in_bytes: view.offset_in_bytes + start * mem::size_of::<T>(),
                len: end - start,
            })
        }
    }

    unsafe fn get_unchecked<'a>(self, view: &'a IndexBufferView<T>) -> IndexBufferView<'a, T> {
        IndexBufferView {
            buffer: view.buffer,
            offset_in_bytes: view.offset_in_bytes + self.start * mem::size_of::<T>(),
            len: self.end - self.start,
        }
    }
}

impl<T> IndexBufferViewSliceIndex<T> for RangeInclusive<usize> {
    fn get<'a>(self, view: &'a IndexBufferView<T>) -> Option<IndexBufferView<'a, T>> {
        if *self.end() == usize::max_value() {
            None
        } else {
            view.get(*self.start()..self.end() + 1)
        }
    }

    unsafe fn get_unchecked<'a>(self, view: &'a IndexBufferView<T>) -> IndexBufferView<'a, T> {
        view.get_unchecked(*self.start()..self.end() + 1)
    }
}

impl<T> IndexBufferViewSliceIndex<T> for RangeFrom<usize> {
    fn get<'a>(self, view: &'a IndexBufferView<T>) -> Option<IndexBufferView<'a, T>> {
        view.get(self.start..view.len)
    }

    unsafe fn get_unchecked<'a>(self, view: &'a IndexBufferView<T>) -> IndexBufferView<'a, T> {
        view.get_unchecked(self.start..view.len)
    }
}

impl<T> IndexBufferViewSliceIndex<T> for RangeTo<usize> {
    fn get<'a>(self, view: &'a IndexBufferView<T>) -> Option<IndexBufferView<'a, T>> {
        view.get(0..self.end)
    }

    unsafe fn get_unchecked<'a>(self, view: &'a IndexBufferView<T>) -> IndexBufferView<'a, T> {
        view.get_unchecked(0..self.end)
    }
}

impl<T> IndexBufferViewSliceIndex<T> for RangeToInclusive<usize> {
    fn get<'a>(self, view: &'a IndexBufferView<T>) -> Option<IndexBufferView<'a, T>> {
        view.get(0..=self.end)
    }

    unsafe fn get_unchecked<'a>(self, view: &'a IndexBufferView<T>) -> IndexBufferView<'a, T> {
        view.get_unchecked(0..=self.end)
    }
}

/// Command for uploading index data to an [IndexBuffer] or a sub-section of a buffer as viewed by a
/// [BufferView].
///
/// See [Buffer::upload_command] and [BufferView::upload_command] for details.
pub struct IndexBufferUploadCommand<T, D> {
    buffer_data: Arc<IndexBufferData>,
    index_data: D,
    offset_in_bytes: usize,
    len: usize,
    _marker: marker::PhantomData<T>,
}

unsafe impl<T, D> GpuTask<Connection> for IndexBufferUploadCommand<T, D>
where
    D: Borrow<[T]>,
{
    type Output = ();

    fn context_id(&self) -> ContextId {
        ContextId::Id(self.buffer_data.context_id)
    }

    fn progress(&mut self, connection: &mut Connection) -> Progress<Self::Output> {
        let (gl, state) = unsafe { connection.unpack_mut() };

        state.bind_vertex_array(None).apply(gl).unwrap();

        unsafe {
            self.buffer_data
                .id()
                .unwrap()
                .with_value_unchecked(|buffer_object| {
                    state
                        .bind_element_array_buffer(Some(&buffer_object))
                        .apply(gl)
                        .unwrap();
                });
        }

        let data = self.index_data.borrow();
        let size = data.len() * mem::size_of::<T>();
        let max_size = self.len * mem::size_of::<T>();

        unsafe {
            let mut data =
                slice::from_raw_parts(self.index_data.borrow() as *const _ as *const u8, size);

            if max_size < size {
                data = &data[0..max_size];
            }

            gl.buffer_sub_data_with_i32_and_u8_array(
                Gl::ELEMENT_ARRAY_BUFFER,
                self.offset_in_bytes as i32,
                data,
            );
        };

        Progress::Finished(())
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

pub(crate) struct IndexBufferData {
    id: UnsafeCell<Option<JsId>>,
    context_id: u64,
    dropper: Box<dyn BufferObjectDropper>,
    len: usize,
    usage_hint: UsageHint,
}

impl IndexBufferData {
    pub(crate) fn id(&self) -> Option<JsId> {
        unsafe { *self.id.get() }
    }

    pub(crate) fn context_id(&self) -> u64 {
        self.context_id
    }
}

impl Drop for IndexBufferData {
    fn drop(&mut self) {
        if let Some(id) = self.id() {
            self.dropper.drop_buffer_object(id);
        }
    }
}

struct AllocateUninitCommand<T>
where
    T: IndexFormat,
{
    data: Arc<IndexBufferData>,
    _marker: marker::PhantomData<T>,
}

unsafe impl<T> GpuTask<Connection> for AllocateUninitCommand<T>
where
    T: IndexFormat,
{
    type Output = ();

    fn context_id(&self) -> ContextId {
        ContextId::Any
    }

    fn progress(&mut self, connection: &mut Connection) -> Progress<Self::Output> {
        let (gl, state) = unsafe { connection.unpack_mut() };
        let data = &self.data;

        state.bind_vertex_array(None).apply(gl).unwrap();

        let buffer_object = Gl::create_buffer(&gl).unwrap();

        state
            .bind_element_array_buffer(Some(&buffer_object))
            .apply(gl)
            .unwrap();

        let size = mem::size_of::<T>() * data.len;

        gl.buffer_data_with_i32(
            Gl::ELEMENT_ARRAY_BUFFER,
            size as i32,
            data.usage_hint.gl_id(),
        );

        unsafe {
            *data.id.get() = Some(JsId::from_value(buffer_object.into()));
        }

        Progress::Finished(())
    }
}

struct AllocateCommand<D, T>
where
    T: IndexFormat,
{
    data: Arc<IndexBufferData>,
    initial: D,
    _marker: marker::PhantomData<T>,
}

unsafe impl<D, T> GpuTask<Connection> for AllocateCommand<D, T>
where
    D: Borrow<[T]>,
    T: IndexFormat,
{
    type Output = ();

    fn context_id(&self) -> ContextId {
        ContextId::Any
    }

    fn progress(&mut self, connection: &mut Connection) -> Progress<Self::Output> {
        let (gl, state) = unsafe { connection.unpack_mut() };
        let data = &self.data;

        state.bind_vertex_array(None).apply(gl).unwrap();

        let buffer_object = Gl::create_buffer(&gl).unwrap();

        state
            .bind_element_array_buffer(Some(&buffer_object))
            .apply(gl)
            .unwrap();

        unsafe {
            let initial = self.initial.borrow();
            let size = initial.len() * mem::size_of::<T>();
            let initial = slice::from_raw_parts(initial as *const _ as *const u8, size);

            gl.buffer_data_with_u8_array(
                Gl::ELEMENT_ARRAY_BUFFER,
                initial,
                data.usage_hint.gl_id(),
            );
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

        let value = unsafe { JsId::into_value(self.id) };

        gl.delete_buffer(Some(&value.unchecked_into()));

        Progress::Finished(())
    }
}
