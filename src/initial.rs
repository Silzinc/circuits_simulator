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

use crate::{dipole::Dipole, component::{Component, ComponentContent}, generator::Generator as Gen};
use Dipole::{C, F, L, R};
use ComponentContent::{Parallel, Serial};

/* ---------------------------------------------------------------------- */

duplicate::duplicate! {
    [float; [f64]; [f32]]
    
    impl Component<float> {

        pub(crate) fn setup(&mut self, input: float) {
            match self.equiv {
                C(_) | F => panic!("Short-circuit observed"),
                _ => self.setup_aux(input),
            }
        }
        fn setup_aux(&mut self, input: float) {
            // For a real generator or an inductance, input is treated as the tension to set
            // For a capacity, it is treated as a current ~ The other values can be inferred from it
            // Here we suppose the tension of capacities, the current of inductances and the sources of real generators are known
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
                                    R(ref g) => {
                                        c.setup_aux(self.current * g.r + g.gen_tens());
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
                            // There are only inductances here
                            for c in components.iter_mut() {
                                c.setup_aux(input);
                            }
                        },
                        _ => (),
                    }
                },
                R(ref g) => {
                    self.tension = input;
                    self.current = (input - g.gen_tens()) / g.r;
                    match self.content {
                        Serial(ref mut components) => {
                            for c in components.iter_mut() {
                                // Here we are sure not to find an inductance
                                match c.equiv {
                                    F | L(_) => unreachable!(),
                                    C(_) => c.setup_aux(self.current),
                                    R(ref _g) => c.setup_aux(_g.r * self.current + _g.gen_tens()),
                                }
                            }
                        },
                        Parallel(ref mut components) => {
                            for c in components.iter_mut() {
                                // Here we are sure not to find a capacity
                                c.setup_aux(input);
                            }
                        },
                        _ => (),
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
                (R(g1), R(g2))  => 
                R(Gen {
                    r: g1.r + g2.r,
                    source: g1.gen_tens() + g2.gen_tens(),
                    t_or_n: true,
                }),
                (C(_),  R(g))   => 
                R(Gen {
                    r: g.r,
                    source: g.gen_tens() + self.tension,
                    t_or_n: true,
                }),
                (R(g),  C(_))   => 
                R(Gen {
                    r: g.r,
                    source: g.gen_tens() + other.tension,
                    t_or_n: true,
                }),
                (C(c1), C(c2))  => C(c1 * c2 / (c1 + c2)),
                _ => self.equiv.clone(),
            };
            // Here we computed the new equivalent using domination relationships
            // between dipoles in serie when it comes to determining the flowing current
            match &mut self.content {
                Serial(components) => {
                    self.equiv = new_equiv;
                    self.tension += other.tension;
                    components.push(other)
                }
                _ => {
                    *self = Self {
                        tension: self.tension + other.tension,
                        current: self.current,
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
                (R(g1), R(g2))  => 
                R(Gen {
                    r: g1.r * g2.r / (g1.r + g2.r),
                    source: g1.gen_cur() + g2.gen_cur(),
                    t_or_n: false,
                }),
                (L(_),  R(g))   => 
                R(Gen {
                    r: g.r,
                    source: g.gen_cur() + self.current,
                    t_or_n: false,
                }),
                (R(g),  L(_))   =>
                R(Gen {
                    r: g.r,
                    source: g.gen_cur() + other.current,
                    t_or_n: false,
                }),
                (L(l1), L(l2))  => L(l1 * l2 / (l1 + l2)),
                _ => self.equiv.clone(),
            };
            // Notice the relationships changed when dealing with parallel components :
            // The tension of one capacitor-like determines the tension of the whole component
            match &mut self.content {
                Parallel(components) => {
                    self.equiv = new_equiv;
                    self.current += other.current;
                    components.push(other)
                }
                _ => {
                    *self = Self {
                        tension: self.tension,
                        current: self.current + other.current,
                        equiv: new_equiv,
                        content: Parallel(vec![std::mem::take(self), other]),
                    }
                }
            }
        }
    }
}
