#![allow(unused_imports, dead_code)]

#[derive(Debug, Clone)]
pub(crate) struct Circuit<T: num::Float + std::fmt::Debug> {
    pub(crate) dt: T,
    pub(crate) source: T,
    pub(crate) charge: crate::component::Component<T>,
}

/*
impl Circuit<f32> {
    pub(crate) fn setup(&mut self) {
        self.charge.setup(self.source)
    }
}
impl Circuit<f64> {
    pub(crate) fn setup(&mut self) {
        self.charge.setup(self.source)
    }
}
*/