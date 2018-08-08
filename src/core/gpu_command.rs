use core::rendering_context::RenderingContext;

pub enum GpuCommand<T> where T: GpuExecutable {
    Static(T),
    Dynamic(Box<T>)
}

impl<T> GpuCommand<T> where T: GpuExecutable {
    pub fn execute(self, context: &mut RenderingContext) -> Result<T::Output, T::Error> {
        match self {
            GpuCommand::Static(executable) => executable.execute(context),
            GpuCommand::Dynamic(boxed_executable) => boxed_executable.execute_boxed(context)
        }
    }
}

impl<T> From<T> for GpuCommand<T> where T: GpuExecutable {
    fn from(executable: T) -> GpuCommand<T> {
        GpuCommand::Static(executable)
    }
}

impl<T> From<Box<T>> for GpuCommand<T> where T: GpuExecutable {
    fn from(boxed_executable: Box<T>) -> GpuCommand<T> {
        GpuCommand::Dynamic(boxed_executable)
    }
}

pub trait GpuExecutable {
    type Output;

    type Error;

    fn execute(self, context: &mut RenderingContext) -> Result<Self::Output, Self::Error>;

    fn execute_boxed(self: Box<Self>, context: &mut RenderingContext) -> Result<Self::Output, Self::Error>;
}

pub struct Empty;

impl GpuExecutable for Empty {
    type Output = ();

    type Error = ();

    fn execute(self, _context: &mut RenderingContext) -> Result<Self::Output, Self::Error> {
        Ok(())
    }

    fn execute_boxed(self: Box<Self>, _context: &mut RenderingContext) -> Result<Self::Output, Self::Error> {
        Ok(())
    }
}

pub struct Map<A, F> where A: GpuExecutable {
    data: Option<MapData<A, F>>
}

struct MapData<A, F> where A: GpuExecutable {
    executable: GpuCommand<A>,
    f: F
}

impl <A, B, F> Map<A, F> where A: GpuExecutable, F: FnOnce(A::Output) -> B {
    pub fn new<T>(executable: T, f: F) -> Self where T: Into<GpuCommand<A>> {
        Map {
            data: Some(MapData {
                executable: executable.into(),
                f
            })
        }
    }

    fn execute_internal(&mut self, context: &mut RenderingContext) -> Result<B, A::Error> {
        let data = self.data.take().expect("Cannot execute Map twice");

        data.executable.execute(context).map(data.f)
    }
}

impl <A, B, F> GpuExecutable for Map<A, F> where A: GpuExecutable, F: FnOnce(A::Output) -> B {
    type Output = B;

    type Error = <A as GpuExecutable>::Error;

    fn execute(mut self, context: &mut RenderingContext) -> Result<Self::Output, Self::Error> {
        self.execute_internal(context)
    }

    fn execute_boxed(mut self: Box<Self>, context: &mut RenderingContext) -> Result<Self::Output, Self::Error> {
        self.execute_internal(context)
    }
}

//struct MapErr<A, F> {
//    future: A,
//    f: Option<F>
//}
//
//impl <A, B, F> UnsafeGpuExecutable for MapErr<A, F> where A: UnsafeGpuExecutable, F: FnOnce(A::Error) -> B {
//    type Output = <A as UnsafeGpuExecutable>::Output;
//
//    type Error = B;
//
//    unsafe fn execute_unsafe(&mut self, context: &mut RenderingContext) -> Result<Self::Output, Self::Error> {
//        self.future.execute(context).map_err(self.f.take().expect("Cannot execute Map twice"))
//    }
//}
//
//struct Then<A, F> {
//    future: A,
//    f: Option<F>
//}
//
//impl <A, B, F> UnsafeGpuExecutable for Then<A, F> where A: UnsafeGpuExecutable, B: UnsafeGpuExecutable, F: FnOnce(Result<A::Item, A::Error>) -> B {
//    type Output = <B as UnsafeGpuExecutable>::Output;
//
//    type Error = <B as UnsafeGpuExecutable>::Error;
//
//    unsafe fn execute_unsafe(&mut self, context: &mut RenderingContext) -> Result<Self::Output, Self::Error> {
//        self.f.take().expect("Cannot execute Then twice")(self.future.execute_unsafe(context))
//    }
//}
//
//struct AndThen<A, F> {
//    future: A,
//    f: Option<F>
//}
//
//impl <A, B, F> UnsafeGpuExecutable for AndThen<A, F> where A: UnsafeGpuExecutable, B: UnsafeGpuExecutable<Error=A::Error>, F: FnOnce(A::Item) -> B {
//    type Output = <A as UnsafeGpuExecutable>::Output;
//
//    type Error = B;
//
//    unsafe fn execute_unsafe(&mut self, context: &mut RenderingContext) -> Result<Self::Output, Self::Error> {
//        self.future.execute(context).and_then(self.f.take().expect("Cannot execute AndThen twice"))
//    }
//}
//
//trait UnsafeGpuExecutableExt {
//
//}
//
//struct CreateBuffer {
//
//}
//
//impl UnsafeGpuExecutable for CreateBuffer {
//    type Output = Buffer;
//
//    type Error = CreateBufferError;
//
//    unsafe fn execute_unsafe(&mut self, context: &mut RenderingContext) -> Result<Self::Output, Self::Error> {
//
//    }
//}
//
//struct BufferUpload {
//
//}
//
//impl UnsafeGpuExecutable for BufferUpload {
//    type Output = ();
//
//    type Error = BufferUploadError;
//
//    unsafe fn execute_unsafe(&mut self, context: &mut RenderingContext) -> Result<Self::Output, Self::Error> {
//
//    }
//}
//
//struct BufferDownload {
//
//}
//
//impl UnsafeGpuExecutable for BufferDownload {
//    type Output = impl Future<Item=Box[u8], Error=()>;
//
//    type Error = BufferDownloadError;
//
//    unsafe fn execute_unsafe(&mut self, context: &mut RenderingContext) -> Result<Self::Output, Self::Error> {
//
//    }
//}
//
//struct BufferDownloadSync {
//
//}
//
//impl UnsafeGpuExecutable for BufferDownloadSync {
//    type Output = Box[u8];
//
//    type Error = BufferDownloadError;
//
//    unsafe fn execute_unsafe(&mut self, context: &mut RenderingContext) -> Result<Self::Output, Self::Error> {
//
//    }
//}
//
//struct CompileShaderCommand<S> where S: Borrow<str> {
//    source: S
//}
//
//struct CreateGraphicsPipelineCommand {
//    builder: GraphicsPipelineBuilder
//}
//
//struct MappedCommand<A, B, F> where A: UnsafeGpuExecutable, F: FnOnce(A::Output) -> B {
//    command: A,
//    f: F
//}

#[cfg(test)]
mod tests {
    use super::*;

    struct Return1;

    impl GpuExecutable for Return1 {
        type Output = i32;

        type Error = ();

        fn execute(self, _context: &mut RenderingContext) -> Result<Self::Output, Self::Error> {
            Ok(1)
        }

        fn execute_boxed(self: Box<Self>, _context: &mut RenderingContext) -> Result<Self::Output, Self::Error> {
            Ok(1)
        }
    }

    #[test]
    fn test_map_execute() {
        let mut context = RenderingContext {};
        let map = Map::new(Return1, |v| v * 2);

        assert_eq!(map.execute(&mut context), Ok(2));
    }

    #[test]
    fn test_map_execute_boxed() {
        let mut context = RenderingContext {};
        let map = Map::new(Return1, |v| v * 2);
        let boxed_map = Box::new(map);

        assert_eq!(boxed_map.execute_boxed(&mut context), Ok(2));
    }

    #[test]
    fn test_map_with_boxed_command() {
        let mut context = RenderingContext {};
        let map = Map::new(Box::new(Return1), |v| v * 2);

        assert_eq!(map.execute(&mut context), Ok(2));
    }
}