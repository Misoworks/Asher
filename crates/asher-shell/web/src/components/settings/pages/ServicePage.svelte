<script lang="ts">
  import type { SettingsPage, SettingsSnapshot } from "../../../settings/model";

  let { page, settings } = $props<{
    page: SettingsPage;
    settings: SettingsSnapshot;
  }>();
</script>

{#if page === "network"}
  <section class="settings-section">
    <h2>Network</h2>
    <div class="settings-group">
      <div class="settings-row">
        <span>Interface</span>
        <strong>{settings.status.network?.name ?? "No active interface"}</strong>
      </div>
      <div class="settings-row">
        <span>Type</span>
        <strong>{settings.status.network?.wireless ? "Wireless" : settings.status.network ? "Wired" : "Unavailable"}</strong>
      </div>
      <div class="settings-row">
        <span>Quick settings tile</span>
        <strong>{settings.status.network ? "Visible" : "Hidden"}</strong>
      </div>
      <div class="settings-row">
        <span>Configuration</span>
        <strong>Detected automatically</strong>
      </div>
    </div>
  </section>
{:else}
  <section class="settings-section">
    <h2>Sound</h2>
    <div class="settings-group">
      <div class="settings-row">
        <span>Output volume</span>
        <strong>{settings.status.audio ? `${settings.status.audio.percent}%` : "Unavailable"}</strong>
      </div>
      <div class="settings-row">
        <span>Mute</span>
        <strong>{settings.status.audio?.muted ? "On" : settings.status.audio ? "Off" : "Unavailable"}</strong>
      </div>
      <div class="settings-row">
        <span>Quick settings slider</span>
        <strong>{settings.status.audio ? "Visible" : "Hidden"}</strong>
      </div>
      <div class="settings-row">
        <span>Device routing</span>
        <strong>Default sink</strong>
      </div>
    </div>
  </section>
{/if}

<section class="settings-section">
  <h2>Panel Integration</h2>
  <div class="settings-group">
    <div class="settings-row"><span>Quick settings</span><strong>{settings.status.audio || settings.status.network ? "Available" : "Limited"}</strong></div>
  </div>
</section>
