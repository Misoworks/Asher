use super::model::{WebDockApp, WebShellSnapshot, WebWindow};

pub(crate) const QUICK_SETTINGS_WIDTH: i32 = 420;
pub(crate) const NOTIFICATION_TOAST_WIDTH: i32 = 380;
pub(crate) const NOTIFICATION_TOAST_HEIGHT: i32 = 136;
pub(crate) const DOCK_MENU_WIDTH: i32 = 184;

pub(crate) fn quick_settings_size(snapshot: &WebShellSnapshot) -> (i32, i32) {
    let status_tiles = 1
        + i32::from(snapshot.status.network.is_some())
        + i32::from(snapshot.status.battery.is_some());
    let status_rows = (status_tiles + 1) / 2;
    let sliders = i32::from(snapshot.status.audio.is_some())
        + i32::from(snapshot.status.brightness.is_some());
    let mut height = 32 + 36 + 13 + status_rows * 58 + (status_rows - 1) * 10;
    if sliders > 0 {
        height += 13 + sliders * 58 + (sliders - 1) * 10;
    }
    (QUICK_SETTINGS_WIDTH, height)
}

pub(crate) fn notification_toast_size() -> (i32, i32) {
    (NOTIFICATION_TOAST_WIDTH, NOTIFICATION_TOAST_HEIGHT)
}

pub(crate) fn dock_menu_size(snapshot: &WebShellSnapshot) -> (i32, i32) {
    let Some(command) = &snapshot.dock_menu_command else {
        return (DOCK_MENU_WIDTH, 128);
    };
    let Some(app) = snapshot
        .dock_apps
        .iter()
        .find(|entry| entry.command == *command)
    else {
        return (DOCK_MENU_WIDTH, 128);
    };
    let window = matched_window(app, &snapshot.windows);
    let action_count = if window.is_some() {
        let focus = i32::from(!window.is_some_and(|window| window.active));
        focus + 4
    } else if app.running {
        3
    } else {
        2
    };
    (DOCK_MENU_WIDTH, dock_menu_height(action_count))
}

fn dock_menu_height(action_count: i32) -> i32 {
    let content_items = action_count + 1;
    let content_height = 16 + 23 + action_count * 34 + (content_items - 1) * 4;
    content_height.clamp(128, 264)
}

fn matched_window<'a>(app: &WebDockApp, windows: &'a [WebWindow]) -> Option<&'a WebWindow> {
    windows
        .iter()
        .find(|window| window.active && window.visible && window_matches_app(window, app))
        .or_else(|| {
            windows
                .iter()
                .find(|window| window.visible && window_matches_app(window, app))
        })
        .or_else(|| {
            windows
                .iter()
                .find(|window| window_matches_app(window, app))
        })
}

fn window_matches_app(window: &WebWindow, app: &WebDockApp) -> bool {
    let command = command_name(&app.command);
    let label = app.label.to_lowercase();
    [window.app_id.as_deref(), Some(window.title.as_str())]
        .into_iter()
        .flatten()
        .map(str::to_lowercase)
        .any(|text| {
            !text.is_empty()
                && ((!command.is_empty() && text.contains(&command))
                    || (!label.is_empty() && text.contains(&label)))
        })
}

fn command_name(command: &str) -> String {
    command
        .split_whitespace()
        .next()
        .and_then(|value| value.rsplit('/').next())
        .unwrap_or_default()
        .trim_matches(['\'', '"'])
        .to_lowercase()
}
