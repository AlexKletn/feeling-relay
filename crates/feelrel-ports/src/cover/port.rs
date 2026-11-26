use crate::Timestamped;

use super::CoverState;

pub trait CoverPort {
    type Error;

    fn cover_state(&self) -> Result<Option<Timestamped<CoverState>>, Self::Error>;

    fn next_cover_change(&mut self) -> Result<Option<Timestamped<CoverState>>, Self::Error>;
}
