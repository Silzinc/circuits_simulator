/* -------------------------------------------------------------------

                            Component module
                            
    -> Component declaration
       This struct represents a block structuring the circuit's charge
       It is made of a tension, a current, an equivalent simple dipole (explained below) and its content

    -> ComponentContent declaration
       This enum allows to represent parallel and serial chains of dipoles/dipole blocks
       It is either a simple dipole, a set of components in parallel or a set in serie

   ------------------------------------------------------------------- */

use crate::dipole::Dipole;

/* ------------------------------------------------------------------- */

#[derive(Debug, Clone)]
pub(crate) enum ComponentContent<T: num::Float + std::fmt::Debug> {
    Simple(Dipole<T>),
    Serial(Vec<Component<T>>),
    Parallel(Vec<Component<T>>),
}

#[derive(Debug, Clone)]
pub(crate) struct Component<T: num::Float + std::fmt::Debug> {
    pub(crate) tension: T,
    pub(crate) current: T,
    pub(crate) equiv: Dipole<T>,
    /*
    Explanation : at t = 0+
    ->  R means the component is equivalent to a resistor
    ->  C means the tension is necessarily 0 (equivalence with a capacitor),
        which happens if the component is a capacitor or if it is a parallel
        component with a capacitor in its branches, or a serial component with only capacitors, etc...
    ->  L means the current is necessarily 0 for reasons similar to the former case's
        (equivalence with a coil)

    ->  This whole is necessary to determine tension of sub-components in a serial component
        and current of sub-components in a parallel component
    */
    pub(crate) content: ComponentContent<T>,
}