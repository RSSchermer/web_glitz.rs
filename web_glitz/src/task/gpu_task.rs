use super::{Join, Join3, Join4, Join5, Map, Sequence, Sequence3, Sequence4, Sequence5, Then};

pub enum Progress<T> {
    Finished(T),
    ContinueFenced,
}

pub trait GpuTask<Ec> {
    type Output;

    fn progress(&mut self, execution_context: &mut Ec) -> Progress<Self::Output>;
}

pub trait GpuTaskExt<Ec>: GpuTask<Ec> {
    fn map<B, F>(self, f: F) -> Map<Self, F, Ec>
    where
        F: FnOnce(Self::Output) -> B,
        B: 'static,
        Self: Sized;

    fn then<B, F>(self, f: F) -> Then<Self, B, F, Ec>
    where
        B: GpuTask<Ec>,
        F: FnOnce(Self::Output) -> B,
        Self: Sized;

    fn join<B>(self, b: B) -> Join<Self, B, Ec>
    where
        B: GpuTask<Ec>,
        Self: Sized;

    fn join3<B, C>(self, b: B, c: C) -> Join3<Self, B, C, Ec>
    where
        B: GpuTask<Ec>,
        C: GpuTask<Ec>,
        Self: Sized;

    fn join4<B, C, D>(self, b: B, c: C, d: D) -> Join4<Self, B, C, D, Ec>
    where
        B: GpuTask<Ec>,
        C: GpuTask<Ec>,
        D: GpuTask<Ec>,
        Self: Sized;

    fn join5<B, C, D, E>(self, b: B, c: C, d: D, e: E) -> Join5<Self, B, C, D, E, Ec>
    where
        B: GpuTask<Ec>,
        C: GpuTask<Ec>,
        D: GpuTask<Ec>,
        E: GpuTask<Ec>,
        Self: Sized;

    fn sequence<B>(self, b: B) -> Sequence<Self, B, Ec>
    where
        B: GpuTask<Ec>,
        Self: Sized;

    fn sequence3<B, C>(self, b: B, c: C) -> Sequence3<Self, B, C, Ec>
    where
        B: GpuTask<Ec>,
        C: GpuTask<Ec>,
        Self: Sized;

    fn sequence4<B, C, D>(self, b: B, c: C, d: D) -> Sequence4<Self, B, C, D, Ec>
    where
        B: GpuTask<Ec>,
        C: GpuTask<Ec>,
        D: GpuTask<Ec>,
        Self: Sized;

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
    fn map<B, F>(self, f: F) -> Map<T, F, Ec>
    where
        F: FnOnce(T::Output) -> B,
        B: 'static,
    {
        Map::new(self, f)
    }

    fn then<B, F>(self, f: F) -> Then<T, B, F, Ec>
    where
        B: GpuTask<Ec>,
        F: FnOnce(T::Output) -> B,
    {
        Then::new(self, f)
    }

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
