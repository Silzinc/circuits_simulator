use num_traits::{
  PrimInt,
  Zero,
};
use rustfft::{
  num_complex::Complex,
  FftPlanner,
};

/// This function takes a real valued function g of period 1/Δf, the fundamental
/// frequency Δf and a number of frequencies n_freqs > 0. It returns the values
/// of ĝ(0), ĝ(Δf), ĝ(2Δf), ..., ĝ(n_freqs * Δf) where ĝ is the Fourier
/// transform of g and Δf is the fundamental frequency.
/// ĝ(f) = Δf * ∫ g(t) exp(-2πi f t) dt with the integral from -1/2Δf to 1/2Δf.
///
/// # Arguments
///
/// * `g` - A closure that takes a `f64` argument and returns a `f64` value.
///   This is the real valued function of period 1/Δf.
/// * `fundamental` - A `f64` value representing the fundamental frequency Δf.
/// * `n_freqs_` - An integer value representing the number of frequencies such
///   that n_freqs_ > 0.
///
/// # Returns
///
/// A `Vec` of `Complex<f64>` values representing the Fourier transform of g
/// evaluated at ĝ(0), ĝ(Δf), ĝ(2Δf), ..., ĝ(n_freqs * Δf).
///
/// # Time complexity
///
/// O(n_freqs log n_freqs) (using FFT)
///
/// # Space complexity
///
/// O(n_freqs)

pub fn fouriers<F, I>(g: F, fundamental: f64, n_freqs_: I) -> Vec<Complex<f64>>
where
  F: Fn(f64) -> f64,
  I: PrimInt,
{
  let n_freqs = n_freqs_.to_usize().unwrap();
  assert!(n_freqs > 0);

  if fundamental.is_sign_negative() {
    let mut result = fouriers(g, -fundamental, n_freqs_);
    for c in result.iter_mut() {
      c.im = -c.im;
    }
    return result;
  }

  let delta_f = fundamental;
  let n = 2 * n_freqs + 1;
  // We take the 0 frequency and make sure we do enough samples for an integration
  // on the interval [-1/2Δf, 1/2Δf]. Since we want the coefficients up to the
  // frequency n_freqs * Δf, we need to take double samples to satisfy
  // Shannon-Nyquist, perform a FFT on the samples and only keep the first
  // (correct) half of the spectrum.

  let t = (delta_f * n as f64).recip();
  let invn = (n as f64).recip();
  let halft = 0.5f64 / delta_f;

  let mut vals = (0..n)
    .map(|i| Complex {
      re: g(t * (i as f64 + 0.5) - halft),
      im: 0f64,
    })
    .collect::<Vec<_>>();

  if fundamental.is_zero() {
    return vec![vals.iter().sum::<Complex<f64>>() * invn];
  }

  let mut planner = FftPlanner::new();
  let fft = planner.plan_fft_forward(n);
  fft.process(&mut vals);

  let mut change_sign = false;
  for val in vals.iter_mut().take(n_freqs + 1) {
    *val *= invn;
    if change_sign {
      *val = -*val;
    }
    change_sign = !change_sign;
  }
  vals.truncate(n_freqs + 1); // We only keep the half of the spectrum that
                              // follows the Shannon-Nyquist criterion
  vals
}
