use std::marker::PhantomData;

use super::{ GpuCommand, Execution };

pub struct MapErr<A, F, Ec> where A: GpuCommand<Ec> {
    command: A,
    f: Option<F>,
    ec: PhantomData<Ec>
}

impl <A, B, F, Ec> MapErr<A, F, Ec> where A: GpuCommand<Ec>, F: FnOnce(A::Error) -> B, B: 'static {
    pub fn new(command: A, f: F) -> Self {
        MapErr {
            command,
            f: Some(f),
            ec: PhantomData
        }
    }
}

impl <A, B, F, Ec> GpuCommand<Ec> for MapErr<A, F, Ec> where A: GpuCommand<Ec>, F: FnOnce(A::Error) -> B, B: 'static {
    type Output = A::Output;

    type Error = B;

    fn execute(&mut self, execution_context: &mut Ec) -> Execution<A::Output, B> {
        match self.command.execute(execution_context) {
            Execution::Finished(result) => {
                let f = self.f.take().expect("Cannot execute MapErr again after it has finished");

                Execution::Finished(result.map_err(f))
            },
            Execution::ContinueFenced => Execution::ContinueFenced
        }
    }
}
