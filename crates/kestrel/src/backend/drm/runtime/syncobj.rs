use super::LoopEvents;
use crate::backend::drm::DrmError;
use crate::{client::ClientState, state::KestrelState};
use calloop::EventLoop;
use smithay::reexports::wayland_server::DisplayHandle;

pub(super) fn clear_ready_syncobj_blockers(
    events: &mut LoopEvents,
    state: &mut KestrelState,
    dh: &DisplayHandle,
) {
    for client in events.syncobj_ready.drain(..) {
        let Some(client_state) = client.get_data::<ClientState>() else {
            continue;
        };
        client_state.compositor_state.blocker_cleared(state, dh);
        state.mark_scene_dirty();
    }
}

pub(super) fn register_syncobj_sources(
    state: &mut KestrelState,
    event_loop: &EventLoop<LoopEvents>,
) -> Result<(), DrmError> {
    for pending in state.take_syncobj_sources() {
        let client = pending.client;
        event_loop
            .handle()
            .insert_source(pending.source, move |(), _, events| {
                events.syncobj_ready.push(client.clone());
                Ok(())
            })
            .map_err(|error| {
                DrmError::Unsupported(format!(
                    "failed to register drm syncobj acquire source: {error}"
                ))
            })?;
    }

    Ok(())
}
