use asher_config::{AsherConfig, PinnedAppConfig};

#[derive(Debug, Clone)]
pub struct SettingsDockApp {
    pub label: String,
    pub command: String,
}

pub fn settings_dock_apps(config: &AsherConfig) -> Vec<SettingsDockApp> {
    if config.dock.customized || !config.dock.pinned.is_empty() {
        return config
            .dock
            .pinned
            .iter()
            .map(|app| SettingsDockApp {
                label: app.label.clone(),
                command: app.command.clone(),
            })
            .collect();
    }

    vec![
        SettingsDockApp {
            label: "Terminal".to_string(),
            command: config.default_apps.terminal.clone(),
        },
        SettingsDockApp {
            label: "Files".to_string(),
            command: config.default_apps.file_manager.clone(),
        },
        SettingsDockApp {
            label: "Browser".to_string(),
            command: config.default_apps.browser.clone(),
        },
        SettingsDockApp {
            label: "Settings".to_string(),
            command: config.default_apps.settings.clone(),
        },
    ]
}

pub fn reorder_dock(config: &mut AsherConfig, commands: Vec<String>) {
    let current = settings_dock_apps(config);
    let mut pins = Vec::new();
    for command in commands {
        if let Some(app) = current.iter().find(|app| app.command == command) {
            pins.push(PinnedAppConfig {
                label: app.label.clone(),
                command: app.command.clone(),
                icon: None,
            });
        }
    }
    if !pins.is_empty() {
        config.dock.customized = true;
        config.dock.pinned = pins;
    }
}
