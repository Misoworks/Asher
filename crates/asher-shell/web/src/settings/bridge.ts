import { emptySettingsSnapshot, type SettingsPatch, type SettingsSnapshot } from "./model";

type NativeBridge = {
  commands: string[];
  invoke<T>(name: string, params?: Record<string, unknown>): Promise<T>;
};

type ReadyPayload = {
  settings?: SettingsSnapshot;
  page?: string;
};

export async function loadSettings(): Promise<SettingsSnapshot> {
  const bridge = await waitForBridge();
  if (bridge?.commands.includes("asher.settings.load")) {
    return bridge.invoke<SettingsSnapshot>("asher.settings.load");
  }
  const ready = await readyPayload(bridge);
  return ready?.settings ?? emptySettingsSnapshot();
}

export async function applySettings(patch: SettingsPatch): Promise<SettingsSnapshot> {
  const bridge = await waitForBridge();
  if (!bridge?.commands.includes("asher.settings.apply")) {
    const fallback = emptySettingsSnapshot();
    return {
      ...fallback,
      appearance: appearancePatch(fallback.appearance, patch),
      general: {
        ...fallback.general,
        enableEffects: patch.enableEffects ?? fallback.general.enableEffects,
        safeMode: patch.safeMode ?? fallback.general.safeMode,
      },
      session: {
        ...fallback.session,
        idleLockSeconds: patch.idleLockSeconds === null ? undefined : (patch.idleLockSeconds ?? fallback.session.idleLockSeconds),
        idleSuspendSeconds:
          patch.idleSuspendSeconds === null ? undefined : (patch.idleSuspendSeconds ?? fallback.session.idleSuspendSeconds),
      },
    };
  }
  return bridge.invoke<SettingsSnapshot>("asher.settings.apply", patch as Record<string, unknown>);
}

export async function pickWallpaper(): Promise<string | undefined> {
  const bridge = await waitForBridge();
  if (!bridge?.commands.includes("asher.settings.pick-wallpaper")) {
    return undefined;
  }
  const result = await bridge.invoke<{ path?: string | null }>("asher.settings.pick-wallpaper");
  return result.path ?? undefined;
}

export function initialSettingsPage() {
  const page = new URLSearchParams(window.location.search).get("settingsPage");
  return page && page.length > 0 ? page : "appearance";
}

export function windowControl(action: "close" | "minimize" | "maximize") {
  const controls = window.fenestra?.window;
  if (action === "close" && controls?.close) {
    controls.close();
    return;
  }
  if (action === "minimize" && controls?.minimize) {
    controls.minimize();
    return;
  }
  if (action === "maximize" && controls?.toggleMaximize) {
    controls.toggleMaximize();
    return;
  }
  window.location.href = `fenestra://window/${action}`;
}

export function startWindowDrag() {
  const controls = window.fenestra?.window;
  if (controls?.startDrag) {
    controls.startDrag();
    return;
  }
  window.location.href = `fenestra://window/start-drag?at=${Date.now()}`;
}

async function readyPayload(bridge?: NativeBridge): Promise<ReadyPayload | undefined> {
  if (!bridge?.commands.includes("asher.ready")) return undefined;
  try {
    return bridge.invoke<ReadyPayload>("asher.ready");
  } catch {
    return undefined;
  }
}

async function waitForBridge(): Promise<NativeBridge | undefined> {
  const deadline = performance.now() + 2000;
  while (performance.now() < deadline) {
    if (window.fenestra?.bridge) {
      return window.fenestra.bridge;
    }
    await new Promise((resolve) => window.setTimeout(resolve, 16));
  }
  return undefined;
}

function appearancePatch(base: SettingsSnapshot["appearance"], patch: SettingsPatch) {
  return {
    ...base,
    materialMode: patch.materialMode ?? base.materialMode,
    shellMode: patch.shellMode ?? base.shellMode,
    animationsEnabled: patch.animationsEnabled ?? base.animationsEnabled,
    performanceMode: patch.performanceMode ?? base.performanceMode,
    dockIconSize: patch.dockIconSize ?? base.dockIconSize,
    dockMagnification: patch.dockMagnification ?? base.dockMagnification,
    taskbarLauncher: patch.taskbarLauncher ?? base.taskbarLauncher,
  };
}
