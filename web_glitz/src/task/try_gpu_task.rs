use super::{
    AndThen, TryJoin, TryJoin3, TryJoin4, TryJoin5, MapOk, MapErr, OrElse, TrySequence, TrySequence3, TrySequence4,
    TrySequence5, Progress, GpuTask
};

pub trait TryGpuTask<Ec> {
    type Ok;

    type Error;

    fn try_progress(&mut self, execution_context: &mut Ec) -> Progress<Result<Self::Ok, Self::Error>>;
}

impl<T, O, E, Ec> TryGpuTask<Ec> for T where T: GpuTask<Ec, Output=Result<O, E>> {
    type Ok = O;

    type Error = E;

    fn try_progress(&mut self, execution_context: &mut Ec) -> Progress<Result<Self::Ok, Self::Error>> {
        self.progress(execution_context)
    }
}

pub trait TryGpuTaskExt<Ec>: TryGpuTask<Ec> {
    fn map_ok<B, F>(self, f: F) -> MapOk<Self, F, Ec>
        where
            F: FnOnce(Self::Ok) -> B,
            B: 'static,
            Self: Sized;

    fn map_err<B, F>(self, f: F) -> MapErr<Self, F, Ec>
        where
            F: FnOnce(Self::Error) -> B,
            B: 'static,
            Self: Sized;

    fn and_then<B, F>(self, f: F) -> AndThen<Self, B, F, Ec>
        where
            B: TryGpuTask<Ec, Error = Self::Error>,
            F: FnOnce(Self::Ok) -> B,
            Self: Sized;

    fn or_else<B, F>(self, f: F) -> OrElse<Self, B, F, Ec>
        where
            B: TryGpuTask<Ec, Ok = Self::Ok>,
            F: FnOnce(Self::Error) -> B,
            Self: Sized;

    fn try_join<B>(self, b: B) -> TryJoin<Self, B, Ec>
        where
            B: TryGpuTask<Ec, Error=Self::Error>,
            Self: Sized;

    fn try_join3<B, C>(self, b: B, c: C) -> TryJoin3<Self, B, C, Ec>
        where
            B: TryGpuTask<Ec, Error=Self::Error>,
            C: TryGpuTask<Ec, Error=Self::Error>,
            Self: Sized;

    fn try_join4<B, C, D>(self, b: B, c: C, d: D) -> TryJoin4<Self, B, C, D, Ec>
        where
            B: TryGpuTask<Ec, Error=Self::Error>,
            C: TryGpuTask<Ec, Error=Self::Error>,
            D: TryGpuTask<Ec, Error=Self::Error>,
            Self: Sized;

    fn try_join5<B, C, D, E>(self, b: B, c: C, d: D, e: E) -> TryJoin5<Self, B, C, D, E, Ec>
        where
            B: TryGpuTask<Ec, Error=Self::Error>,
            C: TryGpuTask<Ec, Error=Self::Error>,
            D: TryGpuTask<Ec, Error=Self::Error>,
            E: TryGpuTask<Ec, Error=Self::Error>,
            Self: Sized;

    fn try_sequence<B>(self, b: B) -> TrySequence<Self, B, Ec>
        where
            B: TryGpuTask<Ec, Error=Self::Error>,
            Self: Sized;

    fn try_sequence3<B, C>(self, b: B, c: C) -> TrySequence3<Self, B, C, Ec>
        where
            B: TryGpuTask<Ec, Error=Self::Error>,
            C: TryGpuTask<Ec, Error=Self::Error>,
            Self: Sized;

    fn try_sequence4<B, C, D>(self, b: B, c: C, d: D) -> TrySequence4<Self, B, C, D, Ec>
        where
            B: TryGpuTask<Ec, Error=Self::Error>,
            C: TryGpuTask<Ec, Error=Self::Error>,
            D: TryGpuTask<Ec, Error=Self::Error>,
            Self: Sized;

    fn try_sequence5<B, C, D, E>(self, b: B, c: C, d: D, e: E) -> TrySequence5<Self, B, C, D, E, Ec>
        where
            B: TryGpuTask<Ec, Error=Self::Error>,
            C: TryGpuTask<Ec, Error=Self::Error>,
            D: TryGpuTask<Ec, Error=Self::Error>,
            E: TryGpuTask<Ec, Error=Self::Error>,
            Self: Sized;
}

impl<T, Ec> TryGpuTaskExt<Ec> for T where T: TryGpuTask<Ec> {
    fn map_ok<B, F>(self, f: F) -> MapOk<Self, F, Ec>
        where
            F: FnOnce(Self::Ok) -> B,
            B: 'static,
            Self: Sized {
        MapOk::new(self, f)
    }

    fn map_err<B, F>(self, f: F) -> MapErr<Self, F, Ec>
        where
            F: FnOnce(Self::Error) -> B,
            B: 'static,
            Self: Sized {
        MapErr::new(self, f)
    }

    fn and_then<B, F>(self, f: F) -> AndThen<Self, B, F, Ec>
        where
            B: TryGpuTask<Ec, Error = Self::Error>,
            F: FnOnce(Self::Ok) -> B,
            Self: Sized {
        AndThen::new(self, f)
    }

    fn or_else<B, F>(self, f: F) -> OrElse<Self, B, F, Ec>
        where
            B: TryGpuTask<Ec, Ok = Self::Ok>,
            F: FnOnce(Self::Error) -> B,
            Self: Sized {
        OrElse::new(self, f)
    }

    fn try_join<B>(self, b: B) -> TryJoin<Self, B, Ec>
        where
            B: TryGpuTask<Ec, Error=Self::Error>,
            Self: Sized {
        TryJoin::new(self, b)
    }

    fn try_join3<B, C>(self, b: B, c: C) -> TryJoin3<Self, B, C, Ec>
        where
            B: TryGpuTask<Ec, Error=Self::Error>,
            C: TryGpuTask<Ec, Error=Self::Error>,
            Self: Sized {
        TryJoin3::new(self, b, c)
    }

    fn try_join4<B, C, D>(self, b: B, c: C, d: D) -> TryJoin4<Self, B, C, D, Ec>
        where
            B: TryGpuTask<Ec, Error=Self::Error>,
            C: TryGpuTask<Ec, Error=Self::Error>,
            D: TryGpuTask<Ec, Error=Self::Error>,
            Self: Sized {
        TryJoin4::new(self, b, c, d)
    }

    fn try_join5<B, C, D, E>(self, b: B, c: C, d: D, e: E) -> TryJoin5<Self, B, C, D, E, Ec>
        where
            B: TryGpuTask<Ec, Error=Self::Error>,
            C: TryGpuTask<Ec, Error=Self::Error>,
            D: TryGpuTask<Ec, Error=Self::Error>,
            E: TryGpuTask<Ec, Error=Self::Error>,
            Self: Sized {
        TryJoin5::new(self, b, c, d, e)
    }

    fn try_sequence<B>(self, b: B) -> TrySequence<Self, B, Ec>
        where
            B: TryGpuTask<Ec, Error=Self::Error>,
            Self: Sized {
        TrySequence::new(self, b)
    }

    fn try_sequence3<B, C>(self, b: B, c: C) -> TrySequence3<Self, B, C, Ec>
        where
            B: TryGpuTask<Ec, Error=Self::Error>,
            C: TryGpuTask<Ec, Error=Self::Error>,
            Self: Sized {
        TrySequence3::new(self, b, c)
    }

    fn try_sequence4<B, C, D>(self, b: B, c: C, d: D) -> TrySequence4<Self, B, C, D, Ec>
        where
            B: TryGpuTask<Ec, Error=Self::Error>,
            C: TryGpuTask<Ec, Error=Self::Error>,
            D: TryGpuTask<Ec, Error=Self::Error>,
            Self: Sized {
        TrySequence4::new(self, b, c, d)
    }

    fn try_sequence5<B, C, D, E>(self, b: B, c: C, d: D, e: E) -> TrySequence5<Self, B, C, D, E, Ec>
        where
            B: TryGpuTask<Ec, Error=Self::Error>,
            C: TryGpuTask<Ec, Error=Self::Error>,
            D: TryGpuTask<Ec, Error=Self::Error>,
            E: TryGpuTask<Ec, Error=Self::Error>,
            Self: Sized {
        TrySequence5::new(self, b, c, d, e)
    }
}