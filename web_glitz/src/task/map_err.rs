use std::marker::PhantomData;

use super::{GpuTask, Progress, TryGpuTask};

pub struct MapErr<A, F, Ec>
where
    A: TryGpuTask<Ec>,
{
    task: A,
    f: Option<F>,
    ec: PhantomData<Ec>,
}

impl<A, B, F, Ec> MapErr<A, F, Ec>
where
    A: TryGpuTask<Ec>,
    F: FnOnce(A::Error) -> B,
    B: 'static,
{
    pub fn new(task: A, f: F) -> Self {
        MapErr {
            task,
            f: Some(f),
            ec: PhantomData,
        }
    }
}

impl<A, B, F, Ec> GpuTask<Ec> for MapErr<A, F, Ec>
where
    A: TryGpuTask<Ec>,
    F: FnOnce(A::Error) -> B,
    B: 'static,
{
    type Output = Result<A::Ok, B>;

    fn progress(&mut self, execution_context: &mut Ec) -> Progress<Self::Output> {
        match self.task.try_progress(execution_context) {
            Progress::Finished(result) => {
                let f = self
                    .f
                    .take()
                    .expect("Cannot execute MapErr again after it has finished");

                Progress::Finished(result.map_err(f))
            }
            Progress::ContinueFenced => Progress::ContinueFenced,
        }
    }
}
