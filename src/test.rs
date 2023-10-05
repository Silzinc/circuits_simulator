#[allow(dead_code)]
pub(crate) fn test() -> crate::error::Result<()>
{
	use crate::structs::{
		circuit::Circuit,
		component::Component,
		dipole::Dipole::{Capacitor, Resistor},
	};
	use std::{fs::File, io::prelude::*};

	let twopi = 2. * std::f64::consts::PI;

	fn door(x: f64) -> f64
	{
		if ((x + 2.) % 4.) > 2. {
			-1.
		} else {
			1.
		}
	}
	fn triangle(x: f64) -> f64 { ((x + 2.) % 4.) - 2. }
	fn weird(x: f64) -> f64 { (x.abs() + 1.).ln() + x.abs().sqrt().sin() - (x / 10.).exp() }
	fn sine(x: f64) -> f64 { (2. * x).sin() + (4. * x).sin() / 2. + (6. * x).sin() / 3. }

	let fundamental = 0.04;
	let n_freqs = 5000;
	let fourier_coefs = crate::fourier::fouriers(sine, fundamental, n_freqs - 1);

	// Create the circuit
	let mut c = Circuit::new();
	c.source.add_pulse(0., fourier_coefs[0]);
	for (i, coef) in fourier_coefs.iter().enumerate().skip(1) {
		c.source.add_pulse(twopi * i as f64 * fundamental, *coef);
		// This line would be required if we did not use the fourier real functions
		// approximation in circuit.rs line 436
		// c.source.add_pulse(-twopi * i as f64 * fundamental, coef.conj());
	}

	c.content.push_serie(Component::try_from(Resistor(1.))?);
	c.content.push_serie(Component::try_from(Capacitor(0.25))?);

	// Simulate the circuit
	let duration = 1. / (2. * fundamental) * 0.99; // Gotta respect the Shannon-Nyquist criterion
	let step = 0.02;
	let (currents, tensions) = c.emulate_one(duration, step, &vec![0u8])?;

	// Save the results to a file
	let out_dir = std::path::Path::new(r"/home/jonasbloch/Projets/circuits_rust/out");
	let mut currents_file =
		File::create(out_dir.join("currents.txt")).expect("Failed to create currents text file");
	let mut tensions_file =
		File::create(out_dir.join("tensions.txt")).expect("Failed to create tensions text file");
	currents_file.write_all(format!("{:?}", currents).as_bytes())
	             .expect("Failed to write currents");
	tensions_file.write_all(format!("{:?}", tensions).as_bytes())
	             .expect("Failed to write tensions");

	// Plot the results using Python
	std::process::Command::new("python3").arg(r"/home/jonasbloch/Projets/circuits_rust/out/plot.py")
	                                     .arg(format!("{duration}"))
	                                     .spawn()
	                                     .expect("Failed to plot the results");
	Ok(())
}
