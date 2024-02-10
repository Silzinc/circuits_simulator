#![allow(dead_code)]
use fractios::RatioFrac;
use num_traits::Zero;

// Note that an impedance is never a multiple of x² or 1/x², where x is the
// pulse. In particular, a component behaves as a wire under a constant tension
// if and only if its impedance is a multiple of x and as an open interruptor if
// and only if its impedance is a multiple of 1/x. The following functions help
// to handle these cases.

// The ratiofrac is assumed to be reduced in the following functions
#[inline]
pub(crate) fn is_multiple_of_x<T: Zero + Clone>(r: &RatioFrac<T>) -> bool { r.numerator[0].is_zero() }
#[inline]
pub(crate) fn is_multiple_of_invx<T: Zero + Clone>(r: &RatioFrac<T>) -> bool { r.denominator[0].is_zero() }

#[inline]
pub(crate) fn evaluate_zero_without_x<T>(r: &RatioFrac<T>) -> T
where
	T: Zero + Clone + std::ops::Div<Output = T>,
{
	r.numerator[1].clone() / r.denominator[0].clone()
}
#[inline]
pub(crate) fn evaluate_zero_without_invx<T>(r: &RatioFrac<T>) -> T
where
	T: Zero + Clone + std::ops::Div<Output = T>,
{
	r.numerator[0].clone() / r.denominator[1].clone()
}
