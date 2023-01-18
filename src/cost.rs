use crate::types::JacobianType;
use std::os::raw::{c_int, c_void};
use std::pin::Pin;
use std::slice;

pub type CostFunctionType<'a> = Box<dyn Fn(&[&[f64]], &mut [f64], JacobianType<'_>) -> bool + 'a>;

pub(crate) struct CostFunctionInner<'a> {
    parameter_sizes: Vec<usize>,
    num_residuals: usize,
    func: CostFunctionType<'a>,
}

/// A cost function for [ResidualBlock](crate::residual_block::ResidualBlock) of the
/// [NllsProblem](crate::nlls_problem::NllsProblem).
pub struct CostFunction<'a>(Pin<Box<CostFunctionInner<'a>>>);

impl<'a> CostFunction<'a> {
    /// Create a new cost function for [ResidualBlock](crate::residual_block::ResidualBlock) from
    /// a Rust function.
    ///
    /// # Arguments
    /// - func - function to find residuals and Jacobian for the problem block. The function itself
    /// must return [false] if it cannot compute Jacobian, [true] otherwise, and accept following
    /// arguments:
    ///   - parameters - slice of [f64] slices representing the current values of the parameters.
    ///   Each parameter is represented as a slice, the slice sizes are specified by
    ///   `parameter_sizes`.
    ///   - residuals - mutable slice of [f64] for residuals outputs, the size is specified by
    ///   `num_residuals`.
    ///   - jacobians - has a complex type [JacobianType](crate::types::JacobianType) and represents
    ///   a mutable structure to output the Jacobian. Sometimes solver doesn't need the Jacobian or
    ///   some of its components, in this case the corresponding value is [None]. For the required
    ///   components it has a 3-D structure: top index is for parameter, middle index is for
    ///   the residual index, and the most inner dimension is for parameter component index. So the
    ///   size of top-level [Some] is defined by `parameter_sizes.len()`, second-level [Some]'s
    ///   length is `num_residuals`, and the bottom-level slice has length of `parameter_sizes[i]`,
    ///    where `i` is the top-level index.
    /// - parameter_sizes - sizes of the parameter vectors.
    /// - num_residuals - length of the residual vector, usually corresponds to the number of
    /// data points.
    pub fn new(
        func: impl Into<CostFunctionType<'a>>,
        parameter_sizes: impl Into<Vec<usize>>,
        num_residuals: usize,
    ) -> Self {
        Self(Box::pin(CostFunctionInner {
            func: func.into(),
            parameter_sizes: parameter_sizes.into(),
            num_residuals,
        }))
    }

    pub(crate) fn cost_function_data(&mut self) -> *mut c_void {
        Pin::into_inner(self.0.as_mut()) as *mut CostFunctionInner as *mut c_void
    }

    /// Lengths of the parameter vectors.
    #[inline]
    pub fn parameter_sizes(&self) -> &[usize] {
        &self.0.parameter_sizes
    }

    /// Length of the residual vector.
    #[inline]
    pub fn num_residuals(&self) -> usize {
        self.0.num_residuals
    }

    /// Parameter count.
    #[inline]
    pub fn num_parameters(&self) -> usize {
        self.0.parameter_sizes.len()
    }

    /// Calls underlying cost function.
    #[inline]
    pub fn cost(
        &self,
        parameters: &[&[f64]],
        residuals: &mut [f64],
        jacobians: JacobianType<'_>,
    ) -> bool {
        (self.0.func)(parameters, residuals, jacobians)
    }
}

struct OwnedJacobian<'a>(Option<Vec<Option<Vec<&'a mut [f64]>>>>);

impl<'a> OwnedJacobian<'a> {
    fn from_pointer(
        pointer: *mut *mut f64,
        parameter_sizes: &[usize],
        num_residuals: usize,
    ) -> Self {
        if pointer.is_null() {
            return Self(None);
        }
        let per_parameter = unsafe { slice::from_raw_parts_mut(pointer, parameter_sizes.len()) };
        let vec = per_parameter
            .iter()
            .zip(parameter_sizes)
            .map(|(&p, &size)| OwnedDerivative::from_pointer(p, size, num_residuals).0)
            .collect();
        Self(Some(vec))
    }

    fn references(&'a mut self) -> Option<Vec<Option<&'a mut [&'a mut [f64]]>>> {
        let v = self
            .0
            .as_mut()?
            .iter_mut()
            .map(|der| der.as_mut().map(|v| &mut v[..]))
            .collect();
        Some(v)
    }
}

struct OwnedDerivative<'a>(Option<Vec<&'a mut [f64]>>);

impl<'a> OwnedDerivative<'a> {
    fn from_pointer(pointer: *mut f64, parameter_size: usize, num_residuals: usize) -> Self {
        if pointer.is_null() {
            return Self(None);
        }
        let per_residual_per_param_component =
            { unsafe { slice::from_raw_parts_mut(pointer, parameter_size * num_residuals) } };
        let v = per_residual_per_param_component
            .chunks_exact_mut(parameter_size)
            .collect();
        Self(Some(v))
    }
}

#[no_mangle]
pub(crate) unsafe extern "C" fn ffi_cost_function(
    user_data: *mut c_void,
    parameters: *mut *mut f64,
    residuals: *mut f64,
    jacobians: *mut *mut f64,
) -> c_int {
    let cost_function_inner = (user_data as *mut CostFunctionInner).as_ref().unwrap();
    let parameter_pointers =
        slice::from_raw_parts(parameters, cost_function_inner.parameter_sizes.len());
    let parameters = parameter_pointers
        .iter()
        .zip(cost_function_inner.parameter_sizes.iter())
        .map(|(&p, &size)| slice::from_raw_parts(p, size))
        .collect::<Vec<_>>();
    let residuals = slice::from_raw_parts_mut(residuals, cost_function_inner.num_residuals);
    let mut jacobians_owned = OwnedJacobian::from_pointer(
        jacobians,
        &cost_function_inner.parameter_sizes,
        cost_function_inner.num_residuals,
    );
    let mut jacobian_references = jacobians_owned.references();
    (cost_function_inner.func)(
        &parameters,
        residuals,
        jacobian_references.as_mut().map(|v| &mut v[..]),
    )
    .into()
}
