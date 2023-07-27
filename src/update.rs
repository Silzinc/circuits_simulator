/*-----------------------------------------------------------------------
|
|                           Update module
|                           
|   In this module are declared methods used to update the state
|   of a circuit.
|
|   `preupdate`, when step_index is 0, updates the 
|   tension on each capacitor, the current in each coil and the sources 
|   in each real generator when advancing in time by dt. It does so with
|   Euler's explicit method, which alone yields errors on long simulations.
|
|   `preupdate`, when step_index is 1, updates the 
|   tension on each capacitor, the current in each coil and the sources 
|   in each real generator when advancing in time by dt and when the currents
|   and tensions at t + dt/2 are known, using the RK2 method (2nd order 
|   Runge-Kutta). This reduces a lot the errors compared to explicit Euler alone.
|
|   The, the method `setup` infers the other variables as it did to
|   determine the circuit's initial state.
|
-----------------------------------------------------------------------*/

#![allow(dead_code)]
use crate::{dipole::Dipole, component::{Component, ComponentContent}, circuit::Circuit};
use Dipole::{C, L, R};
use ComponentContent::{Parallel, Serial, Simple};

/* ------------------------------------------------------------------- */

duplicate::duplicate! {
    [float; [f64]; [f32]]

    impl Component<float> {
        fn preupdate(&mut self, dt: float, step_index: u8) {
            match self.equiv {
                L(l) => {
                    match &mut self.content {
                        Serial(components) => {
                            let mut needs_current = true;
                            for c in components.iter_mut() {
                                // At least one inductance here
                                c.preupdate(dt, step_index);
                                if needs_current && matches!(c.equiv, L(_)) {
                                    self.current = c.current;
                                    needs_current = false;
                                }
                            }
                        },
                        Parallel(components) => {
                            let mut i = 0 as float;
                            for c in components.iter_mut() {
                                // Only inductances here
                                c.preupdate(dt, step_index);
                                i += c.current;
                            }
                            self.current = i;
                        },
                        Simple(_, previous) => {
                            if step_index == 0u8 {
                                *previous = self.current;
                                self.current += dt * self.tension / l;
                            }
                            if step_index == 1u8 {
                                self.current = *previous + dt * self.tension / l;
                            }
                        },
                        // Simple(L(l), previous)
                    }
                },

                C(c) => {
                    match &mut self.content {
                        Serial(components) => {
                            let mut u = 0 as float;
                            for co in components.iter_mut() {
                                // Only capacities here
                                co.preupdate(dt, step_index);
                                u += co.tension;
                            }
                            self.tension = u
                        },
                        Parallel(components) => {
                            let mut needs_tension = true;
                            for co in components.iter_mut() {
                                // At least one capacity here
                                co.preupdate(dt, step_index);
                                if needs_tension && matches!(co.equiv, C(_)) {
                                    self.tension = co.tension;
                                    needs_tension = false;
                                }
                            }
                        },
                        Simple(_, previous) => {
                            if step_index == 0u8 {
                                *previous = self.tension;
                                self.tension += dt * self.current / c;
                            }
                            if step_index == 1u8 {
                                self.tension = *previous + dt * self.current / c;
                            }
                        }
                        // Simple(C(c), previous)
                    }
                },

                R(ref mut g) => match &mut self.content {
                    Serial(components) => {
                        let mut u = 0 as float;
                        for c in components.iter_mut() {
                            c.preupdate(dt, step_index);
                            match c.equiv {
                                C(_) => u += c.tension,
                                R(ref _g) => u += _g.gen_tens(),
                                _ => unreachable!(),
                            };
                        }
                        g.source = u;
                        g.t_or_n = true
                    },
                    Parallel(components) => {
                        let mut i = 0 as float;
                        for c in components.iter_mut() {
                            c.preupdate(dt, step_index);
                            match c.equiv {
                                L(_) => i += c.current,
                                R(ref _g) => i += _g.gen_cur(),
                                _ => unreachable!(),
                            };
                        }
                        g.source = i;
                        g.t_or_n = false
                    },
                    Simple(_, previous) => {
                        if step_index == 0u8 {
                            *previous = self.current;
                        }
                    } 
                    // Simple(R(g), previous)
                },
                _ => unreachable!(),
            }
        }
    }

    impl Circuit<float> {
        pub(crate) fn setup(&mut self) {
            self.charge.setup(self.source, self.dt)
        }
        pub(crate) fn update(&mut self) {
            /*
            // Euler's explicit method alone :
            self.charge.preupdate(self.dt, 0u8);
            self.charge.setup(self.source, self.dt);
            */

            // RK2 method :
            self.charge.preupdate(self.dt * 0.5, 0u8);
            self.charge.setup(self.source, self.dt * 0.5);
            self.charge.preupdate(self.dt, 1u8);
            self.charge.setup(self.source, self.dt * 0.5);
        }
    }
}