use std::time::{Duration, Instant};

/// A stopwatch based on absolute monotonic timestamps, never frame counts.
#[derive(Default)]
pub struct Clock {
    accumulated: Duration,
    started: Option<Instant>,
}

impl Clock {
    pub fn running(&self) -> bool {
        self.started.is_some()
    }

    pub fn elapsed(&self, now: Instant) -> Duration {
        self.accumulated
            + self
                .started
                .map(|start| now.saturating_duration_since(start))
                .unwrap_or_default()
    }

    pub fn start(&mut self, now: Instant) {
        if self.started.is_none() {
            self.started = Some(now);
        }
    }

    pub fn pause(&mut self, now: Instant) {
        self.accumulated = self.elapsed(now);
        self.started = None;
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub fn finish_at(&mut self, duration: Duration) {
        self.accumulated = duration;
        self.started = None;
    }

    pub fn remaining(&self, target: Duration, now: Instant) -> Duration {
        target.saturating_sub(self.elapsed(now))
    }

    pub fn deadline(&self, target: Duration) -> Option<Instant> {
        self.started
            .map(|start| start + target.saturating_sub(self.accumulated))
    }
}

pub fn format_time(duration: Duration) -> String {
    let millis = duration.as_millis();
    format!(
        "{:02}:{:02}:{:02}.{:03}",
        millis / 3_600_000,
        (millis / 60_000) % 60,
        (millis / 1_000) % 60,
        millis % 1_000
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pause_and_resume_exclude_paused_time() {
        let origin = Instant::now();
        let mut clock = Clock::default();
        clock.start(origin);
        clock.pause(origin + Duration::from_millis(1_250));
        assert_eq!(
            clock.elapsed(origin + Duration::from_secs(20)),
            Duration::from_millis(1_250)
        );
        clock.start(origin + Duration::from_secs(20));
        assert_eq!(
            clock.elapsed(origin + Duration::from_millis(20_750)),
            Duration::from_secs(2)
        );
    }

    #[test]
    fn countdown_never_becomes_negative() {
        let origin = Instant::now();
        let mut clock = Clock::default();
        clock.start(origin);
        assert_eq!(
            clock.remaining(Duration::from_secs(5), origin + Duration::from_secs(50)),
            Duration::ZERO
        );
    }

    #[test]
    fn resumed_deadline_uses_remaining_duration() {
        let origin = Instant::now();
        let mut clock = Clock::default();
        clock.start(origin);
        clock.pause(origin + Duration::from_secs(3));
        clock.start(origin + Duration::from_secs(100));
        assert_eq!(
            clock.deadline(Duration::from_secs(10)),
            Some(origin + Duration::from_secs(107))
        );
    }

    #[test]
    fn repeated_start_does_not_lose_time() {
        let origin = Instant::now();
        let mut clock = Clock::default();
        clock.start(origin);
        clock.start(origin + Duration::from_secs(2));
        assert_eq!(
            clock.elapsed(origin + Duration::from_secs(3)),
            Duration::from_secs(3)
        );
    }

    #[test]
    fn long_interval_does_not_require_intermediate_updates() {
        let origin = Instant::now();
        let mut clock = Clock::default();
        clock.start(origin);
        let interval = Duration::from_millis(86_400_123);
        assert_eq!(clock.elapsed(origin + interval), interval);
        assert_eq!(format_time(interval), "24:00:00.123");
    }

    #[test]
    fn reset_clears_running_state_and_elapsed_time() {
        let origin = Instant::now();
        let mut clock = Clock::default();
        clock.start(origin);
        clock.reset();
        assert!(!clock.running());
        assert_eq!(
            clock.elapsed(origin + Duration::from_secs(10)),
            Duration::ZERO
        );
    }
}
