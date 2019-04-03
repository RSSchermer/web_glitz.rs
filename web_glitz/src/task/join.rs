#![allow(non_snake_case)]

use super::maybe_done::{maybe_done, MaybeDone};
use super::{ContextId, GpuTask, Progress};

macro_rules! generate {
    ($(
        $(#[$doc:meta])*
        ($Join:ident, <A, $($B:ident),*>),
    )*) => ($(
        $(#[$doc])*
        pub struct $Join<A, $($B),*, Ec> where A: GpuTask<Ec>, $($B: GpuTask<Ec>),*
        {
            id: ContextId,
            a: MaybeDone<A, A::Output, Ec>,
            $($B: MaybeDone<$B, $B::Output, Ec>),*
        }

        impl<A, $($B),*, Ec> $Join<A, $($B),*, Ec> where A: GpuTask<Ec>, $($B: GpuTask<Ec>),* {
            pub fn new(a: A, $($B: $B),*) -> Self {
                let mut id = a.context_id();

                $(
                    id = id.combine($B.context_id()).unwrap();
                )*

                $Join {
                    id,
                    a: maybe_done(a),
                    $($B: maybe_done($B)),*
                }
            }
        }

        unsafe impl<A, $($B),*, Ec> GpuTask<Ec> for $Join<A, $($B),*, Ec> where A: GpuTask<Ec>, $($B: GpuTask<Ec>),* {
            type Output = (A::Output, $($B::Output),*);

            fn context_id(&self) -> ContextId {
                self.id
            }

            fn progress(&mut self, execution_context: &mut Ec) -> Progress<Self::Output> {
                let mut all_done = self.a.progress(execution_context);

                $(
                    all_done = all_done && self.$B.progress(execution_context);;
                )*

                if all_done {
                    Progress::Finished((self.a.take(), $(self.$B.take()),*))
                } else {
                    Progress::ContinueFenced
                }
            }
        }
    )*)
}

generate! {
    /// Task for the `join` combinator, waiting for two tasks to complete in no particular order.
    ///
    /// See [GpuTaskExt::join].
    (Join, <A, B>),

    /// Task for the `join3` combinator, waiting for three tasks to complete in no particular order.
    ///
    /// See [GpuTaskExt::join3].
    (Join3, <A, B, C>),

    /// Task for the `join4` combinator, waiting for four tasks to complete in no particular order.
    ///
    /// See [GpuTaskExt::join4].
    (Join4, <A, B, C, D>),

    /// Task for the `join5` combinator, waiting for five tasks to complete in no particular order.
    ///
    /// See [GpuTaskExt::join5].
    (Join5, <A, B, C, D, E>),
}
