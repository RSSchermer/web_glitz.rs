use super::{ GpuCommand, GpuCommandExt, CommandObject, Execution };

pub struct And<A, B, Ec> where A: GpuCommand<Ec>, B: GpuCommand<Ec, Error=A::Error> {
    data: Option<AndData<A, B, Ec>>
}

pub struct AndData<A, B, Ec> where A: GpuCommand<Ec>, B: GpuCommand<Ec, Error=A::Error> {
    a: CommandObject<A, Ec>,
    b: CommandObject<B, Ec>
}

impl<A, B, Ec> And<A, B, Ec> where A: GpuCommand<Ec>, B: GpuCommand<Ec, Error=A::Error> {
    pub fn new<Ta, Tb>(a: Ta, b: Tb) -> Self where Ta: Into<CommandObject<A, Ec>>, Tb: Into<CommandObject<B, Ec>> {
        And {
            data: AndData { a, b }
        }
    }

    fn execute_internal(&mut self, execution_context: &mut Ec) -> Execution<<B as GpuCommand<Ec>>::Output, <A as GpuCommand<Ec>>::Error, Ec> {
        let AndData { a, b } = self.data.take().expect("Cannot execute And twice");

        match a.execute(execution_context) {
            Execution::Finished(Ok(_)) => b.execute(execution_context),
            Execution::Finished(Err(err)) => Execution::Finished(Err(err)),
            Execution::ContinueFenced(command) => Execution::ContinueFenced(And::new(command, b))
        }
    }
}

impl<A, B, Ec> GpuCommand<Ec> for And<A, B, Ec> where A: GpuCommand<Ec>, B: GpuCommand<Ec, Error=A::Error> {
    type Output = <B as GpuCommand<Ec>>::Output;

    type Error = <A as GpuCommand<Ec>>::Error;

    fn execute_static(mut self, execution_context: &mut Ec) -> Execution<Self::Output, Self::Error, Ec> {
        self.execute_internal(execution_context)
    }

    fn execute_dynamic(mut self: Box<Self>, execution_context: &mut Ec) -> Execution<Self::Output, Self::Error, Ec> {
        self.execute_internal(execution_context)
    }
}
