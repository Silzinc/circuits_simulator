#[derive(Debug, Clone)]
pub(crate) struct Generator<T: num::Float + std::fmt::Debug> {
    pub(crate) r: T,        // equivalent resistance
    pub(crate) source: T,   // either current or tension
    pub(crate) t_or_n: bool // true if Thevenin, false if Norton
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