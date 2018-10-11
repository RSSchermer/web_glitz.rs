use super::{ GpuCommand, GpuCommandExt, CommandObject, Execution };

pub struct Or<A, B, Ec> where A: GpuCommand<Ec>, B: GpuCommand<Ec, Output=A::Output> {
    data: Option<OrData<A, B, Ec>>
}

pub struct OrData<A, B, Ec> where A: GpuCommand<Ec>, B: GpuCommand<Ec, Output=A::Output> {
    a: CommandObject<A, Ec>,
    b: CommandObject<B, Ec>
}

impl<A, B, Ec> Or<A, B, Ec> where A: GpuCommand<Ec>, B: GpuCommand<Ec, Output=A::Output> {
    pub fn new<Ta, Tb>(a: Ta, b: Tb) -> Self where Ta: Into<CommandObject<A, Ec>>, Tb: Into<CommandObject<B, Ec>> {
        Or {
            data: OrData { a, b }
        }
    }

    fn execute_internal(&mut self, execution_context: &mut Ec) -> Execution<<A as GpuCommand<Ec>>::Output, <B as GpuCommand<Ec>>::Error, Ec> {
        let OrData { a, b } = self.data.take().expect("Cannot execute Or twice");

        match a.execute(execution_context) {
            Execution::Finished(Ok(output)) => Execution::Finished(Ok(output)),
            Execution::Finished(Err(_)) => b.execute(execution_context),
            Execution::ContinueFenced(command) => Execution::ContinueFenced(And::new(command, b))
        }
    }
}

impl<A, B, Ec> GpuCommand<Ec> for Or<A, B, Ec> where A: GpuCommand<Ec>, B: GpuCommand<Ec, Output=A::Output> {
    type Output = <A as GpuCommand<Ec>>::Output;

    type Error = <B as GpuCommand<Ec>>::Error;

    fn execute_static(mut self, execution_context: &mut Ec) -> Execution<Self::Output, Self::Error, Ec> {
        self.execute_internal(execution_context)
    }

    fn execute_dynamic(mut self: Box<Self>, execution_context: &mut Ec) -> Execution<Self::Output, Self::Error, Ec> {
        self.execute_internal(execution_context)
    }
}