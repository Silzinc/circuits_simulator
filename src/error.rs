use std::{
  fmt::{
    Debug,
    Display,
  },
  io::Error as IOError,
};

use fractios::RatioFrac;

use crate::Id;

/// Represents an error that can occur during circuit building or solving.
#[derive(Debug)]
pub enum Error
{
  /// An error that occurred during circuit building.
  CircuitBuild(String),
  /// An error that occurred during circuit solving.
  CircuitSolve(String),
  /// An I/O error that occurred during the algorithm execution.
  IO(IOError),
}

pub type Result<T> = std::result::Result<T, Error>;

impl Display for Error
{
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result
  {
    use Error::*;
    let msg = match self {
      CircuitBuild(s) => format!("CircuitBuild: {}", s),
      CircuitSolve(s) => format!("CircuitSolve: {}", s),
      IO(e) => format!("IOError: {}", e),
    };
    write!(f, "{}", msg)
  }
}

impl std::error::Error for Error {}

impl From<IOError> for Error
{
  fn from(e: IOError) -> Self
  {
    Error::IO(e)
  }
}

pub(crate) fn short_circuit_current<U: Debug, V: Debug, T>(
  id: &Id,
  current: U,
  impedance: &RatioFrac<V>,
) -> Result<T>
{
  Err(Error::CircuitSolve(format!(
    "Short circuit is caused by a non zero constant current source on a zero admittance \
     component\n\nComponent Id: {:?}\nCurrent: {:?} A\nImpedance (rational fraction of \
     pulse):\n{:?}\n-------\n{:?}",
    id, current, impedance.numerator, impedance.denominator
  )))
}

pub(crate) fn short_circuit_tension<U: Debug, V: Debug, T>(
  id: &Id,
  tension: U,
  impedance: &RatioFrac<V>,
) -> Result<T>
{
  Err(Error::CircuitSolve(format!(
    "Short circuit is caused by a non zero constant tension source on a zero impedance \
     component\n\nComponent Id: {:?}\nTension: {:?} V\nImpedance (rational fraction of \
     pulse):\n{:?}\n-------\n{:?}",
    id, tension, impedance.numerator, impedance.denominator
  )))
}
