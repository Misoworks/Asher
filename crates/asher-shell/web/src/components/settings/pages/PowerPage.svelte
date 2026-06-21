<script lang="ts">
  import type { SettingsPatch, SettingsSnapshot } from "../../../settings/model";

  let { settings, apply } = $props<{
    settings: SettingsSnapshot;
    apply: (patch: SettingsPatch) => Promise<void>;
  }>();

  const crashLimits = [1, 2, 3, 5, 8, 12];
  const crashWindows = [30, 60, 120, 300, 600];
  const idleChoices = [
    { label: "Off", value: null },
    { label: "5m", value: 300 },
    { label: "10m", value: 600 },
    { label: "15m", value: 900 },
    { label: "30m", value: 1800 },
  ];
  const suspendChoices = [
    { label: "Off", value: null },
    { label: "15m", value: 900 },
    { label: "30m", value: 1800 },
    { label: "1h", value: 3600 },
    { label: "2h", value: 7200 },
  ];
</script>

<section class="settings-section">
  <h2>Power</h2>
  <div class="settings-group">
    <div class="settings-row">
      <span>Battery</span>
      <strong>{settings.status.battery ? `${settings.status.battery.percent}% ${settings.status.battery.state}` : "Desktop power"}</strong>
    </div>
    <div class="settings-row">
      <span>Brightness</span>
      <strong>{settings.status.brightness ? `${settings.status.brightness.percent}%` : "Unavailable"}</strong>
    </div>
    <div class="settings-row">
      <span>Power actions</span>
      <strong>Quick Settings</strong>
    </div>
    <div class="settings-row">
      <span>Lock after idle</span>
      <div class="choice-row compact">
        {#each idleChoices as choice (choice.label)}
          <button
            type="button"
            class:is-active={(settings.session.idleLockSeconds ?? null) === choice.value}
            onclick={() => apply({ idleLockSeconds: choice.value })}
          >
            {choice.label}
          </button>
        {/each}
      </div>
    </div>
    <div class="settings-row">
      <span>Suspend after idle</span>
      <div class="choice-row compact">
        {#each suspendChoices as choice (choice.label)}
          <button
            type="button"
            class:is-active={(settings.session.idleSuspendSeconds ?? null) === choice.value}
            onclick={() => apply({ idleSuspendSeconds: choice.value })}
          >
            {choice.label}
          </button>
        {/each}
      </div>
    </div>
  </div>
</section>

<section class="settings-section">
  <h2>Recovery</h2>
  <div class="settings-group">
    <div class="settings-row">
      <span>Safe mode</span>
      <button
        type="button"
        class="settings-switch"
        class:is-on={settings.general.safeMode}
        aria-label="Toggle safe mode"
        aria-pressed={settings.general.safeMode}
        onclick={() => apply({ safeMode: !settings.general.safeMode })}
      >
        <span></span>
      </button>
    </div>
    <div class="settings-row">
      <span>Automatic safe mode</span>
      <button
        type="button"
        class="settings-switch"
        class:is-on={settings.recovery.autoSafeMode}
        aria-label="Toggle automatic safe mode"
        aria-pressed={settings.recovery.autoSafeMode}
        onclick={() => apply({ autoSafeMode: !settings.recovery.autoSafeMode })}
      >
        <span></span>
      </button>
    </div>
    <div class="settings-row">
      <span>Backup before writes</span>
      <button
        type="button"
        class="settings-switch"
        class:is-on={settings.recovery.backupBeforeApply}
        aria-label="Toggle config backups"
        aria-pressed={settings.recovery.backupBeforeApply}
        onclick={() => apply({ backupBeforeApply: !settings.recovery.backupBeforeApply })}
      >
        <span></span>
      </button>
    </div>
    <div class="settings-row">
      <span>Crash limit</span>
      <div class="choice-row compact">
        {#each crashLimits as limit (limit)}
          <button type="button" class:is-active={settings.recovery.crashLimit === limit} onclick={() => apply({ crashLimit: limit })}>
            {limit}
          </button>
        {/each}
      </div>
    </div>
    <div class="settings-row">
      <span>Crash window</span>
      <div class="choice-row compact">
        {#each crashWindows as seconds (seconds)}
          <button
            type="button"
            class:is-active={settings.recovery.crashWindowSeconds === seconds}
            onclick={() => apply({ crashWindowSeconds: seconds })}
          >
            {seconds}s
          </button>
        {/each}
      </div>
    </div>
  </div>
</section>
