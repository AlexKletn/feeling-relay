use super::GravitySample;
use crate::Timestamped;

pub trait GravityPort {
    type Error;

    fn next_gravity(&mut self) -> Result<Option<Timestamped<GravitySample>>, Self::Error>;
}
