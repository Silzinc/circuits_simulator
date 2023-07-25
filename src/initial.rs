/*-----------------------------------------------------------------------
|
|                           Initial implementation
|                           
|   -> new_r/l/c functions
|      These functions create simple dipoles based on their main attribute
|      A capacitor also needs its starting tension as an argument to be created
|   
|   -> setup and setup_aux methods
|      setup modifies self by setting to each component its tension and current at t = 0+
|      It takes the input tension as argument and can detect short-circuits
|   
|   -> push_serie/parallel methods
|      These methods help building the circuit by updating the equivalent dipoles of each component
|
-----------------------------------------------------------------------*/
#![allow(dead_code)]
use crate::{dipole::Dipole, component::{Component, ComponentContent}};
use Dipole::{C, F, L, R};
use ComponentContent::{Parallel, Serial, Simple};

/* ---------------------------------------------------------------------- */

duplicate::duplicate! {
    [float; [f64]; [f32]]
    impl Default for Component<float> {
        fn default() -> Self {
            let zero = 0 as float;
            Component {
                tension: zero,
                current: zero,
                equiv: F,
                content: Simple(F),
            }
        }
    }

    impl Component<float> {

        pub(crate) fn new_r(r: float) -> Self {
            if r <= 0 as float {
                panic!("Tried to build a negative or zero resistance")
            } else {
                Component {
                    tension: 0 as float,
                    current: 0 as float,
                    equiv: R(r),
                    content: Simple(R(r)),
                }
            }
        }
        pub(crate) fn new_c(c: float, u: float) -> Self {
            // u is the starting charge of the capacitor
            if c <= 0 as float {
                panic!("Tried to build a negative or zero capacitor")
            } else {
                Component {
                    tension: u,
                    current: 0 as float,
                    equiv: C(c),
                    content: Simple(C(c)),
                }
            }
        }
        pub(crate) fn new_l(l: float) -> Self {
            if l <= 0 as float {
                panic!("Tried to build a negative inductance")
            } else {
                Component {
                    tension: 0 as float,
                    current: 0 as float,
                    // We don't suppose the coil is charged initially
                    equiv: L(l),
                    content: Simple(L(l)),
                }
            }
        }
        /*
        pub(crate) fn new(d: Dipole<float>) -> Self {
            match d {
                R(r) => Self::new_r(r),
                C(c) => Self::new_c(c),
                L(l) => Self::new_l(l),
                F => Self::default(),
            }
        }
        */

        pub(crate) fn setup(&mut self, input: float) {
            match self.equiv {
                C(_) | F => panic!("Short-circuit observed"),
                _ => self.setup_aux(input),
            }
        }
        fn setup_aux(&mut self, input: float) {
            // For an inductance or a resistance, input is treated as the tension to set
            // For a capacity, it is treated as a current ~ The other values can be inferred from it
            match self.equiv {
                L(l) => {
                    self.tension = input;
                    match self.content {
                        Serial(ref mut components) => {
                            let mut u = input;
                            for c in components.iter_mut() {
                                match c.equiv {
                                    C(_) => {
                                        u -= c.tension;
                                        c.setup_aux(self.current);
                                        // In this case, the tension of c is known prior to
                                        // the call of setup, and therefore we need to know what
                                        // tension is "left" to the other components (the coils here)
                                    },
                                    R(r) => {
                                        c.setup_aux(self.current * r);
                                        u -= c.tension;
                                    },
                                    _ => (),
                                };
                            }
                            // Here we are sure to find at least one L
                            for c in components.iter_mut() {
                                match c.equiv {
                                    L(_l) => c.setup_aux(u * _l / l),
                                    _ => (),
                                };
                            }
                        },
                        Parallel(ref mut components) => {
                            for c in components.iter_mut() {
                                c.setup_aux(input);
                            }
                        },
                        _ => (),
                    }
                },
                R(r) => {
                    self.tension = input;
                    match self.content {
                        Serial(ref mut components) => {
                            let mut u = input;
                            for c in components.iter() {
                                // Here we are sure not to find an inductance
                                // We first need to retrieve the tension shared between the sub resistors
                                if matches!(c.equiv, C(_)) {
                                    u -= c.tension;
                                    // c.tension is already known
                                }
                            }
                            self.current = u / r;
                            for c in components.iter_mut() {
                                match c.equiv {
                                    F | L(_) => unreachable!(),
                                    C(_) => c.setup_aux(self.current),
                                    R(_r) => c.setup_aux(_r * self.current),
                                }
                            }
                        },
                        Parallel(ref mut components) => {
                            self.current = input / r;
                            for c in components.iter_mut() {
                                c.setup_aux(input);
                            }
                        },
                        _ => self.current = input / r,
                    }
                },
                C(c) => {
                    self.current = input;
                    match self.content {
                        Serial(ref mut components) => {
                            // There should only be capacities here
                            for co in components.iter_mut() {
                                co.setup_aux(input);
                            }
                        },
                        Parallel(ref mut components) => {
                            let mut i = input;
                            for co in components.iter_mut() {
                                if matches!(co.equiv, R(_) | L(_)) {
                                    co.setup_aux(self.tension);
                                    i -= co.current;
                                }
                            }
                            // Here we are sure to find at least one C
                            for co in components.iter_mut() {
                                match co.equiv {
                                    C(_c) => co.setup_aux(i * _c / c),
                                    _ => (),
                                }
                            }
                        },
                        _ => (),
                    }
                },
                F => unreachable!(),
                // There can only be a F if there is nothing else than a wire among the components
                // That bring necessarily a short-circuit if the input tension is non zero
            }
        }

        // self is modified to add other in serie to the rest of the Component
        // Currents and tensions are not updated as it will be handled by the setup
        pub(crate) fn push_serie(&mut self, other: Self) {
            let new_equiv = match (&self.equiv, &other.equiv) {
                (F,     _)      => {*self = other; return}
                (L(l1), L(l2))  => L(l1 + l2),
                (_,     L(l))   => L(*l),
                (R(r1), R(r2))  => R(r1 + r2),
                (C(_),  R(r))   => R(*r),
                (C(c1), C(c2))  => C(c1 * c2 / (c1 + c2)),
                _ => self.equiv.clone(),
            };
            // Here we computed the new equivalent using domination relationships
            // between dipoles in serie when it comes to determining the flowing current
            match &mut self.content {
                Serial(components) => {
                    self.equiv = new_equiv;
                    components.push(other)
                }
                _ => {
                    let zero = 0 as float;
                    *self = Component {
                        tension: zero,
                        current: zero,
                        equiv: new_equiv,
                        content: Serial(vec![std::mem::take(self), other]),
                    }
                }
            }
        }

        // Does the same for parallel components
        pub(crate) fn push_parallel(&mut self, other: Self) {
            let new_equiv = match (&self.equiv, &other.equiv) {
                (F,     _)      => {*self = other; return}
                (C(c1), C(c2))  => C(c1 + c2),
                (_,     C(c))   => C(*c),
                (R(r1), R(r2))  => R(r1 * r2 / (r1 + r2)),
                (L(_),  R(r))   => R(*r),
                (L(l1), L(l2))  => L(l1 * l2 / (l1 + l2)),
                _ => self.equiv.clone(),
            };
            // Notice the relationships changed when dealing with parallel components :
            // The tension of one capacitor-like determines the tension of the whole component
            match &mut self.content {
                Parallel(components) => {
                    self.equiv = new_equiv;
                    components.push(other)
                }
                _ => {
                    let zero = 0 as float;
                    *self = Component {
                        tension: zero,
                        current: zero,
                        equiv: new_equiv,
                        content: Parallel(vec![std::mem::take(self), other]),
                    }
                }
            }
        }
    }
}
