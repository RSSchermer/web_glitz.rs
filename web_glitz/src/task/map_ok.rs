use std::marker::PhantomData;

use super::{GpuTask, Progress, TryGpuTask};

pub struct MapOk<A, F, Ec>
    where
        A: TryGpuTask<Ec>,
{
    task: A,
    f: Option<F>,
    ec: PhantomData<Ec>,
}

impl<A, B, F, Ec> MapOk<A, F, Ec>
    where
        A: TryGpuTask<Ec>,
        F: FnOnce(A::Ok) -> B,
        B: 'static,
{
    pub fn new(task: A, f: F) -> Self {
        MapOk {
            task,
            f: Some(f),
            ec: PhantomData,
        }
    }
}

impl<A, B, F, Ec> GpuTask<Ec> for MapOk<A, F, Ec>
    where
        A: TryGpuTask<Ec>,
        F: FnOnce(A::Ok) -> B,
        B: 'static,
{
    type Output = Result<B, A::Error>;

    fn progress(&mut self, execution_context: &mut Ec) -> Progress<Self::Output> {
        match self.task.try_progress(execution_context) {
            Progress::Finished(result) => {
                let f = self
                    .f
                    .take()
                    .expect("Cannot execute MapOk again after it has finished");

                Progress::Finished(result.map(f))
            }
            Progress::ContinueFenced => Progress::ContinueFenced,
        }
    }
}
