use super::{Circuit, CircuitInitState, Component, ComponentContent};
use num::Complex;
use std::collections::HashMap;

// IMPROVEMENT: A linked list would probably be a better fit
/// A node is a point in a circuit where two or more circuit components meet. It
/// is represented by a vector of bytes that gives the path to find it in the
/// node tree.
pub type Id = Vec<u8>;

/// A node in a circuit.
#[derive(Clone, Debug, Default)]
pub struct Node
{
  /// The ID of the node.
  pub id: Id,

  /// The tensions of the next components connected to the node for each pulse.
  pub next_component_tensions: Vec<Complex<f64>>,

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
    Node {
      id: Id::new(),
      next_component_tensions: Vec::new(),
      potentials: Vec::new(),
      currents: Vec::new(),
    }
  }
}

impl Component
{
  /// Sets up the nodes of the `Component` and its children. In particular, the
  /// `nodes` HashMap is filled with the nodes of the circuit. It is assumed
  /// that the IDs of the `Component` and its children are already set.
  ///
  /// # Arguments
  ///
  /// * `nodes` - A mutable reference to a `HashMap` containing the nodes of the
  ///   circuit.
  fn init_nodes(&self, nodes: &mut HashMap<Id, Node>) -> &Self
  {
    use ComponentContent::*;
    let id = &self.fore_node_id;
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
    self
  }
}

impl Circuit
{
  /// Sets up the nodes IDs of the `Circuit` and its components.
  #[inline]
  pub fn init_nodes(&mut self) -> &mut Self
  {
    if self.init_state > CircuitInitState::CircuitNodes {
      return self;
    }
    self.content.init_nodes(&mut self.nodes);
    self.init_state = CircuitInitState::CircuitNodes;
    self
  }
}

impl Circuit
{
  /// Retrieves a node from the circuit based on its ID.
  ///
  /// # Arguments
  ///
  /// * `id` - The ID of the node to retrieve.
  ///
  /// # Returns
  ///
  /// An `Option` containing a reference to the node if it exists, otherwise
  /// `None`.
  #[inline]
  pub fn get_node(&self, id: &Id) -> Option<&Node> { self.nodes.get(id) }

  /// Retrieves a mutable node from the circuit based on its ID.
  ///
  /// # Arguments
  ///
  /// * `id` - The ID of the node to retrieve.
  ///
  /// # Returns
  ///
  /// An `Option` containing a mutable reference to the node if it exists,
  /// otherwise `None`.
  #[inline]
  pub fn get_node_mut(&mut self, id: &Id) -> Option<&mut Node> { self.nodes.get_mut(id) }
}
