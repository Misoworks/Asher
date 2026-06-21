<script lang="ts">
  import type { SettingsPatch, SettingsSnapshot } from "../../../settings/model";

  let { settings, apply } = $props<{
    settings: SettingsSnapshot;
    apply: (patch: SettingsPatch) => Promise<void>;
  }>();

  function inputValue(event: Event) {
    return event.currentTarget instanceof HTMLInputElement ? event.currentTarget.value : "";
  }
</script>

<section class="settings-section">
  <h2>Defaults</h2>
  <div class="settings-group">
    <label class="settings-field inline">
      <span>Terminal</span>
      <input value={settings.defaultApps.terminal} spellcheck="false" onchange={(event) => apply({ defaultTerminal: inputValue(event) })} />
    </label>
    <label class="settings-field inline">
      <span>Files</span>
      <input value={settings.defaultApps.fileManager} spellcheck="false" onchange={(event) => apply({ defaultFileManager: inputValue(event) })} />
    </label>
    <label class="settings-field inline">
      <span>Browser</span>
      <input value={settings.defaultApps.browser} spellcheck="false" onchange={(event) => apply({ defaultBrowser: inputValue(event) })} />
    </label>
    <label class="settings-field inline">
      <span>Launcher</span>
      <input value={settings.defaultApps.launcher} spellcheck="false" onchange={(event) => apply({ defaultLauncher: inputValue(event) })} />
    </label>
  </div>
</section>
