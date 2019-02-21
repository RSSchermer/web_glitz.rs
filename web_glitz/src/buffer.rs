use std::borrow::Borrow;
use std::marker;
use std::mem;
use std::ops::{Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive};
use std::slice;
use std::sync::Arc;

use wasm_bindgen::JsCast;
use web_sys::{WebGl2RenderingContext as GL, WebGlBuffer};

use crate::runtime::state::{BufferRange, ContextUpdate};
use crate::runtime::{Connection, RenderingContext, TaskContextMismatch};
use crate::task::{GpuTask, Progress};
use crate::util::{arc_get_mut_unchecked, slice_make_mut, JsId};

#[derive(Clone, Copy, Debug)]
pub enum BufferUsage {
    StaticDraw,
    DynamicDraw,
    StreamDraw,
    StaticRead,
    DynamicRead,
    StreamRead,
    StaticCopy,
    DynamicCopy,
    StreamCopy,
}

impl BufferUsage {
    fn gl_id(&self) -> u32 {
        match self {
            BufferUsage::StaticDraw => GL::STATIC_DRAW,
            BufferUsage::DynamicDraw => GL::DYNAMIC_DRAW,
            BufferUsage::StreamDraw => GL::STREAM_DRAW,
            BufferUsage::StaticRead => GL::STATIC_READ,
            BufferUsage::DynamicRead => GL::DYNAMIC_READ,
            BufferUsage::StreamRead => GL::STREAM_READ,
            BufferUsage::StaticCopy => GL::STATIC_COPY,
            BufferUsage::DynamicCopy => GL::DYNAMIC_COPY,
            BufferUsage::StreamCopy => GL::STREAM_COPY,
        }
    }
}

pub trait IntoBuffer<T>
where
    T: ?Sized,
{
    fn into_buffer<Rc>(self, context: &Rc, usage_hint: BufferUsage) -> Buffer<T>
    where
        Rc: RenderingContext + Clone + 'static;
}

impl<D, T> IntoBuffer<T> for D
where
    D: Borrow<T> + 'static,
    T: 'static,
{
    fn into_buffer<Rc>(self, context: &Rc, usage_hint: BufferUsage) -> Buffer<T>
    where
        Rc: RenderingContext + Clone + 'static,
    {
        let data = Arc::new(BufferData {
            id: None,
            context_id: context.id(),
            dropper: Box::new(context.clone()),
            usage_hint,
            len: 1,
            size_in_bytes: mem::size_of::<T>(),
            recent_uniform_binding: None,
        });

        context.submit(AllocateCommand {
            data: data.clone(),
            initial: self,
            _marker: marker::PhantomData,
        });

        Buffer {
            data,
            _marker: marker::PhantomData,
        }
    }
}

impl<D, T> IntoBuffer<[T]> for D
where
    D: Borrow<[T]> + 'static,
    T: 'static,
{
    fn into_buffer<Rc>(self, context: &Rc, usage_hint: BufferUsage) -> Buffer<[T]>
    where
        Rc: RenderingContext + Clone + 'static,
    {
        let len = self.borrow().len();
        let data = Arc::new(BufferData {
            id: None,
            context_id: context.id(),
            dropper: Box::new(context.clone()),
            usage_hint,
            len,
            size_in_bytes: len * mem::size_of::<T>(),
            recent_uniform_binding: None,
        });

        context.submit(AllocateCommand::<D, [T]> {
            data: data.clone(),
            initial: self,
            _marker: marker::PhantomData,
        });

        Buffer {
            data,
            _marker: marker::PhantomData,
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
    id: Option<JsId>,
    context_id: usize,
    dropper: Box<BufferObjectDropper>,
    len: usize,
    size_in_bytes: usize,
    usage_hint: BufferUsage,
    recent_uniform_binding: Option<u32>,
}

impl Drop for BufferData {
    fn drop(&mut self) {
        if let Some(id) = self.id {
            self.dropper.drop_buffer_object(id);
        }
    }
}

pub struct Buffer<T>
where
    T: ?Sized,
{
    data: Arc<BufferData>,
    _marker: marker::PhantomData<Box<T>>,
}

impl<T> Buffer<T>
where
    T: ?Sized,
{
    pub fn usage_hint(&self) -> BufferUsage {
        self.data.usage_hint
    }

    pub fn view(&self) -> BufferView<T> {
        BufferView {
            data: self.data.clone(),
            offset_in_bytes: 0,
            len: self.data.len,
            _marker: marker::PhantomData,
        }
    }
}

impl<T> Buffer<T> {
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

impl<T> Buffer<[T]> {
    pub fn len(&self) -> usize {
        self.data.len
    }

    pub fn get<I>(&self, index: I) -> Option<I::Output>
    where
        I: BufferIndex<Buffer<[T]>>,
    {
        index.get(self)
    }

    pub unsafe fn get_unchecked<I>(&self, index: I) -> I::Output
    where
        I: BufferIndex<Buffer<[T]>>,
    {
        index.get_unchecked(self)
    }

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

pub struct BufferView<T>
where
    T: ?Sized,
{
    data: Arc<BufferData>,
    offset_in_bytes: usize,
    len: usize,
    _marker: marker::PhantomData<T>,
}

impl<T> BufferView<T>
where
    T: ?Sized,
{
    pub fn usage_hint(&self) -> BufferUsage {
        self.data.usage_hint
    }
}

impl<T> BufferView<T> {
    pub(crate) fn bind_uniform(&self, connection: &mut Connection) -> u32 {
        let (gl, state) = unsafe { connection.unpack_mut() };

        unsafe {
            let data = arc_get_mut_unchecked(&self.data);
            let most_recent_binding = &mut data.recent_uniform_binding;
            let size_in_bytes = self.len * mem::size_of::<T>();

            data.id.unwrap().with_value_unchecked(|buffer_object| {
                let buffer_range = BufferRange::OffsetSize(
                    buffer_object,
                    self.offset_in_bytes as u32,
                    size_in_bytes as u32,
                );

                if most_recent_binding.is_none()
                    || state.bound_uniform_buffer_range(most_recent_binding.unwrap())
                        != buffer_range
                {
                    state.set_active_uniform_buffer_binding_lru();
                    state
                        .set_bound_uniform_buffer_range(buffer_range)
                        .apply(gl)
                        .unwrap();

                    let binding = state.active_uniform_buffer_binding();

                    *most_recent_binding = Some(binding);

                    binding
                } else {
                    most_recent_binding.unwrap()
                }
            })
        }
    }

    pub fn upload_command<D>(&self, data: D) -> UploadCommand<T, D>
    where
        D: Borrow<T> + Send + Sync + 'static,
    {
        UploadCommand {
            buffer_data: self.data.clone(),
            data,
            offset_in_bytes: self.offset_in_bytes,
            len: 1,
            _marker: marker::PhantomData,
        }
    }

    pub fn download_command(&self) -> DownloadCommand<T> {
        DownloadCommand {
            data: self.data.clone(),
            state: DownloadState::Initial,
            offset_in_bytes: self.offset_in_bytes,
            len: 1,
            _marker: marker::PhantomData,
        }
    }
}

impl<T> BufferView<[T]> {
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn get<I>(&self, index: I) -> Option<I::Output>
    where
        I: BufferIndex<BufferView<[T]>>,
    {
        index.get(self)
    }

    pub unsafe fn get_unchecked<I>(&self, index: I) -> I::Output
    where
        I: BufferIndex<BufferView<[T]>>,
    {
        index.get_unchecked(self)
    }

    pub fn upload_command<D>(&self, data: D) -> UploadCommand<[T], D>
    where
        D: Borrow<[T]> + Send + Sync + 'static,
    {
        UploadCommand {
            buffer_data: self.data.clone(),
            data,
            offset_in_bytes: self.offset_in_bytes,
            len: self.len,
            _marker: marker::PhantomData,
        }
    }

    pub fn download_command(&self) -> DownloadCommand<[T]> {
        DownloadCommand {
            data: self.data.clone(),
            state: DownloadState::Initial,
            offset_in_bytes: self.offset_in_bytes,
            len: self.len,
            _marker: marker::PhantomData,
        }
    }
}

impl<T> Into<BufferView<T>> for Buffer<T>
where
    T: ?Sized,
{
    fn into(self) -> BufferView<T> {
        BufferView {
            len: self.data.len,
            data: self.data,
            offset_in_bytes: 0,
            _marker: marker::PhantomData,
        }
    }
}

pub trait BufferIndex<T> {
    type Output;

    fn get(self, buffer: &T) -> Option<Self::Output>;

    unsafe fn get_unchecked(self, buffer: &T) -> Self::Output;
}

impl<T> BufferIndex<Buffer<[T]>> for usize {
    type Output = BufferView<T>;

    fn get(self, buffer: &Buffer<[T]>) -> Option<Self::Output> {
        if self < buffer.data.len {
            Some(BufferView {
                data: buffer.data.clone(),
                offset_in_bytes: self * mem::size_of::<T>(),
                len: 1,
                _marker: marker::PhantomData,
            })
        } else {
            None
        }
    }

    unsafe fn get_unchecked(self, buffer: &Buffer<[T]>) -> Self::Output {
        BufferView {
            data: buffer.data.clone(),
            offset_in_bytes: self * mem::size_of::<T>(),
            len: 1,
            _marker: marker::PhantomData,
        }
    }
}

impl<T> BufferIndex<BufferView<[T]>> for usize {
    type Output = BufferView<T>;

    fn get(self, buffer: &BufferView<[T]>) -> Option<Self::Output> {
        if self < buffer.len {
            Some(BufferView {
                data: buffer.data.clone(),
                offset_in_bytes: buffer.offset_in_bytes + self * mem::size_of::<T>(),
                len: 1,
                _marker: marker::PhantomData,
            })
        } else {
            None
        }
    }

    unsafe fn get_unchecked(self, buffer: &BufferView<[T]>) -> Self::Output {
        BufferView {
            data: buffer.data.clone(),
            offset_in_bytes: buffer.offset_in_bytes + self * mem::size_of::<T>(),
            len: 1,
            _marker: marker::PhantomData,
        }
    }
}

impl<T> BufferIndex<Buffer<[T]>> for RangeFull {
    type Output = BufferView<[T]>;

    fn get(self, buffer: &Buffer<[T]>) -> Option<Self::Output> {
        Some(BufferView {
            data: buffer.data.clone(),
            offset_in_bytes: 0,
            len: buffer.data.len,
            _marker: marker::PhantomData,
        })
    }

    unsafe fn get_unchecked(self, buffer: &Buffer<[T]>) -> Self::Output {
        BufferView {
            data: buffer.data.clone(),
            offset_in_bytes: 0,
            len: buffer.data.len,
            _marker: marker::PhantomData,
        }
    }
}

impl<T> BufferIndex<BufferView<[T]>> for RangeFull {
    type Output = BufferView<[T]>;

    fn get(self, buffer: &BufferView<[T]>) -> Option<Self::Output> {
        Some(BufferView {
            data: buffer.data.clone(),
            offset_in_bytes: buffer.offset_in_bytes,
            len: buffer.len,
            _marker: marker::PhantomData,
        })
    }

    unsafe fn get_unchecked(self, buffer: &BufferView<[T]>) -> Self::Output {
        BufferView {
            data: buffer.data.clone(),
            offset_in_bytes: buffer.offset_in_bytes,
            len: buffer.len,
            _marker: marker::PhantomData,
        }
    }
}

impl<T> BufferIndex<Buffer<[T]>> for Range<usize> {
    type Output = BufferView<[T]>;

    fn get(self, buffer: &Buffer<[T]>) -> Option<Self::Output> {
        let Range { start, end } = self;

        if start > end || end > buffer.data.len {
            None
        } else {
            Some(BufferView {
                data: buffer.data.clone(),
                offset_in_bytes: start * mem::size_of::<T>(),
                len: end - start,
                _marker: marker::PhantomData,
            })
        }
    }

    unsafe fn get_unchecked(self, buffer: &Buffer<[T]>) -> Self::Output {
        BufferView {
            data: buffer.data.clone(),
            offset_in_bytes: self.start * mem::size_of::<T>(),
            len: self.end - self.start,
            _marker: marker::PhantomData,
        }
    }
}

impl<T> BufferIndex<BufferView<[T]>> for Range<usize> {
    type Output = BufferView<[T]>;

    fn get(self, buffer: &BufferView<[T]>) -> Option<Self::Output> {
        let Range { start, end } = self;

        if start > end || end > buffer.len {
            None
        } else {
            Some(BufferView {
                data: buffer.data.clone(),
                offset_in_bytes: buffer.offset_in_bytes + start * mem::size_of::<T>(),
                len: end - start,
                _marker: marker::PhantomData,
            })
        }
    }

    unsafe fn get_unchecked(self, buffer: &BufferView<[T]>) -> Self::Output {
        BufferView {
            data: buffer.data.clone(),
            offset_in_bytes: buffer.offset_in_bytes + self.start * mem::size_of::<T>(),
            len: self.end - self.start,
            _marker: marker::PhantomData,
        }
    }
}

impl<T> BufferIndex<Buffer<[T]>> for RangeInclusive<usize> {
    type Output = BufferView<[T]>;

    fn get(self, buffer: &Buffer<[T]>) -> Option<Self::Output> {
        if *self.end() == usize::max_value() {
            None
        } else {
            (*self.start()..self.end() + 1).get(buffer)
        }
    }

    unsafe fn get_unchecked(self, buffer: &Buffer<[T]>) -> Self::Output {
        (*self.start()..self.end() + 1).get_unchecked(buffer)
    }
}

impl<T> BufferIndex<BufferView<[T]>> for RangeInclusive<usize> {
    type Output = BufferView<[T]>;

    fn get(self, buffer: &BufferView<[T]>) -> Option<Self::Output> {
        if *self.end() == usize::max_value() {
            None
        } else {
            (*self.start()..self.end() + 1).get(buffer)
        }
    }

    unsafe fn get_unchecked(self, buffer: &BufferView<[T]>) -> Self::Output {
        (*self.start()..self.end() + 1).get_unchecked(buffer)
    }
}

impl<T> BufferIndex<Buffer<[T]>> for RangeFrom<usize> {
    type Output = BufferView<[T]>;

    fn get(self, buffer: &Buffer<[T]>) -> Option<Self::Output> {
        (self.start..buffer.data.len).get(buffer)
    }

    unsafe fn get_unchecked(self, buffer: &Buffer<[T]>) -> Self::Output {
        (self.start..buffer.data.len).get_unchecked(buffer)
    }
}

impl<T> BufferIndex<BufferView<[T]>> for RangeFrom<usize> {
    type Output = BufferView<[T]>;

    fn get(self, buffer: &BufferView<[T]>) -> Option<Self::Output> {
        (self.start..buffer.len).get(buffer)
    }

    unsafe fn get_unchecked(self, buffer: &BufferView<[T]>) -> Self::Output {
        (self.start..buffer.len).get_unchecked(buffer)
    }
}

impl<T> BufferIndex<Buffer<[T]>> for RangeTo<usize> {
    type Output = BufferView<[T]>;

    fn get(self, buffer: &Buffer<[T]>) -> Option<Self::Output> {
        (0..self.end).get(buffer)
    }

    unsafe fn get_unchecked(self, buffer: &Buffer<[T]>) -> Self::Output {
        (0..self.end).get_unchecked(buffer)
    }
}

impl<T> BufferIndex<BufferView<[T]>> for RangeTo<usize> {
    type Output = BufferView<[T]>;

    fn get(self, buffer: &BufferView<[T]>) -> Option<Self::Output> {
        (0..self.end).get(buffer)
    }

    unsafe fn get_unchecked(self, buffer: &BufferView<[T]>) -> Self::Output {
        (0..self.end).get_unchecked(buffer)
    }
}

impl<T> BufferIndex<Buffer<[T]>> for RangeToInclusive<usize> {
    type Output = BufferView<[T]>;

    fn get(self, buffer: &Buffer<[T]>) -> Option<Self::Output> {
        (0..=self.end).get(buffer)
    }

    unsafe fn get_unchecked(self, buffer: &Buffer<[T]>) -> Self::Output {
        (0..=self.end).get_unchecked(buffer)
    }
}

impl<T> BufferIndex<BufferView<[T]>> for RangeToInclusive<usize> {
    type Output = BufferView<[T]>;

    fn get(self, buffer: &BufferView<[T]>) -> Option<Self::Output> {
        (0..=self.end).get(buffer)
    }

    unsafe fn get_unchecked(self, buffer: &BufferView<[T]>) -> Self::Output {
        (0..=self.end).get_unchecked(buffer)
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

impl<D, T> GpuTask<Connection> for AllocateCommand<D, T>
where
    D: Borrow<T>,
{
    type Output = ();

    fn progress(&mut self, connection: &mut Connection) -> Progress<Self::Output> {
        let (gl, state) = unsafe { connection.unpack_mut() };
        let data = unsafe { arc_get_mut_unchecked(&mut self.data) };

        let buffer_object = GL::create_buffer(&gl).unwrap();

        state
            .set_bound_copy_write_buffer(Some(&buffer_object))
            .apply(gl)
            .unwrap();

        unsafe {
            let initial = slice::from_raw_parts(
                self.initial.borrow() as *const _ as *const u8,
                mem::size_of::<T>(),
            );

            gl.buffer_data_with_u8_array(
                GL::COPY_WRITE_BUFFER,
                slice_make_mut(initial),
                data.usage_hint.gl_id(),
            );
        }

        data.id = Some(JsId::from_value(buffer_object.into()));

        Progress::Finished(())
    }
}

impl<D, T> GpuTask<Connection> for AllocateCommand<D, [T]>
where
    D: Borrow<[T]>,
{
    type Output = ();

    fn progress(&mut self, connection: &mut Connection) -> Progress<Self::Output> {
        let (gl, state) = unsafe { connection.unpack_mut() };
        let data = unsafe { arc_get_mut_unchecked(&mut self.data) };

        let buffer_object = GL::create_buffer(&gl).unwrap();

        state
            .set_bound_copy_write_buffer(Some(&buffer_object))
            .apply(gl)
            .unwrap();

        unsafe {
            let initial = self.initial.borrow();
            let size = initial.len() * mem::size_of::<T>();
            let initial = slice::from_raw_parts(initial as *const _ as *const u8, size);

            gl.buffer_data_with_u8_array(
                GL::COPY_WRITE_BUFFER,
                slice_make_mut(initial),
                data.usage_hint.gl_id(),
            );
        }

        data.id = Some(JsId::from_value(buffer_object.into()));

        Progress::Finished(())
    }
}

struct DropCommand {
    id: JsId,
}

impl GpuTask<Connection> for DropCommand {
    type Output = ();

    fn progress(&mut self, connection: &mut Connection) -> Progress<Self::Output> {
        let (gl, _) = unsafe { connection.unpack() };
        let value = unsafe { JsId::into_value(self.id) };

        gl.delete_buffer(Some(&value.unchecked_into()));

        Progress::Finished(())
    }
}

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

impl<T, D> GpuTask<Connection> for UploadCommand<T, D>
where
    D: Borrow<T>,
{
    type Output = Result<(), TaskContextMismatch>;

    fn progress(&mut self, connection: &mut Connection) -> Progress<Self::Output> {
        if self.buffer_data.context_id != connection.context_id() {
            return Progress::Finished(Err(TaskContextMismatch));
        }

        let (gl, state) = unsafe { connection.unpack_mut() };

        unsafe {
            self.buffer_data
                .id
                .unwrap()
                .with_value_unchecked(|buffer_object| {
                    state
                        .set_bound_copy_write_buffer(Some(&buffer_object))
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
                slice_make_mut(data),
            );
        };

        Progress::Finished(Ok(()))
    }
}

impl<T, D> GpuTask<Connection> for UploadCommand<[T], D>
where
    D: Borrow<[T]>,
{
    type Output = Result<(), TaskContextMismatch>;

    fn progress(&mut self, connection: &mut Connection) -> Progress<Self::Output> {
        if self.buffer_data.context_id != connection.context_id() {
            return Progress::Finished(Err(TaskContextMismatch));;
        }

        let (gl, state) = unsafe { connection.unpack_mut() };

        unsafe {
            self.buffer_data
                .id
                .unwrap()
                .with_value_unchecked(|buffer_object| {
                    state
                        .set_bound_copy_write_buffer(Some(&buffer_object))
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
                slice_make_mut(data),
            );
        };

        Progress::Finished(Ok(()))
    }
}

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

impl<T> GpuTask<Connection> for DownloadCommand<T> {
    type Output = Result<Box<T>, TaskContextMismatch>;

    fn progress(&mut self, connection: &mut Connection) -> Progress<Self::Output> {
        if self.data.context_id != connection.context_id() {
            return Progress::Finished(Err(TaskContextMismatch));;
        }

        match self.state {
            DownloadState::Initial => {
                let (gl, state) = unsafe { connection.unpack_mut() };
                let read_buffer = GL::create_buffer(&gl).unwrap();
                let size_in_bytes = mem::size_of::<T>();

                state
                    .set_bound_copy_write_buffer(Some(&read_buffer))
                    .apply(gl)
                    .unwrap();

                gl.buffer_data_with_i32(
                    GL::COPY_WRITE_BUFFER,
                    size_in_bytes as i32,
                    GL::STREAM_READ,
                );

                unsafe {
                    self.data.id.unwrap().with_value_unchecked(|buffer_object| {
                        state
                            .set_bound_copy_read_buffer(Some(&buffer_object))
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

                mem::replace(&mut self.state, DownloadState::Copied(Some(read_buffer)));

                Progress::ContinueFenced
            }
            DownloadState::Copied(ref mut read_buffer) => {
                let read_buffer = read_buffer
                    .take()
                    .expect("Cannot make progress on a BufferDownload task after it has finished");
                let (gl, state) = unsafe { connection.unpack_mut() };

                state
                    .set_bound_copy_read_buffer(Some(&read_buffer))
                    .apply(gl)
                    .unwrap();

                let size_in_bytes = self.len * mem::size_of::<T>();
                let mut data = vec![0; size_in_bytes];

                gl.get_buffer_sub_data_with_i32_and_u8_array(GL::COPY_READ_BUFFER, 0, &mut data);

                gl.delete_buffer(Some(&read_buffer));

                let value = unsafe { Box::from_raw(mem::transmute(data.as_mut_ptr())) };

                mem::forget(data);

                Progress::Finished(Ok(value))
            }
        }
    }
}

impl<T> GpuTask<Connection> for DownloadCommand<[T]> {
    type Output = Result<Box<[T]>, TaskContextMismatch>;

    fn progress(&mut self, connection: &mut Connection) -> Progress<Self::Output> {
        if self.data.context_id != connection.context_id() {
            return Progress::Finished(Err(TaskContextMismatch));;
        }

        match self.state {
            DownloadState::Initial => {
                let (gl, state) = unsafe { connection.unpack_mut() };
                let read_buffer = GL::create_buffer(&gl).unwrap();
                let size_in_bytes = self.len * mem::size_of::<T>();

                state
                    .set_bound_copy_write_buffer(Some(&read_buffer))
                    .apply(gl)
                    .unwrap();

                gl.buffer_data_with_i32(
                    GL::COPY_WRITE_BUFFER,
                    size_in_bytes as i32,
                    GL::STREAM_READ,
                );

                unsafe {
                    self.data.id.unwrap().with_value_unchecked(|buffer_object| {
                        state
                            .set_bound_copy_read_buffer(Some(&buffer_object))
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

                mem::replace(&mut self.state, DownloadState::Copied(Some(read_buffer)));

                Progress::ContinueFenced
            }
            DownloadState::Copied(ref mut read_buffer) => {
                let read_buffer = read_buffer
                    .take()
                    .expect("Cannot make progress on a BufferDownload task after it has finished");
                let (gl, state) = unsafe { connection.unpack_mut() };

                state
                    .set_bound_copy_read_buffer(Some(&read_buffer))
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

                    Progress::Finished(Ok(boxed))
                }
            }
        }
    }
}
