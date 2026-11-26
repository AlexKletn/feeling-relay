use super::Rotation;
use crate::Timestamped;

use super::RotationReference;

pub trait RotationPort {
    type Error;

    fn next_rotation(
        &mut self,
        reference: RotationReference,
    ) -> Result<Option<Timestamped<Rotation>>, Self::Error>;
}
