use crate::fourier::fouriers;
use num::Complex;
use num_traits::PrimInt;
use std::fmt::Debug;

#[derive(Clone, Debug)]
pub struct Source
{
	pub voltages: Vec<(f64, Complex<f64>)>, // Map between frequencies (sorted) and voltages
}

impl Source
{
	pub fn new() -> Self { Source { voltages: vec![] } }

	pub fn set_voltage(&mut self, index: usize, voltage: Complex<f64>)
	{
		self.voltages[index].1 = voltage;
	}

	pub fn add_pulse(&mut self, pulse: f64, voltage: Complex<f64>)
	{
		let index = self.voltages
		                .iter()
		                .position(|(f, _)| *f > pulse)
		                .unwrap_or(self.voltages.len());
		self.voltages.insert(index, (pulse, voltage));
	}

	pub fn remove_pulse(&mut self, index: usize) { self.voltages.remove(index); }
}

impl Source
{
	pub fn from_fn<I, F>(f: F, duration: f64, n_freqs_: I) -> Self
		where F: Fn(f64) -> f64,
		      I: PrimInt + Debug
	{
		let fundamental = (duration + duration).recip(); // Shannon's theorem
		let n_freqs = n_freqs_.to_usize()
		                      .expect(&format!("Failed to convert {n_freqs_:?} to usize"));
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
