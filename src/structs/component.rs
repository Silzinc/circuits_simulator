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
	util::{
		evaluate_zero_without_invx, evaluate_zero_without_x, is_multiple_of_invx, is_multiple_of_x,
		strong_link, weak_link, StrongLink, WeakLink,
	},
};
use fractios::{
	traits::{RatioFracComplexFloat, RatioFracFloat},
	RatioFrac,
};
use num::complex::Complex;
use num_traits::Zero;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub enum ComponentContent<T>
{
	Parallel(Vec<StrongLink<Component<T>>>),
	Series(Vec<StrongLink<Component<T>>>),
	Simple(Dipole<T>),
	Poisoned, // Used as a default state
}

#[derive(Clone, Debug)]
pub struct Component<T>
{
	pub content:   ComponentContent<T>,
	pub impedance: RatioFrac<Complex<T>>,
	pub parent:    WeakLink<Component<T>>,
	pub fore_node: Id,
}

impl<T: RatioFracFloat> Component<T> where Complex<T>: RatioFracComplexFloat
{
	pub fn new() -> Self { Component::default() }

	pub fn impedance<Rhs: RatioFracFloat>(&self, pulse: Rhs) -> Complex<T>
	{
		self.impedance.eval(Complex::from(T::from(pulse).unwrap()))
	}
}

impl<T: RatioFracFloat> TryFrom<ComponentContent<T>> for Component<T>
	where Complex<T>: RatioFracComplexFloat
{
	type Error = Error;

	fn try_from(content: ComponentContent<T>) -> Result<Self>
	{
		use ComponentContent::*;
		let impedance: RatioFrac<Complex<T>> =
			match &content {
				Series(components) => {
					let mut impedance = RatioFrac::default();
					for component in components.iter() {
						impedance += &component.borrow().impedance;
					}
					impedance.reduce();
					impedance
				},
				Parallel(components) => {
					let mut impedance = RatioFrac::default();
					for component in components.iter() {
						component.borrow_mut().impedance.inv_inplace();
						impedance += &component.borrow().impedance;
						component.borrow_mut().impedance.inv_inplace();
					}
					impedance.inv_inplace();
					impedance.reduce();
					impedance
				},
				Simple(dipole) => RatioFrac::from(dipole.impedance()?),
				Poisoned => return Err(CircuitBuildError(
					"Cannot create a component from poisoned content".to_string(),
				)),
			};
		Ok(Component { content,
		               impedance,
		               parent: std::rc::Weak::default(),
		               fore_node: Id::default() })
	}
}

impl<T: RatioFracFloat> TryFrom<Dipole<T>> for Component<T> where Complex<T>: RatioFracComplexFloat
{
	type Error = Error;

	fn try_from(content: Dipole<T>) -> Result<Self>
	{
		Self::try_from(ComponentContent::Simple(content))
	}
}

impl<T: RatioFracFloat> Component<T>
	where Complex<T>: RatioFracComplexFloat /* Here is an implementation to setup a circuit without
	                                         * taking care of voltages and currents. All of these
	                                         * will be setup afterwards */
{
	// This does not care about Ids
	// Pushes a component onto self in serie and updates impedance
	pub fn push_serie(self_: &StrongLink<Self>, mut component: Component<T>)
	{
		use ComponentContent::*;
		let true_self = &mut *self_.borrow_mut();
		match true_self.content {
			Poisoned => {
				true_self.content = component.content;
				true_self.impedance = component.impedance;
			},
			Series(ref mut components) => {
				true_self.impedance += &component.impedance;
				true_self.impedance.reduce();
				component.parent = weak_link(&self_);
				components.push(strong_link(component));
			},
			_ => {
				let old_parent = std::mem::take(&mut true_self.parent);
				let mut new_impedance = &true_self.impedance + &component.impedance;
				new_impedance.reduce();

				true_self.parent = weak_link(&self_); // this true_self will be taken one level deeper and its parent points to self_
				component.parent = weak_link(&self_);

				true_self.content = Series(vec![
					strong_link(std::mem::take(true_self)),
					strong_link(component),
				]);

				true_self.impedance = new_impedance;
				true_self.parent = old_parent;
			},
		};
	}

	// Pushes a component onto self in parallel and updates impedance
	pub fn push_parallel(self_: &StrongLink<Self>, mut component: Component<T>)
	{
		use ComponentContent::*;
		let true_self = &mut *self_.borrow_mut();
		match true_self.content {
			Poisoned => {
				true_self.content = component.content;
				true_self.impedance = component.impedance;
			},
			Parallel(ref mut components) => {
				// This is a bit tricky, but it should make the computation faster because only
				// 1 ratiofrac is created instead of 2
				true_self.impedance.inv_inplace();
				component.impedance.inv_inplace();
				true_self.impedance += &component.impedance;
				component.impedance.inv_inplace();
				true_self.impedance.inv_inplace();
				true_self.impedance.reduce();

				component.parent = weak_link(&self_);
				components.push(strong_link(component));
			},
			_ => {
				let old_parent = std::mem::take(&mut true_self.parent);
				// Same as above. Only 1 ratiofrac is created
				true_self.impedance.inv_inplace();
				component.impedance.inv_inplace();
				let mut new_impedance = &true_self.impedance + &component.impedance;
				component.impedance.inv_inplace();
				true_self.impedance.inv_inplace();
				new_impedance.inv_inplace();
				new_impedance.reduce();

				true_self.parent = weak_link(&self_); // this true_self will be taken one level deeper and its parent points to self_
				component.parent = weak_link(&self_);

				true_self.content = Parallel(vec![
					strong_link(std::mem::take(true_self)),
					strong_link(component),
				]);

				true_self.impedance = new_impedance;
				true_self.parent = old_parent;
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

impl<T: RatioFracFloat> Component<T> where Complex<T>: RatioFracComplexFloat
{
	// Function to be called once when tensions and currents are not yet set.
	// This function, given a certain frequency, will
	// infer the tensions and currents of each component and store it in the
	// assiociated	nodes. This function only serves to initialize the circuit as the
	// next values can be inferred from the initial ones and the frequency.
	pub fn init_current_tension(&mut self,
	                            current: Complex<T>,
	                            tension: Complex<T>,
	                            pulse: T,
	                            nodes: &mut HashMap<Id, Node<T>>)
	                            -> Result<()>
	{
		let node = nodes.get_mut(self.fore_node.as_slice()).unwrap();
		node.currents.push(current);
		node.next_comp_tensions.push(tension);

		use ComponentContent::*;
		match &mut self.content {
			Series(components) =>
				for component in components.iter_mut() {
					let mut component = component.borrow_mut();
					if !pulse.is_zero() || !is_multiple_of_invx(&component.impedance) {
						let next_actual_impedance = component.impedance.eval(Complex::from(pulse));
						component.init_current_tension(current, current * next_actual_impedance, pulse, nodes)?;
					} else if current.is_zero() {
						/* We suppose a zero current is always due to a zero admittance
						Otherwise, the emulation for this pulse would not have started or have
						panicked */
						// assert!(is_multiple_of_invx(&self.impedance));
						let tension_factor = evaluate_zero_without_invx(&component.impedance)
						                     / evaluate_zero_without_invx(&self.impedance);
						// We factor by the "impedance ratio"
						component.init_current_tension(current, tension * tension_factor, pulse, nodes)?;
					} else {
						return short_circuit_current(&component.fore_node, current, &component.impedance);
					}
				},
			Parallel(components) =>
				for component in components.iter_mut() {
					let mut component = component.borrow_mut();
					if !pulse.is_zero() || !is_multiple_of_x(&component.impedance) {
						// Better to do inv_inplace instead of calling .inv() on the impendance because
						// NaN.inv() = NaN and not 0, which can lead to false short-circuit detection
						component.impedance.inv_inplace();
						let next_actual_admittance = component.impedance.eval(Complex::from(pulse));
						component.impedance.inv_inplace();

						component.init_current_tension(
						                               tension * next_actual_admittance,
						                               tension,
						                               pulse,
						                               nodes,
						)?;
					} else if tension.is_zero() {
						/* We suppose a zero tension is always due to a zero impedance
						Otherwise, the emulation for this pulse would not have started or have
						panicked */
						// assert!(is_multiple_of_x(&self.impedance));
						let current_factor = evaluate_zero_without_x(&self.impedance)
						                     / evaluate_zero_without_x(&component.impedance);
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

	// Function to be called once tensions and currents are set.
	// This function, given that the circuit is set at a certain frequency, will
	// infer the potentials on each node and store them in the `potentials` field.
	// Like with init_current_tension, this function only serves to initialize the
	// circuit as the next values can be inferred from the initial ones and the
	// frequency.
	pub fn init_potentials(&mut self, fore_potential: Complex<T>, nodes: &mut HashMap<Id, Node<T>>)
	{
		nodes.get_mut(&self.fore_node)
		     .expect("Node not found :/")
		     .potentials
		     .push(fore_potential);
		use ComponentContent::*;
		match &mut self.content {
			Parallel(components) =>
				for component in components {
					component.borrow_mut()
					         .init_potentials(fore_potential, nodes);
				},
			Series(components) => {
				let mut remaining = fore_potential;
				for component in components {
					let mut component = component.borrow_mut();
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
