use super::{ GpuCommand, GpuCommandExt, CommandObject, Execution };

pub struct Then<A, F, Ec> where A: GpuCommand<Ec> {
    data: Option<ThenData<A, F, Ec>>
}

struct ThenData<A, F, Ec> where A: GpuCommand<Ec> {
    command: CommandObject<A, Ec>,
    f: F
}

impl <A, B, F, Ec> Then<A, F, Ec> where A: GpuCommand<Ec>, B: GpuCommand<Ec>, F: FnOnce(Result<A::Output, A::Error>) -> B {
    pub fn new<T>(command: T, f: F) -> Self where T: Into<CommandObject<A, Ec>> {
        Then {
            data: Some(ThenData {
                command: command.into(),
                f
            })
        }
    }

    fn execute_internal(&mut self, execution_context: &mut Ec) -> Execution<<B as GpuCommand<Ec>>::Output, <B as GpuCommand<Ec>>::Error, Ec> {
        let ThenData { command, f } = self.data.take().expect("Cannot execute Then twice");

        match command.execute(execution_context) {
            Execution::Finished(result) => f(result).execute(execution_context),
            Execution::ContinueFenced(command) => Execution::ContinueFenced(Then::new(command, f))
        }
    }
}

impl <A, B, F, Ec> GpuCommand<Ec> for Then<A, F, Ec> where A: GpuCommand<Ec>, B: GpuCommand<Ec>, F: FnOnce(Result<A::Output, A::Error>) -> B {
    type Output = <B as GpuCommand<Ec>>::Output;

    type Error = <B as GpuCommand<Ec>>::Error;

    fn execute_static(mut self, execution_context: &mut Ec) -> Execution<Self::Output, Self::Error, Ec> {
        self.execute_internal(execution_context)
    }

    fn execute_dynamic(mut self: Box<Self>, execution_context: &mut Ec) -> Execution<Self::Output, Self::Error, Ec> {
        self.execute_internal(execution_context)
    }
}
