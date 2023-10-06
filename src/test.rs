fn test_() -> crate::error::Result<()>
{
	use crate::structs::{
		circuit::Circuit,
		component::Component,
		dipole::Dipole::{Capacitor, Resistor},
		source::Source,
	};
	use std::{fs::File, io::prelude::Write, path::Path, process::Command};

	fn door(x: f64) -> f64
	{
		if ((x + 2.) % 4.) > 2. {
			-1.
		} else {
			1.
		}
	}

	let n_freqs = 1000;
	let duration = 10.;

	// Create the serial RC circuit
	let mut c = Circuit::new();
	c.source = Source::from_fn(door, duration, n_freqs);
	c.content.push_serie(Component::try_from(Resistor(1.))?);
	c.content.push_serie(Component::try_from(Capacitor(0.25))?);

	// Simulate the circuit
	let step = 0.02;
	let (currents, tensions, _) = c.emulate_one(duration, step, &vec![0u8])?;

	// Save the results to a file
	let out_dir = Path::new(r"/home/jonasbloch/Projets/circuits_rust/out");
	let mut currents_file =
		File::create(out_dir.join("currents.txt")).expect("Failed to create currents text file");
	let mut tensions_file =
		File::create(out_dir.join("tensions.txt")).expect("Failed to create tensions text file");
	currents_file.write_all(format!("{:?}", currents).as_bytes())
	             .expect("Failed to write currents");
	tensions_file.write_all(format!("{:?}", tensions).as_bytes())
	             .expect("Failed to write tensions");

	// Plot the results using Python
	Command::new("python3").arg(r"/home/jonasbloch/Projets/circuits_rust/out/plot.py")
	                       .arg(format!("{duration}"))
	                       .spawn()
	                       .expect("Failed to plot the results");
	Ok(())
}

#[test]
fn test()
{
	if let Err(e) = test_() {
		eprintln!("{}", e);
	}
}
