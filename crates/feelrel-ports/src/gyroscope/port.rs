use crate::Timestamped;

use super::AngularVelocity;

pub trait GyroscopePort {
    type Error;

    fn next_angular_velocity(
        &mut self,
    ) -> Result<Option<Timestamped<AngularVelocity>>, Self::Error>;
}
