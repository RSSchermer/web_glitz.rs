use std::marker::PhantomData;

use super::{GpuTask, Progress};

pub struct AndThen<C1, C2, F, Ec>
where
    C1: GpuTask<Ec>,
    C2: GpuTask<Ec, Error = C1::Error>,
    F: FnOnce(C1::Output) -> C2,
{
    state: AndThenState<C1, C2, F>,
    ec: PhantomData<Ec>,
}

enum AndThenState<C1, C2, F> {
    A(C1, Option<F>),
    B(C2),
}

impl<C1, C2, F, Ec> AndThen<C1, C2, F, Ec>
where
    C1: GpuTask<Ec>,
    C2: GpuTask<Ec, Error = C1::Error>,
    F: FnOnce(C1::Output) -> C2,
{
    pub fn new(task: C1, f: F) -> Self {
        AndThen {
            state: AndThenState::A(task, Some(f)),
            ec: PhantomData,
        }
    }
}

impl<C1, C2, F, Ec> GpuTask<Ec> for AndThen<C1, C2, F, Ec>
where
    C1: GpuTask<Ec>,
    C2: GpuTask<Ec, Error = C1::Error>,
    F: FnOnce(C1::Output) -> C2,
{
    type Output = C2::Output;

    type Error = C2::Error;

    fn progress(&mut self, execution_context: &mut Ec) -> Progress<C2::Output, C2::Error> {
        match self.state {
            AndThenState::A(ref mut task, ref mut f) => match task.progress(execution_context) {
                Progress::Finished(Ok(output)) => {
                    let f = f
                        .take()
                        .expect("Cannot execute state A again after it finishes");
                    let mut b = f(output);
                    let execution = b.progress(execution_context);

                    self.state = AndThenState::B(b);

                    execution
                }
                Progress::Finished(Err(err)) => Progress::Finished(Err(err)),
                Progress::ContinueFenced => Progress::ContinueFenced,
            },
            AndThenState::B(ref mut task) => task.progress(execution_context),
        }
    }
}