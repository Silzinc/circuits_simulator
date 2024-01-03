use super::{circuit::*, component::*, dipole::Dipole, node::*};
use fractios::RatioFrac;
use std::default::Default;

impl Default for CircuitInitState
{
	#[inline]
	fn default() -> Self { Self::None }
}

impl Default for Dipole
{
	#[inline]
	fn default() -> Self { Dipole::Poisoned }
}

impl Default for ComponentInitState
{
	#[inline]
	fn default() -> Self { ComponentInitState::None }
}

impl Default for ComponentContent
{
	#[inline]
	fn default() -> Self { ComponentContent::Poisoned }
}

impl Default for Component
{
	#[inline]
	fn default() -> Self
	{
		Component { content:    ComponentContent::default(),
		            impedance:  RatioFrac::default(),
		            fore_node:  Id::default(),
		            init_state: ComponentInitState::default(), }
	}
}

impl Default for Node
{
	#[inline]
	fn default() -> Self
	{
		Node { id:                 Id::default(),
		       next_comp_tensions: Vec::new(),
		       potentials:         Vec::new(),
		       currents:           Vec::new(), }
	}
}
