<script lang="ts">
  import Icon from "../../Icon.svelte";
  import type { SettingsPatch, SettingsSnapshot } from "../../../settings/model";

  let { settings, wallpaperPath, chooseWallpaper, apply } = $props<{
    settings: SettingsSnapshot;
    wallpaperPath: string;
    chooseWallpaper: () => Promise<void>;
    apply: (patch: SettingsPatch) => Promise<void>;
  }>();

</script>

<section class="wallpaper-layout">
  <div class="wallpaper-preview" style:--preview-image={settings.wallpaper.uri ? `url("${settings.wallpaper.uri}")` : "none"}>
    {#if !settings.wallpaper.uri}
      <Icon name="image" />
    {/if}
  </div>
  <div class="wallpaper-side">
    <section class="settings-section is-stacked">
      <h2>Wallpaper</h2>
      <div class="settings-group">
        <div class="settings-row">
          <span>Selected image</span>
          <strong>{wallpaperPath || "None"}</strong>
        </div>
        <div class="settings-actions">
          <button type="button" class="secondary-action" onclick={chooseWallpaper}>Browse</button>
          <button type="button" class="secondary-action" onclick={() => apply({ wallpaperPath: null })}>Clear</button>
        </div>
      </div>
    </section>
  </div>
</section>
