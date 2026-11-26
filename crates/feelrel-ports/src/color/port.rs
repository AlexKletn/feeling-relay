use super::ColorSample;
use crate::Timestamped;

pub trait ColorPort {
    type Error;

    fn next_color(&mut self) -> Result<Option<Timestamped<ColorSample>>, Self::Error>;
}
