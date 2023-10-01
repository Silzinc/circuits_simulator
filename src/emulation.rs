use crate::{
	error::{short_circuit_current, Result},
	structs::{circuit::Circuit, node::Id},
};
use fractios::traits::{RatioFracComplexFloat, RatioFracFloat};
use num::Complex;
use num_traits::Zero;

impl<T: RatioFracFloat> Circuit<T> where Complex<T>: RatioFracComplexFloat
{
	// This function is used to emulate a circuit and returns the currents and
	// tensions of a certain node
	pub fn emulate(&mut self, duration: T, step: T, node_id: &Id) -> Result<(Vec<T>, Vec<T>)>
	{
		let two = T::one() + T::one();
		self.init()?;

		let node = self.nodes.get_mut(node_id).expect("Node not found :/");
		let initial_currents = node.currents.clone();
		let initial_tensions = node.next_comp_tensions.clone();

		let nb_iter = (duration / step).ceil()
		                               .to_usize()
		                               .expect(&format!("round({duration:?}/{step:?}) to usize failed"));
		let mut currents = Vec::with_capacity(nb_iter);
		let mut tensions = Vec::with_capacity(nb_iter);
		let mut elapsed = T::zero();

		while elapsed < duration {
			let mut current = initial_currents[0].re;
			let mut tension = initial_tensions[0].re;
			for (k, (pulse, voltage)) in self.source.voltages.iter().enumerate() {
				if voltage.is_zero() || pulse.is_zero() {
					continue;
				}
				if pulse.is_zero() {
					return short_circuit_current(&vec![0u8], voltage, &self.content.borrow().impedance);
				}
				let factor = Complex::new(T::zero(), elapsed * *pulse).exp();
				// This way we know we can approximate a real function such as current or
				// tension if we only use positive pulses
				current += two * (initial_currents[k] * factor).re;
				tension += two * (initial_tensions[k] * factor).re;
			}
			currents.push(current); // We only need the real part to emulate a circuit
			tensions.push(tension);
			elapsed += step;
		}
		Ok((currents, tensions))
	}
}
