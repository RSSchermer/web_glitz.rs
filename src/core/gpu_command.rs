use std::marker::PhantomData;

pub enum CommandObject<T, C> where T: GpuCommand<C> {
    Static(T, PhantomData<C>),
    Dynamic(Box<T>)
}

impl<T, C> CommandObject<T, C> where T: GpuCommand<C> {
    pub fn execute(self, context: &mut C) -> Result<T::Output, T::Error> {
        match self {
            CommandObject::Static(executable, _) => executable.execute(context),
            CommandObject::Dynamic(boxed_executable) => boxed_executable.execute_boxed(context)
        }
    }
}

impl<T, C> From<T> for CommandObject<T, C> where T: GpuCommand<C> {
    fn from(executable: T) -> CommandObject<T, C> {
        CommandObject::Static(executable, PhantomData)
    }
}

impl<T, C> From<Box<T>> for CommandObject<T, C> where T: GpuCommand<C> {
    fn from(boxed_executable: Box<T>) -> CommandObject<T, C> {
        CommandObject::Dynamic(boxed_executable)
    }
}

pub trait GpuCommand<C> {
    type Output;

    type Error;

    fn execute(self, context: &mut C) -> Result<Self::Output, Self::Error>;

    fn execute_boxed(self: Box<Self>, context: &mut C) -> Result<Self::Output, Self::Error>;
}

pub struct Empty;

impl<C> GpuCommand<C> for Empty {
    type Output = ();

    type Error = ();

    fn execute(self, _context: &mut C) -> Result<Self::Output, Self::Error> {
        Ok(())
    }

    fn execute_boxed(self: Box<Self>, _context: &mut C) -> Result<Self::Output, Self::Error> {
        Ok(())
    }
}

pub struct Map<A, F, C> where A: GpuCommand<C> {
    data: Option<MapData<A, F, C>>
}

struct MapData<A, F, C> where A: GpuCommand<C> {
    command: CommandObject<A, C>,
    f: F
}

impl <A, B, F, C> Map<A, F, C> where A: GpuCommand<C>, F: FnOnce(A::Output) -> B {
    pub fn new<T>(command: T, f: F) -> Self where T: Into<CommandObject<A, C>> {
        Map {
            data: Some(MapData {
                command: command.into(),
                f
            })
        }
    }

    fn execute_internal(&mut self, context: &mut C) -> Result<B, A::Error> {
        let data = self.data.take().expect("Cannot execute Map twice");

        data.command.execute(context).map(data.f)
    }
}

impl <A, B, F, C> GpuCommand<C> for Map<A, F, C> where A: GpuCommand<C>, F: FnOnce(A::Output) -> B {
    type Output = B;

    type Error = <A as GpuCommand<C>>::Error;

    fn execute(mut self, context: &mut C) -> Result<Self::Output, Self::Error> {
        self.execute_internal(context)
    }

    fn execute_boxed(mut self: Box<Self>, context: &mut C) -> Result<Self::Output, Self::Error> {
        self.execute_internal(context)
    }
}

pub struct MapErr<A, F, C> where A: GpuCommand<C> {
    data: Option<MapErrData<A, F, C>>
}

struct MapErrData<A, F, C> where A: GpuCommand<C> {
    command: CommandObject<A, C>,
    f: F
}

impl <A, B, F, C> MapErr<A, F, C> where A: GpuCommand<C>, F: FnOnce(A::Error) -> B {
    pub fn new<T>(command: T, f: F) -> Self where T: Into<CommandObject<A, C>> {
        MapErr {
            data: Some(MapErrData {
                command: command.into(),
                f
            })
        }
    }

    fn execute_internal(&mut self, context: &mut C) -> Result<A::Output, B> {
        let data = self.data.take().expect("Cannot execute MapErr twice");

        data.command.execute(context).map_err(data.f)
    }
}

impl <A, B, F, C> GpuCommand<C> for MapErr<A, F, C> where A: GpuCommand<C>, F: FnOnce(A::Error) -> B {
    type Output = <A as GpuCommand<C>>::Output;

    type Error = B;

    fn execute(mut self, context: &mut C) -> Result<Self::Output, Self::Error> {
        self.execute_internal(context)
    }

    fn execute_boxed(mut self: Box<Self>, context: &mut C) -> Result<Self::Output, Self::Error> {
        self.execute_internal(context)
    }
}
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

    struct TestContext;

    struct Return1;

    impl GpuCommand<TestContext> for Return1 {
        type Output = i32;

        type Error = ();

        fn execute(self, _context: &mut TestContext) -> Result<Self::Output, Self::Error> {
            Ok(1)
        }

        fn execute_boxed(self: Box<Self>, _context: &mut TestContext) -> Result<Self::Output, Self::Error> {
            Ok(1)
        }
    }

    #[test]
    fn test_map_execute() {
        let mut context = TestContext;
        let map = Map::new(Return1, |v| v * 2);

        assert_eq!(map.execute(&mut context), Ok(2));
    }

    #[test]
    fn test_map_execute_boxed() {
        let mut context = TestContext;
        let map = Map::new(Return1, |v| v * 2);
        let boxed_map = Box::new(map);

        assert_eq!(boxed_map.execute_boxed(&mut context), Ok(2));
    }

    #[test]
    fn test_map_with_boxed_command() {
        let mut context = TestContext;
        let map = Map::new(Box::new(Return1), |v| v * 2);

        assert_eq!(map.execute(&mut context), Ok(2));
    }
}