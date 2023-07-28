/*-----------------------------------------------------------------------
|
|                           Circuit module
|
|    -> Generator declaration
|
|       This struct represents a real generator. It can be either a Thevenin
|       or a Norton one, provided a boolean that determines the generator type
|
|       `source` represents either the source current or the source tension,
|       depending on the value of `t_or_n` and `r` is the resistor associated
|
|    -> gen_cur and gen_tens defines
|       These methods return the equivalent Norton current source and
|       Thevenin tension source of the generator.
|
-----------------------------------------------------------------------*/

#[derive(Debug, Clone)]
pub(crate) struct Generator<T: num::Float + std::fmt::Debug> {
    pub(crate) r: T,         // equivalent resistance
    pub(crate) source: T,    // either current or tension
    pub(crate) t_or_n: bool, // true if Thevenin, false if Norton
}

impl<T: num::Float + std::fmt::Debug + std::ops::Mul + std::ops::Div> Generator<T> {
    pub(crate) fn gen_cur(&self) -> T {
        if self.t_or_n {
            self.source / self.r
        } else {
            self.source
        }
    }
    pub(crate) fn gen_tens(&self) -> T {
        if self.t_or_n {
            self.source
        } else {
            self.source * self.r
        }
    }
}
