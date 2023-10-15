use super::{
	dipole::Dipole,
	node::{Id, Node},
};
use crate::{
	error::{
		short_circuit_current, short_circuit_tension,
		Error::{self, CircuitBuildError},
		Result,
	},
	util::{evaluate_zero_without_invx, evaluate_zero_without_x, is_multiple_of_invx, is_multiple_of_x},
};
use fractios::RatioFrac;
use num::complex::Complex;
use num_traits::Zero;
use std::collections::HashMap;

/// Represents the content of a circuit component, which can be either a
/// parallel or series combination of other components, a simple dipole, or a
/// poisoned state used as a default.
#[derive(Clone, Debug)]
pub enum ComponentContent
{
	Parallel(Vec<Component>),
	Series(Vec<Component>),
	Simple(Dipole),
	/// Used as a default state
	Poisoned,
}

/// A struct representing a circuit component.
#[derive(Clone, Debug)]
pub struct Component
{
	/// The content of the component.
	pub content:   ComponentContent,
	/// The impedance of the component.
	pub impedance: RatioFrac<Complex<f64>>,
	/// The ID of the node connected to the component's fore port.
	pub fore_node: Id,
}

impl Component
{
	pub fn new() -> Self { Component::default() }

	/// Returns the impedance of the component for a given pulse.
	pub fn impedance(&self, pulse: f64) -> Complex<f64> { self.impedance.eval(Complex::from(pulse)) }
}

impl TryFrom<ComponentContent> for Component
{
	type Error = Error;

	fn try_from(mut content: ComponentContent) -> Result<Self>
	{
		use ComponentContent::*;
		let impedance: RatioFrac<Complex<f64>> = match &mut content {
			Series(components) => {
				let mut impedance = RatioFrac::default();
				for component in components.iter() {
					impedance += &component.impedance;
				}
				impedance.reduce();
				impedance
			},
			Parallel(components) => {
				let mut impedance = RatioFrac::default();
				for component in components.iter_mut() {
					component.impedance.inv_inplace();
					impedance += &component.impedance;
					component.impedance.inv_inplace();
				}
				impedance.inv_inplace();
				impedance.reduce();
				impedance
			},
			Simple(dipole) => RatioFrac::from(dipole.impedance()?),
			Poisoned => return Err(CircuitBuildError("Cannot create a component from poisoned content".to_string())),
		};
		Ok(Component { content,
		               impedance,
		               fore_node: Id::default() })
	}
}

impl TryFrom<Dipole> for Component
{
	type Error = Error;

	fn try_from(content: Dipole) -> Result<Self> { Self::try_from(ComponentContent::Simple(content)) }
}

impl Component
/* Here is an implementation to setup a circuit without
 * taking care of voltages and currents. All of these
 * will be setup afterwards */
{
	/// Pushes a component onto self in series and updates impedance.
	///
	/// # Arguments
	///
	/// * `component` - A `Component` to be pushed onto `self`.
	///
	/// # Remarks
	///
	/// This method does not care about Ids. If `self` is `Poisoned`, it will be
	/// replaced by `component`. If `self` is `Series`, `component` will be pushed
	/// onto the vector of components and the impedance will be updated. If `self`
	/// is anything else, a new `Series` component will be created with `self` and
	/// `component` as its components.
	///
	/// # Example
	///
	/// ```
	/// use circuits_simulator::structs::{
	/// 	component::Component,
	/// 	dipole::Dipole::{Capacitor, Resistor},
	/// };
	///
	/// let mut component1 = Component::try_from(Resistor(10.0)).unwrap();
	/// let mut component2 = Component::try_from(Capacitor(0.1)).unwrap();
	///
	/// component1.push_serie(component2);
	/// ```
	pub fn push_serie(&mut self, component: Component)
	{
		use ComponentContent::*;
		match self.content {
			Poisoned => {
				self.content = component.content;
				self.impedance = component.impedance;
			},
			Series(ref mut components) => {
				self.impedance += &component.impedance;
				self.impedance.reduce();
				components.push(component);
			},
			_ => {
				let mut new_impedance = &self.impedance + &component.impedance;
				new_impedance.reduce();

				self.content = Series(vec![std::mem::take(self), component]);

				self.impedance = new_impedance;
			},
		};
	}

	/// Pushes a component onto self in parallel and updates impedance
	///
	/// # Arguments
	///
	/// * `component` - A `Component` to be pushed onto self in parallel
	///
	/// This method does not care about Ids. If `self` is `Poisoned`, it will be
	/// replaced by `component`. If `self` is `Parallel`, `component` will be
	/// pushed onto the vector of components and the impedance will be updated. If
	/// `self` is anything else, a new `Parallel` component will be created with
	/// `self` and `component` as its components.
	///
	/// # Example
	///
	/// ```
	/// use circuits_simulator::structs::{
	/// 	component::Component,
	/// 	dipole::Dipole::{Capacitor, Resistor},
	/// };
	///
	/// let mut component1 = Component::try_from(Resistor(10.0)).unwrap();
	/// let mut component2 = Component::try_from(Capacitor(0.1)).unwrap();
	///
	/// component1.push_parallel(component2);
	/// ```
	pub fn push_parallel(&mut self, mut component: Component)
	{
		use ComponentContent::*;
		match self.content {
			Poisoned => {
				self.content = component.content;
				self.impedance = component.impedance;
			},
			Parallel(ref mut components) => {
				// This is a bit tricky, but it should make the computation faster because only
				// 1 ratiofrac is created instead of 2
				self.impedance.inv_inplace();
				component.impedance.inv_inplace();
				self.impedance += &component.impedance;
				component.impedance.inv_inplace();
				self.impedance.inv_inplace();
				self.impedance.reduce();

				components.push(component);
			},
			_ => {
				// Same as above. Only 1 ratiofrac is created
				self.impedance.inv_inplace();
				component.impedance.inv_inplace();
				let mut new_impedance = &self.impedance + &component.impedance;
				component.impedance.inv_inplace();
				self.impedance.inv_inplace();
				new_impedance.inv_inplace();
				new_impedance.reduce();

				self.content = Parallel(vec![std::mem::take(self), component]);

				self.impedance = new_impedance;
			},
		};
	}

	// Swaps two components in a branch

	pub fn swap(&mut self, index1: usize, index2: usize) -> Result<()>
	{
		use ComponentContent::*;
		match &mut self.content {
			Series(components) | Parallel(components) => components.swap(index1, index2),
			_ => return Err(CircuitBuildError("Cannot swap components in a non-branch component".to_string())),
		}
		Ok(())
	}
}

impl Component
{
	/// Function to be called once when tensions and currents are not yet set.
	/// This function, given a certain frequency, will
	/// infer the tensions and currents of each component and store it in the
	/// associated nodes. This function only serves to initialize the circuit as
	/// the next values can be inferred from the initial ones and the frequency.
	///
	/// # Arguments
	///
	/// * `current` - The current of the component.
	/// * `tension` - The tension of the component.
	/// * `pulse` - The frequency of the component.
	/// * `nodes` - The nodes of the component.
	///
	/// # Returns
	///
	/// Returns a Result containing Ok(()) if the function is successful,
	/// otherwise it returns an error.
	pub fn init_current_tension(&mut self, current: Complex<f64>, tension: Complex<f64>, pulse: f64, nodes: &mut HashMap<Id, Node>) -> Result<()>
	{
		let node = nodes.get_mut(self.fore_node.as_slice()).unwrap();
		node.currents.push(current);
		node.next_comp_tensions.push(tension);

		use ComponentContent::*;
		match &mut self.content {
			Series(components) =>
				for component in components.iter_mut() {
					if !pulse.is_zero() || !is_multiple_of_invx(&component.impedance) {
						let next_actual_impedance = component.impedance.eval(Complex::from(pulse));
						component.init_current_tension(current, current * next_actual_impedance, pulse, nodes)?;
					} else if current.is_zero() {
						/* We suppose a zero current is always due to a zero admittance
						Otherwise, the emulation for this pulse would not have started or have
						panicked */
						// assert!(is_multiple_of_invx(&self.impedance));
						let tension_factor = evaluate_zero_without_invx(&component.impedance) / evaluate_zero_without_invx(&self.impedance);
						// We factor by the "impedance ratio"
						component.init_current_tension(current, tension * tension_factor, pulse, nodes)?;
					} else {
						return short_circuit_current(&component.fore_node, current, &component.impedance);
					}
				},
			Parallel(components) =>
				for component in components.iter_mut() {
					if !pulse.is_zero() || !is_multiple_of_x(&component.impedance) {
						// Better to do inv_inplace instead of calling .inv() on the impendance because
						// NaN.inv() = NaN and not 0, which can lead to false short-circuit detection
						component.impedance.inv_inplace();
						let next_actual_admittance = component.impedance.eval(Complex::from(pulse));
						component.impedance.inv_inplace();

						component.init_current_tension(tension * next_actual_admittance, tension, pulse, nodes)?;
					} else if tension.is_zero() {
						/* We suppose a zero tension is always due to a zero impedance
						Otherwise, the emulation for this pulse would not have started or have
						panicked */
						// assert!(is_multiple_of_x(&self.impedance));
						let current_factor = evaluate_zero_without_x(&self.impedance) / evaluate_zero_without_x(&component.impedance);
						// We factor by the "admittance ratio"
						component.init_current_tension(current * current_factor, tension, pulse, nodes)?;
					} else {
						return short_circuit_tension(&component.fore_node, tension, &component.impedance);
					}
				},
			_ => (),
		};
		Ok(())
	}

	/// Function to be called once tensions and currents are set.
	/// This function, given that the circuit is set at a certain frequency, will
	/// infer the potentials on each node and store them in the `potentials`
	/// field. Like with `init_current_tension`, this function only serves to
	/// initialize the circuit as the next values can be inferred from the initial
	/// ones and the frequency.
	pub fn init_potentials(&mut self, fore_potential: Complex<f64>, nodes: &mut HashMap<Id, Node>)
	{
		nodes.get_mut(&self.fore_node).expect("Node not found :/").potentials.push(fore_potential);
		use ComponentContent::*;
		match &mut self.content {
			Parallel(components) =>
				for component in components {
					component.init_potentials(fore_potential, nodes);
				},
			Series(components) => {
				let mut remaining = fore_potential;
				for component in components {
					component.init_potentials(remaining, nodes);

					remaining -= *nodes.get(&component.fore_node)
					                   .expect("Node not found :/")
					                   .next_comp_tensions
					                   .last()
					                   .expect("The component has no tension set but we need to infer the potential");
					// The last tension is the one that
					// corresponds to the pulse being emulated
				}
			},
			_ => (),
		}
	}
}
