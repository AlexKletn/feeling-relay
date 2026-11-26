use std::time::Duration;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Timestamped<T> {
    pub timestamp: Duration,
    pub value: T,
}
