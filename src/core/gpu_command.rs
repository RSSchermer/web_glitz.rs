use std::marker::PhantomData;

pub trait GpuCommand<Ec> {
    type Output;

    type Error;

    fn execute_static(self, execution_context: &mut Ec) -> Result<Self::Output, Self::Error>;

    fn execute_dynamic(self: Box<Self>, execution_context: &mut Ec) -> Result<Self::Output, Self::Error>;
}

pub enum CommandObject<T, Ec> where T: GpuCommand<Ec> {
    Static(T, PhantomData<Ec>),
    Dynamic(Box<T>)
}

impl<T, Ec> From<T> for CommandObject<T, Ec> where T: GpuCommand<Ec> {
    fn from(executable: T) -> CommandObject<T, Ec> {
        CommandObject::Static(executable, PhantomData)
    }
}

impl<T, Ec> From<Box<T>> for CommandObject<T, Ec> where T: GpuCommand<Ec> {
    fn from(boxed_executable: Box<T>) -> CommandObject<T, Ec> {
        CommandObject::Dynamic(boxed_executable)
    }
}

pub trait Execute<A, Ec> where A: GpuCommand<Ec> {
    fn execute(self, execution_context: &mut Ec) -> Result<A::Output, A::Error>;
}

impl<T, A, Ec> Execute<A, Ec> for T where T: Into<CommandObject<A, Ec>>, A: GpuCommand<Ec> {
    fn execute(self, execution_context: &mut Ec) -> Result<A::Output, A::Error> {
        match self.into() {
            CommandObject::Static(executable, _) => executable.execute_static(execution_context),
            CommandObject::Dynamic(boxed_executable) => boxed_executable.execute_dynamic(execution_context)
        }
    }
}

pub struct Empty;

impl<Ec> GpuCommand<Ec> for Empty {
    type Output = ();

    type Error = ();

    fn execute_static(self, _execution_context: &mut Ec) -> Result<Self::Output, Self::Error> {
        Ok(())
    }

    fn execute_dynamic(self: Box<Self>, _execution_context: &mut Ec) -> Result<Self::Output, Self::Error> {
        Ok(())
    }
}

pub struct Map<A, F, Ec> where A: GpuCommand<Ec> {
    data: Option<MapData<A, F, Ec>>
}

struct MapData<A, F, Ec> where A: GpuCommand<Ec> {
    command: CommandObject<A, Ec>,
    f: F
}

impl <A, B, F, Ec> Map<A, F, Ec> where A: GpuCommand<Ec>, F: FnOnce(A::Output) -> B {
    pub fn new<T>(command: T, f: F) -> Self where T: Into<CommandObject<A, Ec>> {
        Map {
            data: Some(MapData {
                command: command.into(),
                f
            })
        }
    }

    fn execute_internal(&mut self, execution_context: &mut Ec) -> Result<B, A::Error> {
        let data = self.data.take().expect("Cannot execute Map twice");

        data.command.execute(execution_context).map(data.f)
    }
}

impl <A, B, F, Ec> GpuCommand<Ec> for Map<A, F, Ec> where A: GpuCommand<Ec>, F: FnOnce(A::Output) -> B {
    type Output = B;

    type Error = <A as GpuCommand<Ec>>::Error;

    fn execute_static(mut self, execution_context: &mut Ec) -> Result<Self::Output, Self::Error> {
        self.execute_internal(execution_context)
    }

    fn execute_dynamic(mut self: Box<Self>, execution_context: &mut Ec) -> Result<Self::Output, Self::Error> {
        self.execute_internal(execution_context)
    }
}

pub struct MapErr<A, F, Ec> where A: GpuCommand<Ec> {
    data: Option<MapErrData<A, F, Ec>>
}

struct MapErrData<A, F, Ec> where A: GpuCommand<Ec> {
    command: CommandObject<A, Ec>,
    f: F
}

impl <A, B, F, Ec> MapErr<A, F, Ec> where A: GpuCommand<Ec>, F: FnOnce(A::Error) -> B {
    pub fn new<T>(command: T, f: F) -> Self where T: Into<CommandObject<A, Ec>> {
        MapErr {
            data: Some(MapErrData {
                command: command.into(),
                f
            })
        }
    }

    fn execute_internal(&mut self, execution_context: &mut Ec) -> Result<A::Output, B> {
        let data = self.data.take().expect("Cannot execute MapErr twice");

        data.command.execute(execution_context).map_err(data.f)
    }
}

impl <A, B, F, Ec> GpuCommand<Ec> for MapErr<A, F, Ec> where A: GpuCommand<Ec>, F: FnOnce(A::Error) -> B {
    type Output = <A as GpuCommand<Ec>>::Output;

    type Error = B;

    fn execute_static(mut self, execution_context: &mut Ec) -> Result<Self::Output, Self::Error> {
        self.execute_internal(execution_context)
    }

    fn execute_dynamic(mut self: Box<Self>, execution_context: &mut Ec) -> Result<Self::Output, Self::Error> {
        self.execute_internal(execution_context)
    }
}

pub struct Then<A, F, Ec> where A: GpuCommand<Ec> {
    data: Option<ThenData<A, F, Ec>>
}

struct ThenData<A, F, Ec> where A: GpuCommand<Ec> {
    command: CommandObject<A, Ec>,
    f: F
}

impl <A, B, F, Ec> Then<A, F, Ec> where A: GpuCommand<Ec>, B: GpuCommand<Ec>, F: FnOnce(Result<A::Output, A::Error>) -> B {
    pub fn new<T>(command: T, f: F) -> Self where T: Into<CommandObject<A, Ec>> {
        Then {
            data: Some(ThenData {
                command: command.into(),
                f
            })
        }
    }

    fn execute_internal(&mut self, execution_context: &mut Ec) -> Result<<B as GpuCommand<Ec>>::Output, <B as GpuCommand<Ec>>::Error> {
        let data = self.data.take().expect("Cannot execute Then twice");

        ((data.f)(data.command.execute(execution_context))).execute(execution_context)
    }
}

impl <A, B, F, Ec> GpuCommand<Ec> for Then<A, F, Ec> where A: GpuCommand<Ec>, B: GpuCommand<Ec>, F: FnOnce(Result<A::Output, A::Error>) -> B {
    type Output = <B as GpuCommand<Ec>>::Output;

    type Error = <B as GpuCommand<Ec>>::Error;

    fn execute_static(mut self, execution_context: &mut Ec) -> Result<Self::Output, Self::Error> {
        self.execute_internal(execution_context)
    }

    fn execute_dynamic(mut self: Box<Self>, execution_context: &mut Ec) -> Result<Self::Output, Self::Error> {
        self.execute_internal(execution_context)
    }
}

pub struct AndThen<A, F, Ec> where A: GpuCommand<Ec> {
    data: Option<AndThenData<A, F, Ec>>
}

struct AndThenData<A, F, Ec> where A: GpuCommand<Ec> {
    command: CommandObject<A, Ec>,
    f: F
}

impl <A, B, F, Ec> AndThen<A, F, Ec> where A: GpuCommand<Ec>, B: GpuCommand<Ec, Error=A::Error>, F: FnOnce(A::Output) -> B {
    pub fn new<T>(command: T, f: F) -> Self where T: Into<CommandObject<A, Ec>> {
        AndThen {
            data: Some(AndThenData {
                command: command.into(),
                f
            })
        }
    }

    fn execute_internal(&mut self, execution_context: &mut Ec) -> Result<<B as GpuCommand<Ec>>::Output, <B as GpuCommand<Ec>>::Error> {
        let AndThenData { command, f } = self.data.take().expect("Cannot execute Then twice");

        command.execute(execution_context).and_then(|output| f(output).execute(execution_context))
    }
}

impl <A, B, F, Ec> GpuCommand<Ec> for AndThen<A, F, Ec> where A: GpuCommand<Ec>, B: GpuCommand<Ec, Error=A::Error>, F: FnOnce(A::Output) -> B {
    type Output = <B as GpuCommand<Ec>>::Output;

    type Error = <B as GpuCommand<Ec>>::Error;

    fn execute_static(mut self, execution_context: &mut Ec) -> Result<Self::Output, Self::Error> {
        self.execute_internal(execution_context)
    }

    fn execute_dynamic(mut self: Box<Self>, execution_context: &mut Ec) -> Result<Self::Output, Self::Error> {
        self.execute_internal(execution_context)
    }
}

pub struct OrElse<A, F, Ec> where A: GpuCommand<Ec> {
    data: Option<OrElseData<A, F, Ec>>
}

struct OrElseData<A, F, Ec> where A: GpuCommand<Ec> {
    command: CommandObject<A, Ec>,
    f: F
}

impl <A, B, F, Ec> OrElse<A, F, Ec> where A: GpuCommand<Ec>, B: GpuCommand<Ec, Output=A::Output>, F: FnOnce(A::Error) -> B {
    pub fn new<T>(command: T, f: F) -> Self where T: Into<CommandObject<A, Ec>> {
        OrElse {
            data: Some(OrElseData {
                command: command.into(),
                f
            })
        }
    }

    fn execute_internal(&mut self, execution_context: &mut Ec) -> Result<<B as GpuCommand<Ec>>::Output, <B as GpuCommand<Ec>>::Error> {
        let OrElseData { command, f } = self.data.take().expect("Cannot execute Then twice");

        command.execute(execution_context).or_else(|error| f(error).execute(execution_context))
    }
}

impl <A, B, F, Ec> GpuCommand<Ec> for OrElse<A, F, Ec> where A: GpuCommand<Ec>, B: GpuCommand<Ec, Output=A::Output>, F: FnOnce(A::Error) -> B {
    type Output = <B as GpuCommand<Ec>>::Output;

    type Error = <B as GpuCommand<Ec>>::Error;

    fn execute_static(mut self, execution_context: &mut Ec) -> Result<Self::Output, Self::Error> {
        self.execute_internal(execution_context)
    }

    fn execute_dynamic(mut self: Box<Self>, execution_context: &mut Ec) -> Result<Self::Output, Self::Error> {
        self.execute_internal(execution_context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestContext;

    struct Return1;

    impl GpuCommand<TestContext> for Return1 {
        type Output = i32;

        type Error = ();

        fn execute_static(self, _execution_context: &mut TestContext) -> Result<Self::Output, Self::Error> {
            Ok(1)
        }

        fn execute_dynamic(self: Box<Self>, _execution_context: &mut TestContext) -> Result<Self::Output, Self::Error> {
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

        assert_eq!(boxed_map.execute(&mut context), Ok(2));
    }

    #[test]
    fn test_map_with_boxed_command() {
        let mut context = TestContext;
        let map = Map::new(Box::new(Return1), |v| v * 2);

        assert_eq!(map.execute(&mut context), Ok(2));
    }
}