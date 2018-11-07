#![allow(non_snake_case)]

use super::maybe_done::{maybe_done, MaybeDone};
use super::{GpuTask, Progress};

macro_rules! generate {
    ($(
        $(#[$doc:meta])*
        ($Join:ident, <A, $($B:ident),*>),
    )*) => ($(
        $(#[$doc])*
        pub struct $Join<A, $($B),*, Ec> where A: GpuTask<Ec>, $($B: GpuTask<Ec, Error=A::Error>),*
        {
            a: MaybeDone<A, A::Output, Ec>,
            $($B: MaybeDone<$B, $B::Output, Ec>),*
        }

        impl<A, $($B),*, Ec> $Join<A, $($B),*, Ec> where A: GpuTask<Ec>, $($B: GpuTask<Ec, Error=A::Error>),* {
            pub fn new(a: A, $($B: $B),*) -> Self {
                $Join {
                    a: maybe_done(a),
                    $($B: maybe_done($B)),*
                }
            }
        }

        impl<A, $($B),*, Ec> GpuTask<Ec> for $Join<A, $($B),*, Ec> where A: GpuTask<Ec>, $($B: GpuTask<Ec, Error=A::Error>),* {
            type Output = (A::Output, $($B::Output),*);

            type Error = A::Error;

            fn progress(&mut self, execution_context: &mut Ec) -> Progress<(A::Output, $($B::Output),*), A::Error> {
                let mut all_done = match self.a.progress(execution_context) {
                    Ok(done) => done,
                    Err(err) => return Progress::Finished(Err(err))
                };

                $(
                    all_done = match self.$B.progress(execution_context) {
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
    /// Task for the `join` combinator, waiting for two tasks to complete in no particular order.
    (Join, <A, B>),

    /// Task for the `join3` combinator, waiting for three tasks to complete in no particular order.
    (Join3, <A, B, C>),

    /// Task for the `join4` combinator, waiting for four tasks to complete in no particular order.
    (Join4, <A, B, C, D>),

    /// Task for the `join5` combinator, waiting for five tasks to complete in no particular order.
    (Join5, <A, B, C, D, E>),
}
