use crate::output::DEFAULT_REFRESH_MILLIHERTZ;
use smithay::{
    backend::drm::{DrmEventMetadata, DrmEventTime},
    utils::{Clock, Monotonic, Time},
};
use std::time::{Duration, Instant};

pub(super) fn presentation_time(
    clock: &Clock<Monotonic>,
    metadata: Option<DrmEventMetadata>,
) -> (Option<(Time<Monotonic>, u64)>, Instant) {
    let Some(metadata) = metadata else {
        return (None, Instant::now());
    };
    let DrmEventTime::Monotonic(duration) = metadata.time else {
        return (None, Instant::now());
    };

    let time = Time::<Monotonic>::from(duration);
    let now = Instant::now();
    let age = Time::<Monotonic>::elapsed(&time, clock.now());
    let instant = now.checked_sub(age).unwrap_or(now);
    (Some((time, u64::from(metadata.sequence))), instant)
}

pub(super) fn refresh_interval(refresh_millihertz: i32) -> Duration {
    let refresh = u64::try_from(refresh_millihertz)
        .ok()
        .filter(|refresh| *refresh > 0)
        .unwrap_or(DEFAULT_REFRESH_MILLIHERTZ as u64);
    Duration::from_nanos((1_000_000_000_000u64 + refresh / 2) / refresh)
}
