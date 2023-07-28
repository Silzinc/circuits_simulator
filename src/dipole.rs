/*-----------------------------------------------------------------------
|
|                           Dipole module
|
|    -> Dipole declaration
|       This enum represents a simple passive dipole.
|       It is either a real generator, a coil or a capacitor.
|
-----------------------------------------------------------------------*/

#[derive(Debug, Clone)]
pub(crate) enum Dipole<T: num::Float + std::fmt::Debug> {
    F,                                 // Poisoned state (originally intended to be a simple wire)
    R(crate::generator::Generator<T>), // Resistor/Real generator
    // We actually need a real generator here in general
    L(T), // Coil
    C(T), // Capacitor
}
impl<T: num::Float + std::fmt::Debug> Default for Dipole<T> {
    fn default() -> Self {
        Dipole::F
    }
}
