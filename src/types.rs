//! Various helper types.
use ceres_solver_sys::ffi::{
    RustIterationSummary,
    RustCallbackReturnType,
};

pub type IterationSummary = RustIterationSummary;
pub type CallbackReturnType = RustCallbackReturnType;

pub type JacobianType<'a> = Option<&'a mut [Option<&'a mut [&'a mut [f64]]>]>;

pub(crate) enum Either<A, B> {
    Left(A),
    Right(B),
}

impl<A, B, T> Iterator for Either<A, B>
where
    A: Iterator<Item = T>,
    B: Iterator<Item = T>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Either::Left(a) => a.next(),
            Either::Right(b) => b.next(),
        }
    }
}
