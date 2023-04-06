pub use ceres_solver_sys::ffi::{
    RustIterationSummary as IterationSummary,
    RustCallbackReturnType as CallbackReturnType
};

pub trait IterationCallback {
    fn invoke(&mut self, summary: IterationSummary) -> CallbackReturnType;
}
