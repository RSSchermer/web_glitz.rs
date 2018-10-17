#![allow(non_snake_case)]

use std::marker::PhantomData;
use std::mem;

use super::{GpuTask, Execution };

macro_rules! generate {
    ($(
        $(#[$doc:meta])*
        ($Join:ident, <A, $($B:ident),*>),
    )*) => ($(
        $(#[$doc])*
        pub struct $Join<A, $($B),*, Ec> where A: GpuTask<Ec>, $($B: GpuTask<Ec, Error=A::Error>),*
        {
            a: MaybeDone<A, Ec>,
            $($B: MaybeDone<$B, Ec>),*
        }

        impl<A, $($B),*, Ec> $Join<A, $($B),*, Ec> where A: GpuTask<Ec>, $($B: GpuTask<Ec, Error=A::Error>),* {
            pub fn new(a: A, $($B: $B),*) -> Self {
                $Join {
                    a: MaybeDone::NotYet(a, PhantomData),
                    $($B: MaybeDone::NotYet($B, PhantomData)),*
                }
            }
        }

        impl<A, $($B),*, Ec> GpuTask<Ec> for $Join<A, $($B),*, Ec> where A: GpuTask<Ec>, $($B: GpuTask<Ec, Error=A::Error>),* {
            type Output = (A::Output, $($B::Output),*);

            type Error = A::Error;

            fn progress(&mut self, execution_context: &mut Ec) -> Execution<(A::Output, $($B::Output),*), A::Error> {
                let mut all_done = match self.a.progress(execution_context) {
                    Ok(done) => done,
                    Err(err) => return Execution::Finished(Err(err))
                };

                $(
                    all_done = match self.$B.progress(execution_context) {
                        Ok(done) => all_done && done,
                        Err(err) => return Execution::Finished(Err(err))
                    };
                )*

                if all_done {
                    Execution::Finished(Ok((self.a.take(), $(self.$B.take()),*)))
                } else {
                    Execution::ContinueFenced
                }
            }
        }
    )*)
}

generate! {
    /// Task for the `join` combinator, waiting for two tasks to complete in no particular order.
    (Join, <A, B>),

    /// Task for the `join3` combinator, waiting for three tasks to complete in no particular order.
    (Join3, <A, B, C>),

    /// Task for the `join4` combinator, waiting for four tasks to complete in no particular order.
    (Join4, <A, B, C, D>),

    /// Task for the `join5` combinator, waiting for five tasks to complete in no particular order.
    (Join5, <A, B, C, D, E>),
}

enum MaybeDone<T, Ec> where T: GpuTask<Ec> {
    NotYet(T, PhantomData<Ec>),
    Done(T::Output),
    Gone
}

impl<T, Ec> MaybeDone<T, Ec> where T: GpuTask<Ec> {
    fn progress(&mut self, execution_context: &mut Ec) -> Result<bool, T::Error> {
        let res = match self {
            MaybeDone::Done(_) => return Ok(true),
            MaybeDone::NotYet(ref mut task, _) => task.progress(execution_context),
            MaybeDone::Gone => panic!("Cannot progress a Join twice.")
        };

        match res {
            Execution::Finished(Ok(output)) => {
                *self = MaybeDone::Done(output);

                Ok(true)
            },
            Execution::Finished(Err(err)) => Err(err),
            Execution::ContinueFenced => Ok(false)
        }
    }

    fn take(&mut self) -> T::Output {
        match mem::replace(self, MaybeDone::Gone) {
            MaybeDone::Done(a) => a,
            _ => panic!(),
        }
    }
}