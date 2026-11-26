use std::time::Duration;

use tokio::time::Instant;

pub(super) struct SampleDeadline {
    interval: Duration,
    deadline: Instant,
}

impl SampleDeadline {
    pub fn new(now: Instant, interval: Duration) -> Self {
        Self {
            interval,
            deadline: now + interval,
        }
    }

    pub fn expires_at(&self) -> Instant {
        self.deadline
    }

    pub fn record_sample(&mut self, now: Instant) {
        self.deadline = now + self.interval;
    }
}

#[cfg(test)]
#[path = "sample_deadline.tests.rs"]
mod tests;
