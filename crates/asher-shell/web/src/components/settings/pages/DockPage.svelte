<script lang="ts">
  import SettingsSlider from "../SettingsSlider.svelte";
  import type { SettingsPatch, SettingsSnapshot } from "../../../settings/model";

  let { settings, apply } = $props<{
    settings: SettingsSnapshot;
    apply: (patch: SettingsPatch) => Promise<void>;
  }>();

  const workspaceCounts = [1, 2, 3, 4, 6, 9, 12];
</script>

<section class="settings-section">
  <h2>Dock</h2>
  <div class="settings-group">
    <SettingsSlider
      label="Icon size"
      value={settings.appearance.dockIconSize}
      min={32}
      max={64}
      step={2}
      unit="px"
      onChange={(dockIconSize) => apply({ dockIconSize })}
    />
    <div class="settings-row">
      <span>Hover lift</span>
      <button
        type="button"
        class="settings-switch"
        class:is-on={settings.appearance.dockMagnification}
        aria-label="Toggle hover lift"
        aria-pressed={settings.appearance.dockMagnification}
        onclick={() => apply({ dockMagnification: !settings.appearance.dockMagnification })}
      >
        <span></span>
      </button>
    </div>
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
  </div>
</section>

<section class="settings-section">
  <h2>Workspaces</h2>
  <div class="settings-group">
    <div class="settings-row">
      <span>Startup count</span>
      <div class="choice-row">
        {#each workspaceCounts as count (count)}
          <button type="button" class:is-active={settings.workspaces.count === count} onclick={() => apply({ workspaceCount: count })}>
            {count}
          </button>
        {/each}
      </div>
    </div>
    <div class="settings-row">
      <span>Restore sessions</span>
      <button
        type="button"
        class="settings-switch"
        class:is-on={settings.workspaces.restoreSessions}
        aria-label="Toggle session restore"
        aria-pressed={settings.workspaces.restoreSessions}
        onclick={() => apply({ restoreSessions: !settings.workspaces.restoreSessions })}
      >
        <span></span>
      </button>
    </div>
  </div>
</section>

<section class="settings-section">
  <h2>Pinned Apps</h2>
  <div class="settings-group">
    <div class="settings-row"><span>Pinned apps</span><strong>{settings.dock.apps.length}</strong></div>
    <p class="settings-note">Reorder pinned apps directly in the panel or dock by holding an icon and dragging it into place.</p>
  </div>
</section>
