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

pub trait GpuCommandExt<A, Ec> where A: GpuCommand<Ec> {
    fn execute(self, execution_context: &mut Ec) -> Result<A::Output, A::Error>;

    fn map<B, F>(self, f: F) -> Map<A, F, Ec> where F: FnOnce(A::Output) -> B;

    fn map_err<B, F>(self, f: F) -> MapErr<A, F, Ec> where F: FnOnce(A::Error) -> B;

    fn then<B, F>(self, f: F) -> Then<A, F, Ec> where B: GpuCommand<Ec>, F: FnOnce(Result<A::Output, A::Error>) -> B;

    fn and_then<B, F>(self, f: F) -> AndThen<A, F, Ec> where B: GpuCommand<Ec, Error=A::Error>, F: FnOnce(A::Output) -> B;

    fn or_else<B, F>(self, f: F) -> OrElse<A, F, Ec> where B: GpuCommand<Ec, Output=A::Output>, F: FnOnce(A::Error) -> B;
}

impl<T, A, Ec> GpuCommandExt<A, Ec> for T where T: Into<CommandObject<A, Ec>>, A: GpuCommand<Ec> {
    fn execute(self, execution_context: &mut Ec) -> Result<A::Output, A::Error> {
        match self.into() {
            CommandObject::Static(executable, _) => executable.execute_static(execution_context),
            CommandObject::Dynamic(boxed_executable) => boxed_executable.execute_dynamic(execution_context)
        }
    }

    fn map<B, F>(self, f: F) -> Map<A, F, Ec> where F: FnOnce(A::Output) -> B {
        Map::new(self, f)
    }

    fn map_err<B, F>(self, f: F) -> MapErr<A, F, Ec> where F: FnOnce(A::Error) -> B {
        MapErr::new(self, f)
    }

    fn then<B, F>(self, f: F) -> Then<A, F, Ec> where B: GpuCommand<Ec>, F: FnOnce(Result<A::Output, A::Error>) -> B {
        Then::new(self, f)
    }

    fn and_then<B, F>(self, f: F) -> AndThen<A, F, Ec> where B: GpuCommand<Ec, Error=A::Error>, F: FnOnce(A::Output) -> B {
        AndThen::new(self, f)
    }

    fn or_else<B, F>(self, f: F) -> OrElse<A, F, Ec> where B: GpuCommand<Ec, Output=A::Output>, F: FnOnce(A::Error) -> B {
        OrElse::new(self, f)
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

    #[derive(Debug)]
    struct ReturnOk<T>(T);

    impl<T> GpuCommand<TestContext> for ReturnOk<T> {
        type Output = T;

        type Error = ();

        fn execute_static(self, _execution_context: &mut TestContext) -> Result<Self::Output, Self::Error> {
            Ok(self.0)
        }

        fn execute_dynamic(self: Box<Self>, _execution_context: &mut TestContext) -> Result<Self::Output, Self::Error> {
            Ok(self.0)
        }
    }

    #[derive(Debug)]
    struct ReturnErr<T>(T);

    impl<T> GpuCommand<TestContext> for ReturnErr<T> {
        type Output = ();

        type Error = T;

        fn execute_static(self, _execution_context: &mut TestContext) -> Result<Self::Output, Self::Error> {
            Err(self.0)
        }

        fn execute_dynamic(self: Box<Self>, _execution_context: &mut TestContext) -> Result<Self::Output, Self::Error> {
            Err(self.0)
        }
    }

    #[test]
    fn test_map_execute_static() {
        let mut context = TestContext;
        let map = Map::new(ReturnOk(1), |v| v * 2);

        assert_eq!(map.execute(&mut context), Ok(2));
    }

    #[test]
    fn test_map_execute_dynamic() {
        let mut context = TestContext;
        let map = Map::new(ReturnOk(1), |v| v * 2);
        let boxed_map = Box::new(map);

        assert_eq!(boxed_map.execute(&mut context), Ok(2));
    }

    #[test]
    fn test_map_err_execute_static() {
        let mut context = TestContext;
        let map_err = MapErr::new(ReturnErr(1), |v| v * 2);

        assert_eq!(map_err.execute(&mut context), Err(2));
    }

    #[test]
    fn test_map_err_execute_dynamic() {
        let mut context = TestContext;
        let map = MapErr::new(ReturnErr(1), |v| v * 2);
        let boxed_map_err = Box::new(map);

        assert_eq!(boxed_map_err.execute(&mut context), Err(2));
    }

    #[test]
    fn test_ok_then_execute_static() {
        let mut context = TestContext;
        let then = Then::new(ReturnOk(1), |res| ReturnOk(res.unwrap() * 2));

        assert_eq!(then.execute(&mut context), Ok(2));
    }

    #[test]
    fn test_ok_then_execute_dynamic() {
        let mut context = TestContext;
        let then = Then::new(ReturnOk(1), |res| ReturnOk(res.unwrap() * 2));
        let boxed_then = Box::new(then);

        assert_eq!(boxed_then.execute(&mut context), Ok(2));
    }

    #[test]
    fn test_err_then_execute_static() {
        let mut context = TestContext;
        let then = Then::new(ReturnErr(1), |res| ReturnErr(res.unwrap_err() * 2));

        assert_eq!(then.execute(&mut context), Err(2));
    }

    #[test]
    fn test_err_then_execute_dynamic() {
        let mut context = TestContext;
        let then = Then::new(ReturnErr(1), |res| ReturnErr(res.unwrap_err() * 2));
        let boxed_then = Box::new(then);

        assert_eq!(boxed_then.execute(&mut context), Err(2));
    }

    #[test]
    fn test_ok_and_then_execute_static() {
        let mut context = TestContext;
        let and_then = AndThen::new(ReturnOk(1), |v| ReturnOk(v * 2));

        assert_eq!(and_then.execute(&mut context), Ok(2));
    }

    #[test]
    fn test_ok_and_then_execute_dynamic() {
        let mut context = TestContext;
        let and_then = AndThen::new(ReturnOk(1), |v| ReturnOk(v * 2));
        let boxed_and_then = Box::new(and_then);

        assert_eq!(boxed_and_then.execute(&mut context), Ok(2));
    }

    #[test]
    fn test_err_and_then_execute_static() {
        let mut context = TestContext;
        let and_then = AndThen::new(ReturnErr(1), |_| ReturnErr(2));

        assert_eq!(and_then.execute(&mut context), Err(1));
    }

    #[test]
    fn test_err_and_then_execute_dynamic() {
        let mut context = TestContext;
        let and_then = AndThen::new(ReturnErr(1), |_| ReturnErr(2));
        let boxed_and_then = Box::new(and_then);

        assert_eq!(boxed_and_then.execute(&mut context), Err(1));
    }

    #[test]
    fn test_ok_or_else_execute_static() {
        let mut context = TestContext;
        let or_else = OrElse::new(ReturnOk(1), |_| ReturnOk(2));

        assert_eq!(or_else.execute(&mut context), Ok(1));
    }

    #[test]
    fn test_ok_or_else_execute_dynamic() {
        let mut context = TestContext;
        let or_else = OrElse::new(ReturnOk(1), |_| ReturnOk(2));
        let boxed_or_else = Box::new(or_else);

        assert_eq!(boxed_or_else.execute(&mut context), Ok(1));
    }

    #[test]
    fn test_err_or_else_execute_static() {
        let mut context = TestContext;
        let or_else = OrElse::new(ReturnErr(1), |v| ReturnErr(v * 2));

        assert_eq!(or_else.execute(&mut context), Err(2));
    }

    #[test]
    fn test_err_or_else_execute_dynamic() {
        let mut context = TestContext;
        let or_else = OrElse::new(ReturnErr(1), |v| ReturnErr(v * 2));
        let boxed_or_else = Box::new(or_else);

        assert_eq!(boxed_or_else.execute(&mut context), Err(2));
    }
}