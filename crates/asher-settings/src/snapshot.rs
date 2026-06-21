use crate::{dock::settings_dock_apps, system::SettingsSystemStatus};
use asher_config::{AsherConfig, BackendPreference, ConfigPaths, PerformanceMode};
use asher_ipc::{IpcRequest, IpcResponse, OutputSummary, send_request};
use asher_material::{MaterialPalette, glass_blur_wallpaper_path, shell_material_palette};
use serde::Serialize;
use std::{fmt::Write, os::unix::ffi::OsStrExt, path::Path};

pub(super) fn settings_snapshot(config: &AsherConfig) -> SettingsSnapshot {
    SettingsSnapshot {
        appearance: SettingsAppearance::from(config),
        wallpaper: SettingsWallpaper::from(config),
        palette: SettingsMaterialPalette::from(shell_material_palette(config)),
        dock: SettingsDock::from(config),
        general: SettingsGeneral::from(config),
        compositor: SettingsCompositor::from(config),
        performance: SettingsPerformance::from(config),
        workspaces: SettingsWorkspaces::from(config),
        recovery: SettingsRecovery::from(config),
        session: SettingsSession::from(config),
        display: SettingsDisplay::from(config),
        default_apps: SettingsDefaultApps::from(config),
        status: SettingsSystemStatus::read(),
        outputs: output_summaries(),
        paths: SettingsPaths::discover(),
    }
}

fn output_summaries() -> Vec<OutputSummary> {
    match send_request(&IpcRequest::ListOutputs) {
        Ok(IpcResponse::Outputs { outputs }) => outputs,
        _ => Vec::new(),
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct SettingsSnapshot {
    appearance: SettingsAppearance,
    wallpaper: SettingsWallpaper,
    palette: SettingsMaterialPalette,
    dock: SettingsDock,
    general: SettingsGeneral,
    compositor: SettingsCompositor,
    performance: SettingsPerformance,
    workspaces: SettingsWorkspaces,
    recovery: SettingsRecovery,
    session: SettingsSession,
    display: SettingsDisplay,
    default_apps: SettingsDefaultApps,
    status: SettingsSystemStatus,
    outputs: Vec<OutputSummary>,
    paths: SettingsPaths,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct SettingsGeneral {
    enable_effects: bool,
    safe_mode: bool,
}

impl From<&AsherConfig> for SettingsGeneral {
    fn from(config: &AsherConfig) -> Self {
        Self {
            enable_effects: config.general.enable_effects,
            safe_mode: config.general.safe_mode,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct SettingsCompositor {
    backend: String,
    xwayland: bool,
    debug_overlay: bool,
}

impl From<&AsherConfig> for SettingsCompositor {
    fn from(config: &AsherConfig) -> Self {
        Self {
            backend: backend_name(config.compositor.backend).to_string(),
            xwayland: config.compositor.xwayland,
            debug_overlay: config.compositor.debug_overlay,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct SettingsPerformance {
    reduce_effects_on_battery: bool,
}

impl From<&AsherConfig> for SettingsPerformance {
    fn from(config: &AsherConfig) -> Self {
        Self {
            reduce_effects_on_battery: config.performance.reduce_effects_on_battery,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct SettingsWorkspaces {
    count: u32,
    restore_sessions: bool,
}

impl From<&AsherConfig> for SettingsWorkspaces {
    fn from(config: &AsherConfig) -> Self {
        Self {
            count: config.workspaces.count,
            restore_sessions: config.workspaces.restore_sessions,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct SettingsRecovery {
    crash_limit: u32,
    crash_window_seconds: u64,
    auto_safe_mode: bool,
    backup_before_apply: bool,
}

impl From<&AsherConfig> for SettingsRecovery {
    fn from(config: &AsherConfig) -> Self {
        Self {
            crash_limit: config.recovery.crash_limit,
            crash_window_seconds: config.recovery.crash_window_seconds,
            auto_safe_mode: config.recovery.auto_safe_mode,
            backup_before_apply: config.recovery.backup_before_apply,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct SettingsSession {
    lock_command: String,
    suspend_command: String,
    reboot_command: String,
    poweroff_command: String,
    idle_lock_seconds: Option<u64>,
    idle_suspend_seconds: Option<u64>,
}

impl From<&AsherConfig> for SettingsSession {
    fn from(config: &AsherConfig) -> Self {
        Self {
            lock_command: config.session.lock_command.clone(),
            suspend_command: config.session.suspend_command.clone(),
            reboot_command: config.session.reboot_command.clone(),
            poweroff_command: config.session.poweroff_command.clone(),
            idle_lock_seconds: config.session.idle_lock_seconds,
            idle_suspend_seconds: config.session.idle_suspend_seconds,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct SettingsAppearance {
    material_mode: String,
    shell_mode: String,
    animations_enabled: bool,
    performance_mode: String,
    dock_icon_size: u16,
    dock_magnification: bool,
    taskbar_launcher: bool,
}

impl From<&AsherConfig> for SettingsAppearance {
    fn from(config: &AsherConfig) -> Self {
        Self {
            material_mode: config.appearance.material_mode.as_str().to_string(),
            shell_mode: config.appearance.shell_mode.as_str().to_string(),
            animations_enabled: config.general.enable_animations && config.performance.animations,
            performance_mode: performance_mode_name(config.performance.mode).to_string(),
            dock_icon_size: config.appearance.dock_icon_size,
            dock_magnification: config.appearance.dock_magnification,
            taskbar_launcher: config.appearance.taskbar_launcher,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct SettingsWallpaper {
    path: Option<String>,
    uri: Option<String>,
    glass_blur_uri: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct SettingsMaterialPalette {
    panel: String,
    panel_control: String,
    panel_text: String,
    dock: String,
    accent: String,
    text_soft: String,
    text_muted: String,
}

impl From<MaterialPalette> for SettingsMaterialPalette {
    fn from(value: MaterialPalette) -> Self {
        Self {
            panel: value.panel.css_rgba(),
            panel_control: value.panel_control.css_rgba(),
            panel_text: value.panel_text.css_rgba(),
            dock: value.dock.css_rgba(),
            accent: value.accent.css_rgba(),
            text_soft: value.text_soft.css_rgba(),
            text_muted: value.text_muted.css_rgba(),
        }
    }
}

impl From<&AsherConfig> for SettingsWallpaper {
    fn from(config: &AsherConfig) -> Self {
        let path = config
            .compositor
            .background_image
            .as_ref()
            .map(|path| path.to_string_lossy().into_owned());
        let uri = config
            .compositor
            .background_image
            .as_deref()
            .filter(|path| path.exists())
            .map(local_file_uri);
        let glass_blur_uri = glass_blur_wallpaper_path(config)
            .as_deref()
            .filter(|path| path.exists())
            .map(local_file_uri);
        Self {
            path,
            uri,
            glass_blur_uri,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct SettingsDock {
    customized: bool,
    apps: Vec<SettingsDockApp>,
}

impl From<&AsherConfig> for SettingsDock {
    fn from(config: &AsherConfig) -> Self {
        Self {
            customized: config.dock.customized,
            apps: settings_dock_apps(config)
                .into_iter()
                .map(|app| SettingsDockApp {
                    label: app.label,
                    command: app.command,
                })
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct SettingsDockApp {
    label: String,
    command: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct SettingsDisplay {
    default_scale: f64,
}

impl From<&AsherConfig> for SettingsDisplay {
    fn from(config: &AsherConfig) -> Self {
        Self {
            default_scale: config.display.default_scale,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct SettingsDefaultApps {
    terminal: String,
    file_manager: String,
    browser: String,
    settings: String,
    launcher: String,
}

impl From<&AsherConfig> for SettingsDefaultApps {
    fn from(config: &AsherConfig) -> Self {
        Self {
            terminal: config.default_apps.terminal.clone(),
            file_manager: config.default_apps.file_manager.clone(),
            browser: config.default_apps.browser.clone(),
            settings: config.default_apps.settings.clone(),
            launcher: config.default_apps.launcher.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct SettingsPaths {
    config_file: Option<String>,
}

impl SettingsPaths {
    fn discover() -> Self {
        Self {
            config_file: ConfigPaths::discover()
                .ok()
                .map(|paths| paths.config_file.to_string_lossy().into_owned()),
        }
    }
}

fn performance_mode_name(value: PerformanceMode) -> &'static str {
    match value {
        PerformanceMode::Quality => "quality",
        PerformanceMode::Balanced => "balanced",
        PerformanceMode::Performance => "performance",
        PerformanceMode::Battery => "battery",
    }
}

fn backend_name(value: BackendPreference) -> &'static str {
    match value {
        BackendPreference::Auto => "auto",
        BackendPreference::Nested => "nested",
        BackendPreference::Headless => "headless",
        BackendPreference::Session => "session",
    }
}

fn local_file_uri(path: &Path) -> String {
    let path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let mut uri = String::from("file://");
    for byte in path.as_os_str().as_bytes() {
        match *byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' | b'/' => {
                uri.push(*byte as char);
            }
            byte => {
                let _ = write!(uri, "%{byte:02X}");
            }
        }
    }
    uri
}
