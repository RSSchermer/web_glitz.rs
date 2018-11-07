use std::marker::PhantomData;
use std::mem;

use super::{GpuTask, Progress};

pub(crate) enum MaybeDone<T, O, Ec> {
    NotYet(T, PhantomData<Ec>),
    Done(O),
    Gone,
}

impl<T, O, Ec> MaybeDone<T, O, Ec>
where
    T: GpuTask<Ec, Output = O>,
{
    pub fn progress(&mut self, execution_context: &mut Ec) -> Result<bool, T::Error> {
        let res = match self {
            MaybeDone::Done(_) => return Ok(true),
            MaybeDone::NotYet(ref mut task, _) => task.progress(execution_context),
            MaybeDone::Gone => panic!("Cannot progress a Join twice."),
        };

        match res {
            Progress::Finished(Ok(output)) => {
                *self = MaybeDone::Done(output);

                Ok(true)
            }
            Progress::Finished(Err(err)) => Err(err),
            Progress::ContinueFenced => Ok(false),
        }
    }

    pub fn take(&mut self) -> O {
        match mem::replace(self, MaybeDone::Gone) {
            MaybeDone::Done(a) => a,
            _ => panic!(),
        }
    }
}

pub(crate) fn maybe_done<T, O, Ec>(task: T) -> MaybeDone<T, O, Ec>
where
    T: GpuTask<Ec, Output = O>,
{
    MaybeDone::NotYet(task, PhantomData)
}
