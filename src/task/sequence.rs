#![allow(non_snake_case)]

use super::{GpuTask, Execution};
use super::maybe_done::{MaybeDone, maybe_done};

macro_rules! generate {
    ($(
        $(#[$doc:meta])*
        ($Sequence:ident, <A, $($B:ident),*>),
    )*) => ($(
        $(#[$doc])*
        pub struct $Sequence<A, $($B),*, Ec> where A: GpuTask<Ec>, $($B: GpuTask<Ec, Error=A::Error>),*
        {
            a: MaybeDone<A, Ec>,
            $($B: MaybeDone<$B, Ec>),*
        }

        impl<A, $($B),*, Ec> $Sequence<A, $($B),*, Ec> where A: GpuTask<Ec>, $($B: GpuTask<Ec, Error=A::Error>),* {
            pub fn new(a: A, $($B: $B),*) -> Self {
                $Sequence {
                    a: maybe_done(a),
                    $($B: maybe_done($B)),*
                }
            }
        }

        impl<A, $($B),*, Ec> GpuTask<Ec> for $Sequence<A, $($B),*, Ec> where A: GpuTask<Ec>, $($B: GpuTask<Ec, Error=A::Error>),* {
            type Output = (A::Output, $($B::Output),*);

            type Error = A::Error;

            fn progress(&mut self, execution_context: &mut Ec) -> Execution<(A::Output, $($B::Output),*), A::Error> {
                let mut all_done = match self.a.progress(execution_context) {
                    Ok(done) => done,
                    Err(err) => return Execution::Finished(Err(err))
                };

                while all_done {
                    $(
                        all_done = match self.$B.progress(execution_context) {
                            Ok(done) => all_done && done,
                            Err(err) => return Execution::Finished(Err(err))
                        };
                    )*
                }

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
    /// Task for the `sequence` combinator, waiting for two tasks to complete in no particular order.
    (Sequence, <A, B>),

    /// Task for the `sequence3` combinator, waiting for three tasks to complete in no particular order.
    (Sequence3, <A, B, C>),

    /// Task for the `sequence4` combinator, waiting for four tasks to complete in no particular order.
    (Sequence4, <A, B, C, D>),

    /// Task for the `sequence5` combinator, waiting for five tasks to complete in no particular order.
    (Sequence5, <A, B, C, D, E>),
}
