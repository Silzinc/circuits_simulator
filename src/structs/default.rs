use super::component::*;
use super::dipole::Dipole;
use super::node::*;
use fractios::{
	traits::{RatioFracComplexFloat, RatioFracFloat},
	RatioFrac,
};
use num::Complex;
use std::default::Default;

impl<T> Default for Dipole<T>
{
	fn default() -> Self { Dipole::Poisoned }
}

impl<T> Default for ComponentContent<T>
{
	fn default() -> Self { ComponentContent::Poisoned }
}

impl<T: RatioFracFloat> Default for Component<T> where Complex<T>: RatioFracComplexFloat
{
	fn default() -> Self
	{
		Component { content:   ComponentContent::default(),
		            impedance: RatioFrac::default(),
		            fore_node: Id::default(), }
	}
}

impl<T: RatioFracFloat + Default> Default for Node<T>
{
	fn default() -> Self
	{
		Node { id:                 Id::default(),
		       next_comp_tensions: Vec::new(),
		       potentials:         Vec::new(),
		       currents:           Vec::new(), }
	}
}
