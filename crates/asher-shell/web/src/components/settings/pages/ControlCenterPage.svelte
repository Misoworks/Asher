<script lang="ts">
  import type { SettingsPatch, SettingsSnapshot } from "../../../settings/model";

  let { settings, apply } = $props<{
    settings: SettingsSnapshot;
    apply: (patch: SettingsPatch) => Promise<void>;
  }>();
</script>

<section class="settings-section">
  <h2>Panel</h2>
  <div class="settings-group">
    <div class="settings-row">
      <span>Start menu button</span>
      <button
        type="button"
        class="settings-switch"
        class:is-on={settings.appearance.taskbarLauncher}
        aria-label="Toggle Start menu button"
        aria-pressed={settings.appearance.taskbarLauncher}
        onclick={() => apply({ taskbarLauncher: !settings.appearance.taskbarLauncher })}
      >
        <span></span>
      </button>
    </div>
    <div class="settings-row"><span>Clock action</span><strong>Notification Center</strong></div>
    <div class="settings-row"><span>Status action</span><strong>Quick Settings</strong></div>
  </div>
</section>

<section class="settings-section">
  <h2>Menus</h2>
  <div class="settings-group">
    <div class="settings-row">
      <span>Quick Settings</span>
      <strong>Network, sound, brightness, power</strong>
    </div>
    <div class="settings-row">
      <span>Date menu</span>
      <strong>Notifications over calendar</strong>
    </div>
    <div class="settings-row">
      <span>Reduce effects on battery</span>
      <button
        type="button"
        class="settings-switch"
        class:is-on={settings.performance.reduceEffectsOnBattery}
        aria-label="Toggle battery effect reduction"
        aria-pressed={settings.performance.reduceEffectsOnBattery}
        onclick={() => apply({ reduceEffectsOnBattery: !settings.performance.reduceEffectsOnBattery })}
      >
        <span></span>
      </button>
    </div>
  </div>
</section>

<section class="settings-section">
  <h2>Live Modules</h2>
  <div class="settings-group">
    <div class="settings-row"><span>Network tile</span><strong>{settings.status.network ? "Visible" : "Hidden"}</strong></div>
    <div class="settings-row"><span>Volume slider</span><strong>{settings.status.audio ? "Visible" : "Hidden"}</strong></div>
    <div class="settings-row"><span>Brightness slider</span><strong>{settings.status.brightness ? "Visible" : "Hidden"}</strong></div>
    <div class="settings-row"><span>Battery tile</span><strong>{settings.status.battery ? "Visible" : "Hidden"}</strong></div>
  </div>
</section>
