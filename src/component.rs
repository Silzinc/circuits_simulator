/*-----------------------------------------------------------------------
|
|                           Component module
|                           
|    -> Component declaration
|       This struct represents a block structuring the circuit's charge.
|       It is made of a tension, a current, an equivalent simple dipole
|       (explained below) and its content.
| 
|    -> ComponentContent declaration
|       This enum allows to represent parallel and serial chains of 
|       dipoles/dipole blocks. It is either a simple dipole,
|       a set of components in parallel or a set in serie.
|
|    -> new_r/l/c functions defined
|       These functions create simple dipoles based on their main attribute.
|       A capacitor also needs its starting tension as an argument to be created.
|
-----------------------------------------------------------------------*/

use crate::{dipole::Dipole, generator::Generator as Gen};
use Dipole::{F, R, L, C};

/* ------------------------------------------------------------------- */

#[derive(Debug, Clone)]
pub(crate) enum ComponentContent<T: num::Float + std::fmt::Debug> {
    Simple(Dipole<T>),
    Serial(Vec<Component<T>>),
    Parallel(Vec<Component<T>>),
}
use ComponentContent::Simple;

#[derive(Debug, Clone)]
pub(crate) struct Component<T: num::Float + std::fmt::Debug> {
    pub(crate) tension: T,
    pub(crate) current: T,
    pub(crate) equiv: Dipole<T>,
    /*
    Explanation :
    ->  R means the component is equivalent to a Norton/Thevenin generator
    ->  C means the tension is imposed by the dipole (equivalence with a capacitor),
        which happens if the component is a capacitor or if it is a parallel
        component with a capacitor in its branches, or a serial component with only capacitors, etc...
    ->  L means the current is imposed by the dipole for reasons similar to the former case's
        (equivalence with a coil)

    ->  This whole is necessary to determine tension of sub-components in a serial component
        and current of sub-components in a parallel component
    */
    pub(crate) energy: T,
    pub(crate) content: ComponentContent<T>,
}

impl<T: num::Float + num::Zero + std::fmt::Debug> Default for Component<T> {
    fn default() -> Self {
        let zero = T::zero();
        Self {
            tension: zero,
            current: zero,
            equiv: F,
            energy: zero,
            content: Simple(F),
        }
    }
}

impl<T: num::Float + num::NumCast + std::fmt::Debug> Component<T> {
    pub(crate) fn new_r(_r: T) -> Self {
        let zero = T::zero();
        if _r <= zero {
            panic!("Tried to build a negative or zero resistance")
        } else {
            Self {
                tension: zero,
                current: zero,
                equiv: R(Gen {
                    r: _r,
                    source: zero,
                    t_or_n: true,
                }),
                energy: zero,
                content: Simple(R(Gen {
                    r: _r,
                    source: zero,
                    t_or_n: true,
                })),
            }
        }
    }
    pub(crate) fn new_c(c: T, u: T) -> Self {
        // u is the starting charge of the capacitor
        let zero = T::from(0.).unwrap();
        if c <= zero {
            panic!("Tried to build a negative or zero capacitor")
        } else {
            Self {
                tension: u,
                current: zero,
                equiv: C(c),
                energy: c * u * u * T::from(0.5).unwrap(),
                content: Simple(C(c)),
            }
        }
    }
    pub(crate) fn new_l(l: T) -> Self {
        let zero = T::zero();
        if l <= zero {
            panic!("Tried to build a negative or zero inductance")
        } else {
            Self {
                tension: zero,
                current: zero,
                // We don't suppose the coil is charged initially
                equiv: L(l),
                energy: zero,
                content: Simple(L(l)),
            }
        }
    }
}