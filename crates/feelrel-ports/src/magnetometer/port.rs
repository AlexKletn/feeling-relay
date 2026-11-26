use crate::Timestamped;

use super::MagneticField;

pub trait MagnetometerPort {
    type Error;

    fn next_magnetic_field(&mut self) -> Result<Option<Timestamped<MagneticField>>, Self::Error>;
}
