use super::{Join, Join3, Join4, Join5, Sequence, Sequence3, Sequence4, Sequence5};

/// Trait for types that represent a computational task is to be partly or completely executed on a
/// GPU.
///
/// A [GpuTask] implementation is associated with a specific execution context type `Ec` (e.g. the
/// base [Connection], a [RenderPassContext], or a [PipelineTaskContext]). A task executor
/// associated with that context may attempt to make progress on the task by calling
/// [GpuTask::progress] and providing an exclusive reference to an instance of that context, but
/// only if this context instance is compatible with [GpuTask::context_id].
///
/// # Unsafe
///
/// If a context instance is compatible with [GpuTask::context_id], then invoking
/// [GpuTask::progress] with this instance must not result in undefined behaviour.
pub unsafe trait GpuTask<Ec> {
    /// The type of output that results from this task finishing.
    type Output;

    /// Identifies the context(s) a [GpuTask] may be used with.
    ///
    /// If the [GpuTask] may be used with any context, then [GpuTask::context_id] should return
    /// `ContextId::Any`; if it may only be used with one specific context instance, then it should
    /// return `ContextId::Id(context_id)`, where `context_id` is the ID associated with the context
    /// (see [RenderingContext::id]).
    fn context_id(&self) -> ContextId;

    /// Attempts to progress this [GpuTask] towards its finished state using the given
    /// [execution_context].
    ///
    /// If this call to [GpuTask::progress] resulted in the task finishing, then it should return
    /// `Progress::Finished(output)`, where `output` is the task's output. The task executor should
    /// then drop this [GpuTask]; it should never call this method again.
    ///
    /// Otherwise, [GpuTask::progress] may return `Progress::ContinueFenced`. In this case the task
    /// executor will insert a GPU fence into the command stream. It will call this method again
    /// once that fence has become signalled.
    fn progress(&mut self, execution_context: &mut Ec) -> Progress<Self::Output>;
}

pub trait GpuTaskExt<Ec>: GpuTask<Ec> {
    /// Combines this task with another task `b`, waiting for both tasks to complete in no
    /// particular order.
    ///
    /// This returns a new "joined" task. This joined task may progress the its sub-tasks in any
    /// order. The joined task will finish when both sub-tasks have finished. When it finishes, it
    /// will output a tuple `(A, B)` where `A` is this tasks output and `B` is task `b`'s output.
    ///
    /// # Panics
    ///
    /// Panics if the [ContextId] of `b` is not compatible with this task's [ContextId].
    fn join<B>(self, b: B) -> Join<Self, B, Ec>
    where
        B: GpuTask<Ec>,
        Self: Sized;

    /// Combines this task with 2 other tasks `b` and `c`, waiting for all tasks to complete in no
    /// particular order.
    ///
    /// This returns a new "joined" task. This joined task may progress the its sub-tasks in any
    /// order. The joined task will finish when all sub-tasks have finished. When it finishes, it
    /// will output a tuple `(A, B, C)` where `A` is this tasks output, `B` is task `b`'s output and
    /// `C` is task `c`'s output.
    ///
    /// # Panics
    ///
    /// Panics if any of the [ContextId]s of `b` and `c` are not compatible with this task's
    /// [ContextId].
    fn join3<B, C>(self, b: B, c: C) -> Join3<Self, B, C, Ec>
    where
        B: GpuTask<Ec>,
        C: GpuTask<Ec>,
        Self: Sized;

    /// Combines this task with 3 other tasks `b`, `c` and `d`, waiting for all tasks to complete in
    /// no particular order.
    ///
    /// This returns a new "joined" task. This joined task may progress the its sub-tasks in any
    /// order. The joined task will finish when all sub-tasks have finished. When it finishes, it
    /// will output a tuple `(A, B, C, D)` where `A` is this tasks output, `B` is task `b`'s output,
    /// `C` is task `c`'s output and `D` is task `d`'s output.
    ///
    /// # Panics
    ///
    /// Panics if any of the [ContextId]s of `b`, `c` and `d` are not compatible with this task's
    /// [ContextId].
    fn join4<B, C, D>(self, b: B, c: C, d: D) -> Join4<Self, B, C, D, Ec>
    where
        B: GpuTask<Ec>,
        C: GpuTask<Ec>,
        D: GpuTask<Ec>,
        Self: Sized;

    /// Combines this task with 4 other tasks `b`, `c`, `d` and `e`, waiting for all tasks to
    /// complete in no particular order.
    ///
    /// This returns a new "joined" task. This joined task may progress the its sub-tasks in any
    /// order. The joined task will finish when all sub-tasks have finished. When it finishes, it
    /// will output a tuple `(A, B, C, D, E)` where `A` is this tasks output, `B` is task `b`'s
    /// output, `C` is task `c`'s output, `D` is task `d`'s output and `E` is task `e`'s output.
    ///
    /// # Panics
    ///
    /// Panics if any of the [ContextId]s of `b`, `c`, `d` and `e` are not compatible with this
    /// task's [ContextId].
    fn join5<B, C, D, E>(self, b: B, c: C, d: D, e: E) -> Join5<Self, B, C, D, E, Ec>
    where
        B: GpuTask<Ec>,
        C: GpuTask<Ec>,
        D: GpuTask<Ec>,
        E: GpuTask<Ec>,
        Self: Sized;

    /// Combines this task with another task `b`, waiting for both tasks to complete in order.
    ///
    /// This returns a new "sequenced" task. This sequenced task must progress its sub-tasks in
    /// order: it may only start to progress the next task if the previous task has finished. The
    /// sequenced task will finish when the last sub-task has finished. When it finishes, it will
    /// output a tuple `(A, B)` where `A` is this tasks output and `B` is task `b`'s output.
    ///
    /// # Panics
    ///
    /// Panics if the [ContextId] of `b` is not compatible with this task's [ContextId].
    fn sequence<B>(self, b: B) -> Sequence<Self, B, Ec>
    where
        B: GpuTask<Ec>,
        Self: Sized;

    /// Combines this task with 2 other tasks `b` and `c`, waiting for all tasks to complete in
    /// order.
    ///
    /// This returns a new "sequenced" task. This sequenced task must progress its sub-tasks in
    /// order: it may only start to progress the next task if the previous task has finished. The
    /// sequenced task will finish when the last sub-task has finished. When it finishes, it will
    /// output a tuple `(A, B, C)` where `A` is this tasks output, `B` is task `b`'s output and `C`
    /// is task `c`'s output.
    ///
    /// # Panics
    ///
    /// Panics if any of the [ContextId]s of `b` and `c` are not compatible with this task's
    /// [ContextId].
    fn sequence3<B, C>(self, b: B, c: C) -> Sequence3<Self, B, C, Ec>
    where
        B: GpuTask<Ec>,
        C: GpuTask<Ec>,
        Self: Sized;

    /// Combines this task with 3 other tasks `b`, `c` and `d`, waiting for all tasks to complete in
    /// order.
    ///
    /// This returns a new "sequenced" task. This sequenced task must progress its sub-tasks in
    /// order: it may only start to progress the next task if the previous task has finished. The
    /// sequenced task will finish when the last sub-task has finished. When it finishes, it will
    /// output a tuple `(A, B, C, D)` where `A` is this tasks output, `B` is task `b`'s output, `C`
    /// is task `c`'s output and `D` is task `d`'s output.
    ///
    /// # Panics
    ///
    /// Panics if any of the [ContextId]s of `b`, `c` and `d` are not compatible with this task's
    /// [ContextId].
    fn sequence4<B, C, D>(self, b: B, c: C, d: D) -> Sequence4<Self, B, C, D, Ec>
    where
        B: GpuTask<Ec>,
        C: GpuTask<Ec>,
        D: GpuTask<Ec>,
        Self: Sized;

    /// Combines this task with 4 other tasks `b`, `c`, `d` and `e`, waiting for all tasks to
    /// complete in order.
    ///
    /// This returns a new "sequenced" task. This sequenced task must progress its sub-tasks in
    /// order: it may only start to progress the next task if the previous task has finished. The
    /// sequenced task will finish when the last sub-task has finished. When it finishes, it will
    /// output a tuple `(A, B, C, D, E)` where `A` is this tasks output, `B` is task `b`'s output,
    /// `C` is task `c`'s output, `D` is task `d`'s output and `E` is task `e`'s output.
    ///
    /// # Panics
    ///
    /// Panics if any of the [ContextId]s of `b`, `c`, `d` and `e` are not compatible with this
    /// task's [ContextId].
    fn sequence5<B, C, D, E>(self, b: B, c: C, d: D, e: E) -> Sequence5<Self, B, C, D, E, Ec>
    where
        B: GpuTask<Ec>,
        C: GpuTask<Ec>,
        D: GpuTask<Ec>,
        E: GpuTask<Ec>,
        Self: Sized;
}

impl<T, Ec> GpuTaskExt<Ec> for T
where
    T: GpuTask<Ec>,
{
    fn join<B>(self, b: B) -> Join<T, B, Ec>
    where
        B: GpuTask<Ec>,
    {
        Join::new(self, b)
    }

    fn join3<B, C>(self, b: B, c: C) -> Join3<T, B, C, Ec>
    where
        B: GpuTask<Ec>,
        C: GpuTask<Ec>,
    {
        Join3::new(self, b, c)
    }

    fn join4<B, C, D>(self, b: B, c: C, d: D) -> Join4<T, B, C, D, Ec>
    where
        B: GpuTask<Ec>,
        C: GpuTask<Ec>,
        D: GpuTask<Ec>,
    {
        Join4::new(self, b, c, d)
    }

    fn join5<B, C, D, E>(self, b: B, c: C, d: D, e: E) -> Join5<T, B, C, D, E, Ec>
    where
        B: GpuTask<Ec>,
        C: GpuTask<Ec>,
        D: GpuTask<Ec>,
        E: GpuTask<Ec>,
    {
        Join5::new(self, b, c, d, e)
    }

    fn sequence<B>(self, b: B) -> Sequence<T, B, Ec>
    where
        B: GpuTask<Ec>,
    {
        Sequence::new(self, b)
    }

    fn sequence3<B, C>(self, b: B, c: C) -> Sequence3<T, B, C, Ec>
    where
        B: GpuTask<Ec>,
        C: GpuTask<Ec>,
    {
        Sequence3::new(self, b, c)
    }

    fn sequence4<B, C, D>(self, b: B, c: C, d: D) -> Sequence4<T, B, C, D, Ec>
    where
        B: GpuTask<Ec>,
        C: GpuTask<Ec>,
        D: GpuTask<Ec>,
    {
        Sequence4::new(self, b, c, d)
    }

    fn sequence5<B, C, D, E>(self, b: B, c: C, d: D, e: E) -> Sequence5<T, B, C, D, E, Ec>
    where
        B: GpuTask<Ec>,
        C: GpuTask<Ec>,
        D: GpuTask<Ec>,
        E: GpuTask<Ec>,
    {
        Sequence5::new(self, b, c, d, e)
    }
}


/// Returned from [GpuTask::progress], signifies the current state of progress for the task.
///
/// See [GpuTask::progress] for details.
pub enum Progress<T> {
    Finished(T),
    ContinueFenced,
}

/// Returned from [GpuTask::context_id], identifies the context(s) a [GpuTask] may be used with.
///
/// See [GpuTask::context_id] for details.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ContextId {
    Any,
    Id(usize),
}

impl ContextId {
    /// Attempts to combine two [ContextId]s, or returns an [IncompatibleContextIds] error when they
    /// are incompatible.
    ///
    /// Two [ContextId]s are compatible if:
    ///
    /// - Either or both [ContextId]s are [ContextId::Any]. In this case [ContextId::Any] will be
    ///   returned.
    /// - Both are [ContextId::Id] and the two IDs are identical. In this case [ContextId::Id(id)]
    ///   will be returned, where `id` is the ID shared by both [ContextId]s.
    ///
    /// Two [ContextId]s are incompatible if:
    ///
    /// - Both are [ContextId::Id] and the two IDs are not identical. In this case an
    ///   [IncompatibleContextIds] error will be returned.
    pub fn combine(&self, other: ContextId) -> Result<ContextId, IncompatibleContextIds> {
        match self {
            ContextId::Any => Ok(other),
            ContextId::Id(id) => {
                if other == ContextId::Any || other == ContextId::Id(*id) {
                    Ok(*self)
                } else {
                    Err(IncompatibleContextIds(*self, other))
                }
            }
        }
    }
}

/// Error returned by [ContextId::combine] if the two context IDs that are combined are
/// incompatible.
///
/// Two context IDs are incompatible if they are both [ContextId::Id] and the ID values are not
/// identical.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct IncompatibleContextIds(ContextId, ContextId);
