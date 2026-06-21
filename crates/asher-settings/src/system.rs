use serde::Serialize;
use std::{fs, path::Path, process::Command};

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsSystemStatus {
    pub battery: Option<SettingsBattery>,
    pub network: Option<SettingsNetwork>,
    pub audio: Option<SettingsAudio>,
    pub brightness: Option<SettingsBrightness>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsBattery {
    pub percent: u8,
    pub state: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsNetwork {
    pub name: String,
    pub wireless: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsAudio {
    pub percent: u8,
    pub muted: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsBrightness {
    pub percent: u8,
}

impl SettingsSystemStatus {
    pub fn read() -> Self {
        Self {
            battery: read_battery(),
            network: read_network(),
            audio: read_audio(),
            brightness: read_brightness(),
        }
    }
}

fn read_battery() -> Option<SettingsBattery> {
    let batteries = fs::read_dir("/sys/class/power_supply")
        .ok()?
        .filter_map(Result::ok)
        .filter(|entry| read_trimmed(entry.path().join("type")).as_deref() == Some("Battery"))
        .filter_map(|entry| battery_from_path(&entry.path()))
        .collect::<Vec<_>>();
    if batteries.is_empty() {
        return None;
    }
    let percent = batteries
        .iter()
        .map(|battery| battery.percent as u32)
        .sum::<u32>()
        / batteries.len() as u32;
    let state = batteries
        .iter()
        .find(|battery| battery.state.eq_ignore_ascii_case("charging"))
        .or_else(|| {
            batteries
                .iter()
                .find(|battery| battery.state.eq_ignore_ascii_case("discharging"))
        })
        .or_else(|| batteries.first())
        .map(|battery| battery.state.clone())
        .unwrap_or_else(|| "Unknown".to_string());
    Some(SettingsBattery {
        percent: percent.min(100) as u8,
        state,
    })
}

fn battery_from_path(path: &Path) -> Option<SettingsBattery> {
    Some(SettingsBattery {
        percent: read_trimmed(path.join("capacity"))?
            .parse::<u8>()
            .ok()?
            .min(100),
        state: read_trimmed(path.join("status")).unwrap_or_else(|| "Unknown".to_string()),
    })
}

fn read_network() -> Option<SettingsNetwork> {
    let entries = fs::read_dir("/sys/class/net").ok()?;
    let networks = entries
        .filter_map(Result::ok)
        .filter_map(|entry| network_from_path(&entry.path()))
        .collect::<Vec<_>>();
    networks
        .iter()
        .find(|network| network.wireless)
        .cloned()
        .or_else(|| networks.into_iter().next())
}

fn network_from_path(path: &Path) -> Option<SettingsNetwork> {
    let name = path.file_name()?.to_string_lossy().to_string();
    if name == "lo" {
        return None;
    }
    let state = read_trimmed(path.join("operstate"))?;
    if !matches!(state.as_str(), "up" | "unknown" | "dormant") {
        return None;
    }
    Some(SettingsNetwork {
        name,
        wireless: path.join("wireless").exists(),
    })
}

fn read_audio() -> Option<SettingsAudio> {
    let output = Command::new("wpctl")
        .args(["get-volume", "@DEFAULT_AUDIO_SINK@"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let value = text
        .split_whitespace()
        .find_map(|part| part.parse::<f32>().ok())?;
    Some(SettingsAudio {
        percent: (value * 100.0).round().clamp(0.0, 100.0) as u8,
        muted: text.contains("MUTED"),
    })
}

fn read_brightness() -> Option<SettingsBrightness> {
    fs::read_dir("/sys/class/backlight")
        .ok()?
        .filter_map(Result::ok)
        .filter_map(|entry| brightness_from_path(&entry.path()))
        .max_by_key(|brightness| brightness.percent)
}

fn brightness_from_path(path: &Path) -> Option<SettingsBrightness> {
    let current = read_trimmed(path.join("brightness"))?.parse::<u32>().ok()?;
    let max = read_trimmed(path.join("max_brightness"))?
        .parse::<u32>()
        .ok()?
        .max(1);
    Some(SettingsBrightness {
        percent: ((current * 100) / max).min(100) as u8,
    })
}

fn read_trimmed(path: impl AsRef<Path>) -> Option<String> {
    fs::read_to_string(path)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}
