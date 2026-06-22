use crate::{
    dock::reorder_dock,
    ipc::{load_model, reload_config, set_workspace_profile},
    snapshot::settings_snapshot,
};
use asher_config::{
    AsherConfig, BackendPreference, MaterialModePreference, PerformanceMode, ShellModePreference,
    load_config, save_config,
};
use asher_layout::ProfileId;
use clap::ValueEnum;
use fenestra_cef::{
    BridgeCommand, BridgeCommandDescriptor, BridgeError, BridgeResponse, BridgeResult,
    FenestraWindow, RuntimeConfig, RuntimeMode, WebViewSecurity, WindowBackgroundEffect,
    WindowRegion, WindowRegionRect,
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

const WINDOW_WIDTH: u32 = 980;
const WINDOW_HEIGHT: u32 = 660;
const MIN_WINDOW_WIDTH: u32 = 820;
const MIN_WINDOW_HEIGHT: u32 = 560;
const SIDEBAR_WIDTH: i32 = 264;
const TITLEBAR_HEIGHT: i32 = 48;
const WINDOW_RADIUS: i32 = 18;

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum SettingsPage {
    Appearance,
    ControlCenter,
    Dock,
    Wallpaper,
    Display,
    Bluetooth,
    Network,
    Sound,
    Keyboard,
    Mouse,
    Multitasking,
    Notifications,
    Privacy,
    Search,
    Accessibility,
    Power,
    Apps,
    System,
    Wellbeing,
    About,
}

impl SettingsPage {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Appearance => "appearance",
            Self::ControlCenter => "control-center",
            Self::Dock => "dock",
            Self::Wallpaper => "wallpaper",
            Self::Display => "display",
            Self::Bluetooth => "bluetooth",
            Self::Network => "network",
            Self::Sound => "sound",
            Self::Keyboard => "keyboard",
            Self::Mouse => "mouse",
            Self::Multitasking => "multitasking",
            Self::Notifications => "notifications",
            Self::Privacy => "privacy",
            Self::Search => "search",
            Self::Accessibility => "accessibility",
            Self::Power => "power",
            Self::Apps => "apps",
            Self::System => "system",
            Self::Wellbeing => "wellbeing",
            Self::About => "about",
        }
    }
}

pub fn run(config: AsherConfig, page: SettingsPage) -> Result<(), String> {
    let process = build_window(config, page)
        .launch_or_install()
        .map_err(|error| error.to_string())?;
    process.wait().map_err(|error| error.to_string())?;
    Ok(())
}

fn build_window(config: AsherConfig, page: SettingsPage) -> FenestraWindow {
    let material = background_effect(config.appearance.material_mode);
    let state = Arc::new(Mutex::new(config));
    let mut window = FenestraWindow::new()
        .title("Asher Settings")
        .size(WINDOW_WIDTH, WINDOW_HEIGHT)
        .min_size(MIN_WINDOW_WIDTH, MIN_WINDOW_HEIGHT)
        .frameless()
        .resizable(false)
        .glass_material(material)
        .runtime(runtime_config())
        .security(WebViewSecurity::default())
        .blur_region(WindowRegion::adaptive_rounded_left(
            SIDEBAR_WIDTH,
            WINDOW_RADIUS,
        ))
        .input_region(WindowRegion::adaptive_rounded_rect(WINDOW_RADIUS))
        .drag_region(WindowRegionRect::new(210, 0, SIDEBAR_WIDTH - 210, 92))
        .drag_region(WindowRegionRect::new(
            SIDEBAR_WIDTH + 280,
            0,
            i32::MAX,
            TITLEBAR_HEIGHT,
        ));

    window = register_bridge(window, state, page);
    match settings_entry(page) {
        SettingsEntry::Dev(url) => window.dev_url(url),
        SettingsEntry::File(path) => window.entry(path),
    }
}

fn background_effect(material: MaterialModePreference) -> WindowBackgroundEffect {
    match material {
        MaterialModePreference::Glass => WindowBackgroundEffect::Glass,
    }
}

fn register_bridge(
    mut window: FenestraWindow,
    state: Arc<Mutex<AsherConfig>>,
    page: SettingsPage,
) -> FenestraWindow {
    let ready_state = Arc::clone(&state);
    window = window.bridge_descriptor_handler(
        BridgeCommandDescriptor::new("asher.ready").target("desktop"),
        move |_| {
            let config = locked_config(&ready_state)?;
            json_ok(serde_json::json!({
                "surface": "settings",
                "snapshot": null,
                "settings": settings_snapshot(&config),
                "page": page.as_str(),
            }))
        },
    );

    let load_state = Arc::clone(&state);
    window = window.bridge_descriptor_handler(
        BridgeCommandDescriptor::new("asher.settings.load").target("desktop"),
        move |_| {
            if let Ok(loaded) = load_config()
                && let Ok(mut config) = load_state.lock()
            {
                *config = loaded.config;
            }
            let config = locked_config(&load_state)?;
            json_ok(settings_snapshot(&config))
        },
    );

    let apply_state = Arc::clone(&state);
    window = window.bridge_descriptor_handler(
        BridgeCommandDescriptor::new("asher.settings.apply").target("desktop"),
        move |command| {
            let patch: SettingsPatch = params(&command)?;
            let mut config = locked_config(&apply_state)?;
            let profile = apply_patch(&mut config, patch);
            save_config(&config).map_err(|error| BridgeError::new(error.to_string()))?;
            if let Ok(mut current) = apply_state.lock() {
                *current = config.clone();
            }
            let _ = reload_config();
            if let Some(profile) = profile {
                apply_active_profile(profile);
            }
            json_ok(settings_snapshot(&config))
        },
    );

    window = window.bridge_descriptor_handler(
        BridgeCommandDescriptor::new("asher.settings.pick-wallpaper").target("desktop"),
        move |_| pick_wallpaper(),
    );

    window
}

fn pick_wallpaper() -> BridgeResult {
    let path = rfd::FileDialog::new()
        .set_title("Choose wallpaper")
        .add_filter("Images", &["png", "jpg", "jpeg", "webp"])
        .pick_file()
        .map(|path| path.to_string_lossy().into_owned());
    json_ok(serde_json::json!({ "path": path }))
}

fn apply_patch(config: &mut AsherConfig, patch: SettingsPatch) -> Option<ShellModePreference> {
    if let Some(value) = patch.material_mode.and_then(material_mode) {
        config.appearance.material_mode = value;
    }
    if let Some(value) = patch.enable_effects {
        config.general.enable_effects = value;
    }
    if let Some(value) = patch.safe_mode {
        config.general.safe_mode = value;
    }
    if let Some(value) = patch.backend.and_then(backend_preference) {
        config.compositor.backend = value;
    }
    if let Some(value) = patch.xwayland {
        config.compositor.xwayland = value;
    }
    if let Some(value) = patch.debug_overlay {
        config.compositor.debug_overlay = value;
    }
    config.general.enable_blur = true;
    config.effects.blur = true;
    if let Some(value) = patch.animations_enabled {
        config.general.enable_animations = value;
        config.performance.animations = value;
    }
    config.effects.background_effect_protocol = true;
    if let Some(value) = patch.performance_mode.and_then(performance_mode) {
        config.performance.mode = value;
    }
    if let Some(value) = patch.reduce_effects_on_battery {
        config.performance.reduce_effects_on_battery = value;
    }
    if let Some(value) = patch.default_scale {
        config.display.default_scale = value.clamp(0.5, 4.0);
    }
    if let Some(value) = patch.dock_icon_size {
        config.appearance.dock_icon_size = value.clamp(32, 64);
    }
    if let Some(value) = patch.dock_magnification {
        config.appearance.dock_magnification = value;
    }
    if let Some(value) = patch.taskbar_launcher {
        config.appearance.taskbar_launcher = value;
    }
    if let Some(value) = patch.wallpaper_path {
        config.compositor.background_image =
            value.and_then(|path| (!path.trim().is_empty()).then(|| PathBuf::from(path.trim())));
    }
    if let Some(commands) = patch.pinned_commands {
        reorder_dock(config, commands);
    }
    if let Some(value) = patch.workspace_count {
        config.workspaces.count = value.clamp(1, 12);
    }
    if let Some(value) = patch.restore_sessions {
        config.workspaces.restore_sessions = value;
    }
    if let Some(value) = patch.crash_limit {
        config.recovery.crash_limit = value.clamp(1, 12);
    }
    if let Some(value) = patch.crash_window_seconds {
        config.recovery.crash_window_seconds = value.clamp(10, 600);
    }
    if let Some(value) = patch.auto_safe_mode {
        config.recovery.auto_safe_mode = value;
    }
    if let Some(value) = patch.backup_before_apply {
        config.recovery.backup_before_apply = value;
    }
    if let Some(value) = patch.default_terminal {
        config.default_apps.terminal = cleaned_command(value, &config.default_apps.terminal);
    }
    if let Some(value) = patch.default_file_manager {
        config.default_apps.file_manager =
            cleaned_command(value, &config.default_apps.file_manager);
    }
    if let Some(value) = patch.default_browser {
        config.default_apps.browser = cleaned_command(value, &config.default_apps.browser);
    }
    if let Some(value) = patch.default_launcher {
        config.default_apps.launcher = cleaned_command(value, &config.default_apps.launcher);
    }
    if let Some(value) = patch.lock_command {
        config.session.lock_command = cleaned_command(value, &config.session.lock_command);
    }
    if let Some(value) = patch.suspend_command {
        config.session.suspend_command = cleaned_command(value, &config.session.suspend_command);
    }
    if let Some(value) = patch.reboot_command {
        config.session.reboot_command = cleaned_command(value, &config.session.reboot_command);
    }
    if let Some(value) = patch.poweroff_command {
        config.session.poweroff_command = cleaned_command(value, &config.session.poweroff_command);
    }
    if let Some(value) = patch.idle_lock_seconds {
        config.session.idle_lock_seconds = clean_idle_seconds(value);
    }
    if let Some(value) = patch.idle_suspend_seconds {
        config.session.idle_suspend_seconds = clean_idle_seconds(value);
    }

    patch.shell_mode.and_then(shell_mode).inspect(|mode| {
        config.appearance.shell_mode = *mode;
        config.general.default_profile = mode.profile_id().to_string();
    })
}

fn apply_active_profile(mode: ShellModePreference) {
    let Ok(model) = load_model() else {
        return;
    };
    let _ = set_workspace_profile(
        model.active_workspace,
        ProfileId(mode.profile_id().to_string()),
    );
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SettingsPatch {
    material_mode: Option<String>,
    shell_mode: Option<String>,
    enable_effects: Option<bool>,
    safe_mode: Option<bool>,
    backend: Option<String>,
    xwayland: Option<bool>,
    debug_overlay: Option<bool>,
    animations_enabled: Option<bool>,
    performance_mode: Option<String>,
    reduce_effects_on_battery: Option<bool>,
    default_scale: Option<f64>,
    dock_icon_size: Option<u16>,
    dock_magnification: Option<bool>,
    taskbar_launcher: Option<bool>,
    wallpaper_path: Option<Option<String>>,
    pinned_commands: Option<Vec<String>>,
    workspace_count: Option<u32>,
    restore_sessions: Option<bool>,
    crash_limit: Option<u32>,
    crash_window_seconds: Option<u64>,
    auto_safe_mode: Option<bool>,
    backup_before_apply: Option<bool>,
    default_terminal: Option<String>,
    default_file_manager: Option<String>,
    default_browser: Option<String>,
    default_launcher: Option<String>,
    lock_command: Option<String>,
    suspend_command: Option<String>,
    reboot_command: Option<String>,
    poweroff_command: Option<String>,
    idle_lock_seconds: Option<Option<u64>>,
    idle_suspend_seconds: Option<Option<u64>>,
}

fn params<T: DeserializeOwned>(command: &BridgeCommand) -> Result<T, BridgeError> {
    serde_json::from_value(command.params.clone())
        .map_err(|error| BridgeError::new(format!("invalid {} params: {error}", command.name)))
}

fn material_mode(value: String) -> Option<MaterialModePreference> {
    match value.as_str() {
        "glass" => Some(MaterialModePreference::Glass),
        _ => None,
    }
}

fn locked_config(state: &Arc<Mutex<AsherConfig>>) -> Result<AsherConfig, BridgeError> {
    state
        .lock()
        .map(|config| config.clone())
        .map_err(|_| BridgeError::new("failed to read settings state"))
}

fn json_ok<T: Serialize>(value: T) -> BridgeResult {
    serde_json::to_value(value)
        .map(BridgeResponse::json)
        .map_err(|error| BridgeError::new(error.to_string()))
}

fn shell_mode(value: String) -> Option<ShellModePreference> {
    match value.as_str() {
        "panel" => Some(ShellModePreference::Panel),
        "dock" => Some(ShellModePreference::Dock),
        "tiling" => Some(ShellModePreference::Tiling),
        "focus" => Some(ShellModePreference::Focus),
        "browser" => Some(ShellModePreference::Browser),
        _ => None,
    }
}

fn performance_mode(value: String) -> Option<PerformanceMode> {
    match value.as_str() {
        "quality" => Some(PerformanceMode::Quality),
        "balanced" => Some(PerformanceMode::Balanced),
        "performance" => Some(PerformanceMode::Performance),
        "battery" => Some(PerformanceMode::Battery),
        _ => None,
    }
}

fn backend_preference(value: String) -> Option<BackendPreference> {
    match value.as_str() {
        "auto" => Some(BackendPreference::Auto),
        "nested" => Some(BackendPreference::Nested),
        "headless" => Some(BackendPreference::Headless),
        "session" => Some(BackendPreference::Session),
        _ => None,
    }
}

fn cleaned_command(value: String, fallback: &str) -> String {
    let value = value.trim();
    if value.is_empty() {
        fallback.to_string()
    } else {
        value.to_string()
    }
}

fn clean_idle_seconds(value: Option<u64>) -> Option<u64> {
    value
        .filter(|seconds| *seconds > 0)
        .map(|seconds| seconds.clamp(60, 86_400))
}

enum SettingsEntry {
    Dev(String),
    File(String),
}

fn settings_entry(page: SettingsPage) -> SettingsEntry {
    if let Ok(url) = std::env::var("ASHER_SHELL_WEB_DEV_URL") {
        return SettingsEntry::Dev(append_settings_query(url.trim_end_matches('/'), page));
    }
    SettingsEntry::File(append_settings_query(
        &workspace_root()
            .join("crates/asher-shell/web/dist/index.html")
            .display()
            .to_string(),
        page,
    ))
}

fn append_settings_query(base: &str, page: SettingsPage) -> String {
    let separator = if base.contains('?') { '&' } else { '?' };
    format!(
        "{base}{separator}surface=settings&settingsPage={}&fenestra=1",
        page.as_str()
    )
}

fn runtime_config() -> RuntimeConfig {
    RuntimeConfig {
        mode: RuntimeMode::SharedPreferred,
        allow_user_install: true,
        bundled_dir: Some(workspace_root()),
        ..RuntimeConfig::default()
    }
}

fn workspace_root() -> PathBuf {
    manifest_dir()
        .parent()
        .and_then(|path| path.parent())
        .map(PathBuf::from)
        .unwrap_or_else(manifest_dir)
}

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}
