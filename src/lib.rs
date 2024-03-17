//! A crate that can be used to simulate simple electrical circuits. It
//! currently supports analyzing circuits with resistors, capacitors, inductors,
//! and "equivalent impedance" components represented by their impedance. Said
//! impedance is represented by a rational fraction of the pulse with complex
//! coefficients. This crate makes extensive use of complex number
//! representation for electronics, including the Fourier transform.
//!
//! A circuit can only be made of one source and a tree of components. Said
//! components are either made of parallel or serial combinations of other
//! components.
//!
//! # Example
//! ```
//! // Emulate a serial RLC circuit with a square wave of period
//! // 4 ms that starts after 2 ms as input
//! use std::{
//!   env,
//!   time::Instant,
//! };
//!
//! use circuits_simulator::{
//!   Circuit,
//!   Component,
//!   Dipole::{
//!     Capacitor,
//!     Inductor,
//!     Resistor,
//!   },
//! };
//! use plotters::prelude::*;
//!
//! fn square_wave(x: f64) -> f64
//! {
//!   if x < 2e-3 {
//!     0.
//!   } else if ((x + 2e-3) % 4e-3) > 2e-3 {
//!     -1.
//!   } else {
//!     1.
//!   }
//! }
//!
//! let n_freqs = 1000;
//! let duration = 10e-3;
//! let step = duration / n_freqs as f64;
//!
//! // Create the serial RLC circuit
//! let mut c = Circuit::new();
//! c.set_generator_fn(square_wave, duration, n_freqs);
//! c.content_mut()
//!   .push_serie(Component::from(Resistor(200.))) // 0.2 kΩ, at position [0]
//!   .push_serie(Component::from(Capacitor(10e-9))) // 10 nF at position [1]
//!   .push_serie(Component::from(Inductor(100e-3))); // 100 mH at position [2]
//!
//! // With this, the attenuation time is 500 µs
//! // and the pseudo-period is close to 200 µs
//!
//! // Alternatively, we could have used the following:
//! // `c.get_comp_by_id_mut(&[]).unwrap().push_serie(...)`
//! // The `&[]` represents the ID [], corresponding to the root of the circuit.
//!
//! // Simulate the circuit
//! let start = Instant::now();
//! let result = c
//!   .emulate_many(duration, step, &vec![vec![], vec![1u8]])
//!   .unwrap(); // May return an error message if a short circuit is detected
//! let time_required = start.elapsed().as_secs_f64();
//! ```

mod emulation;
mod error;
mod fourier;
mod structs;
mod util;

pub use error::{
  Error,
  Result,
};
pub use structs::*;
