use std::marker::PhantomData;

use super::{GpuTask, Progress};

pub struct Map<A, F, Ec> {
    task: A,
    f: Option<F>,
    ec: PhantomData<Ec>,
}

impl<A, B, F, Ec> Map<A, F, Ec>
where
    A: GpuTask<Ec>,
    F: FnOnce(A::Output) -> B,
    B: 'static,
{
    pub fn new(task: A, f: F) -> Self {
        Map {
            task,
            f: Some(f),
            ec: PhantomData,
        }
    }
}

impl<A, B, F, Ec> GpuTask<Ec> for Map<A, F, Ec>
where
    A: GpuTask<Ec>,
    F: FnOnce(A::Output) -> B,
    B: 'static,
{
    type Output = B;

    fn progress(&mut self, execution_context: &mut Ec) -> Progress<Self::Output> {
        match self.task.progress(execution_context) {
            Progress::Finished(output) => {
                let f = self
                    .f
                    .take()
                    .expect("Cannot execute Map again after it has finished");

                Progress::Finished(f(output))
            }
            Progress::ContinueFenced => Progress::ContinueFenced,
        }
    }
}
