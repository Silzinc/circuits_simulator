use super::{
	circuit::Circuit,
	component::{Component, ComponentContent},
};
use num::Complex;
use std::collections::HashMap;

// A linked list would be a better fit
pub(crate) type Id = Vec<u8>;

#[derive(Clone, Debug)]
pub struct Node
{
	pub id:                 Id,
	pub next_comp_tensions: Vec<Complex<f64>>,
	pub potentials:         Vec<Complex<f64>>,
	pub currents:           Vec<Complex<f64>>,
}

impl Node
{
	pub fn new() -> Self
	{
		Node { id:                 Id::default(),
		       next_comp_tensions: Vec::new(),
		       potentials:         Vec::new(),
		       currents:           Vec::new(), }
	}
}

impl Component
{
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

	pub fn get_comp_by_id(&mut self, id: Id) -> Option<&Component>
	{
		self.get_comp_by_slice(id.as_slice())
	}

	pub fn set_ids(&mut self, id: &Id)
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

	pub fn setup_nodes(&self, nodes: &mut HashMap<Id, Node>)
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

impl Circuit
{
	pub fn setup_ids(&mut self) { self.content.set_ids(&vec![0u8]); }

	pub fn setup_nodes(&mut self)
	{
		self.setup_ids();
		self.content.setup_nodes(&mut self.nodes);
	}
}
