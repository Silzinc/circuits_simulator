use super::{Circuit, CircuitInitState, Component, ComponentContent};
use num::Complex;
use std::collections::HashMap;

// IMPROVEMENT: A linked list would probably be a better fit
/// A node is a point in a circuit where two or more circuit components meet. It
/// is represented by a vector of bytes that gives the path to find it in the
/// node tree.
pub type Id = Vec<u8>;

/// A node in a circuit.
#[derive(Clone, Debug)]
pub struct Node
{
	/// The ID of the node.
	pub id: Id,

	/// The tensions of the next components connected to the node for each pulse.
	pub next_comp_tensions: Vec<Complex<f64>>,

	/// The potentials of the node for each pulse.
	pub potentials: Vec<Complex<f64>>,

	/// The currents flowing through the node for each pulse.
	pub currents: Vec<Complex<f64>>,
}

impl Node
{
	#[inline]
	pub fn new() -> Self
	{
		Node { id:                 Id::new(),
		       next_comp_tensions: Vec::new(),
		       potentials:         Vec::new(),
		       currents:           Vec::new(), }
	}
}

/// Implementation of the `Component` trait for the `Node` struct.
impl Component
{
	/// Same as `get_comp_by_id` but the input is a slice of bytes instead of an
	/// `Id`.
	fn get_comp_by_slice(&self, id: &[u8]) -> Option<&Component>
	{
		if id.len() == 0 {
			panic!("The id is empty")
		} else {
			match &self.content {
				ComponentContent::Series(components) | ComponentContent::Parallel(components) => {
					let index = id[0] as usize;
					if index < components.len() {
						if id.len() == 1 {
							Some(&components[index])
						} else {
							components[index].get_comp_by_slice(&id[1..])
						}
					} else {
						None
					}
				},
				_ => None,
			}
		}
	}

	/// Same as `get_comp_by_id_mut` but the input is a slice of bytes instead of
	/// an `Id`.
	fn get_comp_by_slice_mut(&mut self, id: &[u8]) -> Option<&mut Component>
	{
		if id.len() == 0 {
			panic!("The id is empty")
		} else {
			match &mut self.content {
				ComponentContent::Series(components) | ComponentContent::Parallel(components) => {
					let index = id[0] as usize;
					if index < components.len() {
						if id.len() == 1 {
							Some(&mut components[index])
						} else {
							components[index].get_comp_by_slice_mut(&id[1..])
						}
					} else {
						None
					}
				},
				_ => None,
			}
		}
	}

	/// Returns an `Option` containing a reference to a `Component` based on its
	/// ID.
	///
	/// # Arguments
	///
	/// * `id` - An `Id` representing the ID of the `Component`.
	///
	/// # Returns
	///
	/// An `Option` containing a reference to a `Component` if it exists,
	/// otherwise `None`.
	#[inline]
	pub fn get_comp_by_id(&mut self, id: Id) -> Option<&Component> { self.get_comp_by_slice(id.as_slice()) }

	/// Returns an `Option` containing a mutable reference to a `Component` based
	/// on its ID.
	///
	/// # Arguments
	///
	/// * `id` - An `Id` representing the ID of the `Component`.
	///
	/// # Returns
	///
	/// An `Option` containing a reference to a `Component` if it exists,
	/// otherwise `None`.
	#[inline]
	pub fn get_comp_by_id_mut(&mut self, id: Id) -> Option<&mut Component> { self.get_comp_by_slice_mut(id.as_slice()) }

	/// Sets the IDs of the `Component` and its children.
	///
	/// # Arguments
	///
	/// * `id` - An `Id` representing the ID of the `Component`.
	fn set_id(&mut self, id: &Id)
	{
		use ComponentContent::*;
		self.fore_node = id.clone();
		match &mut self.content {
			Series(components) | Parallel(components) => {
				let mut next_id = Id::new();
				next_id.reserve_exact(self.fore_node.len() + 1);
				next_id.extend_from_slice(&self.fore_node);
				next_id.push(0u8);
				for component in components.iter_mut() {
					component.set_id(&next_id);
					next_id[self.fore_node.len()] += 1u8;
				}
			},
			_ => (),
		}
	}

	/// Sets up the nodes of the `Component` and its children. In particular, the
	/// `nodes` HashMap is filled with the nodes of the circuit. It is assumed
	/// that the IDs of the `Component` and its children are already set.
	///
	/// # Arguments
	///
	/// * `nodes` - A mutable reference to a `HashMap` containing the nodes of the
	///   circuit.
	fn init_nodes(&self, nodes: &mut HashMap<Id, Node>)
	{
		use ComponentContent::*;
		let id = &self.fore_node;
		let mut node = Node::new();
		node.id = id.clone();
		nodes.insert(id.clone(), node);
		match &self.content {
			Series(components) | Parallel(components) =>
				for component in components.iter() {
					component.init_nodes(nodes);
				},
			_ => (),
		}
	}
}

/// Implementation of the `Circuit` struct.
impl Circuit
{
	/// Sets up the nodes IDs of the `Circuit` and its components.
	#[inline]
	pub fn init_nodes(&mut self)
	{
		if self.init_state > CircuitInitState::CircuitNodes {
			return;
		}
		self.content.set_id(&vec![0u8]);
		self.content.init_nodes(&mut self.nodes);
		self.init_state = CircuitInitState::CircuitNodes;
	}
}
