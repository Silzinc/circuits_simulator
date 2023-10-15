//! A crate that can be used to simulate simple electrical circuits. It
//! currently supports analyzing circuits with resistors, capacitors, inductors,
//! and "equivalent impedance" components represented by their impedance. Said
//! impedance is represented by a rational fraction of the pulse with complex
//! coefficients.
//!
//! A circuit can only be made of one source and a tree of components. Said
//! components are either made of parallel or serial combinations of other
//! components.

mod emulation;
pub mod error;
pub mod fourier;
pub mod structs;
mod util;

#[cfg(test)]
mod test;
