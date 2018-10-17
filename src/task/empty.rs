use super::{GpuTask, Execution };

pub struct Empty;

impl<Ec> GpuTask<Ec> for Empty {
    type Output = ();

    type Error = ();

    fn progress(&mut self, _execution_context: &mut Ec) -> Execution<Self::Output, Self::Error> {
        Execution::Finished(Ok(()))
    }
}