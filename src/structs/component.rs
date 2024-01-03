use super::{
	dipole::Dipole,
	node::{Id, Node},
};
use crate::{
	error::{short_circuit_current, short_circuit_tension, Error::CircuitBuildError, Result},
	util::{evaluate_zero_without_invx, evaluate_zero_without_x, is_multiple_of_invx, is_multiple_of_x},
};
use fractios::RatioFrac;
use num::complex::Complex;
use num_traits::Zero;
use std::collections::HashMap;

/// Represents the initialisation state of a component.
#[derive(Clone, Debug, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ComponentInitState
{
	/// No initialisation
	None      = 0,
	/// The impedances have been computed
	Impedance = 1,
	/// The initial currents, tensions, and potentials have been computed,
	/// provided a source
	CurrentTensionPotential = 2,
}

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
	pub content:    ComponentContent,
	/// The impedance of the component.
	pub impedance:  RatioFrac<Complex<f64>>,
	/// The ID of the node connected to the component's fore port.
	pub fore_node:  Id,
	pub init_state: ComponentInitState,
}

impl Component
{
	#[inline]
	pub fn new() -> Self { Component::default() }

	/// Returns the impedance of the component for a given pulse.
	#[inline]
	pub fn impedance(&self, pulse: f64) -> Complex<f64> { self.impedance.eval(Complex::from(pulse)) }
}

impl From<ComponentContent> for Component
{
	#[inline]
	fn from(content: ComponentContent) -> Self
	{
		Component { content,
		            impedance: RatioFrac::default(),
		            fore_node: Id::default(),
		            init_state: ComponentInitState::default() }
	}
}

impl From<Dipole> for Component
{
	#[inline]
	fn from(content: Dipole) -> Self { Self::from(ComponentContent::Simple(content)) }
}

impl Component
// Here is an implementation to setup a circuit without taking care of impedances,
// currents, tensions and potentials. All of these will be setup afterwards
{
	/// Pushes a component onto self in series.
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
	/// let mut component1 = Component::from(Resistor(10.0));
	/// let mut component2 = Component::from(Capacitor(0.1));
	///
	/// component1.push_serie(component2);
	/// ```
	#[inline]
	pub fn push_serie(&mut self, component: Component)
	{
		use ComponentContent::*;
		match self.content {
			Poisoned => self.content = component.content,
			Series(ref mut components) => components.push(component),
			_ => self.content = Series(vec![std::mem::take(self), component]),
		};
	}

	/// Pushes a component onto self in parallel.
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
	#[inline]
	pub fn push_parallel(&mut self, component: Component)
	{
		use ComponentContent::*;
		match self.content {
			Poisoned => self.content = component.content,
			Parallel(ref mut components) => components.push(component),
			_ => self.content = Parallel(vec![std::mem::take(self), component]),
		};
	}

	// Swaps two components in a branch
	#[inline]
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
	/// Initializes the impedance of the component.
	///
	/// This method calculates and sets the impedance of the component based on
	/// its content. If the component has already been initialized with a higher
	/// state than `ComponentInitState::Impedance`, this method does nothing and
	/// returns `Ok(())`.
	///
	/// # Errors
	///
	/// Returns an error of type `CircuitBuildError` if the component is in a
	/// `Poisoned` state, case in which no initialisation is possible.
	///
	/// # Examples
	///
	/// ```
	/// use circuits_simulator::structs::{
	/// 	component::Component,
	/// 	dipole::Dipole::{Capacitor, Resistor},
	/// };
	///
	/// let mut component1 = Component::from(Resistor(10.0));
	/// let mut component2 = Component::from(Capacitor(0.1));
	///
	/// component1.push_serie(component2);
	/// let result = component1.init_impedance();
	///
	/// assert!(result.is_ok());
	/// ```
	pub fn init_impedance(&mut self) -> Result<()>
	{
		if self.init_state > ComponentInitState::Impedance {
			return Ok(());
		}
		use ComponentContent::*;
		match &mut self.content {
			Series(components) => {
				let mut impedance = RatioFrac::default();
				for component in components.iter_mut() {
					component.init_impedance()?;
					impedance += &component.impedance;
				}
				impedance.reduce();
				self.impedance = impedance;
			},
			Parallel(components) => {
				let mut impedance = RatioFrac::default();
				for component in components.iter_mut() {
					component.init_impedance()?;
					// This is a bit tricky, but it should make the computation faster because
					// only one additional ratiofrac is created, instead of 2 without inv_inplace
					component.impedance.inv_inplace();
					impedance += &component.impedance;
					component.impedance.inv_inplace();
				}
				impedance.inv_inplace();
				impedance.reduce();
				self.impedance = impedance;
			},
			Simple(dipole) => self.impedance = dipole.impedance()?,
			Poisoned => return Err(CircuitBuildError("Cannot initialize impedance of poisoned component".to_string())),
		};
		self.init_state = ComponentInitState::Impedance;
		Ok(())
	}

	/// Initializes the current, tension, and potential for a component. Requires
	/// the `nodes` HashMap to be initialized.
	///
	/// # Arguments
	///
	/// * `current` - The current value for the component.
	/// * `tension` - The tension value for the component.
	/// * `fore_potential` - The potential value for the component's fore node.
	/// * `pulse` - The pulse value for the component.
	/// * `nodes` - A mutable reference to the HashMap of nodes.
	///
	/// # Errors
	///
	/// Returns an error if the component's initialization state is not
	/// appropriate.
	pub fn init_current_tension_potential(&mut self,
	                                      current: Complex<f64>,
	                                      tension: Complex<f64>,
	                                      fore_potential: Complex<f64>,
	                                      pulse: f64,
	                                      nodes: &mut HashMap<Id, Node>)
	                                      -> Result<()>
	{
		if self.init_state > ComponentInitState::CurrentTensionPotential {
			return Ok(());
		} else if self.init_state < ComponentInitState::Impedance {
			return Err(CircuitBuildError("Cannot initialize currents and tensions before the impedance".to_string()));
		}

		let node = nodes.get_mut(self.fore_node.as_slice()).expect("Node not found :/");
		node.currents.push(current);
		node.next_comp_tensions.push(tension);
		node.potentials.push(fore_potential);

		use ComponentContent::*;
		match &mut self.content {
			Series(components) => {
				let mut remaining_potential = fore_potential;
				for component in components.iter_mut() {
					if !pulse.is_zero() || !is_multiple_of_invx(&component.impedance) {
						let next_tension = current * component.impedance.eval(Complex::from(pulse));
						component.init_current_tension_potential(current, next_tension, remaining_potential, pulse, nodes)?;
						remaining_potential -= next_tension;
					} else if current.is_zero() {
						/* We suppose a zero current is always due to a zero admittance
						Otherwise, the emulation for this pulse would not have started or have
						panicked */
						assert!(is_multiple_of_invx(&self.impedance));
						// We factor by the "impedance ratio"
						let next_tension = tension * evaluate_zero_without_invx(&component.impedance) / evaluate_zero_without_invx(&self.impedance);
						component.init_current_tension_potential(current, next_tension, remaining_potential, pulse, nodes)?;
						remaining_potential -= next_tension;
					} else {
						return short_circuit_current(&component.fore_node, current, &component.impedance);
					}
				}
			},
			Parallel(components) =>
				for component in components.iter_mut() {
					if !pulse.is_zero() || !is_multiple_of_x(&component.impedance) {
						// Better to do inv_inplace instead of calling .inv() on the impendance because
						// NaN.inv() = NaN and not 0, which can lead to false short-circuit detection
						component.impedance.inv_inplace();
						let evaluated_admittance = component.impedance.eval(Complex::from(pulse));
						component.impedance.inv_inplace();

						component.init_current_tension_potential(tension * evaluated_admittance, tension, fore_potential, pulse, nodes)?;
					} else if tension.is_zero() {
						/* We suppose a zero tension is always due to a zero impedance
						Otherwise, the emulation for this pulse would not have started or have
						panicked */
						assert!(is_multiple_of_x(&self.impedance));
						let current_factor = evaluate_zero_without_x(&self.impedance) / evaluate_zero_without_x(&component.impedance);
						// We factor by the "admittance ratio"
						component.init_current_tension_potential(current * current_factor, tension, fore_potential, pulse, nodes)?;
					} else {
						return short_circuit_tension(&component.fore_node, tension, &component.impedance);
					}
				},
			_ => (),
		};
		self.init_state = ComponentInitState::CurrentTensionPotential;
		Ok(())
	}

	#[inline]
	pub fn uninit_current_tension_potential(&mut self) { self.init_state = self.init_state.min(ComponentInitState::Impedance) }

	#[inline]
	pub fn uninit_all(&mut self) { self.init_state = ComponentInitState::None }
}
