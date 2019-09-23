#![allow(non_snake_case)]

use super::maybe_done::{maybe_done, MaybeDone};
use super::{ContextId, GpuTask, Progress};

macro_rules! generate_join {
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
            pub(crate) fn new(a: A, $($B: $B),*) -> Self {
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

        unsafe impl<A, $($B),*, Ec> GpuTask<Ec> for $Join<A, $($B),*, Ec>
        where
            A: GpuTask<Ec>,
            $($B: GpuTask<Ec>),*
        {
            type Output = (A::Output, $($B::Output),*);

            fn context_id(&self) -> ContextId {
                self.id
            }

            fn progress(&mut self, execution_context: &mut Ec) -> Progress<Self::Output> {
                let mut all_done = self.a.progress(execution_context);

                $(
                    all_done = all_done && self.$B.progress(execution_context);
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

generate_join! {
    /// Task for the `join` combinator, waiting for two tasks to complete in no particular order.
    ///
    /// See [join] and [GpuTaskExt::join].
    (Join, <A, B>),

    /// Task for the `join3` combinator, waiting for three tasks to complete in no particular order.
    ///
    /// See [join3] and [GpuTaskExt::join3].
    (Join3, <A, B, C>),

    /// Task for the `join4` combinator, waiting for four tasks to complete in no particular order.
    ///
    /// See [join4] and [GpuTaskExt::join4].
    (Join4, <A, B, C, D>),

    /// Task for the `join5` combinator, waiting for five tasks to complete in no particular order.
    ///
    /// See [join5] and [GpuTaskExt::join5].
    (Join5, <A, B, C, D, E>),
}

macro_rules! generate_join_left {
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
            pub(crate) fn new(a: A, $($B: $B),*) -> Self {
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

        unsafe impl<A, $($B),*, Ec> GpuTask<Ec> for $Join<A, $($B),*, Ec>
        where
            A: GpuTask<Ec>,
            $($B: GpuTask<Ec>),*
        {
            type Output = A::Output;

            fn context_id(&self) -> ContextId {
                self.id
            }

            fn progress(&mut self, execution_context: &mut Ec) -> Progress<Self::Output> {
                let mut all_done = self.a.progress(execution_context);

                $(
                    all_done = all_done && self.$B.progress(execution_context);
                )*

                if all_done {
                    Progress::Finished(self.a.take())
                } else {
                    Progress::ContinueFenced
                }
            }
        }
    )*)
}

generate_join_left! {
    /// Task for the `join_left` combinator, waiting for two tasks to complete in no particular
    /// order and only outputting the output of the left-most task.
    ///
    /// See [join_left] and [GpuTaskExt::join_left].
    (JoinLeft, <A, B>),

    /// Task for the `join3_left` combinator, waiting for three tasks to complete in no particular
    /// order and only outputting the output of the left-most task.
    ///
    /// See [join3_left] and [GpuTaskExt::join3_left].
    (Join3Left, <A, B, C>),

    /// Task for the `join4_left` combinator, waiting for four tasks to complete in no particular
    /// order and only outputting the output of the left-most task.
    ///
    /// See [join4_left] and [GpuTaskExt::join4_left].
    (Join4Left, <A, B, C, D>),

    /// Task for the `join5_left` combinator, waiting for five tasks to complete in no particular
    /// order and only outputting the output of the left-most task.
    ///
    /// See [join5_left] and [GpuTaskExt::join5_left].
    (Join5Left, <A, B, C, D, E>),
}

macro_rules! generate_join_right {
    ($(
        $(#[$doc:meta])*
        ($Join:ident, <$($A:ident),*> $B:ident),
    )*) => ($(
        $(#[$doc])*
        pub struct $Join<$($A,)* $B, Ec> where $($A: GpuTask<Ec>,)* $B: GpuTask<Ec>
        {
            id: ContextId,
            $($A: MaybeDone<$A, $A::Output, Ec>,)*
            $B: MaybeDone<$B, $B::Output, Ec>,
        }

        impl<$($A,)* $B, Ec> $Join<$($A,)* $B, Ec> where $($A: GpuTask<Ec>,)* $B: GpuTask<Ec> {
            pub(crate) fn new($($A: $A,)* $B: $B) -> Self {
                let mut id = ContextId::Any;

                $(
                    id = id.combine($A.context_id()).unwrap();
                )*

                id = id.combine($B.context_id()).unwrap();

                $Join {
                    id,
                    $($A: maybe_done($A),)*
                    $B: maybe_done($B)
                }
            }
        }

        unsafe impl<$($A,)* $B, Ec> GpuTask<Ec> for $Join<$($A,)* $B, Ec>
        where
            $($A: GpuTask<Ec>,)*
            $B: GpuTask<Ec>
        {
            type Output = $B::Output;

            fn context_id(&self) -> ContextId {
                self.id
            }

            fn progress(&mut self, execution_context: &mut Ec) -> Progress<Self::Output> {
                let mut all_done = true;

                $(
                    all_done = all_done && self.$A.progress(execution_context);
                )*

                all_done = all_done && self.$B.progress(execution_context);

                if all_done {
                    Progress::Finished(self.$B.take())
                } else {
                    Progress::ContinueFenced
                }
            }
        }
    )*)
}

generate_join_right! {
    /// Task for the `join_right` combinator, waiting for two tasks to complete in no particular
    /// order and only outputting the output of the right-most task.
    ///
    /// See [join_right] and [GpuTaskExt::join_right].
    (JoinRight, <A> B),

    /// Task for the `join3_right` combinator, waiting for three tasks to complete in no particular
    /// order and only outputting the output of the right-most task.
    ///
    /// See [join3_right] and [GpuTaskExt::join3_right].
    (Join3Right, <A, B> C),

    /// Task for the `join4_right` combinator, waiting for four tasks to complete in no particular
    /// order and only outputting the output of the right-most task.
    ///
    /// See [join4_right] and [GpuTaskExt::join4_right].
    (Join4Right, <A, B, C> D),

    /// Task for the `join5_right` combinator, waiting for five tasks to complete in no particular
    /// order and only outputting the output of the right-most task.
    ///
    /// See [join5_right] and [GpuTaskExt::join5_right].
    (Join5Right, <A, B, C, D> E),
}

#[doc(hidden)]
#[macro_export]
macro_rules! join_all {
    ($e0:expr, $e1:expr) => (join_all!($e0, $e1,));
    ($e0:expr, $e1:expr, $($e:expr,)+) => (join_all!($e0, $e1, $($e),*));
    ($e0:expr, $e1:expr, $($e:expr),*) => {
        {
            let joined = $crate::task::join($e0, $e1);

            $(
                let joined = $crate::task::join(joined, $e);
            )*

            joined
        }
    }
}

/// Combines task `a` with another task `b`, waiting for both tasks to complete in no particular
/// order.
///
/// This returns a new "joined" task. This joined task may progress the its sub-tasks in any order.
/// The joined task will finish when both sub-tasks have finished. When it finishes, it will output
/// a tuple `(A, B)` where `A` is this task's output and `B` is task `b`'s output.
///
/// # Panics
///
/// Panics if the [ContextId]s of `a` and `b` are not compatible.
pub fn join<A, B, Ec>(a: A, b: B) -> Join<A, B, Ec>
where
    A: GpuTask<Ec>,
    B: GpuTask<Ec>,
{
    Join::new(a, b)
}

/// Combines task `a` with another task `b`, waiting for both tasks to complete in no particular
/// order, with the output of task `a`.
///
/// Similar to [join], except that instead of returning a tuple of the outputs of `a` and `b`, it
/// only returns the output of `a`.
///
/// See also [join_right].
///
/// # Panics
///
/// Panics if the [ContextId]s of `a` and `b` are not compatible.
pub fn join_left<A, B, Ec>(a: A, b: B) -> JoinLeft<A, B, Ec>
where
    A: GpuTask<Ec>,
    B: GpuTask<Ec>,
{
    JoinLeft::new(a, b)
}

/// Combines task `a` with another task `b`, waiting for both tasks to complete in no particular
/// order, with the output of task `a`.
///
/// Similar to [join], except that instead of returning a tuple of the outputs of `a` and `b`, it
/// only returns the output of `b`.
///
/// See also [join_left].
///
/// # Panics
///
/// Panics if the [ContextId]s of `a` and `b` are not compatible.
pub fn join_right<A, B, Ec>(a: A, b: B) -> JoinRight<A, B, Ec>
where
    A: GpuTask<Ec>,
    B: GpuTask<Ec>,
{
    JoinRight::new(a, b)
}

/// Combines task `a`, `b` and `c`, waiting for all tasks to complete in no particular order.
///
/// This returns a new "joined" task. This joined task may progress the its sub-tasks in any order.
/// The joined task will finish when all sub-tasks have finished. When it finishes, it will output a
/// tuple `(A, B, C)` where `A` is this task's output, `B` is task `b`'s output and `C` is task
/// `c`'s output.
///
/// # Panics
///
/// Panics if the [ContextId]s `a`, `b` and `c` are not compatible.
pub fn join3<A, B, C, Ec>(a: A, b: B, c: C) -> Join3<A, B, C, Ec>
where
    A: GpuTask<Ec>,
    B: GpuTask<Ec>,
    C: GpuTask<Ec>,
{
    Join3::new(a, b, c)
}

/// Combines task `a`, `b` and `c`, waiting for all tasks to complete in no particular order, with
/// the output of task `a`.
///
/// Similar to [join3], except that instead of returning a tuple of the outputs of `a`, `b` and `c`,
/// it only returns the output of `a`.
///
/// See also [join3_right].
///
/// # Panics
///
/// Panics if the [ContextId]s `a`, `b` and `c` are not compatible.
pub fn join3_left<A, B, C, Ec>(a: A, b: B, c: C) -> Join3Left<A, B, C, Ec>
where
    A: GpuTask<Ec>,
    B: GpuTask<Ec>,
    C: GpuTask<Ec>,
{
    Join3Left::new(a, b, c)
}

/// Combines task `a`, `b` and `c`, waiting for all tasks to complete in no particular order, with
/// the output of task `c`.
///
/// Similar to [join3], except that instead of returning a tuple of the outputs of `a`, `b` and `c`,
/// it only returns the output of `c`.
///
/// See also [join3_left].
///
/// # Panics
///
/// Panics if the [ContextId]s `a`, `b` and `c` are not compatible.
pub fn join3_right<A, B, C, Ec>(a: A, b: B, c: C) -> Join3Right<A, B, C, Ec>
where
    A: GpuTask<Ec>,
    B: GpuTask<Ec>,
    C: GpuTask<Ec>,
{
    Join3Right::new(a, b, c)
}

/// Combines task `a`, `b`, `c` and `d`, waiting for all tasks to complete in no particular order.
///
/// This returns a new "joined" task. This joined task may progress the its sub-tasks in any order.
/// The joined task will finish when all sub-tasks have finished. When it finishes, it will output a
/// tuple `(A, B, C, D)` where `A` is this task's output, `B` is task `b`'s output, `C` is task
/// `c`'s output and `D` is task `d`'s output.
///
/// # Panics
///
/// Panics if the [ContextId]s `a`, `b`, `c` and `d` are not compatible.
pub fn join4<A, B, C, D, Ec>(a: A, b: B, c: C, d: D) -> Join4<A, B, C, D, Ec>
where
    A: GpuTask<Ec>,
    B: GpuTask<Ec>,
    C: GpuTask<Ec>,
    D: GpuTask<Ec>,
{
    Join4::new(a, b, c, d)
}

/// Combines task `a`, `b`, `c` and `d`, waiting for all tasks to complete in no particular order,
/// with the output of task `a`.
///
/// Similar to [join4], except that instead of returning a tuple of the outputs of `a`, `b`, `c` and
/// `d`, it only returns the output of `a`.
///
/// See also [join4_right].
///
/// # Panics
///
/// Panics if the [ContextId]s `a`, `b`, `c` and `d` are not compatible.
pub fn join4_left<A, B, C, D, Ec>(a: A, b: B, c: C, d: D) -> Join4Left<A, B, C, D, Ec>
where
    A: GpuTask<Ec>,
    B: GpuTask<Ec>,
    C: GpuTask<Ec>,
    D: GpuTask<Ec>,
{
    Join4Left::new(a, b, c, d)
}

/// Combines task `a`, `b`, `c` and `d`, waiting for all tasks to complete in no particular order,
/// with the output of task `d`.
///
/// Similar to [join4], except that instead of returning a tuple of the outputs of `a`, `b`, `c` and
/// `d`, it only returns the output of `d`.
///
/// See also [join4_left].
///
/// # Panics
///
/// Panics if the [ContextId]s `a`, `b`, `c` and `d` are not compatible.
pub fn join4_right<A, B, C, D, Ec>(a: A, b: B, c: C, d: D) -> Join4Right<A, B, C, D, Ec>
where
    A: GpuTask<Ec>,
    B: GpuTask<Ec>,
    C: GpuTask<Ec>,
    D: GpuTask<Ec>,
{
    Join4Right::new(a, b, c, d)
}

/// Combines task `a`, `b`, `c`, `d` and `e`, waiting for all tasks to complete in no particular
/// order.
///
/// This returns a new "joined" task. This joined task may progress the its sub-tasks in any order.
/// The joined task will finish when all sub-tasks have finished. When it finishes, it will output a
/// tuple `(A, B, C, D, E)` where `A` is this task's output, `B` is task `b`'s output, `C` is task
/// `c`'s output, `D` is task `d`'s output and `E` is task `e`'s output.
///
/// # Panics
///
/// Panics if the [ContextId]s `a`, `b`, `c`, `d` and `e` are not compatible.
pub fn join5<A, B, C, D, E, Ec>(a: A, b: B, c: C, d: D, e: E) -> Join5<A, B, C, D, E, Ec>
where
    A: GpuTask<Ec>,
    B: GpuTask<Ec>,
    C: GpuTask<Ec>,
    D: GpuTask<Ec>,
    E: GpuTask<Ec>,
{
    Join5::new(a, b, c, d, e)
}

/// Combines task `a`, `b`, `c`, `d` and `e`, waiting for all tasks to complete in no particular
/// order, with the output of task `a`.
///
/// Similar to [join5], except that instead of returning a tuple of the outputs of `a`, `b`, `c`,
/// `d` and `e`, it only returns the output of `a`.
///
/// See also [join5_right].
///
/// # Panics
///
/// Panics if the [ContextId]s `a`, `b`, `c`, `d` and `e` are not compatible.
pub fn join5_left<A, B, C, D, E, Ec>(a: A, b: B, c: C, d: D, e: E) -> Join5Left<A, B, C, D, E, Ec>
where
    A: GpuTask<Ec>,
    B: GpuTask<Ec>,
    C: GpuTask<Ec>,
    D: GpuTask<Ec>,
    E: GpuTask<Ec>,
{
    Join5Left::new(a, b, c, d, e)
}

/// Combines task `a`, `b`, `c`, `d` and `e`, waiting for all tasks to complete in no particular
/// order, with the output of task `e`.
///
/// Similar to [join5], except that instead of returning a tuple of the outputs of `a`, `b`, `c`,
/// `d` and `e`, it only returns the output of `e`.
///
/// See also [join5_left].
///
/// # Panics
///
/// Panics if the [ContextId]s `a`, `b`, `c`, `d` and `e` are not compatible.
pub fn join5_right<A, B, C, D, E, Ec>(a: A, b: B, c: C, d: D, e: E) -> Join5Right<A, B, C, D, E, Ec>
where
    A: GpuTask<Ec>,
    B: GpuTask<Ec>,
    C: GpuTask<Ec>,
    D: GpuTask<Ec>,
    E: GpuTask<Ec>,
{
    Join5Right::new(a, b, c, d, e)
}
