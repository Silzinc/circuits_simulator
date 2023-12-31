use super::component::*;
use super::dipole::Dipole;
use super::node::*;
use fractios::RatioFrac;
use std::default::Default;

impl Default for Dipole
{
	fn default() -> Self { Dipole::Poisoned }
}

impl Default for ComponentContent
{
	fn default() -> Self { ComponentContent::Poisoned }
}

impl Default for Component
{
	fn default() -> Self
	{
		Component { content:   ComponentContent::default(),
		            impedance: RatioFrac::default(),
		            fore_node: Id::default(), }
	}
}

impl Default for Node
{
	fn default() -> Self
	{
		Node { id:                 Id::default(),
		       next_comp_tensions: Vec::new(),
		       potentials:         Vec::new(),
		       currents:           Vec::new(), }
	}
}
