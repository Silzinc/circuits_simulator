/*-----------------------------------------------------------------------
|
|                           Update module
|                           
|   In this module are declared methods used to update the state
|   of a circuit. 
|
|   In particular, `update_step1` uses Euler's explicit
|   method to compute the tension on each capacitor, the current in 
|   each coil and the sources in each real generator.
|
|   The, the method `setup` infers the other variables as it did to
|   determine the circuit's initial state.
|
-----------------------------------------------------------------------*/

use crate::{dipole::Dipole, component::{Component, ComponentContent}, circuit::Circuit};
use Dipole::{C, L, R};
use ComponentContent::{Parallel, Serial};

/* ------------------------------------------------------------------- */

duplicate::duplicate! {
    [float; [f64]; [f32]]

    impl Component<float> {
        fn update_step1(&mut self, dt: float) {
            match self.equiv {
                L(l) => match self.content {
                    Serial(ref mut components) => {
                        let mut needs_current = true;
                        for c in components.iter_mut() {
                            // At least one inductance here
                            c.update_step1(dt);
                            if needs_current && matches!(c.equiv, L(_)) {
                                self.current = c.current;
                                needs_current = false;
                            }
                        }
                    },
                    Parallel(ref mut components) => {
                        let mut i = 0 as float;
                        for c in components.iter_mut() {
                            // Only inductances here
                            c.update_step1(dt);
                            i += c.current;
                        }
                        self.current = i;
                    },
                    _ => self.current += dt * self.tension / l,
                    // Simple(L(l))
                },
                C(c) => match self.content {
                    Serial(ref mut components) => {
                        let mut u = 0 as float;
                        for co in components.iter_mut() {
                            // Only capacities here
                            co.update_step1(dt);
                            u += co.tension;
                        }
                        self.tension = u;
                    },
                    Parallel(ref mut components) => {
                        let mut needs_tension = true;
                        for co in components.iter_mut() {
                            // At least one capacity here
                            co.update_step1(dt);
                            if needs_tension && matches!(co.equiv, C(_)) {
                                self.tension = co.tension;
                                needs_tension = false;
                            }
                        }
                    },
                    _ => self.tension += dt * self.current / c,
                    // Simple(C(c))
                },
                R(ref mut g) => match self.content {
                    Serial(ref mut components) => {
                        let mut u = 0 as float;
                        for c in components.iter_mut() {
                            c.update_step1(dt);
                            match c.equiv {
                                C(_) => u += c.tension,
                                R(ref _g) => u += _g.gen_tens(),
                                _ => unreachable!(),
                            };
                        }
                        g.source = u;
                        g.t_or_n = true;
                    },
                    Parallel(ref mut components) => {
                        let mut i = 0 as float;
                        for c in components.iter_mut() {
                            c.update_step1(dt);
                            match c.equiv {
                                L(_) => i += c.current,
                                R(ref _g) => i += _g.gen_cur(),
                                _ => unreachable!(),
                            };
                        }
                        g.source = i;
                        g.t_or_n = false;
                    },
                    _ => (),
                },
                _ => (),
            }
        }
    }

    impl Circuit<float> {
        pub(crate) fn setup(&mut self) {
            self.charge.setup(self.source)
        }
        pub(crate) fn update(&mut self) {
            self.charge.update_step1(self.dt);
            self.charge.setup(self.source);
        }
    }
}