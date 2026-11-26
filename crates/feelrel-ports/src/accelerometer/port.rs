use crate::Timestamped;

use super::Acceleration;

pub trait AccelerometerPort {
    type Error;

    fn next_acceleration(&mut self) -> Result<Option<Timestamped<Acceleration>>, Self::Error>;
}
