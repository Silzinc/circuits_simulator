use crate::error::{Error::CircuitBuild, Result};
use fractios::RatioFrac;
use num::Complex;
use num_traits::Zero;
use polyx::{polynomial, Polynomial};

#[derive(Clone, Debug, Default)]
/// Represents a dipole, which is an electrical component with two terminals.
/// It can be a resistor, capacitor, inductor, or an equivalent component.
pub enum Dipole
{
	/// A resistor with a given resistance in ohms.
	Resistor(f64),
	/// A capacitor with a given capacitance in farads.
	Capacitor(f64),
	/// An inductor with a given inductance in henries.
	Inductor(f64),
	/// An equivalent component represented by a rational fraction the pulse with
	/// complex coefficients.
	Equivalent(RatioFrac<Complex<f64>>),
	/// A poisoned state, used as a default state.
	#[default]
	Poisoned,
}

impl Dipole
{
	/// Calculates the impedance of a dipole.
	#[inline]
	pub fn impedance(&self) -> Result<RatioFrac<Complex<f64>>>
	{
		match self {
			Dipole::Resistor(r) => Ok(RatioFrac::from(Complex::from(*r))),
			Dipole::Capacitor(c) => Ok(RatioFrac::from((
				polynomial![Complex { re: 0f64, im: -(*c).recip() }],
				polynomial![Complex::zero(), Complex::from(1f64)],
			))),
			Dipole::Inductor(l) => Ok(RatioFrac::from(polynomial![Complex::zero(), Complex { re: 0f64, im: *l }])),
			Dipole::Equivalent(e) => Ok(e.clone()),
			Dipole::Poisoned => Err(CircuitBuild("Called impedance on poisoned dipole".to_string())),
		}
	}
}
