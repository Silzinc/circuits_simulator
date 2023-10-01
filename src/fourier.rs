use fractios::traits::{RatioFracComplexFloat, RatioFracFloat};
use num_traits::PrimInt;
use rustfft::{num_complex::Complex, FftNum, FftPlanner};

/*
This function takes a real valued function g of period 1/Δf, the fundamental frequency Δf and a number of frequencies n_freqs > 0.
It returns the values of ĝ(0), ĝ(Δf), ĝ(2Δf), ..., ĝ(n_freqs * Δf) where ĝ is the Fourier transform of g and Δf is the fundamental frequency.
ĝ(f) = Δf * ∫ g(t) exp(-2πi f t) dt with the integral from -1/2Δf to 1/2Δf.
*/

// Time complexity: O(n_freqs log n_freqs) (using FFT)
// Space complexity: O(n_freqs)

pub(crate) fn fouriers<F, R, I>(g: F, fundamental: R, n_freqs_: I) -> Vec<Complex<R>>
	where F: Fn(R) -> R,
	      R: FftNum + RatioFracFloat,
	      I: PrimInt,
	      Complex<R>: RatioFracComplexFloat
{
	let n_freqs = n_freqs_.to_usize().unwrap();
	assert!(n_freqs > 0);

	if fundamental.is_negative() {
		let mut result = fouriers(g, -fundamental, n_freqs_.clone());
		for c in result.iter_mut() {
			c.im = -c.im;
		}
		return result;
	}

	let delta_f = fundamental;
	let n = 2 * n_freqs + 1; // We take the 0 frequency and respect the Shannon-Nyquist criterion

	let t = (delta_f * R::from_usize(n).unwrap()).inv();
	let invn = R::from_usize(n).unwrap().inv();
	let halft = R::from_f64(0.5).unwrap() * delta_f.inv();

	let mut vals = (0..n).map(|i| Complex { re: g(t * R::from_usize(i).unwrap() - halft),
	                                        im: R::zero(), })
	                     .collect::<Vec<_>>();

	if fundamental.is_zero() {
		return vec![vals.iter().sum::<Complex<R>>() * invn.into()];
	}

	let mut planner = FftPlanner::new();
	let fft = planner.plan_fft_forward(n);
	fft.process(&mut vals);

	let mut change_sign = false;
	for i in 0..=n_freqs {
		vals[i] = vals[i] * invn.into();
		if change_sign {
			vals[i] = -vals[i];
		}
		change_sign = !change_sign;
	}
	vals.truncate(n_freqs + 1); // We only keep the half of the spectrum that
														// follows the Shannon-Nyquist criterion
	return vals;
}
