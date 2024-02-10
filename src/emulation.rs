use crate::{
  error::{short_circuit_current, Result},
  Circuit, Id,
};
use num::Complex;
use num_traits::Zero;

/// Return type of the emulation functions.
/// - The first vector contains the current values of the node.
/// - The second vector contains the voltage values of the node.
/// - The third vector contains the potential values of the node.
type EmulationData = (Vec<f64>, Vec<f64>, Vec<f64>);

impl Circuit
{
  /// This function is used to emulate a circuit and returns the currents and
  /// voltages of the node as well as the tensions on the following component
  ///
  /// # Arguments
  ///
  /// * `duration` - The duration of the emulation in seconds
  /// * `step` - The time step used for the emulation in seconds
  /// * `node_id` - The ID of the node to emulate
  ///
  /// # Returns
  ///
  /// A tuple containing the vectors of currents, tensions, and potentials of
  /// the node
  ///
  /// # Errors
  ///
  /// Returns an error if the initialization of the circuit fails
  pub fn emulate_one(&mut self, duration: f64, step: f64, node_id: &Id) -> Result<EmulationData>
  {
    self.init()?;

    let node = self
      .nodes
      .get_mut(node_id)
      .unwrap_or_else(|| panic!("Node of id {node_id:?} not found :/"));
    let initial_currents = &node.currents;
    let initial_tensions = &node.next_comp_tensions;
    let initial_potentials = &node.potentials;

    let nb_iter = (duration / step).ceil() as usize;
    let mut currents = Vec::with_capacity(nb_iter);
    let mut tensions = Vec::with_capacity(nb_iter);
    let mut potentials = Vec::with_capacity(nb_iter);
    let mut elapsed = 0f64;

    while elapsed < duration {
      let mut current = initial_currents[0].re;
      let mut tension = initial_tensions[0].re;
      let mut potential = initial_potentials[0].re;
      for (k, (pulse, voltage)) in self.source.voltages.iter().enumerate() {
        if voltage.is_zero() || pulse.is_zero() {
          continue;
        }
        if pulse.is_zero() {
          return short_circuit_current(&vec![0u8], voltage, &self.content.impedance);
        }
        let factor = Complex::new(0f64, elapsed * *pulse).exp();
        // This way we know we can approximate a real function such as current or
        // tension if we only use positive pulses
        current += 2f64 * (initial_currents[k] * factor).re;
        tension += 2f64 * (initial_tensions[k] * factor).re;
        potential += 2f64 * (initial_potentials[k] * factor).re;
      }
      currents.push(current);
      tensions.push(tension);
      potentials.push(potential);
      elapsed += step;
    }
    Ok((currents, tensions, potentials))
  }

  /// Emulates the circuit for multiple nodes for a given duration and step
  /// size.
  ///
  /// # Arguments
  ///
  /// * `duration` - The duration of the emulation in seconds.
  /// * `step` - The step size of the emulation in seconds.
  /// * `node_ids` - A vector of node IDs to emulate.
  ///
  /// # Returns
  ///
  /// A `Result` containing a vector of tuples, where each tuple contains three
  /// vectors: the vectors of currents, tensions, and potentials of
  /// the node.
  ///
  /// # Errors
  ///
  /// Returns an error if the initialization of the circuit fails or if the
  /// emulation of a node fails.
  #[inline]
  pub fn emulate_many(&mut self, duration: f64, step: f64, node_ids: &Vec<Id>) -> Result<Vec<EmulationData>>
  {
    self.init()?;
    let mut results = Vec::with_capacity(node_ids.len());
    for node_id in node_ids {
      results.push(self.emulate_one(duration, step, node_id)?);
    }
    Ok(results)
  }
}
