use std::marker::PhantomData;

use super::{GpuTask, Execution };

pub struct MapErr<A, F, Ec> where A: GpuTask<Ec> {
    task: A,
    f: Option<F>,
    ec: PhantomData<Ec>
}

impl <A, B, F, Ec> MapErr<A, F, Ec> where A: GpuTask<Ec>, F: FnOnce(A::Error) -> B, B: 'static {
    pub fn new(task: A, f: F) -> Self {
        MapErr {
            task,
            f: Some(f),
            ec: PhantomData
        }
    }
}

impl <A, B, F, Ec> GpuTask<Ec> for MapErr<A, F, Ec> where A: GpuTask<Ec>, F: FnOnce(A::Error) -> B, B: 'static {
    type Output = A::Output;

    type Error = B;

    fn progress(&mut self, execution_context: &mut Ec) -> Execution<A::Output, B> {
        match self.task.progress(execution_context) {
            Execution::Finished(result) => {
                let f = self.f.take().expect("Cannot execute MapErr again after it has finished");

                Execution::Finished(result.map_err(f))
            },
            Execution::ContinueFenced => Execution::ContinueFenced
        }
    }
}
