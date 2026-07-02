use crate::{shell::ShellProcess, xwayland::XwaylandSatellite};
use std::time::{Duration, Instant};

pub(super) fn process_timeout(
    now: Instant,
    fallback: Duration,
    shell: &ShellProcess,
    xwayland: &XwaylandSatellite,
) -> Duration {
    [
        shell.next_restart_deadline(),
        xwayland.next_restart_deadline(),
    ]
    .into_iter()
    .flatten()
    .min()
    .map(|deadline| deadline.saturating_duration_since(now).min(fallback))
    .unwrap_or(fallback)
}
