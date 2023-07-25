/*-----------------------------------------------------------------------
|
|                           Circuit module
|                           
|    -> Cicruit declaration
|       This struct represents a simulated circuit, made of a time step (dt),
|       a tension source (source) and a charging component (charge)
|
-----------------------------------------------------------------------*/

#[derive(Debug, Clone)]
pub(crate) struct Circuit<T: num::Float + std::fmt::Debug> {
    pub(crate) dt: T,
    pub(crate) source: T,
    pub(crate) charge: crate::component::Component<T>,
}