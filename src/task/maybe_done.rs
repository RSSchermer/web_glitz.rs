use std::marker::PhantomData;
use std::mem;

use super::{GpuTask, Execution };

pub(crate) enum MaybeDone<T, Ec> where T: GpuTask<Ec> {
    NotYet(T, PhantomData<Ec>),
    Done(T::Output),
    Gone
}

impl<T, Ec> MaybeDone<T, Ec> where T: GpuTask<Ec> {
    pub fn progress(&mut self, execution_context: &mut Ec) -> Result<bool, T::Error> {
        let res = match self {
            MaybeDone::Done(_) => return Ok(true),
            MaybeDone::NotYet(ref mut task, _) => task.progress(execution_context),
            MaybeDone::Gone => panic!("Cannot progress a Join twice.")
        };

        match res {
            Execution::Finished(Ok(output)) => {
                *self = MaybeDone::Done(output);

                Ok(true)
            },
            Execution::Finished(Err(err)) => Err(err),
            Execution::ContinueFenced => Ok(false)
        }
    }

    pub fn take(&mut self) -> T::Output {
        match mem::replace(self, MaybeDone::Gone) {
            MaybeDone::Done(a) => a,
            _ => panic!(),
        }
    }
}

pub(crate) fn maybe_done<T, Ec>(task: T) -> MaybeDone<T, Ec> where T: GpuTask<Ec> {
    MaybeDone::NotYet(task, PhantomData)
}
