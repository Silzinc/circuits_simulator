use crate::fourier::fouriers;
use num::Complex;
use num_traits::PrimInt;
use std::fmt::Debug;

#[derive(Clone, Debug)]
/// A source of voltage.
pub struct Source
{
	/// Map between pulses (sorted) and voltages.
	pub voltages: Vec<(f64, Complex<f64>)>,
}

impl Source
{
	/// Creates a new `Source` with an empty vector of voltages.
	pub fn new() -> Self { Source { voltages: vec![] } }

	/// Sets the voltage at a specific index in the `voltages` vector.
	pub fn set_voltage(&mut self, index: usize, voltage: Complex<f64>) { self.voltages[index].1 = voltage; }

	/// Adds a new pulse to the `voltages` vector at the specified time.
	/// The pulse is represented by a voltage value.
	pub fn add_pulse(&mut self, pulse: f64, voltage: Complex<f64>)
	{
		let index = self.voltages.iter().position(|(f, _)| *f > pulse).unwrap_or(self.voltages.len());
		self.voltages.insert(index, (pulse, voltage));
	}

	/// Removes the pulse at the specified index from the `voltages` vector.
	pub fn remove_pulse(&mut self, index: usize) { self.voltages.remove(index); }
}

impl Source
{
	/// Creates a new `Source` from a real valued function that generates voltage
	/// values using its Fourier transform. The function takes a time value as
	/// input and returns a voltage value. The `duration` parameter specifies the
	/// total duration of the voltage source (henceforth the duration of the
	/// simulation). The `n_freqs_` parameter specifies the number of frequencies
	/// to use in the Fourier series.
	pub fn from_fn<I, F>(f: F, duration: f64, n_freqs_: I) -> Self
		where F: Fn(f64) -> f64,
		      I: PrimInt + Debug
	{
		let fundamental = (duration + duration).recip(); // Shannon's theorem
		let n_freqs = n_freqs_.to_usize().expect(&format!("Failed to convert {n_freqs_:?} to usize"));
		let mut source = Self { voltages: Vec::with_capacity(n_freqs), };
		let fourier_coefs = fouriers(f, fundamental, n_freqs - 1);
		let twopif = fundamental * 2. * std::f64::consts::PI;

		let mut pulse = 0f64;
		for coef in fourier_coefs {
			source.add_pulse(pulse, coef);
			pulse += twopif;
		}
		source
	}
}
