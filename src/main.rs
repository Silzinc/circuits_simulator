mod dipole;
mod initial;
use initial::*;

#[allow(dead_code)]
pub(crate) fn type_of<T>(_: &T) -> String {
    String::from(std::any::type_name::<T>())
}

/* ------------------------------------------------------------------------------------------------------------ */

fn main() {
    use Component as Com;
    let mut comp = Com::<f32>::default();
    let mut rc   = Com::<f32>::default();
    let mut rlc  = Com::<f32>::default();
    rc.push_serie(Com::<f32>::new_r(5.));
    rc.push_serie(Com::<f32>::new_c(72.));
    
    rlc.push_serie(Com::<f32>::new_r(50.));
    rlc.push_serie(Com::<f32>::new_l(40.));
    rlc.push_serie(Com::<f32>::new_c(25.));

    comp.push_parallel(rc);
    comp.push_parallel(rlc);

    comp.setup(100.);

    let mut comp2 = Com::<f64>::default();
    let mut rc2   = Com::<f64>::default();
    let mut rlc2  = Com::<f64>::default();
    rc2.push_serie(Com::<f64>::new_r(5.));
    rc2.push_serie(Com::<f64>::new_c(72.));
    
    rlc2.push_serie(Com::<f64>::new_r(50.));
    rlc2.push_serie(Com::<f64>::new_l(40.));
    rlc2.push_serie(Com::<f64>::new_c(25.));

    comp2.push_parallel(rc2);
    comp2.push_parallel(rlc2);

    comp2.setup(100.);

    assert_eq!(format!("{:?}", comp), format!("{:?}", comp2));
}
