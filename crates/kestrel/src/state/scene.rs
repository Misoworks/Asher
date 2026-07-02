use super::KestrelState;
use crate::workspace_transition::{WorkspaceTransition, WorkspaceTransitionSnapshot};

impl KestrelState {
    pub fn workspace_transition(&self) -> Option<WorkspaceTransitionSnapshot> {
        self.workspace_transition
            .as_ref()
            .and_then(WorkspaceTransition::snapshot)
    }

    pub fn mark_scene_dirty(&mut self) {
        self.scene_dirty = true;
    }

    #[cfg(feature = "session-backend")]
    pub fn scene_dirty(&self) -> bool {
        self.scene_dirty
    }

    pub fn take_scene_dirty(&mut self) -> bool {
        let dirty = self.scene_dirty;
        self.scene_dirty = false;
        dirty
    }

    pub fn animations_active(&self) -> bool {
        self.windows.animations_active()
            || self
                .workspace_transition
                .as_ref()
                .is_some_and(WorkspaceTransition::is_active)
    }
}
