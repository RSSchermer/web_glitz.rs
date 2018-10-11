use super::{ GpuCommand, GpuCommandExt, CommandObject, Execution };

pub struct Map<A, F, Ec> where A: GpuCommand<Ec> {
    data: Option<MapData<A, F, Ec>>
}

struct MapData<A, F, Ec> where A: GpuCommand<Ec> {
    command: CommandObject<A, Ec>,
    f: F
}

impl <A, B, F, Ec> Map<A, F, Ec> where A: GpuCommand<Ec>, F: FnOnce(A::Output) -> B {
    pub fn new<T>(command: T, f: F) -> Self where T: Into<CommandObject<A, Ec>> {
        Map {
            data: Some(MapData {
                command: command.into(),
                f
            })
        }
    }

    fn execute_internal(&mut self, execution_context: &mut Ec) -> Execution<B, A::Error, Ec> {
        let MapData { command, f } = self.data.take().expect("Cannot execute Map twice");

        match command.execute(execution_context) {
            Execution::Finished(result) => Execution::Finished(result.map(f)),
            Execution::ContinueFenced(command) => Execution::ContinueFenced(Map::new(command, f))
        }
    }
}

impl <A, B, F, Ec> GpuCommand<Ec> for Map<A, F, Ec> where A: GpuCommand<Ec>, F: FnOnce(A::Output) -> B {
    type Output = B;

    type Error = <A as GpuCommand<Ec>>::Error;

    fn execute_static(mut self, execution_context: &mut Ec) -> Execution<Self::Output, Self::Error, Ec> {
        self.execute_internal(execution_context)
    }

    fn execute_dynamic(mut self: Box<Self>, execution_context: &mut Ec) -> Execution<Self::Output, Self::Error, Ec> {
        self.execute_internal(execution_context)
    }
}