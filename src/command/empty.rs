use super::{ GpuCommand, Execution };

pub struct Empty;

impl<Ec> GpuCommand<Ec> for Empty {
    type Output = ();

    type Error = ();

    fn execute_static(self, _execution_context: &mut Ec) -> Execution<Self::Output, Self::Error, Ec> {
        Execution::Finished(Ok(()))
    }

    fn execute_dynamic(self: Box<Self>, _execution_context: &mut Ec) -> Execution<Self::Output, Self::Error, Ec> {
        Execution::Finished(Ok(()))
    }
}