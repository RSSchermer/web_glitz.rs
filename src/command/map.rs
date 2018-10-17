use std::marker::PhantomData;

use super::{ GpuCommand, Execution };

pub struct Map<A, F, Ec> where A: GpuCommand<Ec> {
    command: A,
    f: Option<F>,
    ec: PhantomData<Ec>
}

impl <A, B, F, Ec> Map<A, F, Ec> where A: GpuCommand<Ec>, F: FnOnce(A::Output) -> B, B: 'static {
    pub fn new(command: A, f: F) -> Self {
        Map {
            command,
            f: Some(f),
            ec: PhantomData
        }
    }
}

impl <A, B, F, Ec> GpuCommand<Ec> for Map<A, F, Ec> where A: GpuCommand<Ec>, F: FnOnce(A::Output) -> B, B: 'static {
    type Output = B;

    type Error = A::Error;

    fn execute(&mut self, execution_context: &mut Ec) -> Execution<B, A::Error> {
        match self.command.execute(execution_context) {
            Execution::Finished(result) => {
                let f = self.f.take().expect("Cannot execute Map again after it has finished");

                Execution::Finished(result.map(f))
            },
            Execution::ContinueFenced => Execution::ContinueFenced
        }
    }
}
