use super::Orientation;
use crate::Timestamped;

pub trait OrientationPort {
    type Error;

    fn next_orientation(&mut self) -> Result<Option<Timestamped<Orientation>>, Self::Error>;
}
