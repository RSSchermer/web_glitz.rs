use std::marker::PhantomData;

use super::{GpuTask, Execution };

pub struct Map<A, F, Ec> where A: GpuTask<Ec> {
    task: A,
    f: Option<F>,
    ec: PhantomData<Ec>
}

impl <A, B, F, Ec> Map<A, F, Ec> where A: GpuTask<Ec>, F: FnOnce(A::Output) -> B, B: 'static {
    pub fn new(task: A, f: F) -> Self {
        Map {
            task,
            f: Some(f),
            ec: PhantomData
        }
    }
}

impl <A, B, F, Ec> GpuTask<Ec> for Map<A, F, Ec> where A: GpuTask<Ec>, F: FnOnce(A::Output) -> B, B: 'static {
    type Output = B;

    type Error = A::Error;

    fn progress(&mut self, execution_context: &mut Ec) -> Execution<B, A::Error> {
        match self.task.progress(execution_context) {
            Execution::Finished(result) => {
                let f = self.f.take().expect("Cannot execute Map again after it has finished");

                Execution::Finished(result.map(f))
            },
            Execution::ContinueFenced => Execution::ContinueFenced
        }
    }
}
