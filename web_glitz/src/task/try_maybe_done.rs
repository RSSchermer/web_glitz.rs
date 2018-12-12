use std::marker::PhantomData;
use std::mem;

use super::{Progress, TryGpuTask};

pub(crate) enum TryMaybeDone<T, O, Ec> {
    NotYet(T, PhantomData<Ec>),
    Done(O),
    Gone,
}

impl<T, O, Ec> TryMaybeDone<T, O, Ec>
where
    T: TryGpuTask<Ec, Ok = O>,
{
    pub fn try_progress(&mut self, execution_context: &mut Ec) -> Result<bool, T::Error> {
        let res = match self {
            TryMaybeDone::Done(_) => return Ok(true),
            TryMaybeDone::NotYet(ref mut task, _) => task.try_progress(execution_context),
            TryMaybeDone::Gone => panic!("Cannot progress a Join twice."),
        };

        match res {
            Progress::Finished(Ok(output)) => {
                *self = TryMaybeDone::Done(output);

                Ok(true)
            }
            Progress::Finished(Err(err)) => Err(err),
            Progress::ContinueFenced => Ok(false),
        }
    }

    pub fn take(&mut self) -> O {
        match mem::replace(self, TryMaybeDone::Gone) {
            TryMaybeDone::Done(a) => a,
            _ => panic!(),
        }
    }
}

pub(crate) fn try_maybe_done<T, O, Ec>(task: T) -> TryMaybeDone<T, O, Ec>
where
    T: TryGpuTask<Ec, Ok = O>,
{
    TryMaybeDone::NotYet(task, PhantomData)
}
