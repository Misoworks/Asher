use std::time::{Duration, Instant};

const NORMAL_REFRESH_SAFETY_MARGIN: Duration = Duration::from_micros(900);
const HIGH_REFRESH_SAFETY_MARGIN: Duration = Duration::from_micros(1_500);
const HIGH_REFRESH_INTERVAL: Duration = Duration::from_millis(9);
const MIN_RENDER_ESTIMATE: Duration = Duration::from_micros(500);
const RENDER_ESTIMATE_DECAY: u128 = 8;

#[derive(Debug)]
pub struct RenderScheduler {
    refresh_interval: Duration,
    render_estimate: Duration,
    safety_margin: Duration,
    last_presentation: Option<Instant>,
    scheduled_render: Option<Instant>,
}

impl RenderScheduler {
    pub fn new(refresh_interval: Duration) -> Self {
        Self {
            refresh_interval,
            render_estimate: MIN_RENDER_ESTIMATE,
            safety_margin: safety_margin(refresh_interval),
            last_presentation: None,
            scheduled_render: None,
        }
    }

    pub fn set_refresh_interval(&mut self, refresh_interval: Duration) {
        self.refresh_interval = refresh_interval;
        self.safety_margin = safety_margin(refresh_interval);
        self.scheduled_render = None;
    }

    pub fn request_repaint(&mut self, now: Instant) {
        let render_at = self.next_render_time(now);
        self.scheduled_render = Some(match self.scheduled_render {
            Some(current) => current.min(render_at),
            None => render_at,
        });
    }

    pub fn cancel_repaint(&mut self) {
        self.scheduled_render = None;
    }

    pub fn frame_presented(&mut self, now: Instant) {
        self.last_presentation = Some(now);
    }

    pub fn frame_rendered(&mut self, render_time: Duration) {
        self.scheduled_render = None;
        self.render_estimate = if render_time > self.render_estimate {
            render_time
        } else {
            weighted_duration(self.render_estimate, render_time)
        }
        .max(MIN_RENDER_ESTIMATE);
    }

    pub fn should_render(&self, now: Instant) -> bool {
        self.scheduled_render
            .is_some_and(|render_at| now >= render_at)
    }

    pub fn dispatch_timeout(&self, now: Instant, idle_timeout: Duration) -> Duration {
        let Some(render_at) = self.scheduled_render else {
            return idle_timeout;
        };
        render_at.saturating_duration_since(now).min(idle_timeout)
    }

    fn next_render_time(&self, now: Instant) -> Instant {
        let Some(last_presentation) = self.last_presentation else {
            return now;
        };
        let budget = self.render_budget();
        let earliest_presentation = now + budget;
        let target_presentation = self.presentation_after(last_presentation, earliest_presentation);
        target_presentation
            .checked_sub(budget)
            .unwrap_or(now)
            .max(now)
    }

    fn render_budget(&self) -> Duration {
        self.render_estimate
            .saturating_add(self.safety_margin)
            .min(duration_mul(self.refresh_interval, 2))
    }

    fn presentation_after(&self, anchor: Instant, earliest: Instant) -> Instant {
        if earliest <= anchor {
            return anchor + self.refresh_interval;
        }

        let elapsed = earliest.duration_since(anchor);
        let interval = self.refresh_interval.as_nanos().max(1);
        let intervals = div_ceil(elapsed.as_nanos(), interval).max(1);
        anchor + duration_mul_u128(self.refresh_interval, intervals)
    }
}

fn safety_margin(refresh_interval: Duration) -> Duration {
    if refresh_interval <= HIGH_REFRESH_INTERVAL {
        HIGH_REFRESH_SAFETY_MARGIN
    } else {
        NORMAL_REFRESH_SAFETY_MARGIN
    }
}

fn weighted_duration(current: Duration, sample: Duration) -> Duration {
    duration_from_nanos(
        ((current.as_nanos() * (RENDER_ESTIMATE_DECAY - 1)) + sample.as_nanos())
            / RENDER_ESTIMATE_DECAY,
    )
}

fn duration_mul(duration: Duration, factor: u32) -> Duration {
    duration_mul_u128(duration, u128::from(factor))
}

fn duration_mul_u128(duration: Duration, factor: u128) -> Duration {
    duration_from_nanos(duration.as_nanos().saturating_mul(factor))
}

fn duration_from_nanos(nanos: u128) -> Duration {
    Duration::from_nanos(nanos.min(u128::from(u64::MAX)) as u64)
}

fn div_ceil(value: u128, divisor: u128) -> u128 {
    value.saturating_add(divisor - 1) / divisor
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_repaint_is_immediate() {
        let now = Instant::now();
        let mut scheduler = RenderScheduler::new(Duration::from_micros(6_944));

        scheduler.request_repaint(now);

        assert!(scheduler.should_render(now));
    }

    #[test]
    fn repaint_after_presentation_waits_for_render_deadline() {
        let now = Instant::now();
        let mut scheduler = RenderScheduler::new(Duration::from_micros(6_944));
        scheduler.frame_presented(now);
        scheduler.request_repaint(now);

        assert!(!scheduler.should_render(now));
        assert!(
            scheduler.dispatch_timeout(now, Duration::from_millis(16)) < Duration::from_millis(16)
        );
    }

    #[test]
    fn slow_render_moves_deadline_earlier() {
        let now = Instant::now();
        let refresh = Duration::from_micros(6_944);
        let mut fast = RenderScheduler::new(refresh);
        fast.frame_presented(now);
        fast.request_repaint(now);

        let mut slow = RenderScheduler::new(refresh);
        slow.frame_presented(now);
        slow.frame_rendered(Duration::from_millis(5));
        slow.request_repaint(now);

        assert!(
            slow.dispatch_timeout(now, Duration::from_millis(16))
                < fast.dispatch_timeout(now, Duration::from_millis(16))
        );
    }
}
