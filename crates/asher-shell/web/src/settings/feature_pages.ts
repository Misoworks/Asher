import type { SettingsPage, SettingsPatch, SettingsSnapshot } from "./model";

export type FeatureDefinition = {
  groups: FeatureGroup[];
};

export type FeatureGroup = {
  title: string;
  rows: FeatureRow[];
};

export type FeatureRow = {
  label: string;
  value?: string;
  detail?: string;
  control?: FeatureControl;
};

export type FeatureControl =
  | { type: "switch"; active: boolean; patch: SettingsPatch }
  | { type: "choices"; selected: string | number | null; options: FeatureChoice[] };

export type FeatureChoice = {
  label: string;
  value: string | number | null;
  patch: SettingsPatch;
};

const workspaceCounts = [1, 2, 3, 4, 6, 9, 12];
const idleLockChoices = [
  { label: "Off", value: null },
  { label: "5m", value: 300 },
  { label: "10m", value: 600 },
  { label: "15m", value: 900 },
  { label: "30m", value: 1800 },
];

export function featureDefinition(page: SettingsPage, settings: SettingsSnapshot): FeatureDefinition {
  switch (page) {
    case "bluetooth":
      return {
        groups: [
          {
            title: "Bluetooth",
            rows: [
              row("Adapter", "Unavailable", "No Bluetooth service data is available."),
              row("Devices", "No devices", "Paired and nearby devices appear here when detected."),
              row("Discoverable", "Off"),
              row("File sharing", "Off"),
            ],
          },
        ],
      };
    case "network":
      return {
        groups: [
          {
            title: "Connection",
            rows: [
              row("Active interface", settings.status.network?.name ?? "No active interface"),
              row("Type", settings.status.network?.wireless ? "Wi-Fi" : settings.status.network ? "Wired" : "Unavailable"),
              row("Wi-Fi networks", settings.status.network?.wireless ? "Connected" : "No wireless interface"),
              row("VPN", "Not configured"),
            ],
          },
          {
            title: "Advanced",
            rows: [
              row("Proxy", "Automatic"),
              row("Hotspot", settings.status.network?.wireless ? "Available" : "Requires Wi-Fi"),
              row("Quick Settings tile", settings.status.network ? "Visible" : "Hidden"),
            ],
          },
        ],
      };
    case "sound":
      return {
        groups: [
          {
            title: "Output",
            rows: [
              row("Volume", settings.status.audio ? `${settings.status.audio.percent}%` : "Unavailable"),
              row("Mute", settings.status.audio?.muted ? "On" : settings.status.audio ? "Off" : "Unavailable"),
              row("Output device", "Default sink"),
              row("Configuration", "System default"),
            ],
          },
          {
            title: "Input",
            rows: [
              row("Input device", "Default source"),
              row("Input level", "Unavailable"),
              row("Alert sound", "Default"),
              row("Per-app volume", "No streams reported"),
            ],
          },
        ],
      };
    case "keyboard":
      return {
        groups: [
          {
            title: "Input Sources",
            rows: [
              row("Current source", "System layout"),
              row("Switch source shortcut", "Super + Space"),
              row("Compose key", "Off"),
            ],
          },
          {
            title: "Typing",
            rows: [
              row("Repeat keys", "On"),
              row("Repeat delay", "System default"),
              row("Cursor blink", "On"),
            ],
          },
          {
            title: "Shortcuts",
            rows: [
              row("Overview", "Super"),
              row("Workspaces", "Configured in Kestrel"),
              row("Custom shortcuts", "Not configured"),
            ],
          },
        ],
      };
    case "mouse":
      return {
        groups: [
          {
            title: "Mouse",
            rows: [
              row("Primary button", "Left"),
              row("Pointer speed", "System default"),
              row("Acceleration", "Adaptive"),
              row("Scroll direction", "Traditional"),
            ],
          },
          {
            title: "Touchpad",
            rows: [
              row("Touchpad", "Detected by libinput"),
              row("Tap to click", "System default"),
              row("Disable while typing", "On"),
              row("Natural scrolling", "System default"),
            ],
          },
        ],
      };
    case "multitasking":
      return {
        groups: [
          {
            title: "Workspaces",
            rows: [
              {
                label: "Startup count",
                control: choices(
                  settings.workspaces.count,
                  workspaceCounts.map((count) => ({ label: String(count), value: count, patch: { workspaceCount: count } })),
                ),
              },
              {
                label: "Reopen windows after login",
                control: { type: "switch", active: settings.workspaces.restoreSessions, patch: { restoreSessions: !settings.workspaces.restoreSessions } },
              },
              row("Multi-monitor workspaces", settings.outputs.length > 1 ? "All displays" : "Primary display"),
            ],
          },
          {
            title: "Window Management",
            rows: [
              row("Edge resize", "On"),
              row("Panel scroll switching", "Kestrel workspace action"),
              row("App switcher scope", "Current workspace"),
            ],
          },
        ],
      };
    case "notifications":
      return {
        groups: [
          {
            title: "Notifications",
            rows: [
              row("Do Not Disturb", "Quick Settings"),
              row("Banners", "Top right"),
              row("Notification Center", "Date menu"),
              row("Lock screen notifications", "Hidden"),
            ],
          },
          {
            title: "Apps",
            rows: [
              row("Per-app notifications", "No app rules yet"),
              row("Sound alerts", "System default"),
              row("Critical alerts", "Allowed"),
            ],
          },
        ],
      };
    case "privacy":
      return {
        groups: [
          {
            title: "Screen Lock",
            rows: [
              {
                label: "Lock after idle",
                control: choices(
                  settings.session.idleLockSeconds ?? null,
                  idleLockChoices.map((choice) => ({ ...choice, patch: { idleLockSeconds: choice.value } })),
                ),
              },
              row("Show notifications", "Hidden"),
            ],
          },
          {
            title: "Permissions",
            rows: [
              row("Location", "Off"),
              row("Camera", "No permission data"),
              row("Microphone", "No permission data"),
              row("Thunderbolt", "No controller data"),
            ],
          },
          {
            title: "Device Security",
            rows: [
              row("Firmware security", "Unavailable"),
              row("Diagnostics reporting", "Off"),
              row("File history", "Off"),
            ],
          },
        ],
      };
    case "search":
      return {
        groups: [
          {
            title: "Overview Search",
            rows: [
              row("Applications", "On"),
              row("Settings", "On"),
              row("Files", "No index data"),
              row("Web suggestions", "Off"),
            ],
          },
          {
            title: "Providers",
            rows: [
              row("Launcher command", settings.defaultApps.launcher),
              row("Provider order", "Apps, Settings, Files"),
              row("Search locations", "Home"),
            ],
          },
        ],
      };
    case "accessibility":
      return {
        groups: [
          {
            title: "Seeing",
            rows: [
              row("High contrast", "Off"),
              row("Zoom", "Off"),
              row("Large text", "Off"),
              row("Cursor size", "Default"),
            ],
          },
          {
            title: "Hearing",
            rows: [row("Visual alerts", "Off"), row("Mono audio", "Off"), row("Sound keys", "Off")],
          },
          {
            title: "Typing & Pointing",
            rows: [row("Sticky keys", "Off"), row("Slow keys", "Off"), row("Mouse keys", "Off"), row("Locate pointer", "Off")],
          },
        ],
      };
    case "system":
      return {
        groups: [
          {
            title: "System",
            rows: [
              row("Date & time", "System clock"),
              row("Region & language", "System locale"),
              row("Users", "Local accounts"),
              row("Remote desktop", "Off"),
              row("Secure shell", "Off"),
            ],
          },
          {
            title: "Defaults",
            rows: [
              row("Terminal", settings.defaultApps.terminal),
              row("Files", settings.defaultApps.fileManager),
              row("Browser", settings.defaultApps.browser),
              row("Settings", settings.defaultApps.settings),
            ],
          },
        ],
      };
    case "wellbeing":
      return {
        groups: [
          {
            title: "Screen Time",
            rows: [
              row("Recording", "Off"),
              row("Daily limit", "Off"),
              row("Grayscale during limits", "Off"),
            ],
          },
          {
            title: "Break Reminders",
            rows: [
              row("Eyesight reminders", "Off"),
              row("Movement reminders", "Off"),
              row("Movement schedule", "Every hour"),
              row("Sounds", "Off"),
            ],
          },
        ],
      };
    default:
      return { groups: [] };
  }
}

function row(label: string, value: string, detail?: string): FeatureRow {
  return { label, value, detail };
}

function choices(selected: string | number | null, options: FeatureChoice[]): FeatureControl {
  return { type: "choices", selected, options };
}
