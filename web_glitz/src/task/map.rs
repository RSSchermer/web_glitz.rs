use crate::task::{ContextId, GpuTask, Progress};

#[derive(Clone)]
pub struct Map<T, F> {
    task: T,
    f: Option<F>,
}

impl<T, F> Map<T, F> {
    pub(crate) fn new(task: T, f: F) -> Self {
        Map { task, f: Some(f) }
    }
}

unsafe impl<Ec, T, F, U> GpuTask<Ec> for Map<T, F>
where
    T: GpuTask<Ec>,
    F: FnOnce(T::Output) -> U,
{
    type Output = U;

    fn context_id(&self) -> ContextId {
        self.task.context_id()
    }

    fn progress(&mut self, execution_context: &mut Ec) -> Progress<Self::Output> {
        self.task.progress(execution_context).map(|output| {
            let f = self
                .f
                .take()
                .expect("Cannot progress Map after it has finished.");

            f(output)
        })
    }
}
