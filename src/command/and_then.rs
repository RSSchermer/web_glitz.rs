use std::marker::PhantomData;

use super::{ GpuCommand, Execution };

pub struct AndThen<C1, C2, F, Ec> where C1: GpuCommand<Ec>, C2: GpuCommand<Ec, Error=C1::Error>, F: FnOnce(C1::Output) -> C2 {
    state: AndThenState<C1, C2, F>,
    ec: PhantomData<Ec>
}

enum AndThenState<C1, C2, F> {
    A(C1, Option<F>),
    B(C2)
}

impl <C1, C2, F, Ec> AndThen<C1, C2, F, Ec> where C1: GpuCommand<Ec>, C2: GpuCommand<Ec, Error=C1::Error>, F: FnOnce(C1::Output) -> C2 {
    pub fn new(command: C1, f: F) -> Self {
        AndThen {
            state: AndThenState::A(command, Some(f)),
            ec: PhantomData
        }
    }
}

impl <C1, C2, F, Ec> GpuCommand<Ec> for AndThen<C1, C2, F, Ec> where C1: GpuCommand<Ec>, C2: GpuCommand<Ec, Error=C1::Error>, F: FnOnce(C1::Output) -> C2 {
    type Output = C2::Output;

    type Error = C2::Error;

    fn execute(&mut self, execution_context: &mut Ec) -> Execution<C2::Output, C2::Error> {
        match self.state {
            AndThenState::A(ref mut command, ref mut f) => {
                match command.execute(execution_context) {
                    Execution::Finished(Ok(output)) => {
                        let f = f.take().expect("Cannot execute state A again after it finishes");
                        let mut b = f(output);
                        let execution = b.execute(execution_context);

                        self.state = AndThenState::B(b);

                        execution
                    }
                    Execution::Finished(Err(err)) => Execution::Finished(Err(err)),
                    Execution::ContinueFenced => Execution::ContinueFenced
                }
            }
            AndThenState::B(ref mut command) => command.execute(execution_context)
        }
    }
}
