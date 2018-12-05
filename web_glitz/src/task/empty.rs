use super::{GpuTask, Progress};

pub struct Empty;

impl<Ec> GpuTask<Ec> for Empty {
    type Output = ();

    fn progress(&mut self, _execution_context: &mut Ec) -> Progress<Self::Output> {
        Progress::Finished(())
    }
}
