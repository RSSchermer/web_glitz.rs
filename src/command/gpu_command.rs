use std::marker::PhantomData;

use super::{ Map, MapErr, Then, AndThen, OrElse };

pub trait GpuCommand<Ec> {
    type Output;

    type Error;

    fn execute_static(self, execution_context: &mut Ec) -> Execution<Self::Output, Self::Error, Ec>;

    fn execute_dynamic(self: Box<Self>, execution_context: &mut Ec) -> Execution<Self::Output, Self::Error, Ec>;
}

pub enum Execution<O, E, Ec> {
    Finished(Result<O, E>),
    ContinueFenced(Box<GpuCommand<Ec, Output=O, Error=E>>)
}

pub enum CommandObject<T, Ec> where T: GpuCommand<Ec> {
    Static(T, PhantomData<Ec>),
    Dynamic(Box<GpuCommand<Ec, Output=T::Output, Error=T::Error>>)
}

impl<T, Ec> CommandObject<T, Ec> where T: GpuCommand<Ec> {
    pub fn into_box(self) -> Box<T> {
        match self {
            CommandObject::Static(command, _) => Box::new(command),
            CommandObject::Dynamic(command) => command
        }
    }
}

impl<T, Ec> From<T> for CommandObject<T, Ec> where T: GpuCommand<Ec> {
    fn from(executable: T) -> CommandObject<T, Ec> {
        CommandObject::Static(executable, PhantomData)
    }
}

impl<T, Ec> From<Box<GpuCommand<Ec, Output=T::Output, Error=T::Error>>> for CommandObject<T, Ec> where T: GpuCommand<Ec> {
    fn from(boxed_executable: Box<T>) -> CommandObject<T, Ec> {
        CommandObject::Dynamic(boxed_executable)
    }
}

pub trait GpuCommandExt<A, Ec> where A: GpuCommand<Ec> {
    fn execute(self, execution_context: &mut Ec) -> Execution<A::Output, A::Error, Ec>;

    fn map<B, F>(self, f: F) -> Map<A, F, Ec> where F: FnOnce(A::Output) -> B;

    fn map_err<B, F>(self, f: F) -> MapErr<A, F, Ec> where F: FnOnce(A::Error) -> B;

    fn then<B, F>(self, f: F) -> Then<A, F, Ec> where B: GpuCommand<Ec>, F: FnOnce(Result<A::Output, A::Error>) -> B;

    fn and_then<B, F>(self, f: F) -> AndThen<A, F, Ec> where B: GpuCommand<Ec, Error=A::Error>, F: FnOnce(A::Output) -> B;

    fn or_else<B, F>(self, f: F) -> OrElse<A, F, Ec> where B: GpuCommand<Ec, Output=A::Output>, F: FnOnce(A::Error) -> B;
}

impl<T, A, Ec> GpuCommandExt<A, Ec> for T where T: Into<CommandObject<A, Ec>>, A: GpuCommand<Ec> {
    fn execute(self, execution_context: &mut Ec) -> Execution<A::Output, A::Error, Ec> {
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
