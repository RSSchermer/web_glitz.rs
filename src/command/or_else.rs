use super::{ GpuCommand, GpuCommandExt, CommandObject, Execution };

pub struct OrElse<A, F, Ec> where A: GpuCommand<Ec> {
    data: Option<OrElseData<A, F, Ec>>
}

struct OrElseData<A, F, Ec> where A: GpuCommand<Ec> {
    command: CommandObject<A, Ec>,
    f: F
}

impl <A, B, F, Ec> OrElse<A, F, Ec> where A: GpuCommand<Ec>, B: GpuCommand<Ec, Output=A::Output>, F: FnOnce(A::Error) -> B {
    pub fn new<T>(command: T, f: F) -> Self where T: Into<CommandObject<A, Ec>> {
        OrElse {
            data: Some(OrElseData {
                command: command.into(),
                f
            })
        }
    }

    fn execute_internal(&mut self, execution_context: &mut Ec) -> Execution<<B as GpuCommand<Ec>>::Output, <B as GpuCommand<Ec>>::Error, Ec> {
        let OrElseData { command, f } = self.data.take().expect("Cannot execute OrElse twice");

        match command.execute(execution_context) {
            Execution::Finished(Ok(output)) => Execution::Finished(Ok(output)),
            Execution::Finished(Err(err)) => f(err).execute(execution_context),
            Execution::ContinueFenced(command) => Execution::ContinueFenced(OrElse::new(command, f))
        }
    }
}

impl <A, B, F, Ec> GpuCommand<Ec> for OrElse<A, F, Ec> where A: GpuCommand<Ec>, B: GpuCommand<Ec, Output=A::Output>, F: FnOnce(A::Error) -> B {
    type Output = <B as GpuCommand<Ec>>::Output;

    type Error = <B as GpuCommand<Ec>>::Error;

    fn execute_static(mut self, execution_context: &mut Ec) -> Execution<Self::Output, Self::Error, Ec> {
        self.execute_internal(execution_context)
    }

    fn execute_dynamic(mut self: Box<Self>, execution_context: &mut Ec) -> Execution<Self::Output, Self::Error, Ec> {
        self.execute_internal(execution_context)
    }
}