use asher_config::AsherConfig;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WebAppearance {
    pub material_mode: String,
    pub shell_mode: String,
    pub animations_enabled: bool,
    pub dock_icon_size: u16,
    pub dock_magnification: bool,
    pub taskbar_launcher: bool,
}

impl WebAppearance {
    pub fn from_config(config: &AsherConfig) -> Self {
        Self {
            material_mode: config.appearance.material_mode.as_str().to_string(),
            shell_mode: config.appearance.shell_mode.as_str().to_string(),
            animations_enabled: config.general.enable_animations && config.performance.animations,
            dock_icon_size: config.appearance.dock_icon_size,
            dock_magnification: config.appearance.dock_magnification,
            taskbar_launcher: config.appearance.taskbar_launcher,
        }
    }
}
