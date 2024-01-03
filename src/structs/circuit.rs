use super::{Component, Id, Node, Source};
use crate::{
	error::{short_circuit_current, Result},
	util::is_multiple_of_x,
};
use num::complex::Complex;
use num_traits::Zero;
use std::collections::HashMap;

/// The initialisation state of a circuit.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Copy)]
pub enum CircuitInitState
{
	/// The circuit is not initialised.
	None         = 0,
	/// The nodes of the circuit are initialised, though they have no current or
	/// tension set. The circuit is built and the impedances are calculated.
	CircuitNodes = 1,
	/// The source of the circuit is initialised. The circuit is ready to be
	/// emulated.
	Source       = 2,
}

#[derive(Clone, Debug)]
/// Represents an electronic circuit.
pub struct Circuit
{
	/// The initialisation state of the circuit.
	pub init_state: CircuitInitState,
	/// The source component of the circuit.
	pub source:     Source,
	/// The main component of the circuit.
	pub content:    Component,
	/// A HashMap that is used to access a Node's voltage and current once the
	/// simulation has started. This won't be used at all during the setup and
	/// shall be initialized when the simulation starts.
	pub nodes:      HashMap<Id, Node>,
}

impl Circuit
{
	#[inline]
	pub fn new() -> Self
	{
		Self { init_state: CircuitInitState::default(),
		       source:     Source::new(),
		       content:    Component::default(),
		       nodes:      HashMap::new(), }
	}

	/// Initializes the circuit by setting up the nodes and calculating the
	/// initial current and tension for each voltage source. Assumes that the
	/// circuit tree is already constructed.
	///
	/// # Errors
	///
	/// Returns an error if there is a short circuit in the circuit. A short
	/// circuit happens when either the tension or the current is infinite in the
	/// circuit. This happens if a capacitor is connected to a constant current
	/// source or if an inductor is connected to a constant tension source. Note
	/// that a capacitor receiving a constant tension is not considered as a short
	/// circuit here : the current will simply be zero. Same for the tension if an
	/// inductor receives a constant current.
	///
	/// # Returns
	///
	/// Returns `Ok(())` if the circuit was successfully initialized.
	pub fn init(&mut self) -> Result<()>
	{
		if self.init_state == CircuitInitState::Source {
			return Ok(());
		}
		self.init_nodes();
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
		self.init_state = CircuitInitState::Source;
		Ok(())
	}

	// To be called when the circuit is changed
	#[inline]
	pub fn uninit_all(&mut self)
	{
		self.init_state = CircuitInitState::None;
		self.nodes.clear();
		self.content.uninit_all();
	}

	// To be called when the source is changed
	#[inline]
	pub fn uninit_source(&mut self)
	{
		self.init_state = self.init_state.min(CircuitInitState::CircuitNodes);
		for node in self.nodes.values_mut() {
			node.next_comp_tensions.clear();
			node.currents.clear();
			node.potentials.clear();
		}
		self.content.uninit_current_tension_potential();
	}
}
