use super::Illuminance;
use crate::Timestamped;

pub trait IlluminancePort {
    type Error;

    fn next_illuminance(&mut self) -> Result<Option<Timestamped<Illuminance>>, Self::Error>;
}
