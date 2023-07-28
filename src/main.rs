/*-----------------------------------------------------------------------
|
|                           Main script
|
|    -> Modules declaration
|    -> Defining `type_of`, a function returning the type of its argument
|    -> Test and debug
|
-----------------------------------------------------------------------*/

mod circuit;
mod component;
mod dipole;
mod generator;
mod initial;
mod update;

#[allow(dead_code)]
pub(crate) fn type_of<T>(_: &T) -> String {
    String::from(std::any::type_name::<T>())
}

use std::io::Write;
use text_io::read;

/* ------------------------------------------------------------------- */

fn main() -> Result<(), Box<dyn std::error::Error>> {
    use component::Component as Com;

    let mut comp = Com::<f32>::default();

    comp.push_serie(Com::<f32>::new_c(1., 0.));
    comp.push_serie(Com::<f32>::new_r(1.));
    comp.push_serie(Com::<f32>::new_l(1.));
    comp.push_parallel(Com::<f32>::new_r(1.));

    /*
    get_by_id test
    let id = [0u8, 2u8];

    println!("{:?}", comp.get_by_id(&id));
    */

    print!("Entrez la source en volts : ");
    let s: f32 = read!("{}\n");

    let mut circuit = circuit::Circuit {
        dt: 0.001,
        source: s,
        charge: comp,
    };

    circuit.setup();
    let ini_energ = circuit.charge.energy;
    let mut charge_energy = vec![0f32];
    let mut produced_energy = vec![0f32];
    let mut currents = vec![circuit.charge.current];

    for _ in 0..30000 {
        let old = circuit.charge.current;
        circuit.update();
        charge_energy.push(circuit.charge.energy - ini_energ);
        produced_energy
            .push(produced_energy[produced_energy.len() - 1] + circuit.source * circuit.dt * old);
        currents.push(circuit.charge.current);
    }

    let mut charge_txt = std::fs::File::create("out/charge_energy.txt")?;
    let mut produced_txt = std::fs::File::create("out/produced_energy.txt")?;
    let mut currents_txt = std::fs::File::create("out/currents.txt")?;

    charge_txt.write_all(format!("{:?}", charge_energy).as_bytes())?;
    produced_txt.write_all(format!("{:?}", produced_energy).as_bytes())?;
    currents_txt.write_all(format!("{:?}", currents).as_bytes())?;
    Ok(())
}
