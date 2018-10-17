use super::{ GpuCommand, Execution };

pub struct Empty;

impl<Ec> GpuCommand<Ec> for Empty {
    type Output = ();

    type Error = ();

    fn execute(&mut self, _execution_context: &mut Ec) -> Execution<Self::Output, Self::Error> {
        Execution::Finished(Ok(()))
    }
}