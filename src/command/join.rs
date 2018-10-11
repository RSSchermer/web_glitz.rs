use std::mem;
use std::boxed::Box;

use super::{ GpuCommand, CommandObject, Execution };

macro_rules! generate {
    ($(
        $(#[$doc:meta])*
        ($Join:ident, $JoinData:ident, <A, $($B:ident),*>, <T_a, $($T_b:ident),*>),
    )*) => ($(
        $(#[$doc])*
        pub struct $Join<A, $($B),*, Ec> where A: GpuCommand<Ec>, $($B: GpuCommand<Ec, Error=A::Error>),*
        {
            data: Option<$JoinData<A, $($B),*, Ec>>
        }

        struct $JoinData<A, $($B),*, Ec> where A: GpuCommand<Ec>, $($B: GpuCommand<Ec, Error=A::Error>),*
        {
            a: MaybeDone<A::Output, A::Error, Ec>,
            $($B: MaybeDone<$B::Output, A::Error, Ec>),*
        }

        impl<A, $($B),*, Ec> $Join<A, $($B),*, Ec> where A: GpuCommand<Ec> + 'static, $($B: GpuCommand<Ec, Error=A::Error> + 'static),*, Ec: 'static {
            pub fn new<T_a, $($T_b),*>(a: T_a, $($B: $T_b),*) -> Self where T_a: Into<CommandObject<A, Ec>>, $($T_b: Into<CommandObject<$B, Ec>>),*{
                $Join {
                    data: Some($JoinData {
                        a: MaybeDone::NotYet(a.into().into_box()),
                        $($B: MaybeDone::NotYet($B.into().into_box())),*
                    })
                }
            }

            fn execute_internal(&mut self, execution_context: &mut Ec) -> Execution<(A::Output, $($B::Output),*), A::Error, Ec> {
                let $JoinData { mut a, $(mut $B),* } = self.data.take().expect("Cannot execute a Join twice");

                let mut all_done = match a.progress(execution_context) {
                    Ok(done) => done,
                    Err(err) => return Execution::Finished(Err(err))
                };

                $(
                    all_done = match $B.progress(execution_context) {
                        Ok(done) => all_done && done,
                        Err(err) => return Execution::Finished(Err(err))
                    };
                )*

                if all_done {
                    Execution::Finished(Ok((a.take(), $($B.take()),*)))
                } else {
                    Execution::ContinueFenced(Box::new($Join {
                        data: Some($JoinData {
                            a,
                            $($B),*
                        })
                    }))
                }
            }
        }

        impl<A, $($B),*, Ec> GpuCommand<Ec> for $Join<A, $($B),*, Ec> where A: GpuCommand<Ec> + 'static, $($B: GpuCommand<Ec, Error=A::Error> + 'static),*, Ec: 'static {
            type Output = (A::Output, $($B::Output),*);

            type Error = A::Error;

            fn execute_static(mut self, execution_context: &mut Ec) -> Execution<Self::Output, Self::Error, Ec> {
                self.execute_internal(execution_context)
            }

            fn execute_dynamic(mut self: Box<Self>, execution_context: &mut Ec) -> Execution<Self::Output, Self::Error, Ec> {
                self.execute_internal(execution_context)
            }
        }
    )*)
}

generate! {
    /// Command for the `join` combinator, waiting for two commands to complete in no particular
    /// order.
    (Join, JoinData, <A, B>, <T_a, T_b>),

    /// Command for the `join3` combinator, waiting for three commands to complete in no particular
    /// order.
    (Join3, Join3Data, <A, B, C>, <T_a, T_b, T_c>),

    /// Command for the `join4` combinator, waiting for four commands to complete in no particular
    /// order.
    (Join4, Join4Data, <A, B, C, D>, <T_a, T_b, T_c, T_d>),

    /// Command for the `join5` combinator, waiting for five commands to complete in no particular
    /// order.
    (Join5, Join5Data, <A, B, C, D, E>, <T_a, T_b, T_c, T_d, T_e>),
}

enum MaybeDone<O, E, Ec> {
    NotYet(Box<GpuCommand<Ec, Output=O, Error=E>>),
    Done(O),
    Gone
}

impl<O, E, Ec> MaybeDone<O, E, Ec> {
    fn progress(&mut self, execution_context: &mut Ec) -> Result<bool, E> {
        let res = match *self {
            MaybeDone::Done(_) => return Ok(true),
            MaybeDone::NotYet(command) => command.execute_dynamic(execution_context),
            MaybeDone::Gone => panic!("Cannot execute a Join twice.")
        };

        match res {
            Execution::Finished(Ok(output)) => {
                *self = MaybeDone::Done(output);

                Ok(true)
            },
            Execution::Finished(Err(err)) => Err(err),
            Execution::ContinueFenced(command) => {
                *self = MaybeDone::NotYet(command);

                Ok(false)
            }
        }
    }

    fn take(&mut self) -> O {
        match mem::replace(self, MaybeDone::Gone) {
            MaybeDone::Done(a) => a,
            _ => panic!(),
        }
    }
}
