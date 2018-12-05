use std::marker::PhantomData;

use super::{GpuTask, Progress};

pub struct Then<C1, C2, F, Ec>
where
    C1: GpuTask<Ec>,
    C2: GpuTask<Ec>,
    F: FnOnce(C1::Output) -> C2,
{
    state: ThenState<C1, C2, F>,
    ec: PhantomData<Ec>,
}

enum ThenState<C1, C2, F> {
    A(C1, Option<F>),
    B(C2),
}

impl<C1, C2, F, Ec> Then<C1, C2, F, Ec>
where
    C1: GpuTask<Ec>,
    C2: GpuTask<Ec>,
    F: FnOnce(C1::Output) -> C2,
{
    pub fn new(task: C1, f: F) -> Self {
        Then {
            state: ThenState::A(task, Some(f)),
            ec: PhantomData,
        }
    }
}

impl<C1, C2, F, Ec> GpuTask<Ec> for Then<C1, C2, F, Ec>
where
    C1: GpuTask<Ec>,
    C2: GpuTask<Ec>,
    F: FnOnce(C1::Output) -> C2,
{
    type Output = C2::Output;

    fn progress(&mut self, execution_context: &mut Ec) -> Progress<Self::Output> {
        match self.state {
            ThenState::A(ref mut task, ref mut f) => match task.progress(execution_context) {
                Progress::Finished(output) => {
                    let f = f
                        .take()
                        .expect("Cannot execute state A again after it finishes");
                    let mut b = f(output);
                    let execution = b.progress(execution_context);

                    self.state = ThenState::B(b);

                    execution
                }
                Progress::ContinueFenced => Progress::ContinueFenced,
            },
            ThenState::B(ref mut task) => task.progress(execution_context),
        }
    }
}
