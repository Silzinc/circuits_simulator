use super::{
	component::Component,
	node::{Id, Node},
	source::Source,
};
use crate::{
	error::{short_circuit_current, Result},
	util::{is_multiple_of_x, strong_link, StrongLink},
};
use fractios::traits::{RatioFracComplexFloat, RatioFracFloat};
use num::complex::Complex;
use num_traits::Zero;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Circuit<T: RatioFracFloat>
	where Complex<T>: RatioFracComplexFloat
{
	pub source:  Source<T>,
	pub content: StrongLink<Component<T>>,
	// This HashMap is only used to access a Node's voltage and current once the simulation has
	// started This won't be used at all during the setup and shall be initialized when the
	// simulation starts
	pub nodes:   HashMap<Id, Node<T>>,
}

impl<T: RatioFracFloat> Circuit<T> where Complex<T>: RatioFracComplexFloat
{
	pub fn new() -> Self
	{
		Self { source:  Source::new(),
		       content: strong_link(Component::default()),
		       nodes:   HashMap::new(), }
	}
}

impl<T: RatioFracFloat> Circuit<T> where Complex<T>: RatioFracComplexFloat
{
	// The following function only assumes the circuit tree is constructed
	pub fn init(&mut self) -> Result<()>
	{
		self.setup_nodes();
		let mut content = self.content.borrow_mut();
		for (pulse, voltage) in self.source.voltages.iter() {
			if voltage.is_zero() {
				continue;
			}
			if pulse.is_zero() && is_multiple_of_x(&content.impedance) {
				return short_circuit_current(&vec![0u8], voltage, &content.impedance);
			}
			let initial_tension = Complex::from(*voltage);
			content.impedance.inv_inplace();
			let initial_current = initial_tension * content.impedance.eval(Complex::from(*pulse));
			content.impedance.inv_inplace();
			content.init_current_tension(initial_current, initial_tension, *pulse, &mut self.nodes)?;
			content.init_potentials(initial_tension, &mut self.nodes);
		}
		Ok(())
	}
}
