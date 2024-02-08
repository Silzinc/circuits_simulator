use crate::fourier::fouriers;
use num::Complex;
use num_traits::PrimInt;
use std::fmt::Debug;

#[derive(Clone, Debug, Default)]
/// A source of voltage.
pub struct Source
{
	/// Map between pulses (sorted) and voltages.
	pub voltages: Vec<(f64, Complex<f64>)>,
}

// Utility struct to enable binary search on f64
#[derive(Clone, Debug, PartialEq, PartialOrd)]
struct NonNan(f64);

impl Eq for NonNan {}
#[allow(clippy::derive_ord_xor_partial_ord)]
impl Ord for NonNan
{
	#[inline]
	fn cmp(&self, other: &Self) -> std::cmp::Ordering { self.partial_cmp(other).unwrap() }
}

impl Source
{
	/// Creates a new `Source` with an empty vector of voltages.
	#[inline]
	pub fn new() -> Self { Source { voltages: vec![] } }

	/// Sets the voltage at a specific index in the `voltages` vector.
	#[inline]
	pub fn set_voltage(&mut self, index: usize, voltage: Complex<f64>) -> &mut Self
	{
		self.voltages[index].1 = voltage;
		self
	}

	/// Adds a new pulse to the `voltages` vector at the specified time. If the
	/// pulse is already present, its voltage is updated. The pulse is represented
	/// by a voltage value.
	#[inline]
	pub fn add_pulse(&mut self, pulse: f64, voltage: Complex<f64>) -> &mut Self
	{
		match self.voltages.binary_search_by_key(&NonNan(pulse), |&(f, _)| NonNan(f)) {
			Ok(index) => self.voltages[index].1 = voltage,
			Err(index) => self.voltages.insert(index, (pulse, voltage)),
		};
		self
	}

	/// Removes the pulse at the specified index from the `voltages` vector.
	#[inline]
	pub fn remove_pulse(&mut self, index: usize) -> &mut Self
	{
		self.voltages.remove(index);
		self
	}

	/// Clears the `voltages` vector.
	#[inline]
	pub fn clear(&mut self) -> &mut Self
	{
		self.voltages.clear();
		self
	}

	/// Creates a new `Source` from a real valued function that generates voltage
	/// values using its Fourier transform. The function takes a time value as
	/// input and returns a voltage value. The `duration` parameter specifies the
	/// total duration of the voltage source (henceforth the duration of the
	/// simulation). The `n_freqs_` parameter specifies the number of frequencies
	/// to use in the Fourier series.
	#[inline]
	pub fn from_fn<I, F>(f: F, duration: f64, n_freqs_: I) -> Self
	where
		F: Fn(f64) -> f64,
		I: PrimInt + Debug,
	{
		let mut source = Self::new();
		source.set_fn(f, duration, n_freqs_);
		source
	}

	/// Clears and updates `self` using a real valued function that generates
	/// voltage values using its Fourier transform. The function takes a time
	/// value as input and returns a voltage value. The `duration` parameter
	/// specifies the total duration of the voltage source (henceforth the
	/// duration of the simulation). The `n_freqs_` parameter specifies the number
	/// of frequencies to use in the Fourier series.
	pub fn set_fn<I, F>(&mut self, f: F, duration: f64, n_freqs_: I) -> &mut Self
	where
		F: Fn(f64) -> f64,
		I: PrimInt + Debug,
	{
		self.clear();
		let fundamental = (duration + duration).recip(); // Shannon's theorem
		let n_freqs = n_freqs_.to_usize().unwrap_or_else(|| panic!("Failed to convert {n_freqs_:?} to usize"));
		let fourier_coefs = fouriers(f, fundamental, n_freqs - 1);
		let twopif = fundamental * 2. * std::f64::consts::PI;

		let mut pulse = 0f64;
		for coef in fourier_coefs {
			self.add_pulse(pulse, coef);
			pulse += twopif;
		}
		self
	}
}
