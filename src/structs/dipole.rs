use crate::error::{Error::CircuitBuildError, Result};
use fractios::RatioFrac;
use num::Complex;
use num_traits::Zero;
use polyx::{polynomial, Polynomial};

#[derive(Clone, Debug)]
pub enum Dipole
{
	Resistor(f64),
	Capacitor(f64),
	Inductor(f64),
	Equivalent(RatioFrac<Complex<f64>>),
	Poisoned, // Used as a default state
}

impl Dipole
{
	pub(crate) fn impedance(&self) -> Result<RatioFrac<Complex<f64>>>
	{
		match self {
			Dipole::Resistor(r) => Ok(RatioFrac::from(Complex::from(*r))),
			Dipole::Capacitor(c) => Ok(RatioFrac::from((
				polynomial![Complex { re: 0f64,
				                      im: -(*c).recip(), }],
				polynomial![Complex::zero(), Complex::from(1f64)],
			))),
			Dipole::Inductor(l) =>
				Ok(RatioFrac::from(polynomial![Complex::zero(), Complex { re: 0f64, im: *l }])),
			Dipole::Equivalent(e) => Ok(e.clone()),
			Dipole::Poisoned => Err(CircuitBuildError("Called impedance on poisoned dipole".to_string())),
		}
	}
}
