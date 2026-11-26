use super::Motion;
use crate::Timestamped;

pub trait MotionPort {
    type Error;

    fn next_motion(&mut self) -> Result<Option<Timestamped<Motion>>, Self::Error>;
}
