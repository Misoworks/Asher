<script lang="ts">
  import { onMount } from "svelte";
  import { applySettings, initialSettingsPage, loadSettings, pickWallpaper, windowControl } from "../../settings/bridge";
  import { emptySettingsSnapshot, type SettingsPage, type SettingsPatch, type SettingsSnapshot } from "../../settings/model";
  import Icon from "../Icon.svelte";
  import AboutPage from "./pages/AboutPage.svelte";
  import AppearancePage from "./pages/AppearancePage.svelte";
  import AppsPage from "./pages/AppsPage.svelte";
  import ControlCenterPage from "./pages/ControlCenterPage.svelte";
  import DisplayPage from "./pages/DisplayPage.svelte";
  import DockPage from "./pages/DockPage.svelte";
  import FeaturePage from "./pages/FeaturePage.svelte";
  import PowerPage from "./pages/PowerPage.svelte";
  import WallpaperPage from "./pages/WallpaperPage.svelte";

  let settings = $state.raw<SettingsSnapshot>(emptySettingsSnapshot());
  let activePage = $state<SettingsPage>("appearance");
  let wallpaperPath = $state("");
  let searchQuery = $state("");
  let saving = $state(false);
  let settingsRoot: HTMLElement | undefined;
  const glassBlurImage = $derived(settings.wallpaper.glassBlurUri ? `url("${settings.wallpaper.glassBlurUri}")` : "none");

  const pages: { id: SettingsPage; label: string; icon: string }[] = [
    { id: "appearance", label: "Appearance", icon: "palette" },
    { id: "control-center", label: "Control Center", icon: "sliders" },
    { id: "dock", label: "Dock", icon: "panel" },
    { id: "wallpaper", label: "Wallpaper", icon: "image" },
    { id: "display", label: "Display", icon: "laptop" },
    { id: "bluetooth", label: "Bluetooth", icon: "bluetooth" },
    { id: "network", label: "Network", icon: "network" },
    { id: "sound", label: "Sound", icon: "volume" },
    { id: "keyboard", label: "Keyboard", icon: "keyboard" },
    { id: "mouse", label: "Mouse & Touchpad", icon: "mouse" },
    { id: "multitasking", label: "Multitasking", icon: "panel" },
    { id: "notifications", label: "Notifications", icon: "bell" },
    { id: "privacy", label: "Privacy & Security", icon: "shield-check" },
    { id: "search", label: "Search", icon: "scan-search" },
    { id: "accessibility", label: "Accessibility", icon: "accessibility" },
    { id: "power", label: "Power", icon: "power" },
    { id: "apps", label: "Apps", icon: "app" },
    { id: "system", label: "System", icon: "settings" },
    { id: "wellbeing", label: "Wellbeing", icon: "activity" },
    { id: "about", label: "About", icon: "info" },
  ];

  const featurePages = new Set<SettingsPage>([
    "bluetooth",
    "network",
    "sound",
    "keyboard",
    "mouse",
    "multitasking",
    "notifications",
    "privacy",
    "search",
    "accessibility",
    "system",
    "wellbeing",
  ]);

  onMount(() => {
    let frame = 0;
    let cancelled = false;

    activePage = normalizePage(initialSettingsPage());
    void (async () => {
      const next = await loadSettings();
      if (cancelled) return;
      settings = next;
      wallpaperPath = next.wallpaper.path ?? "";
    })();

    const syncGlassBlurPosition = () => {
      settingsRoot?.style.setProperty("--settings-glass-blur-x", `${-Math.round(window.screenX || 0)}px`);
      settingsRoot?.style.setProperty("--settings-glass-blur-y", `${-Math.round(window.screenY || 0)}px`);
      frame = window.requestAnimationFrame(syncGlassBlurPosition);
    };
    syncGlassBlurPosition();

    return () => {
      cancelled = true;
      window.cancelAnimationFrame(frame);
    };
  });

  async function apply(patch: SettingsPatch) {
    saving = true;
    try {
      const next = await applySettings(patch);
      settings = next;
      if (patch.wallpaperPath !== undefined) {
        wallpaperPath = next.wallpaper.path ?? "";
      }
    } finally {
      saving = false;
    }
  }

  async function chooseWallpaper() {
    const path = await pickWallpaper();
    if (!path) return;
    wallpaperPath = path;
    await apply({ wallpaperPath: path });
  }

  function normalizePage(value: string): SettingsPage {
    return pages.some((page) => page.id === value) ? (value as SettingsPage) : "appearance";
  }

  function keydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      windowControl("close");
    }
  }

  function pageTitle() {
    return pages.find((page) => page.id === activePage)?.label ?? "Settings";
  }

  function visiblePages() {
    const needle = searchQuery.trim().toLowerCase();
    if (!needle) return pages;
    return pages.filter((page) => `${page.label} ${page.id}`.toLowerCase().includes(needle));
  }
</script>

<svelte:window onkeydown={keydown} />

<section
  bind:this={settingsRoot}
  class="settings-shell"
  data-material={settings.appearance.materialMode}
  style:--settings-glass-sidebar={settings.palette.panel}
  style:--settings-glass-control={settings.palette.panelControl}
  style:--settings-glass-text={settings.palette.panelText}
  style:--settings-glass-soft={settings.palette.textSoft}
  style:--settings-glass-muted={settings.palette.textMuted}
  style:--settings-glass-accent={settings.palette.accent}
  style:--settings-glass-dock={settings.palette.dock}
  style:--settings-glass-blur-image={glassBlurImage}
>
  <aside class="settings-sidebar" data-effect="translucent">
    <div class="settings-window-controls">
      <button type="button" class="settings-window-control" aria-label="Close" onclick={() => windowControl("close")}>
        <Icon name="close" />
      </button>
      <button type="button" class="settings-window-control" aria-label="Minimize" onclick={() => windowControl("minimize")}>
        <Icon name="minimize" />
      </button>
      <button type="button" class="settings-window-control" aria-label="Maximize" onclick={() => windowControl("maximize")}>
        <Icon name="maximize" />
      </button>
    </div>

    <label class="settings-search">
      <Icon name="search" />
      <input bind:value={searchQuery} type="text" aria-label="Search settings" placeholder="Search settings" />
    </label>

    <nav class="settings-nav" aria-label="Settings sections">
      {#each visiblePages() as page (page.id)}
        <button
          type="button"
          class="settings-nav-item"
          class:is-active={activePage === page.id}
          onclick={() => (activePage = page.id)}
        >
          <Icon name={page.icon} />
          <span>{page.label}</span>
        </button>
      {/each}
    </nav>

  </aside>

  <main class="settings-content">
    <header class="settings-titlebar">
      <button type="button" class="settings-icon-button" aria-label="Back">
        <Icon name="chevron-left" />
      </button>
      <button type="button" class="settings-icon-button" aria-label="Forward">
        <Icon name="chevron-right" />
      </button>
      <h1>{pageTitle()}</h1>
      {#if saving}<span class="settings-save-state is-saving">Saving</span>{/if}
    </header>

    <div class="settings-page">
      {#if activePage === "appearance"}
        <AppearancePage {settings} {apply} />
      {:else if activePage === "control-center"}
        <ControlCenterPage {settings} {apply} />
      {:else if activePage === "dock"}
        <DockPage {settings} {apply} />
      {:else if activePage === "wallpaper"}
        <WallpaperPage {settings} {wallpaperPath} {chooseWallpaper} {apply} />
      {:else if activePage === "display"}
        <DisplayPage {settings} {apply} />
      {:else if activePage === "apps"}
        <AppsPage {settings} {apply} />
      {:else if featurePages.has(activePage)}
        <FeaturePage page={activePage} {settings} {apply} />
      {:else if activePage === "power"}
        <PowerPage {settings} {apply} />
      {:else}
        <AboutPage {settings} />
      {/if}
    </div>
  </main>
</section>
