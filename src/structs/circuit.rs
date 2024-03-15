use super::{Component, Id, Node, Source};
use crate::{
  error::{short_circuit_current, Result},
  util::is_multiple_of_x,
};
use fractios::RatioFrac;
use num::{complex::Complex, PrimInt};
use num_traits::Zero;
use std::{collections::HashMap, fmt::Debug};

/// The initialisation state of a circuit.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Default)]
pub enum CircuitInitState {
  /// The circuit is not initialised.
  #[default]
  None = 0,
  /// The nodes of the circuit are initialised, though they have no current or
  /// tension set. The circuit is built and the impedances are calculated.
  CircuitNodes = 1,
  /// The source of the circuit is initialised. The circuit is ready to be
  /// emulated.
  Source = 2,
}

#[derive(Clone, Debug, Default)]
/// Represents an electronic circuit.
pub struct Circuit {
  /// The initialisation state of the circuit.
  pub(super) init_state: CircuitInitState,
  /// The source component of the circuit.
  pub(super) source: Source,
  /// The main component of the circuit.
  pub(super) content: Component, // TODO: make this pub(crate)
  /// A HashMap that is used to access a Node's voltage and current once the
  /// simulation has started. This won't be used at all during the setup and
  /// shall be initialized when the simulation starts.
  pub(super) nodes: HashMap<Id, Node>,
}

impl Circuit {
  #[inline]
  pub fn new() -> Self {
    Self {
      init_state: CircuitInitState::default(),
      source: Source::new(),
      content: Component::default(),
      nodes: HashMap::new(),
    }
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
  /// Returns `Ok(self)` if the circuit was successfully initialized.
  pub fn init(&mut self) -> Result<&mut Self> {
    if self.init_state == CircuitInitState::Source {
      return Ok(self);
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
      let initial_tension = *voltage;
      self.content.impedance.inv_inplace();
      let initial_current = initial_tension * self.content.impedance.eval(Complex::from(*pulse));
      self.content.impedance.inv_inplace();
      self.content.init_current_tension_potential(
        initial_current,
        initial_tension,
        initial_tension,
        *pulse,
        &mut self.nodes,
      )?;
    }
    self.init_state = CircuitInitState::Source;
    Ok(self)
  }

  // To be called when the circuit is changed
  #[inline]
  pub fn uninit_all(&mut self) -> &mut Self {
    self.init_state = CircuitInitState::None;
    self.nodes.clear();
    self.content.uninit_all();
    self
  }

  // To be called when the source is changed
  #[inline]
  pub fn uninit_source(&mut self) -> &mut Self {
    self.init_state = self.init_state.min(CircuitInitState::CircuitNodes);
    for node in self.nodes.values_mut() {
      node.next_component_tensions.clear();
      node.currents.clear();
      node.potentials.clear();
    }
    self.content.uninit_current_tension_potential();
    self
  }

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
  pub fn get_node(&self, id: &Id) -> Option<&Node> {
    self.nodes.get(id)
  }

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
  pub fn get_node_mut(&mut self, id: &Id) -> Option<&mut Node> {
    self.nodes.get_mut(id)
  }
}

impl Circuit {
  /// Gives a reference to the total impedance of the circuit.
  #[inline]
  pub fn impedance(&self) -> &RatioFrac<Complex<f64>> {
    &self.content.impedance
  }

  /// Gives a reference to the main component of the circuit.
  #[inline]
  pub fn content(&self) -> &Component {
    &self.content
  }

  /// Gives a mutable reference to the main component of the circuit.
  /// This method assumes that the circuit will be modified and uninitializes
  /// it completely.
  #[inline]
  pub fn content_mut(&mut self) -> &mut Component {
    &mut self.uninit_all().content
  }

  /// Gives a reference to a component of the circuit based on its ID.
  ///
  /// # Arguments
  ///
  /// * `id` - The ID of the component to retrieve.
  ///
  /// # Returns
  ///
  /// An `Option` containing a reference to the component if it exists,
  /// otherwise `None`.
  #[inline]
  pub fn get_comp_by_id(&self, id: &[u8]) -> Option<&Component> {
    self.content.get_comp_by_id(id)
  }

  /// Gives a mutable reference to a component of the circuit based on its ID.
  /// This method assumes that the circuit will be modified and uninitializes it
  /// completely.
  ///
  /// # Arguments
  ///
  /// * `id` - The ID of the component to retrieve.
  ///
  /// # Returns
  ///
  /// An `Option` containing a mutable reference to the component if it exists,
  /// otherwise `None`.
  #[inline]
  pub fn get_comp_by_id_mut(&mut self, id: &[u8]) -> Option<&mut Component> {
    self.uninit_all().content.get_comp_by_id_mut(id)
  }

  /// Sets up the nodes IDs of the `Circuit` and its components.
  #[inline]
  pub fn init_nodes(&mut self) -> &mut Self {
    if self.init_state > CircuitInitState::CircuitNodes {
      return self;
    }
    self.content.init_nodes(&mut self.nodes);
    self.init_state = CircuitInitState::CircuitNodes;
    self
  }

  /// Sets the voltage at a specific index in the `voltages` vector.
  #[inline]
  pub fn set_voltage(&mut self, index: usize, voltage: Complex<f64>) -> &mut Self {
    self.source.set_voltage(index, voltage);
    self.uninit_source()
  }

  /// Adds a new pulse to the `voltages` vector at the specified time. If the
  /// pulse is already present, its voltage is updated. The pulse is represented
  /// by a voltage value.
  #[inline]
  pub fn add_pulse(&mut self, pulse: f64, voltage: Complex<f64>) -> &mut Self {
    self.source.add_pulse(pulse, voltage);
    self.uninit_source()
  }

  /// Removes the pulse at the specified index from the generator.
  #[inline]
  pub fn remove_pulse(&mut self, index: usize) -> &mut Self {
    self.source.remove_pulse(index);
    self.uninit_source()
  }

  /// Clears the pulses of the generator.
  #[inline]
  pub fn clear_source(&mut self) -> &mut Self {
    self.source.clear();
    self.uninit_source()
  }

  /// Clears and updates the generator using a real valued function that
  /// generates voltage values using its Fourier transform. The function takes
  /// a time value as input and returns a voltage value. The `duration`
  /// parameter specifies the total duration of the voltage source (henceforth
  /// the duration of the simulation). The `n_freqs_` parameter specifies the
  /// number of frequencies to use in the Fourier series.
  #[inline]
  pub fn set_generator_fn<I, F>(&mut self, f: F, duration: f64, n_freqs: I) -> &mut Self
  where
    F: Fn(f64) -> f64,
    I: PrimInt + Debug,
  {
    self.source.set_fn(f, duration, n_freqs);
    self.uninit_source()
  }

  /// Non-consuming iterator over the pulses and their voltages
  #[inline]
  pub fn voltages(&self) -> impl Iterator<Item = &(f64, Complex<f64>)> {
    self.source.voltages()
  }
}
