use crate::error::{self, Error::CircuitBuild};
use fractios::RatioFrac;
use num::Complex;
use num_traits::Zero;
use polyx::{polynomial, Polynomial};
use serde::{ser::SerializeStruct, Serialize, Serializer};

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
  pub fn impedance(&self) -> error::Result<RatioFrac<Complex<f64>>>
  {
    match self {
      Dipole::Resistor(r) => Ok(RatioFrac::from(Complex::from(r))),
      Dipole::Capacitor(c) => Ok(RatioFrac::from((
        polynomial![Complex { re: 0f64, im: -c.recip() }],
        polynomial![Complex::zero(), Complex::from(1f64)],
      ))),
      Dipole::Inductor(l) => Ok(RatioFrac::from(polynomial![Complex::zero(), Complex { re: 0f64, im: *l }])),
      Dipole::Equivalent(e) => Ok(e.clone()),
      Dipole::Poisoned => Err(CircuitBuild("Called impedance on poisoned dipole".to_string())),
    }
  }
}

impl Serialize for Dipole
{
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    let mut state = serializer.serialize_struct("Dipole", 2)?;
    match self {
      Dipole::Resistor(r) => {
        state.serialize_field("type", "resistor")?;
        state.serialize_field("value", r)?;
      },
      Dipole::Capacitor(c) => {
        state.serialize_field("type", "capacitor")?;
        state.serialize_field("value", c)?;
      },
      Dipole::Inductor(l) => {
        state.serialize_field("type", "inductor")?;
        state.serialize_field("value", l)?;
      },
      Dipole::Equivalent(_e) => {
        state.serialize_field("type", "equivalent")?;
        todo!("Serialize equivalent dipole");
        // state.serialize_field("value", e)?;
      },
      Dipole::Poisoned => {
        state.serialize_field("type", "poisoned")?;
      },
    }
    state.end()
  }
}
