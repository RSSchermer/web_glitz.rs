use std::marker::PhantomData;

use super::{GpuTask, Progress};

pub struct OrElse<C1, C2, F, Ec>
where
    C1: GpuTask<Ec>,
    C2: GpuTask<Ec, Output = C1::Output>,
    F: FnOnce(C1::Error) -> C2,
{
    state: OrElseState<C1, C2, F>,
    ec: PhantomData<Ec>,
}

enum OrElseState<C1, C2, F> {
    A(C1, Option<F>),
    B(C2),
}

impl<C1, C2, F, Ec> OrElse<C1, C2, F, Ec>
where
    C1: GpuTask<Ec>,
    C2: GpuTask<Ec, Output = C1::Output>,
    F: FnOnce(C1::Error) -> C2,
{
    pub fn new(task: C1, f: F) -> Self {
        OrElse {
            state: OrElseState::A(task, Some(f)),
            ec: PhantomData,
        }
    }
}

impl<C1, C2, F, Ec> GpuTask<Ec> for OrElse<C1, C2, F, Ec>
where
    C1: GpuTask<Ec>,
    C2: GpuTask<Ec, Output = C1::Output>,
    F: FnOnce(C1::Error) -> C2,
{
    type Output = C2::Output;

    type Error = C2::Error;

    fn progress(&mut self, execution_context: &mut Ec) -> Progress<C2::Output, C2::Error> {
        match self.state {
            OrElseState::A(ref mut task, ref mut f) => match task.progress(execution_context) {
                Progress::Finished(Ok(output)) => Progress::Finished(Ok(output)),
                Progress::Finished(Err(err)) => {
                    let f = f
                        .take()
                        .expect("Cannot execute state A again after it finishes");
                    let mut b = f(err);
                    let execution = b.progress(execution_context);

                    self.state = OrElseState::B(b);

                    execution
                }
                Progress::ContinueFenced => Progress::ContinueFenced,
            },
            OrElseState::B(ref mut task) => task.progress(execution_context),
        }
    }
}
