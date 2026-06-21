use super::DockApp;
use asher_config::{AsherConfig, PinnedAppConfig};

pub fn pin_app(
    config: &mut AsherConfig,
    current: &[DockApp],
    label: String,
    command: String,
    icon: Option<String>,
) -> bool {
    let command = command.trim().to_string();
    if command.is_empty() {
        return false;
    }

    materialize(config, current);
    let pin = PinnedAppConfig {
        label: clean_label(label, &command),
        command,
        icon: clean_icon(icon),
    };

    match config
        .dock
        .pinned
        .iter()
        .position(|app| commands_equal(&app.command, &pin.command))
    {
        Some(index) => config.dock.pinned[index] = pin,
        None => config.dock.pinned.push(pin),
    }

    true
}

pub fn unpin_app(config: &mut AsherConfig, current: &[DockApp], command: &str) -> bool {
    materialize(config, current);
    let before = config.dock.pinned.len();
    config
        .dock
        .pinned
        .retain(|pin| !commands_equal(&pin.command, command));
    before != config.dock.pinned.len()
}

pub fn reorder_apps(config: &mut AsherConfig, current: &[DockApp], commands: Vec<String>) -> bool {
    if commands.is_empty() {
        return false;
    }
    materialize(config, current);

    let before = config.dock.pinned.clone();
    let mut remaining = before.clone();
    let mut next = Vec::new();
    for command in commands {
        if let Some(index) = remaining
            .iter()
            .position(|pin| commands_equal(&pin.command, &command))
        {
            next.push(remaining.remove(index));
        }
    }
    next.extend(remaining);
    if next == before {
        return false;
    }
    config.dock.pinned = next;
    true
}

fn materialize(config: &mut AsherConfig, current: &[DockApp]) {
    if config.dock.customized || !config.dock.pinned.is_empty() {
        config.dock.customized = true;
        return;
    }

    config.dock.pinned = current
        .iter()
        .map(|app| PinnedAppConfig {
            label: app.label.clone(),
            command: app.command.clone(),
            icon: app
                .icon_path
                .as_ref()
                .map(|path| path.to_string_lossy().into_owned()),
        })
        .collect();
    config.dock.customized = true;
}

fn clean_label(label: String, command: &str) -> String {
    let label = label.trim();
    if label.is_empty() {
        command
            .split_whitespace()
            .next()
            .unwrap_or(command)
            .to_string()
    } else {
        label.to_string()
    }
}

fn clean_icon(icon: Option<String>) -> Option<String> {
    icon.map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn commands_equal(left: &str, right: &str) -> bool {
    left.trim() == right.trim()
}
