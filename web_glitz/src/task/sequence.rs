#![allow(non_snake_case)]

use super::maybe_done::{maybe_done, MaybeDone};
use super::{ContextId, GpuTask, Progress};

macro_rules! generate_sequence {
    ($(
        $(#[$doc:meta])*
        ($Sequence:ident, <A, $($B:ident),*>),
    )*) => ($(
        $(#[$doc])*
        pub struct $Sequence<A, $($B),*, Ec> where A: GpuTask<Ec>, $($B: GpuTask<Ec>),*
        {
            id: ContextId,
            a: MaybeDone<A, A::Output, Ec>,
            $($B: MaybeDone<$B, $B::Output, Ec>),*
        }

        impl<A, $($B),*, Ec> $Sequence<A, $($B),*, Ec> where A: GpuTask<Ec>, $($B: GpuTask<Ec>),* {
            pub(crate) fn new(a: A, $($B: $B),*) -> Self {
                let mut id = a.context_id();

                $(
                    id = id.combine($B.context_id()).unwrap();
                )*

                $Sequence {
                    id,
                    a: maybe_done(a),
                    $($B: maybe_done($B)),*
                }
            }
        }

        unsafe impl<A, $($B),*, Ec> GpuTask<Ec> for $Sequence<A, $($B),*, Ec>
        where
            A: GpuTask<Ec>,
            $($B: GpuTask<Ec>),*
        {
            type Output = (A::Output, $($B::Output),*);

            fn context_id(&self) -> ContextId {
                self.id
            }

            fn progress(&mut self, execution_context: &mut Ec) -> Progress<Self::Output> {
                if !self.a.progress(execution_context) {
                    return Progress::ContinueFenced;
                }

                $(
                    if !self.$B.progress(execution_context) {
                        return Progress::ContinueFenced;
                    }
                )*

                Progress::Finished((self.a.take(), $(self.$B.take()),*))
            }
        }

        impl<A, $($B),*, Ec> Clone for $Sequence<A, $($B),*, Ec>
        where
            A: GpuTask<Ec> + Clone,
            A::Output: Clone,
            $(
                $B: GpuTask<Ec> + Clone,
                $B::Output: Clone,
            )*
        {
            fn clone(&self) -> Self {
                $Sequence {
                    id: self.id.clone(),
                    a: self.a.clone(),
                    $($B: self.$B.clone()),*
                }
            }
        }
    )*)
}

generate_sequence! {
    /// Task for the `sequence` combinator, waiting for two tasks to complete in order.
    ///
    /// See [sequence] and [GpuTaskExt::sequence].
    (Sequence, <A, B>),

    /// Task for the `sequence3` combinator, waiting for three tasks to complete in order.
    ///
    /// See [sequence3] and [GpuTaskExt::sequence3].
    (Sequence3, <A, B, C>),

    /// Task for the `sequence4` combinator, waiting for four tasks to complete in order.
    ///
    /// See [sequence4] and [GpuTaskExt::sequence4].
    (Sequence4, <A, B, C, D>),

    /// Task for the `sequence5` combinator, waiting for five tasks to complete in order.
    ///
    /// See [sequence5] and [GpuTaskExt::sequence5].
    (Sequence5, <A, B, C, D, E>),
}

macro_rules! generate_sequence_left {
    ($(
        $(#[$doc:meta])*
        ($Sequence:ident, <A, $($B:ident),*>),
    )*) => ($(
        $(#[$doc])*
        pub struct $Sequence<A, $($B),*, Ec> where A: GpuTask<Ec>, $($B: GpuTask<Ec>),*
        {
            id: ContextId,
            a: MaybeDone<A, A::Output, Ec>,
            $($B: MaybeDone<$B, $B::Output, Ec>),*
        }

        impl<A, $($B),*, Ec> $Sequence<A, $($B),*, Ec> where A: GpuTask<Ec>, $($B: GpuTask<Ec>),* {
            pub(crate) fn new(a: A, $($B: $B),*) -> Self {
                let mut id = a.context_id();

                $(
                    id = id.combine($B.context_id()).unwrap();
                )*

                $Sequence {
                    id,
                    a: maybe_done(a),
                    $($B: maybe_done($B)),*
                }
            }
        }

        unsafe impl<A, $($B),*, Ec> GpuTask<Ec> for $Sequence<A, $($B),*, Ec>
        where
            A: GpuTask<Ec>,
            $($B: GpuTask<Ec>),*
        {
            type Output = A::Output;

            fn context_id(&self) -> ContextId {
                self.id
            }

            fn progress(&mut self, execution_context: &mut Ec) -> Progress<Self::Output> {
                if !self.a.progress(execution_context) {
                    return Progress::ContinueFenced;
                }

                $(
                    if !self.$B.progress(execution_context) {
                        return Progress::ContinueFenced;
                    }
                )*

                Progress::Finished(self.a.take())
            }
        }

        impl<A, $($B),*, Ec> Clone for $Sequence<A, $($B),*, Ec>
        where
            A: GpuTask<Ec> + Clone,
            A::Output: Clone,
            $(
                $B: GpuTask<Ec> + Clone,
                $B::Output: Clone,
            )*
        {
            fn clone(&self) -> Self {
                $Sequence {
                    id: self.id.clone(),
                    a: self.a.clone(),
                    $($B: self.$B.clone()),*
                }
            }
        }
    )*)
}

generate_sequence_left! {
    /// Task for the `sequence_left` combinator, waiting for two tasks to complete in order and only
    /// outputting the output of the left-most task.
    ///
    /// See [sequence_left] and [GpuTaskExt::sequence_left].
    (SequenceLeft, <A, B>),

    /// Task for the `sequence3_left` combinator, waiting for three tasks to complete order and only
    /// outputting the output of the left-most task.
    ///
    /// See [sequence3_left] and [GpuTaskExt::sequence3_left].
    (Sequence3Left, <A, B, C>),

    /// Task for the `sequence4_left` combinator, waiting for four tasks to complete in order and
    /// only outputting the output of the left-most task.
    ///
    /// See [sequence4_left] and [GpuTaskExt::sequence4_left].
    (Sequence4Left, <A, B, C, D>),

    /// Task for the `sequence5_left` combinator, waiting for five tasks to complete in order and
    /// only outputting the output of the left-most task.
    ///
    /// See [sequence5_left] and [GpuTaskExt::sequence5_left].
    (Sequence5Left, <A, B, C, D, E>),
}

macro_rules! generate_sequence_right {
    ($(
        $(#[$doc:meta])*
        ($Sequence:ident, <$($A:ident),*> $B:ident),
    )*) => ($(
        $(#[$doc])*
        pub struct $Sequence<$($A,)* $B, Ec> where $($A: GpuTask<Ec>,)* $B: GpuTask<Ec>
        {
            id: ContextId,
            $($A: MaybeDone<$A, $A::Output, Ec>,)*
            $B: MaybeDone<$B, $B::Output, Ec>,
        }

        impl<$($A,)* $B, Ec> $Sequence<$($A,)* $B, Ec> where $($A: GpuTask<Ec>,)* $B: GpuTask<Ec> {
            pub(crate) fn new($($A: $A,)* $B: $B) -> Self {
                let mut id = ContextId::Any;

                $(
                    id = id.combine($A.context_id()).unwrap();
                )*

                id = id.combine($B.context_id()).unwrap();

                $Sequence {
                    id,
                    $($A: maybe_done($A),)*
                    $B: maybe_done($B)
                }
            }
        }

        unsafe impl<$($A,)* $B, Ec> GpuTask<Ec> for $Sequence<$($A,)* $B, Ec>
        where
            $($A: GpuTask<Ec>,)*
            $B: GpuTask<Ec>
        {
            type Output = $B::Output;

            fn context_id(&self) -> ContextId {
                self.id
            }

            fn progress(&mut self, execution_context: &mut Ec) -> Progress<Self::Output> {
                $(
                    if !self.$A.progress(execution_context) {
                        return Progress::ContinueFenced;
                    }
                )*

                if !self.$B.progress(execution_context) {
                    return Progress::ContinueFenced;
                }

                Progress::Finished(self.$B.take())
            }
        }

        impl<$($A,)* $B, Ec> Clone for $Sequence<$($A,)* $B, Ec>
        where
            $(
                $A: GpuTask<Ec> + Clone,
                $A::Output: Clone,
            )*
            $B: GpuTask<Ec> + Clone,
            $B::Output: Clone,
        {
            fn clone(&self) -> Self {
                $Sequence {
                    id: self.id.clone(),
                    $($A: self.$A.clone(),)*
                    $B: self.$B.clone(),
                }
            }
        }
    )*)
}

generate_sequence_right! {
    /// Task for the `sequence_right` combinator, waiting for two tasks to complete in order and
    /// only outputting the output of the right-most task.
    ///
    /// See [sequence_right] and [GpuTaskExt::sequence_right].
    (SequenceRight, <A> B),

    /// Task for the `sequence3_right` combinator, waiting for three tasks to complete in order and
    /// only outputting the output of the right-most task.
    ///
    /// See [sequence3_right] and [GpuTaskExt::sequence3_right].
    (Sequence3Right, <A, B> C),

    /// Task for the `sequence4_right` combinator, waiting for four tasks to complete in order and
    /// only outputting the output of the right-most task.
    ///
    /// See [sequence4_right] and [GpuTaskExt::sequence4_right].
    (Sequence4Right, <A, B, C> D),

    /// Task for the `sequence5_right` combinator, waiting for five tasks to complete in order and
    /// only outputting the output of the right-most task.
    ///
    /// See [sequence5_right] and [GpuTaskExt::sequence5_right].
    (Sequence5Right, <A, B, C, D> E),
}

#[doc(hidden)]
#[macro_export]
macro_rules! sequence_all {
    ($e0:expr, $e1:expr) => (sequence_all!($e0, $e1,));
    ($e0:expr, $e1:expr, $($e:expr,)+) => (sequence_all!($e0, $e1, $($e),*));
    ($e0:expr, $e1:expr, $($e:expr),*) => {
        {
            let sequenced = $crate::task::sequence($e0, $e1);

            $(
                let sequenced = $crate::task::sequence(sequenced, $e);
            )*

            sequenced
        }
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! sequence_all_left {
    ($e0:expr, $e1:expr) => (sequence_all_left!($e0, $e1,));
    ($e0:expr, $e1:expr, $($e:expr,)+) => (sequence_all_left!($e0, $e1, $($e),*));
    ($e0:expr, $e1:expr, $($e:expr),*) => {
        {
            let sequenced = $crate::task::sequence_left($e0, $e1);

            $(
                let sequenced = $crate::task::sequence_left(sequenced, $e);
            )*

            sequenced
        }
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! sequence_all_right {
    ($e0:expr, $e1:expr) => (sequence_all_right!($e0, $e1,));
    ($e0:expr, $e1:expr, $($e:expr,)+) => (sequence_all_right!($e0, $e1, $($e),*));
    ($e0:expr, $e1:expr, $($e:expr),*) => {
        {
            let sequenced = $crate::task::sequence_right($e0, $e1);

            $(
                let sequenced = $crate::task::sequence_right(sequenced, $e);
            )*

            sequenced
        }
    }
}

/// Task for the [sequence_iter] combinator, waiting for all tasks in the iterator to complete in no
/// particular order, outputting `()`.
///
/// See [sequence_iter].
pub struct SequenceIter<T, Ec> {
    id: ContextId,
    vec: Vec<MaybeDone<T, (), Ec>>,
}

impl<T, Ec> SequenceIter<T, Ec>
where
    T: GpuTask<Ec, Output = ()>,
{
    fn new<I>(tasks: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        let mut id = ContextId::Any;

        let vec: Vec<MaybeDone<T, (), Ec>> = tasks
            .into_iter()
            .map(|t| {
                id = id.combine(t.context_id()).unwrap();

                maybe_done(t)
            })
            .collect();

        SequenceIter { id, vec }
    }
}

unsafe impl<T, Ec> GpuTask<Ec> for SequenceIter<T, Ec>
where
    T: GpuTask<Ec, Output = ()>,
{
    type Output = ();

    fn context_id(&self) -> ContextId {
        self.id
    }

    fn progress(&mut self, execution_context: &mut Ec) -> Progress<Self::Output> {
        for task in &mut self.vec {
            if !task.progress(execution_context) {
                return Progress::ContinueFenced;
            }
        }

        Progress::Finished(())
    }
}

impl<T, Ec> Clone for SequenceIter<T, Ec>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        SequenceIter {
            id: self.id.clone(),
            vec: self.vec.clone(),
        }
    }
}

/// Combines task `a` with another task `b`, waiting for both tasks to complete in order.
///
/// This returns a new "sequenced" task. This sequenced task must progress its sub-tasks in order.
/// The sequenced task will finish when both sub-tasks have finished. When it finishes, it will
/// output a tuple `(A, B)` where `A` is this tasks output and `B` is task `b`'s output.
///
/// # Panics
///
/// Panics if the [ContextId]s of `a` and `b` are not compatible.
pub fn sequence<A, B, Ec>(a: A, b: B) -> Sequence<A, B, Ec>
where
    A: GpuTask<Ec>,
    B: GpuTask<Ec>,
{
    Sequence::new(a, b)
}

/// Combines task `a` with another task `b`, waiting for both tasks to complete in order, with the
/// output of task `a`.
///
/// Similar to [sequence], except that instead of returning a tuple of the outputs of `a` and `b`,
/// it only returns the output of `a`.
///
/// See also [sequence_right].
///
/// # Panics
///
/// Panics if the [ContextId]s of `a` and `b` are not compatible.
pub fn sequence_left<A, B, Ec>(a: A, b: B) -> SequenceLeft<A, B, Ec>
where
    A: GpuTask<Ec>,
    B: GpuTask<Ec>,
{
    SequenceLeft::new(a, b)
}

/// Combines task `a` with another task `b`, waiting for both tasks to complete in order, with the
/// output of task `a`.
///
/// Similar to [sequence], except that instead of returning a tuple of the outputs of `a` and `b`,
/// it only returns the output of `b`.
///
/// See also [sequence_left].
///
/// # Panics
///
/// Panics if the [ContextId]s of `a` and `b` are not compatible.
pub fn sequence_right<A, B, Ec>(a: A, b: B) -> SequenceRight<A, B, Ec>
where
    A: GpuTask<Ec>,
    B: GpuTask<Ec>,
{
    SequenceRight::new(a, b)
}

/// Combines task `a`, `b` and `c`, waiting for all tasks to complete in order.
///
/// This returns a new "sequenced" task. This sequenced task must progress its sub-tasks in order.
/// The sequenced task will finish when all sub-tasks have finished. When it finishes, it will
/// output a tuple `(A, B, C)` where `A` is this task's output, `B` is task `b`'s output and `C` is
/// task `c`'s output.
///
/// # Panics
///
/// Panics if the [ContextId]s `a`, `b` and `c` are not compatible.
pub fn sequence3<A, B, C, Ec>(a: A, b: B, c: C) -> Sequence3<A, B, C, Ec>
where
    A: GpuTask<Ec>,
    B: GpuTask<Ec>,
    C: GpuTask<Ec>,
{
    Sequence3::new(a, b, c)
}

/// Combines task `a`, `b` and `c`, waiting for all tasks to complete in order, with the output of
/// task `a`.
///
/// Similar to [sequence3], except that instead of returning a tuple of the outputs of `a`, `b` and
/// `c`, it only returns the output of `a`.
///
/// See also [sequence3_right].
///
/// # Panics
///
/// Panics if the [ContextId]s `a`, `b` and `c` are not compatible.
pub fn sequence3_left<A, B, C, Ec>(a: A, b: B, c: C) -> Sequence3Left<A, B, C, Ec>
where
    A: GpuTask<Ec>,
    B: GpuTask<Ec>,
    C: GpuTask<Ec>,
{
    Sequence3Left::new(a, b, c)
}

/// Combines task `a`, `b` and `c`, waiting for all tasks to complete in order, with
/// the output of task `c`.
///
/// Similar to [sequence3], except that instead of returning a tuple of the outputs of `a`, `b` and
/// `c`, it only returns the output of `c`.
///
/// See also [sequence3_left].
///
/// # Panics
///
/// Panics if the [ContextId]s `a`, `b` and `c` are not compatible.
pub fn sequence3_right<A, B, C, Ec>(a: A, b: B, c: C) -> Sequence3Right<A, B, C, Ec>
where
    A: GpuTask<Ec>,
    B: GpuTask<Ec>,
    C: GpuTask<Ec>,
{
    Sequence3Right::new(a, b, c)
}

/// Combines task `a`, `b`, `c` and `d`, waiting for all tasks to complete in order.
///
/// This returns a new "sequenced" task. This sequenced task must progress its sub-tasks in order.
/// The sequenced task will finish when all sub-tasks have finished. When it finishes, it will
/// output a tuple `(A, B, C, D)` where `A` is this tasks output, `B` is task `b`'s output, `C` is
/// task `c`'s output and `D` is task `d`'s output.
///
/// # Panics
///
/// Panics if the [ContextId]s `a`, `b`, `c` and `d` are not compatible.
pub fn sequence4<A, B, C, D, Ec>(a: A, b: B, c: C, d: D) -> Sequence4<A, B, C, D, Ec>
where
    A: GpuTask<Ec>,
    B: GpuTask<Ec>,
    C: GpuTask<Ec>,
    D: GpuTask<Ec>,
{
    Sequence4::new(a, b, c, d)
}

/// Combines task `a`, `b`, `c` and `d`, waiting for all tasks to complete in order,
/// with the output of task `a`.
///
/// Similar to [sequence4], except that instead of returning a tuple of the outputs of `a`, `b`, `c`
/// and `d`, it only returns the output of `a`.
///
/// See also [sequence4_right].
///
/// # Panics
///
/// Panics if the [ContextId]s `a`, `b`, `c` and `d` are not compatible.
pub fn sequence4_left<A, B, C, D, Ec>(a: A, b: B, c: C, d: D) -> Sequence4Left<A, B, C, D, Ec>
where
    A: GpuTask<Ec>,
    B: GpuTask<Ec>,
    C: GpuTask<Ec>,
    D: GpuTask<Ec>,
{
    Sequence4Left::new(a, b, c, d)
}

/// Combines task `a`, `b`, `c` and `d`, waiting for all tasks to complete in order, with the output
/// of task `d`.
///
/// Similar to [sequence4], except that instead of returning a tuple of the outputs of `a`, `b`, `c`
/// and `d`, it only returns the output of `d`.
///
/// See also [sequence4_left].
///
/// # Panics
///
/// Panics if the [ContextId]s `a`, `b`, `c` and `d` are not compatible.
pub fn sequence4_right<A, B, C, D, Ec>(a: A, b: B, c: C, d: D) -> Sequence4Right<A, B, C, D, Ec>
where
    A: GpuTask<Ec>,
    B: GpuTask<Ec>,
    C: GpuTask<Ec>,
    D: GpuTask<Ec>,
{
    Sequence4Right::new(a, b, c, d)
}

/// Combines task `a`, `b`, `c`, `d` and `e`, waiting for all tasks to complete in order.
///
/// This returns a new "sequenced" task. This sequenced task must progress its sub-tasks in order.
/// The sequenced task will finish when all sub-tasks have finished. When it finishes, it will
/// output a tuple `(A, B, C, D, E)` where `A` is this tasks output, `B` is task `b`'s output, `C`
/// is task `c`'s output, `D` is task `d`'s output and `E` is task `e`'s output.
///
/// # Panics
///
/// Panics if the [ContextId]s `a`, `b`, `c`, `d` and `e` are not compatible.
pub fn sequence5<A, B, C, D, E, Ec>(a: A, b: B, c: C, d: D, e: E) -> Sequence5<A, B, C, D, E, Ec>
where
    A: GpuTask<Ec>,
    B: GpuTask<Ec>,
    C: GpuTask<Ec>,
    D: GpuTask<Ec>,
    E: GpuTask<Ec>,
{
    Sequence5::new(a, b, c, d, e)
}

/// Combines task `a`, `b`, `c`, `d` and `e`, waiting for all tasks to complete in order, with the
/// output of task `a`.
///
/// Similar to [sequence5], except that instead of returning a tuple of the outputs of `a`, `b`,
/// `c`, `d` and `e`, it only returns the output of `a`.
///
/// See also [sequence5_right].
///
/// # Panics
///
/// Panics if the [ContextId]s `a`, `b`, `c`, `d` and `e` are not compatible.
pub fn sequence5_left<A, B, C, D, E, Ec>(
    a: A,
    b: B,
    c: C,
    d: D,
    e: E,
) -> Sequence5Left<A, B, C, D, E, Ec>
where
    A: GpuTask<Ec>,
    B: GpuTask<Ec>,
    C: GpuTask<Ec>,
    D: GpuTask<Ec>,
    E: GpuTask<Ec>,
{
    Sequence5Left::new(a, b, c, d, e)
}

/// Combines task `a`, `b`, `c`, `d` and `e`, waiting for all tasks to complete in order, with the
/// output of task `e`.
///
/// Similar to [sequence5], except that instead of returning a tuple of the outputs of `a`, `b`,
/// `c`, `d` and `e`, it only returns the output of `e`.
///
/// See also [sequence5_left].
///
/// # Panics
///
/// Panics if the [ContextId]s `a`, `b`, `c`, `d` and `e` are not compatible.
pub fn sequence5_right<A, B, C, D, E, Ec>(
    a: A,
    b: B,
    c: C,
    d: D,
    e: E,
) -> Sequence5Right<A, B, C, D, E, Ec>
where
    A: GpuTask<Ec>,
    B: GpuTask<Ec>,
    C: GpuTask<Ec>,
    D: GpuTask<Ec>,
    E: GpuTask<Ec>,
{
    Sequence5Right::new(a, b, c, d, e)
}

/// Combines all tasks in an iterator, waiting for all tasks to complete in order.
///
/// This returns a new "sequenced" task. This sequenced task must progress its sub-tasks in order.
/// The sequenced task will finish when all sub-tasks have finished. When it finishes, it will
/// output `()`. All tasks in the iterator must also output `()`.
///
/// This combinator allocates. See also the [sequence_all] macro for an alternative that does not
/// allocate if the set of tasks that are to be joined is statically known.
///
/// # Panics
///
/// Panics if the [ContextId]s of any of the tasks in the iterator are not compatible.
pub fn sequence_iter<I, Ec>(iterator: I) -> SequenceIter<I::Item, Ec>
where
    I: IntoIterator,
    I::Item: GpuTask<Ec, Output = ()>,
{
    SequenceIter::new(iterator)
}
