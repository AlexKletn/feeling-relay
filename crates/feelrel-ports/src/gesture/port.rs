use crate::Timestamped;

use super::Gesture;

pub trait GesturePort {
    type Error;

    fn next_gesture(&mut self) -> Result<Option<Timestamped<Gesture>>, Self::Error>;
}
