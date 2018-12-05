#![allow(non_snake_case)]

use super::try_maybe_done::{try_maybe_done, TryMaybeDone};
use super::{GpuTask, Progress, TryGpuTask};

macro_rules! generate {
    ($(
        $(#[$doc:meta])*
        ($TryJoin:ident, <A, $($B:ident),*>),
    )*) => ($(
        $(#[$doc])*
        pub struct $TryJoin<A, $($B),*, Ec> where A: TryGpuTask<Ec>, $($B: TryGpuTask<Ec, Error=A::Error>),*
        {
            a: TryMaybeDone<A, A::Ok, Ec>,
            $($B: TryMaybeDone<$B, $B::Ok, Ec>),*
        }

        impl<A, $($B),*, Ec> $TryJoin<A, $($B),*, Ec> where A: TryGpuTask<Ec>, $($B: TryGpuTask<Ec, Error=A::Error>),* {
            pub fn new(a: A, $($B: $B),*) -> Self {
                $TryJoin {
                    a: try_maybe_done(a),
                    $($B: try_maybe_done($B)),*
                }
            }
        }

        impl<A, $($B),*, Ec> GpuTask<Ec> for $TryJoin<A, $($B),*, Ec> where A: TryGpuTask<Ec>, $($B: TryGpuTask<Ec, Error=A::Error>),* {
            type Output = Result<(A::Ok, $($B::Ok),*), A::Error>;

            fn progress(&mut self, execution_context: &mut Ec) -> Progress<Self::Output> {
                let mut all_done = match self.a.try_progress(execution_context) {
                    Ok(done) => done,
                    Err(err) => return Progress::Finished(Err(err))
                };

                $(
                    all_done = match self.$B.try_progress(execution_context) {
                        Ok(done) => all_done && done,
                        Err(err) => return Progress::Finished(Err(err))
                    };
                )*

                if all_done {
                    Progress::Finished(Ok((self.a.take(), $(self.$B.take()),*)))
                } else {
                    Progress::ContinueFenced
                }
            }
        }
    )*)
}

generate! {
    /// Task for the `try_join` combinator, waiting for two tasks to complete in no particular order.
    (TryJoin, <A, B>),

    /// Task for the `try_join3` combinator, waiting for three tasks to complete in no particular order.
    (TryJoin3, <A, B, C>),

    /// Task for the `try_join4` combinator, waiting for four tasks to complete in no particular order.
    (TryJoin4, <A, B, C, D>),

    /// Task for the `try_join5` combinator, waiting for five tasks to complete in no particular order.
    (TryJoin5, <A, B, C, D, E>),
}
