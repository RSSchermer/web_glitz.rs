use super::{ GpuTask, GpuTaskExt, Execution };
use super::maybe_done::{ MaybeDone, maybe_done };

pub struct JoinAll<E, Ec> {
    tasks: Vec<MaybeDone<Box<GpuTask<Ec, Output=(), Error=E>>, (), Ec>>
}

impl<E, Ec> JoinAll<E, Ec> {
    pub fn new<I>(tasks: I) -> Self where I: IntoIterator, I::Item: GpuTask<Ec, Error=E> {
        JoinAll {
            tasks: tasks.into_iter().map(|t| maybe_done(t.map(|_| ()).into())).collect()
        }
    }
}
