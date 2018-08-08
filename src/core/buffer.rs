struct Buffer {
    context: Weak<RenderingContextInternal>,
    id: GLUint,
    usage_hint: UsageHint,
}

impl Buffer {
    pub fn bind<'a>(&'a mut self, context: &'a mut RenderingContext) -> Result<BoundBuffer, BindingError> {

    }

    pub fn download(&self) -> impl Future<Item = Vec<u8>, Error = DownloadError> {
        // TODO: use fencing and PromiseFuture? Possibly WEBGL_get_buffer_sub_data_async extension?
    }

    pub fn download_sync(&self) -> Result<Vec<u8>, DownloadError> {}
}

struct BoundBuffer<'a> {
    buffer: &'a mut Buffer,
    context: PhantomData<&'a mut RenderingContext>
}
