use super::KestrelState;
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;

impl KestrelState {
    pub fn frame_callback_surfaces(&self) -> Vec<WlSurface> {
        let mut workspaces = vec![self.layout.active_workspace().clone()];
        if let Some(transition) = self.workspace_transition() {
            workspaces.push(transition.from);
            workspaces.push(transition.to);
        }

        let mut surfaces = Vec::new();
        for workspace in workspaces {
            for surface in self.windows.visible_surfaces_for_workspace(&workspace) {
                push_unique_surface(&mut surfaces, surface);
            }
        }
        for surface in self.layer_surfaces() {
            push_unique_surface(&mut surfaces, surface);
        }
        surfaces
    }
}

fn push_unique_surface(surfaces: &mut Vec<WlSurface>, surface: WlSurface) {
    if !surfaces.iter().any(|current| current == &surface) {
        surfaces.push(surface);
    }
}
