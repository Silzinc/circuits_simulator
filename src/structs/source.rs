use crate::fourier::fouriers;
use fractios::traits::{RatioFracComplexFloat, RatioFracFloat};
use num::Complex;
use num_traits::PrimInt;
use rustfft::FftNum;
use std::fmt::Debug;

#[derive(Clone, Debug)]
pub struct Source<T>
{
	pub voltages: Vec<(T, Complex<T>)>, // Map between frequencies (sorted) and voltages
}

impl<T: RatioFracFloat> Source<T>
{
	pub fn new() -> Self { Source { voltages: vec![] } }

	pub fn set_voltage(&mut self, index: usize, voltage: Complex<T>)
	{
		self.voltages[index].1 = voltage;
	}

	pub fn add_pulse(&mut self, pulse: T, voltage: Complex<T>)
	{
		let index = self.voltages
		                .iter()
		                .position(|(f, _)| *f > pulse)
		                .unwrap_or(self.voltages.len());
		self.voltages.insert(index, (pulse, voltage));
	}

	pub fn remove_pulse(&mut self, index: usize) { self.voltages.remove(index); }
}

impl<T: RatioFracFloat + FftNum> Source<T> where Complex<T>: RatioFracComplexFloat
{
	pub fn from_fn<I, F>(f: F, duration: T, n_freqs_: I) -> Self
		where F: Fn(T) -> T,
		      I: PrimInt + Debug
	{
		let fundamental = (duration + duration).recip(); // Shannon's theorem
		let n_freqs = n_freqs_.to_usize()
		                      .expect(&format!("Failed to convert {n_freqs_:?} to usize"));
		let mut source = Self { voltages: Vec::with_capacity(n_freqs), };
		let fourier_coefs = fouriers(f, fundamental, n_freqs - 1);
		let twopif = fundamental
		             * T::from(2. * std::f64::consts::PI).expect("Failed to convert 2pi to the \
		                                                          circuit's preferred scalar type");

		let mut pulse = T::zero();
		for coef in fourier_coefs {
			source.add_pulse(pulse, coef);
			pulse += twopif;
		}
		source
	}
}
