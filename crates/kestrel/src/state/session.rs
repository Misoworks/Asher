#![cfg(feature = "session-backend")]

use super::KestrelState;
use smithay::{
    backend::{allocator::format::FormatSet, drm::DrmDeviceFd},
    reexports::wayland_server::Client,
    wayland::{
        dmabuf::DmabufFeedbackBuilder,
        drm_syncobj::{DrmSyncPointSource, DrmSyncobjState, supports_syncobj_eventfd},
    },
};
use tracing::debug;

impl KestrelState {
    pub fn enable_dmabuf(&mut self, main_device: u64, formats: FormatSet) {
        use smithay::backend::allocator::Format;

        if self.protocol_state.dmabuf_global.is_some() {
            return;
        }

        let advertised_formats = formats.iter().copied().collect::<Vec<Format>>();
        if advertised_formats.is_empty() {
            return;
        }

        let global = match DmabufFeedbackBuilder::new(main_device as _, advertised_formats.clone())
            .build()
        {
            Ok(feedback) => self
                .protocol_state
                .dmabuf
                .create_global_with_default_feedback::<Self>(&self.display_handle, &feedback),
            Err(error) => {
                tracing::warn!(%error, "failed to build dmabuf feedback; falling back to v3 dmabuf");
                self.protocol_state
                    .dmabuf
                    .create_global::<Self>(&self.display_handle, advertised_formats)
            }
        };
        self.protocol_state.dmabuf_global = Some(global);
        self.dmabuf_formats = formats;
    }

    pub fn enable_drm_syncobj(&mut self, import_device: DrmDeviceFd) {
        if let Some(syncobj) = self.protocol_state.drm_syncobj.as_mut() {
            syncobj.update_device(import_device);
            return;
        }

        if !supports_syncobj_eventfd(&import_device) {
            debug!("DRM device does not support syncobj eventfd; explicit sync disabled");
            return;
        }

        self.protocol_state.drm_syncobj = Some(DrmSyncobjState::new::<Self>(
            &self.display_handle,
            import_device,
        ));
        debug!("enabled linux-drm-syncobj explicit sync");
    }

    pub(crate) fn queue_syncobj_source(&mut self, client: Client, source: DrmSyncPointSource) {
        self.pending_syncobj_sources
            .push(PendingSyncobjSource { client, source });
    }

    pub(crate) fn take_syncobj_sources(&mut self) -> Vec<PendingSyncobjSource> {
        std::mem::take(&mut self.pending_syncobj_sources)
    }
}

pub(crate) struct PendingSyncobjSource {
    pub client: Client,
    pub source: DrmSyncPointSource,
}
