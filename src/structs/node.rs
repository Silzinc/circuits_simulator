use super::{
	circuit::Circuit,
	component::{Component, ComponentContent},
};
use num::Complex;
use std::collections::HashMap;

// TODO: A linked list would be a better fit
/// A node is a point in a circuit where two or more circuit components meet. It
/// is represented by a vector of bytes that gives the path to find it in the
/// node tree.
pub(crate) type Id = Vec<u8>;

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
	/// Returns an `Option` containing a reference to a `Component` based on its
	/// ID.
	///
	/// # Arguments
	///
	/// * `id` - A slice of bytes representing the ID of the `Component`.
	///
	/// # Panics
	///
	/// Panics if the ID is empty.
	///
	/// # Returns
	///
	/// An `Option` containing a reference to a `Component` if it exists,
	/// otherwise `None`.
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
	pub fn get_comp_by_id(&mut self, id: Id) -> Option<&Component> { self.get_comp_by_slice(id.as_slice()) }

	/// Sets the IDs of the `Component` and its children.
	///
	/// # Arguments
	///
	/// * `id` - An `Id` representing the ID of the `Component`.
	fn set_ids(&mut self, id: &Id)
	{
		use ComponentContent::*;
		self.fore_node = id.clone();
		match &mut self.content {
			Series(components) | Parallel(components) => {
				let mut next_id = Id::with_capacity(self.fore_node.len() + 1);
				next_id.extend_from_slice(&self.fore_node);
				next_id.push(0u8);
				for component in components.iter_mut() {
					component.set_ids(&next_id);
					next_id[self.fore_node.len()] += 1u8;
				}
			},
			_ => (),
		}
	}

	/// Sets up the nodes of the `Component` and its children. In particular, the
	/// `nodes` HashMap is filled with the nodes of the circuit.
	///
	/// # Arguments
	///
	/// * `nodes` - A mutable reference to a `HashMap` containing the nodes of the
	///   circuit.
	fn setup_nodes(&self, nodes: &mut HashMap<Id, Node>)
	{
		let id = &self.fore_node;
		let mut node = Node::new();
		node.id = id.clone();
		nodes.insert(id.clone(), node);
		match &self.content {
			ComponentContent::Series(components) | ComponentContent::Parallel(components) => {
				let mut next_id = Id::with_capacity(id.len() + 1);
				next_id.extend_from_slice(id);
				next_id.push(0u8);
				for component in components.iter() {
					component.setup_nodes(nodes);
					next_id[id.len()] += 1u8;
				}
			},
			_ => (),
		}
	}
}

/// Implementation of the `Circuit` struct.
impl Circuit
{
	/// Sets up the IDs of the `Circuit` and its components.
	pub fn setup_ids(&mut self) { self.content.set_ids(&vec![0u8]); }

	/// Sets up the nodes IDs of the `Circuit` and its components.
	pub fn setup_nodes(&mut self)
	{
		self.setup_ids();
		self.content.setup_nodes(&mut self.nodes);
	}
}
