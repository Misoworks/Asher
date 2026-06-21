export type SettingsPage =
  | "appearance"
  | "control-center"
  | "dock"
  | "wallpaper"
  | "display"
  | "bluetooth"
  | "network"
  | "sound"
  | "keyboard"
  | "mouse"
  | "multitasking"
  | "notifications"
  | "privacy"
  | "search"
  | "accessibility"
  | "power"
  | "apps"
  | "system"
  | "wellbeing"
  | "about";

export type MaterialMode = "glass";

export type SettingsSnapshot = {
  appearance: SettingsAppearance;
  wallpaper: SettingsWallpaper;
  palette: SettingsPalette;
  dock: SettingsDock;
  general: SettingsGeneral;
  compositor: SettingsCompositor;
  performance: SettingsPerformance;
  workspaces: SettingsWorkspaces;
  recovery: SettingsRecovery;
  session: SettingsSession;
  display: SettingsDisplay;
  defaultApps: SettingsDefaultApps;
  status: SettingsSystemStatus;
  outputs: SettingsOutput[];
  paths: SettingsPaths;
};

export type SettingsAppearance = {
  materialMode: MaterialMode;
  shellMode: string;
  animationsEnabled: boolean;
  performanceMode: string;
  dockIconSize: number;
  dockMagnification: boolean;
  taskbarLauncher: boolean;
};

export type SettingsGeneral = {
  enableEffects: boolean;
  safeMode: boolean;
};

export type SettingsCompositor = {
  backend: string;
  xwayland: boolean;
  debugOverlay: boolean;
};

export type SettingsPerformance = {
  reduceEffectsOnBattery: boolean;
};

export type SettingsWorkspaces = {
  count: number;
  restoreSessions: boolean;
};

export type SettingsRecovery = {
  crashLimit: number;
  crashWindowSeconds: number;
  autoSafeMode: boolean;
  backupBeforeApply: boolean;
};

export type SettingsSession = {
  lockCommand: string;
  suspendCommand: string;
  rebootCommand: string;
  poweroffCommand: string;
  idleLockSeconds?: number;
  idleSuspendSeconds?: number;
};

export type SettingsWallpaper = {
  path?: string;
  uri?: string;
  glassBlurUri?: string;
};

export type SettingsPalette = {
  panel: string;
  panelControl: string;
  panelText: string;
  dock: string;
  accent: string;
  textSoft: string;
  textMuted: string;
};

export type SettingsDock = {
  customized: boolean;
  apps: SettingsDockApp[];
};

export type SettingsDockApp = {
  label: string;
  command: string;
};

export type SettingsDisplay = {
  defaultScale: number;
};

export type SettingsDefaultApps = {
  terminal: string;
  fileManager: string;
  browser: string;
  settings: string;
  launcher: string;
};

export type SettingsSystemStatus = {
  battery?: { percent: number; state: string };
  network?: { name: string; wireless: boolean };
  audio?: { percent: number; muted: boolean };
  brightness?: { percent: number };
};

export type SettingsOutput = {
  name: string;
  make: string;
  model: string;
  width: number;
  height: number;
  refresh_millihertz: number;
  scale: number;
  primary: boolean;
  enabled: boolean;
};

export type SettingsPaths = {
  configFile?: string;
};

export type SettingsPatch = {
  materialMode?: MaterialMode;
  shellMode?: string;
  enableEffects?: boolean;
  safeMode?: boolean;
  backend?: string;
  xwayland?: boolean;
  debugOverlay?: boolean;
  animationsEnabled?: boolean;
  performanceMode?: string;
  reduceEffectsOnBattery?: boolean;
  defaultScale?: number;
  dockIconSize?: number;
  dockMagnification?: boolean;
  taskbarLauncher?: boolean;
  wallpaperPath?: string | null;
  pinnedCommands?: string[];
  workspaceCount?: number;
  restoreSessions?: boolean;
  crashLimit?: number;
  crashWindowSeconds?: number;
  autoSafeMode?: boolean;
  backupBeforeApply?: boolean;
  defaultTerminal?: string;
  defaultFileManager?: string;
  defaultBrowser?: string;
  defaultLauncher?: string;
  lockCommand?: string;
  suspendCommand?: string;
  rebootCommand?: string;
  poweroffCommand?: string;
  idleLockSeconds?: number | null;
  idleSuspendSeconds?: number | null;
};

export const emptySettingsSnapshot = (): SettingsSnapshot => ({
  appearance: {
    materialMode: "glass",
    shellMode: "panel",
    animationsEnabled: true,
    performanceMode: "balanced",
    dockIconSize: 40,
    dockMagnification: true,
    taskbarLauncher: true,
  },
  wallpaper: {},
  palette: {
    panel: "rgba(22, 22, 20, 0.62)",
    panelControl: "rgba(255, 255, 255, 0.08)",
    panelText: "rgba(248, 248, 246, 0.96)",
    dock: "rgba(24, 23, 20, 0.34)",
    accent: "rgba(210, 192, 130, 1)",
    textSoft: "rgba(218, 216, 205, 0.91)",
    textMuted: "rgba(164, 162, 154, 0.87)",
  },
  dock: {
    customized: false,
    apps: [],
  },
  general: {
    enableEffects: true,
    safeMode: false,
  },
  compositor: {
    backend: "auto",
    xwayland: true,
    debugOverlay: false,
  },
  performance: {
    reduceEffectsOnBattery: false,
  },
  workspaces: {
    count: 1,
    restoreSessions: true,
  },
  recovery: {
    crashLimit: 3,
    crashWindowSeconds: 60,
    autoSafeMode: true,
    backupBeforeApply: true,
  },
  session: {
    lockCommand: "loginctl lock-session",
    suspendCommand: "systemctl suspend",
    rebootCommand: "systemctl reboot",
    poweroffCommand: "systemctl poweroff",
    idleLockSeconds: undefined,
    idleSuspendSeconds: undefined,
  },
  display: {
    defaultScale: 1,
  },
  defaultApps: {
    terminal: "ghostty",
    fileManager: "nautilus",
    browser: "google-chrome-stable",
    settings: "asher-settings",
    launcher: "vicinae",
  },
  status: {},
  outputs: [],
  paths: {},
});
