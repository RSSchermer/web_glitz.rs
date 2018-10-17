use std::marker::PhantomData;

use super::{ GpuCommand, Execution };

pub struct OrElse<C1, C2, F, Ec> where C1: GpuCommand<Ec>, C2: GpuCommand<Ec, Output=C1::Output>, F: FnOnce(C1::Error) -> C2 {
    state: OrElseState<C1, C2, F>,
    ec: PhantomData<Ec>
}

enum OrElseState<C1, C2, F> {
    A(C1, Option<F>),
    B(C2)
}

impl <C1, C2, F, Ec> OrElse<C1, C2, F, Ec> where C1: GpuCommand<Ec>, C2: GpuCommand<Ec, Output=C1::Output>, F: FnOnce(C1::Error) -> C2 {
    pub fn new(command: C1, f: F) -> Self {
        OrElse {
            state: OrElseState::A(command, Some(f)),
            ec: PhantomData
        }
    }
}

impl <C1, C2, F, Ec> GpuCommand<Ec> for OrElse<C1, C2, F, Ec> where C1: GpuCommand<Ec>, C2: GpuCommand<Ec, Output=C1::Output>, F: FnOnce(C1::Error) -> C2 {
    type Output = C2::Output;

    type Error = C2::Error;

    fn execute(&mut self, execution_context: &mut Ec) -> Execution<C2::Output, C2::Error> {
        match self.state {
            OrElseState::A(ref mut command, ref mut f) => {
                match command.execute(execution_context) {
                    Execution::Finished(Ok(output)) => Execution::Finished(Ok(output)),
                    Execution::Finished(Err(err)) => {
                        let f = f.take().expect("Cannot execute state A again after it finishes");
                        let mut b = f(err);
                        let execution = b.execute(execution_context);

                        self.state = OrElseState::B(b);

                        execution
                    },
                    Execution::ContinueFenced => Execution::ContinueFenced
                }
            }
            OrElseState::B(ref mut command) => command.execute(execution_context)
        }
    }
}
