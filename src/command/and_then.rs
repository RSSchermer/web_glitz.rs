use super::{ GpuCommand, GpuCommandExt, CommandObject, Execution };

pub struct AndThen<A, F, Ec> where A: GpuCommand<Ec> {
    data: Option<AndThenData<A, F, Ec>>
}

struct AndThenData<A, F, Ec> where A: GpuCommand<Ec> {
    command: CommandObject<A, Ec>,
    f: F
}

impl <A, B, F, Ec> AndThen<A, F, Ec> where A: GpuCommand<Ec>, B: GpuCommand<Ec, Error=A::Error>, F: FnOnce(A::Output) -> B {
    pub fn new<T>(command: T, f: F) -> Self where T: Into<CommandObject<A, Ec>> {
        AndThen {
            data: Some(AndThenData {
                command: command.into(),
                f
            })
        }
    }

    fn execute_internal(&mut self, execution_context: &mut Ec) -> Execution<<B as GpuCommand<Ec>>::Output, <B as GpuCommand<Ec>>::Error, Ec> {
        let AndThenData { command, f } = self.data.take().expect("Cannot execute AndThen twice");

        match command.execute(execution_context) {
            Execution::Finished(Ok(output)) => f(output).execute(execution_context),
            Execution::Finished(Err(err)) => Execution::Finished(Err(err)),
            Execution::ContinueFenced(command) => Execution::ContinueFenced(Box::new(AndThen::new(CommandObject::<A, Ec>::from(command), f)))
        }
    }
}

impl <A, B, F, Ec> GpuCommand<Ec> for AndThen<A, F, Ec> where A: GpuCommand<Ec>, B: GpuCommand<Ec, Error=A::Error>, F: FnOnce(A::Output) -> B {
    type Output = <B as GpuCommand<Ec>>::Output;

    type Error = <B as GpuCommand<Ec>>::Error;

    fn execute_static(mut self, execution_context: &mut Ec) -> Execution<Self::Output, Self::Error, Ec> {
        self.execute_internal(execution_context)
    }

    fn execute_dynamic(mut self: Box<Self>, execution_context: &mut Ec) -> Execution<Self::Output, Self::Error, Ec> {
        self.execute_internal(execution_context)
    }
}