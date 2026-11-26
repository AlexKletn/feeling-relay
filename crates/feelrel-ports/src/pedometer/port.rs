use super::StepCount;
use crate::Timestamped;

pub trait PedometerPort {
    type Error;

    fn next_step_count(&mut self) -> Result<Option<Timestamped<StepCount>>, Self::Error>;
}
