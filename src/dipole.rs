#[derive(Debug, Clone)]
pub(crate) enum Dipole<T: num::Float + std::fmt::Debug> {
    F,    // Poisoned state (originally intended to be a simple wire)
    R(T), // Resistor / Impedance
    L(T), // Coil
    C(T), // Capacitor
}
impl<T: num::Float + std::fmt::Debug> Default for Dipole<T> {
    fn default() -> Self {
        Dipole::F
    }
}
