<script lang="ts">
  import type { SettingsSnapshot } from "../../../settings/model";

  let { settings } = $props<{ settings: SettingsSnapshot }>();
</script>

<section class="settings-section">
  <h2>Asher</h2>
  <div class="settings-group">
    <div class="settings-row"><span>Layout</span><strong>{settings.appearance.shellMode}</strong></div>
    <div class="settings-row"><span>Effects</span><strong>{settings.general.enableEffects ? "Enabled" : "Off"}</strong></div>
  </div>
</section>
