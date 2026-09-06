use std::time::Duration;

const START: Duration = Duration::from_secs(2);
const CEILING: Duration = Duration::from_secs(60);

/// How long to wait before asking Bakong about a payment again.
///
/// The client never sleeps on your behalf, so a customer who leaves a QR on
/// screen for an hour cannot quietly burn the rate limit. Drive the loop
/// yourself and let this decide the gaps.
#[derive(Debug, Clone)]
pub struct Backoff {
    start: Duration,
    ceiling: Duration,
    attempt: u32,
}

impl Backoff {
    /// Doubles from two seconds up to a minute.
    pub fn new() -> Self {
        Self {
            start: START,
            ceiling: CEILING,
            attempt: 0,
        }
    }

    /// Doubles from `start` up to `ceiling`.
    pub fn with_bounds(start: Duration, ceiling: Duration) -> Self {
        Self {
            start,
            ceiling,
            attempt: 0,
        }
    }

    /// The next gap, longer each time until it reaches the ceiling.
    pub fn next_delay(&mut self) -> Duration {
        let delay = self
            .start
            .saturating_mul(2u32.saturating_pow(self.attempt.min(16)))
            .min(self.ceiling);

        self.attempt = self.attempt.saturating_add(1);
        delay
    }

    /// Starts the sequence over, after a payment lands or a new QR is shown.
    pub fn reset(&mut self) {
        self.attempt = 0;
    }
}

impl Default for Backoff {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delays_double_until_they_hit_the_ceiling() {
        let mut backoff = Backoff::new();

        assert_eq!(backoff.next_delay(), Duration::from_secs(2));
        assert_eq!(backoff.next_delay(), Duration::from_secs(4));
        assert_eq!(backoff.next_delay(), Duration::from_secs(8));
        assert_eq!(backoff.next_delay(), Duration::from_secs(16));
        assert_eq!(backoff.next_delay(), Duration::from_secs(32));
        assert_eq!(backoff.next_delay(), Duration::from_secs(60));
        assert_eq!(backoff.next_delay(), Duration::from_secs(60));
    }

    #[test]
    fn resetting_starts_over() {
        let mut backoff = Backoff::new();
        backoff.next_delay();
        backoff.next_delay();
        backoff.reset();

        assert_eq!(backoff.next_delay(), Duration::from_secs(2));
    }

    #[test]
    fn a_long_lived_poll_does_not_overflow() {
        let mut backoff = Backoff::new();

        for _ in 0..10_000 {
            assert!(backoff.next_delay() <= CEILING);
        }
    }
}
