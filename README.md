# Circuits simulator

An algorithm to emulate a certain class of electronical analogical circuits written in Rust. This will also serve as a backend for my electronics simulation software [`asce`](https://github.com/Silzinc/asce).

## Installation

1. Install Rust and Cargo: https://www.rust-lang.org/tools/install
2. Clone the repository: `git clone https://github.com/Silzinc/circuits_simulator.git`
3. Change directory to the project: `cd circuits_simulator`
4. Test the project's RLC circuit example: `cargo +stable run --release --example rlc_square_wave`. A plot of the tension felt by the capacitor and the tension outputed by the generator will be created in the `out/` directory.

## Documentation

Run `cargo doc` and open `target/doc/circuits_simulator/index.html`.
