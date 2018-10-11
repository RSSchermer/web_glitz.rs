use super::{ GpuCommand, GpuCommandExt, CommandObject, Execution };

pub struct MapErr<A, F, Ec> where A: GpuCommand<Ec> {
    data: Option<MapErrData<A, F, Ec>>
}

struct MapErrData<A, F, Ec> where A: GpuCommand<Ec> {
    command: CommandObject<A, Ec>,
    f: F
}

impl <A, B, F, Ec> MapErr<A, F, Ec> where A: GpuCommand<Ec>, F: FnOnce(A::Error) -> B {
    pub fn new<T>(command: T, f: F) -> Self where T: Into<CommandObject<A, Ec>> {
        MapErr {
            data: Some(MapErrData {
                command: command.into(),
                f
            })
        }
    }

    fn execute_internal(&mut self, execution_context: &mut Ec) -> Execution<A::Output, B, Ec> {
        let MapErrData { command, f } = self.data.take().expect("Cannot execute MapErr twice");

        match command.execute(execution_context) {
            Execution::Finished(result) => Execution::Finished(result.map_err(f)),
            Execution::ContinueFenced(command) => Execution::ContinueFenced(MapErr::new(command, f))
        }
    }
}

impl <A, B, F, Ec> GpuCommand<Ec> for MapErr<A, F, Ec> where A: GpuCommand<Ec>, F: FnOnce(A::Error) -> B {
    type Output = <A as GpuCommand<Ec>>::Output;

    type Error = B;

    fn execute_static(mut self, execution_context: &mut Ec) -> Execution<Self::Output, Self::Error, Ec> {
        self.execute_internal(execution_context)
    }

    fn execute_dynamic(mut self: Box<Self>, execution_context: &mut Ec) -> Execution<Self::Output, Self::Error, Ec> {
        self.execute_internal(execution_context)
    }
}