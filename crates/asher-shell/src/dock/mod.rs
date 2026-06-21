mod app;
mod pins;
mod state;

pub use app::DockApp;
pub(crate) use pins::{pin_app, reorder_apps, unpin_app};
pub use state::{DockAppState, dock_app_matches_window};
