use crate::Acceleration;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GravitySample {
    pub gravity: Acceleration,
    pub linear_acceleration: Acceleration,
}
