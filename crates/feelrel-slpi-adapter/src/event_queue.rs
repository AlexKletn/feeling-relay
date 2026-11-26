use std::collections::VecDeque;
use std::io;

const CAPACITY: usize = 256;

pub(super) struct EventQueue<T> {
    events: VecDeque<T>,
    dropped: usize,
}

impl<T> Default for EventQueue<T> {
    fn default() -> Self {
        Self {
            events: VecDeque::new(),
            dropped: 0,
        }
    }
}

impl<T> EventQueue<T> {
    pub fn push(&mut self, event: T) {
        if self.events.len() == CAPACITY {
            self.events.pop_front();
            self.dropped = self.dropped.saturating_add(1);
        }
        self.events.push_back(event);
    }

    pub fn next(&mut self) -> io::Result<Option<T>> {
        let dropped = std::mem::take(&mut self.dropped);
        if dropped != 0 {
            return Err(io::Error::other(format!(
                "SLPI event queue dropped {dropped} events"
            )));
        }
        Ok(self.events.pop_front())
    }
}

#[cfg(test)]
#[path = "event_queue.tests.rs"]
mod tests;
