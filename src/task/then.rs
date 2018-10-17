use std::marker::PhantomData;

use super::{GpuTask, Execution };

pub struct Then<C1, C2, F, Ec> where C1: GpuTask<Ec>, C2: GpuTask<Ec>, F: FnOnce(Result<C1::Output, C1::Error>) -> C2 {
    state: ThenState<C1, C2, F>,
    ec: PhantomData<Ec>
}

enum ThenState<C1, C2, F> {
    A(C1, Option<F>),
    B(C2)
}

impl <C1, C2, F, Ec> Then<C1, C2, F, Ec> where C1: GpuTask<Ec>, C2: GpuTask<Ec>, F: FnOnce(Result<C1::Output, C1::Error>) -> C2 {
    pub fn new(task: C1, f: F) -> Self {
        Then {
            state: ThenState::A(task, Some(f)),
            ec: PhantomData
        }
    }
}

impl <C1, C2, F, Ec> GpuTask<Ec> for Then<C1, C2, F, Ec> where C1: GpuTask<Ec>, C2: GpuTask<Ec>, F: FnOnce(Result<C1::Output, C1::Error>) -> C2 {
    type Output = C2::Output;

    type Error = C2::Error;

    fn progress(&mut self, execution_context: &mut Ec) -> Execution<C2::Output, C2::Error> {
        match self.state {
            ThenState::A(ref mut task, ref mut f) => {
                match task.progress(execution_context) {
                    Execution::Finished(result) => {
                        let f = f.take().expect("Cannot execute state A again after it finishes");
                        let mut b = f(result);
                        let execution = b.progress(execution_context);

                        self.state = ThenState::B(b);

                        execution
                    }
                    Execution::ContinueFenced => Execution::ContinueFenced
                }
            }
            ThenState::B(ref mut task) => task.progress(execution_context)
        }
    }
}
