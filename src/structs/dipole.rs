use crate::error::{Error::CircuitBuildError, Result};
use fractios::{
	traits::{RatioFracComplexFloat, RatioFracFloat},
	RatioFrac,
};
use num::Complex;
use num_traits::Zero;
use polyx::{polynomial, Polynomial};

#[derive(Clone, Debug)]
pub enum Dipole<T>
{
	Resistor(T),
	Capacitor(T),
	Inductor(T),
	Equivalent(RatioFrac<Complex<T>>),
	Poisoned, // Used as a default state
}

impl<T: RatioFracFloat> Dipole<T> where Complex<T>: RatioFracComplexFloat
{
	pub(crate) fn impedance(&self) -> Result<RatioFrac<Complex<T>>>
	{
		match self {
			Dipole::Resistor(r) => Ok(RatioFrac::from(Complex::from(*r))),
			Dipole::Capacitor(c) => Ok(RatioFrac::from((
				polynomial![Complex { re: T::zero(),
				                      im: -(*c).recip(), }],
				polynomial![Complex::zero(), Complex::from(T::one())],
			))),
			Dipole::Inductor(l) => Ok(RatioFrac::from(polynomial![
				Complex::zero(),
				Complex { re: T::zero(),
				          im: *l, }
			])),
			Dipole::Equivalent(e) => Ok(e.clone()),
			Dipole::Poisoned => Err(CircuitBuildError("Called impedance on poisoned dipole".to_string())),
		}
	}
}
