use super::{
	component::Component,
	node::{Id, Node},
	source::Source,
};
use crate::{
	error::{short_circuit_current, Result},
	util::is_multiple_of_x,
};
use num::complex::Complex;
use num_traits::Zero;
use std::collections::HashMap;

#[derive(Clone, Debug)]
/// Represents an electronic circuit.
pub struct Circuit
{
	/// Indicates whether the circuit has been initialised or not.
	pub is_init: bool,
	/// The source component of the circuit.
	pub source:  Source,
	/// The main component of the circuit.
	pub content: Component,
	/// A HashMap that is used to access a Node's voltage and current once the
	/// simulation has started. This won't be used at all during the setup and
	/// shall be initialized when the simulation starts.
	pub nodes:   HashMap<Id, Node>,
}

impl Circuit
{
	#[inline]
	pub fn new() -> Self
	{
		Self { is_init: false,
		       source:  Source::new(),
		       content: Component::default(),
		       nodes:   HashMap::new(), }
	}
}

impl Circuit
{
	/// Initializes the circuit by setting up the nodes and calculating the
	/// initial current and tension for each voltage source. Assumes that the
	/// circuit tree is already constructed.
	///
	/// # Errors
	///
	/// Returns an error if there is a short circuit in the circuit.
	///
	/// # Returns
	///
	/// Returns `Ok(())` if the circuit was successfully initialized.
	pub fn init(&mut self) -> Result<()>
	{
		if self.is_init {
			return Ok(());
		}
		self.setup_nodes();
		self.content.init_impedance()?;
		for (pulse, voltage) in self.source.voltages.iter() {
			if voltage.is_zero() {
				continue;
			}
			if pulse.is_zero() && is_multiple_of_x(&self.content.impedance) {
				return short_circuit_current(&vec![0u8], voltage, &self.content.impedance);
			}
			let initial_tension = Complex::from(*voltage);
			self.content.impedance.inv_inplace();
			let initial_current = initial_tension * self.content.impedance.eval(Complex::from(*pulse));
			self.content.impedance.inv_inplace();
			self.content
			    .init_current_tension_potential(initial_current, initial_tension, initial_tension, *pulse, &mut self.nodes)?;
		}
		self.is_init = true;
		Ok(())
	}

	#[inline]
	pub fn uninit(&mut self)
	{
		self.is_init = false;
		self.nodes.clear();
	}
}
