use super::{Dipole, Id, Node};
use crate::{
  error::{self, short_circuit_current, short_circuit_tension, Error::CircuitBuild},
  util::{evaluate_zero_without_invx, evaluate_zero_without_x, is_multiple_of_invx, is_multiple_of_x},
};
use fractios::RatioFrac;
use num::complex::Complex;
use num_traits::Zero;
use serde::{ser::SerializeStruct, Serialize, Serializer};
use std::collections::HashMap;

/// Represents the initialisation state of a component.
#[derive(Clone, Debug, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum ComponentInitState
{
  /// No initialisation
  #[default]
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
#[derive(Clone, Debug, Default)]
pub enum ComponentContent
{
  Parallel(Vec<Component>),
  Series(Vec<Component>),
  Simple(Dipole),
  /// Used as a default state
  #[default]
  Poisoned,
}

impl Serialize for ComponentContent
{
  #[inline]
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    let mut state = serializer.serialize_struct("ComponentContent", 2)?;
    match self {
      ComponentContent::Parallel(components) => {
        state.serialize_field("type", "parallel")?;
        state.serialize_field("components", components)?;
      },
      ComponentContent::Series(components) => {
        state.serialize_field("type", "series")?;
        state.serialize_field("components", components)?;
      },
      ComponentContent::Simple(dipole) => {
        state.serialize_field("type", "simple")?;
        state.serialize_field("dipole", dipole)?;
      },
      ComponentContent::Poisoned => {
        state.serialize_field("type", "poisoned")?;
      },
    }
    state.end()
  }
}

/// A struct representing a circuit component.
#[derive(Clone, Debug, Default)]
pub struct Component
{
  /// The content of the component.
  pub content:      ComponentContent,
  /// The impedance of the component.
  pub impedance:    RatioFrac<Complex<f64>>,
  /// The ID of the node connected to the component's fore port.
  pub fore_node_id: Id,
  pub init_state:   ComponentInitState,
}

impl Component
{
  #[inline]
  pub fn new() -> Self { Component::default() }

  /// Returns the impedance of the component for a given pulse.
  #[inline]
  pub fn impedance(&self, pulse: f64) -> Complex<f64> { self.impedance.eval(Complex::from(pulse)) }
}

impl Serialize for Component
{
  #[inline]
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    let mut state = serializer.serialize_struct("Component", 2)?;
    state.serialize_field("content", &self.content)?;
    state.serialize_field("foreNodeId", &String::from_utf8(self.fore_node_id.clone()).unwrap())?;
    state.end()
  }
}

impl From<ComponentContent> for Component
{
  #[inline]
  fn from(content: ComponentContent) -> Self
  {
    Component {
      content,
      impedance: RatioFrac::default(),
      fore_node_id: Id::default(),
      init_state: ComponentInitState::default(),
    }
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
  /// use circuits_simulator::{
  ///   Component,
  ///   Dipole::{Capacitor, Resistor},
  /// };
  ///
  /// let mut component1 = Component::from(Resistor(10.0));
  /// let mut component2 = Component::from(Capacitor(0.1));
  ///
  /// component1.push_serie(component2);
  /// ```
  #[inline]
  pub fn push_serie(&mut self, component: Self) -> &mut Self
  {
    use ComponentContent::*;
    match self.content {
      Poisoned => {
        // This should be the case, but we never know
        // A poisoned state should only be at the root of a newly instanciated circuit
        assert!(self.fore_node_id.is_empty());
        self.content = component.content;
        self.fore_node_id.push(0u8);
      },
      Series(ref mut components) => {
        let mut id = components.last().unwrap().fore_node_id.clone();
        *id.last_mut().unwrap() += 1u8;
        components.push(component);
        components.last_mut().unwrap().fore_node_id = id;
      },
      _ => {
        let mut id = std::mem::take(&mut self.fore_node_id);
        let mut new_components = vec![std::mem::take(self), component];
        // Set back self's id
        self.fore_node_id = id.clone();
        // Set first serial component's id
        id.push(0u8);
        new_components[0].fore_node_id = id.clone();
        // Set second serial component's id
        *id.last_mut().unwrap() += 1u8;
        new_components[1].fore_node_id = id;
        // Set self's new content
        self.content = Series(new_components);
      },
    };
    self
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
  /// use circuits_simulator::{
  ///   Component,
  ///   Dipole::{Capacitor, Resistor},
  /// };
  ///
  /// let mut component1 = Component::try_from(Resistor(10.0)).unwrap();
  /// let mut component2 = Component::try_from(Capacitor(0.1)).unwrap();
  ///
  /// component1.push_parallel(component2);
  /// ```
  #[inline]
  pub fn push_parallel(&mut self, component: Self) -> &mut Self
  {
    use ComponentContent::*;
    match self.content {
      Poisoned => {
        // This should be the case, but we never know
        // A poisoned state should only be at the root of a newly instanciated circuit
        assert!(self.fore_node_id.is_empty());
        self.content = component.content;
        self.fore_node_id.push(0u8);
      },
      Parallel(ref mut components) => {
        let mut id = components.last().unwrap().fore_node_id.clone();
        *id.last_mut().unwrap() += 1u8;
        components.push(component);
        components.last_mut().unwrap().fore_node_id = id;
      },
      _ => {
        let mut id = std::mem::take(&mut self.fore_node_id);
        let mut new_components = vec![std::mem::take(self), component];
        // Set back self's id
        self.fore_node_id = id.clone();
        // Set first serial component's id
        id.push(0u8);
        new_components[0].fore_node_id = id.clone();
        // Set second serial component's id
        *id.last_mut().unwrap() += 1u8;
        new_components[1].fore_node_id = id;
        // Set self's new content
        self.content = Parallel(new_components);
      },
    };
    self
  }

  // Swaps two components in a branch
  #[inline]
  pub fn swap(&mut self, index1: usize, index2: usize) -> error::Result<&mut Self>
  {
    use ComponentContent::*;
    match &mut self.content {
      Series(components) | Parallel(components) => components.swap(index1, index2),
      _ => return Err(CircuitBuild("Cannot swap components in a non-branch component".to_string())),
    }
    Ok(self)
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
  /// Returns an error of type `CircuitBuild` if the component is in a
  /// `Poisoned` state, case in which no initialisation is possible.
  ///
  /// # Examples
  ///
  /// ```
  /// use circuits_simulator::{
  ///   Component,
  ///   Dipole::{Capacitor, Resistor},
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
  pub fn init_impedance(&mut self) -> error::Result<&mut Self>
  {
    if self.init_state > ComponentInitState::Impedance {
      return Ok(self);
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
      Poisoned => return Err(CircuitBuild("Cannot initialize impedance of poisoned component".to_string())),
    };
    self.init_state = ComponentInitState::Impedance;
    Ok(self)
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
  /// appropriate or if there is a short circuit in the circuit. For more
  /// details on short circuits, see `[Circuit::init]`.
  pub fn init_current_tension_potential(
    &mut self,
    current: Complex<f64>,
    tension: Complex<f64>,
    fore_potential: Complex<f64>,
    pulse: f64,
    nodes: &mut HashMap<Id, Node>,
  ) -> error::Result<&mut Self>
  {
    if self.init_state > ComponentInitState::CurrentTensionPotential {
      return Ok(self);
    } else if self.init_state < ComponentInitState::Impedance {
      return Err(CircuitBuild(
        "Cannot initialize currents and tensions before the impedance".to_string(),
      ));
    }

    let node = nodes.get_mut(self.fore_node_id.as_slice()).expect("Node not found :/");
    node.currents.push(current);
    node.next_component_tensions.push(tension);
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
            return short_circuit_current(&component.fore_node_id, current, &component.impedance);
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
            return short_circuit_tension(&component.fore_node_id, tension, &component.impedance);
          }
        },
      _ => (),
    };
    self.init_state = ComponentInitState::CurrentTensionPotential;
    Ok(self)
  }

  #[inline]
  pub fn uninit_current_tension_potential(&mut self) -> &mut Self
  {
    self.init_state = self.init_state.min(ComponentInitState::Impedance);
    self
  }

  #[inline]
  pub fn uninit_all(&mut self) -> &mut Self
  {
    self.init_state = ComponentInitState::None;
    self
  }
}
