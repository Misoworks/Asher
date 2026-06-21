<script lang="ts">
  import Icon from "../../Icon.svelte";
  import SettingsSlider from "../SettingsSlider.svelte";
  import type { SettingsPatch, SettingsSnapshot } from "../../../settings/model";

  let { settings, apply } = $props<{
    settings: SettingsSnapshot;
    apply: (patch: SettingsPatch) => Promise<void>;
  }>();

  function outputLabel(refresh: number) {
    return `${Math.round(refresh / 1000)} Hz`;
  }
</script>

<section class="settings-section">
  <h2>Scale</h2>
  <div class="settings-group">
    <SettingsSlider
      label="Default scale"
      value={settings.display.defaultScale}
      min={0.75}
      max={2}
      step={0.25}
      format={(value) => `${value}x`}
      onChange={(defaultScale) => apply({ defaultScale })}
    />
    <p class="settings-note">New and unconfigured displays use this scale.</p>
  </div>
</section>

<section class="settings-section">
  <h2>Outputs</h2>
  <div class="settings-group output-list">
    {#each settings.outputs as output (output.name)}
      <div class="output-row">
        <Icon name="laptop" />
        <div>
          <strong>{output.name}</strong>
          <span>{output.width} x {output.height} / {outputLabel(output.refresh_millihertz)} / {output.scale}x</span>
        </div>
        {#if output.primary}<em>Primary</em>{/if}
      </div>
    {:else}
      <div class="settings-row"><span>No displays reported</span><strong>{settings.display.defaultScale}x default</strong></div>
    {/each}
  </div>
</section>
