use crate::structs::node::Id;
use fractios::RatioFrac;
use std::{fmt::Debug, io::Error as IOError};

/// Represents an error that can occur during circuit building or solving.
#[derive(Debug)]
pub enum Error
{
	/// An error that occurred during circuit building.
	CircuitBuildError(String),
	/// An error that occurred during circuit solving.
	CircuitSolveError(String),
	/// An I/O error that occurred during the algorithm execution.
	IOError(IOError),
}

pub type Result<T> = std::result::Result<T, Error>;

impl Error
{
	pub fn to_string(&self) -> String
	{
		use Error::*;
		match self {
			CircuitBuildError(s) => format!("CircuitBuildError: {}", s),
			CircuitSolveError(s) => format!("CircuitSolveError: {}", s),
			IOError(e) => format!("IOError: {}", e),
		}
	}
}

impl std::fmt::Display for Error
{
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result { write!(f, "{}", self.to_string()) }
}

impl std::error::Error for Error {}

impl From<IOError> for Error
{
	fn from(e: IOError) -> Self { Error::IOError(e) }
}

pub(crate) fn short_circuit_current<U: Debug, V: Debug, T>(id: &Id, current: U, impedance: &RatioFrac<V>) -> Result<T>
{
	Err(Error::CircuitSolveError(format!("Short circuit is caused by a non zero constant current source on a zero admittance component\n\nComponent \
	                                      Id: {:?}\nCurrent: {:?} A\nImpedance (rational fraction of pulse):\n{:?}\n-------\n{:?}",
	                                     id, current, impedance.numerator, impedance.denominator)))
}

pub(crate) fn short_circuit_tension<U: Debug, V: Debug, T>(id: &Id, tension: U, impedance: &RatioFrac<V>) -> Result<T>
{
	Err(Error::CircuitSolveError(format!("Short circuit is caused by a non zero constant tension source on a zero impedance component\n\nComponent \
	                                      Id: {:?}\nTension: {:?} V\nImpedance (rational fraction of pulse):\n{:?}\n-------\n{:?}",
	                                     id, tension, impedance.numerator, impedance.denominator)))
}
