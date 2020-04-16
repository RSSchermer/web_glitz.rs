use crate::task::{GpuTask, Progress, ContextId};

pub struct OptionTask<T> {
    option: Option<T>
}

unsafe impl<Ec, T> GpuTask<Ec> for OptionTask<T> where T: GpuTask<Ec> {
    type Output = Option<T::Output>;

    fn context_id(&self) -> ContextId {
        match &self.option {
            Some(task) => task.context_id(),
            None => ContextId::Any
        }
    }

    fn progress(&mut self, execution_context: &mut Ec) -> Progress<Self::Output> {
        match &mut self.option {
            Some(task) => task.progress(execution_context).map(Some),
            None => Progress::Finished(None)
        }
    }
}

impl<T> From<Option<T>> for OptionTask<T> {
    fn from(option: Option<T>) -> Self {
        OptionTask {
            option
        }
    }
}
