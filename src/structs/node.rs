use num::Complex;

/// A node is a point in a circuit where two or more circuit components meet. It
/// is represented by a vector of bytes that gives the path to find it in the
/// node tree.
pub type Id = Vec<u8>;

/// A macro to create an ID. This is just a copy of the `vec!` macro.
#[macro_export]
macro_rules! id {
  ($($x:expr),*) => (vec![$($x),*]);
  ($($x:expr);*) => (vec![$($x);*]);
}

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
