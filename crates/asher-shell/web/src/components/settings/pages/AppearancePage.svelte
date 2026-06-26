<script lang="ts">
  import type { SettingsPatch, SettingsSnapshot } from "../../../settings/model";
  import SettingsSlider from "../SettingsSlider.svelte";

  let { settings, apply } = $props<{
    settings: SettingsSnapshot;
    apply: (patch: SettingsPatch) => Promise<void>;
  }>();

  const shellModes = [
    { id: "panel", label: "Panel" },
    { id: "dock", label: "Dock" },
    { id: "tiling", label: "Tiling" },
    { id: "focus", label: "Focus" },
    { id: "browser", label: "Browser" },
  ];

  const performanceModes = [
    { id: "quality", label: "Quality" },
    { id: "balanced", label: "Balanced" },
    { id: "performance", label: "Performance" },
    { id: "battery", label: "Battery" },
  ];
</script>

<section class="settings-section">
  <h2>Layout</h2>
  <div class="settings-group">
    <div class="segmented-control is-compact">
      {#each shellModes as mode (mode.id)}
        <button
          type="button"
          class="segmented-option"
          class:is-active={settings.appearance.shellMode === mode.id}
          onclick={() => apply({ shellMode: mode.id })}
        >
          {mode.label}
        </button>
      {/each}
    </div>
    <div class="settings-row">
      <span>Effects</span>
      <button
        type="button"
        class="settings-switch"
        class:is-on={settings.general.enableEffects}
        aria-label="Toggle effects"
        aria-pressed={settings.general.enableEffects}
        onclick={() => apply({ enableEffects: !settings.general.enableEffects })}
      >
        <span></span>
      </button>
    </div>
    <div class="settings-row">
      <span>Animations</span>
      <button
        type="button"
        class="settings-switch"
        class:is-on={settings.appearance.animationsEnabled}
        aria-label="Toggle animations"
        aria-pressed={settings.appearance.animationsEnabled}
        onclick={() => apply({ animationsEnabled: !settings.appearance.animationsEnabled })}
      >
        <span></span>
      </button>
    </div>
  </div>
</section>

<section class="settings-section">
  <h2>Visual quality</h2>
  <div class="settings-group">
    <SettingsSlider
      label="Dock icon size"
      value={settings.appearance.dockIconSize}
      min={32}
      max={64}
      step={2}
      unit="px"
      onChange={(dockIconSize) => apply({ dockIconSize })}
    />
    <div class="segmented-control is-compact">
      {#each performanceModes as mode (mode.id)}
        <button
          type="button"
          class="segmented-option"
          class:is-active={settings.appearance.performanceMode === mode.id}
          onclick={() => apply({ performanceMode: mode.id })}
        >
          {mode.label}
        </button>
      {/each}
    </div>
    <div class="settings-row">
      <span>Reduce effects on battery</span>
      <button
        type="button"
        class="settings-switch"
        class:is-on={settings.performance.reduceEffectsOnBattery}
        aria-label="Reduce effects on battery"
        aria-pressed={settings.performance.reduceEffectsOnBattery}
        onclick={() => apply({ reduceEffectsOnBattery: !settings.performance.reduceEffectsOnBattery })}
      >
        <span></span>
      </button>
    </div>
  </div>
</section>
