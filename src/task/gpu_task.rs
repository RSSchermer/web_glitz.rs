use super::{ Map, MapErr, Then, AndThen, OrElse, Join, Join3, Join4, Join5 };

pub trait GpuTask<Ec> {
    type Output;

    type Error;

    fn progress(&mut self, execution_context: &mut Ec) -> Execution<Self::Output, Self::Error>;
}


pub enum Execution<O, E> {
    Finished(Result<O, E>),
    ContinueFenced
}

pub trait GpuTaskExt<T, Ec> where T: GpuTask<Ec> {
    fn map<B, F>(self, f: F) -> Map<T, F, Ec> where F: FnOnce(T::Output) -> B, B: 'static;

    fn map_err<B, F>(self, f: F) -> MapErr<T, F, Ec> where F: FnOnce(T::Error) -> B, B: 'static;

    fn then<B, F>(self, f: F) -> Then<T, B, F, Ec> where B: GpuTask<Ec>, F: FnOnce(Result<T::Output, T::Error>) -> B;

    fn and_then<B, F>(self, f: F) -> AndThen<T, B, F, Ec> where B: GpuTask<Ec, Error=T::Error>, F: FnOnce(T::Output) -> B;

    fn or_else<B, F>(self, f: F) -> OrElse<T, B, F, Ec> where B: GpuTask<Ec, Output=T::Output>, F: FnOnce(T::Error) -> B;

    fn join<B>(self, b: B) -> Join<T, B, Ec> where B: GpuTask<Ec, Error=T::Error>;

    fn join3<B, C>(self, b: B, c: C) -> Join3<T, B, C, Ec> where B: GpuTask<Ec, Error=T::Error>, C: GpuTask<Ec, Error=T::Error>;

    fn join4<B, C, D>(self, b: B, c: C, d: D) -> Join4<T, B, C, D, Ec> where B: GpuTask<Ec, Error=T::Error>, C: GpuTask<Ec, Error=T::Error>, D: GpuTask<Ec, Error=T::Error>;

    fn join5<B, C, D, E>(self, b: B, c: C, d: D, e: E) -> Join5<T, B, C, D, E, Ec> where B: GpuTask<Ec, Error=T::Error>, C: GpuTask<Ec, Error=T::Error>, D: GpuTask<Ec, Error=T::Error>, E: GpuTask<Ec, Error=T::Error>;
}

impl<T, Ec> GpuTaskExt<T, Ec> for T where T: GpuTask<Ec> {
    fn map<B, F>(self, f: F) -> Map<T, F, Ec> where F: FnOnce(T::Output) -> B, B: 'static {
        Map::new(self, f)
    }

    fn map_err<B, F>(self, f: F) -> MapErr<T, F, Ec> where F: FnOnce(T::Error) -> B, B: 'static {
        MapErr::new(self, f)
    }

    fn then<B, F>(self, f: F) -> Then<T, B, F, Ec> where B: GpuTask<Ec>, F: FnOnce(Result<T::Output, T::Error>) -> B {
        Then::new(self, f)
    }

    fn and_then<B, F>(self, f: F) -> AndThen<T, B, F, Ec> where B: GpuTask<Ec, Error=T::Error>, F: FnOnce(T::Output) -> B {
        AndThen::new(self, f)
    }

    fn or_else<B, F>(self, f: F) -> OrElse<T, B, F, Ec> where B: GpuTask<Ec, Output=T::Output>, F: FnOnce(T::Error) -> B {
        OrElse::new(self, f)
    }

    fn join<B>(self, b: B) -> Join<T, B, Ec> where B: GpuTask<Ec, Error=T::Error> {
        Join::new(self, b)
    }

    fn join3<B, C>(self, b: B, c: C) -> Join3<T, B, C, Ec> where B: GpuTask<Ec, Error=T::Error>, C: GpuTask<Ec, Error=T::Error> {
        Join3::new(self, b, c)
    }

    fn join4<B, C, D>(self, b: B, c: C, d: D) -> Join4<T, B, C, D, Ec> where B: GpuTask<Ec, Error=T::Error>, C: GpuTask<Ec, Error=T::Error>, D: GpuTask<Ec, Error=T::Error> {
        Join4::new(self, b, c, d)
    }

    fn join5<B, C, D, E>(self, b: B, c: C, d: D, e: E) -> Join5<T, B, C, D, E, Ec> where B: GpuTask<Ec, Error=T::Error>, C: GpuTask<Ec, Error=T::Error>, D: GpuTask<Ec, Error=T::Error>, E: GpuTask<Ec, Error=T::Error> {
        Join5::new(self, b, c, d, e)
    }
}
