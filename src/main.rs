/*-----------------------------------------------------------------------
|
|                           Main script
|                           
|   -> Modules declaration
|   -> Defining `type_of`, a function returning the type of its argument
|   -> Test and debug
|
-----------------------------------------------------------------------*/

mod generator;
mod dipole;
mod component;
mod initial;
mod circuit;
mod update;

#[allow(dead_code)]
pub(crate) fn type_of<T>(_: &T) -> String {
    String::from(std::any::type_name::<T>())
}

use std::io::Write;

/* ------------------------------------------------------------------- */

fn main() -> Result<(), Box<dyn std::error::Error>> {
    use component::Component as Com;
    
    let mut comp = Com::<f32>::default();
    let mut rc = Com::<f32>::default();
    let mut rlc = Com::<f32>::default();

    rc.push_serie(Com::<f32>::new_r(1.));
    rc.push_serie(Com::<f32>::new_c(1., 0.));

    // rlc.push_serie(Com::<f32>::new_r(1.));
    rlc.push_serie(Com::<f32>::new_l(1.));
    rlc.push_serie(Com::<f32>::new_c(1., 0.)); 

    // comp.push_parallel(rc);
    comp.push_parallel(rlc);    
    
    let mut circuit = circuit::Circuit {
        dt: 0.01,
        source: 10.,
        charge: comp,
    };

    let mut values = Vec::<f32>::new();
    circuit.setup();
    
    for _ in 0..=10000 {
        values.push(circuit.charge.current);
        circuit.update();
    }

    let mut txt = std::fs::File::create("values.txt")?;
    txt.write_all(format!("{:?}", values).as_bytes())?;
    Ok(())
}
