<script lang="ts">
  import { sendAction } from "../shell/bridge";
  import type { DockApp, ShellSnapshot, WindowItem } from "../shell/model";

  let { snapshot }: { snapshot: ShellSnapshot } = $props();
  const app = $derived(snapshot.dockApps.find((entry) => entry.command === snapshot.dockMenuCommand));
  const window = $derived(app ? matchedWindow(app, snapshot.windows) : undefined);
  const isRunning = $derived(Boolean(window ?? app?.running));

  function close() {
    sendAction({ type: "dock-menu-close" });
  }

  function open(app: DockApp, forceNew = false) {
    close();
    if (!forceNew && window) {
      sendAction({ type: "window-activate", window: window.id });
    } else {
      sendAction({ type: "dock-launch", command: app.command });
    }
  }

  function unpin(app: DockApp) {
    close();
    sendAction({ type: "dock-unpin", command: app.command });
  }

  function minimize(window: WindowItem) {
    close();
    sendAction({ type: "window-minimize", window: window.id });
  }

  function closeWindow(window: WindowItem) {
    close();
    sendAction({ type: "window-close", window: window.id });
  }

  function forceQuit(app: DockApp) {
    close();
    sendAction({ type: "dock-force-quit", command: app.command });
  }

  function matchedWindow(app: DockApp, windows: WindowItem[]) {
    return (
      windows.find((window) => window.active && window.visible && windowMatchesApp(window, app)) ??
      windows.find((window) => window.visible && windowMatchesApp(window, app)) ??
      windows.find((window) => windowMatchesApp(window, app))
    );
  }

  function windowMatchesApp(window: WindowItem, app: DockApp) {
    const command = commandName(app.command);
    const label = app.label.toLowerCase();
    return [window.appId, window.title].some((value) => {
      const text = value?.toLowerCase() ?? "";
      return Boolean(text && ((command && text.includes(command)) || (label && text.includes(label))));
    });
  }

  function commandName(command: string) {
    return command.trim().split(/\s+/)[0]?.split("/").at(-1)?.replace(/^['"]|['"]$/g, "").toLowerCase() ?? "";
  }
</script>

<section class="dock-menu-shell">
  {#if app}
    <div class="dock-menu" role="menu" tabindex="-1" data-command={app.command} onpointerdown={(event) => event.stopPropagation()}>
      <strong>{app.label}</strong>
      {#if window}
        {#if !window.active}
          <button type="button" class="dock-menu-item" role="menuitem" onclick={() => open(app)}>
            <span>Focus</span>
          </button>
        {/if}
        <button type="button" class="dock-menu-item" role="menuitem" onclick={() => open(app, true)}>
          <span>Open New Window</span>
        </button>
        <button type="button" class="dock-menu-item" role="menuitem" onclick={() => minimize(window)}>
          <span>Minimize</span>
        </button>
        <button type="button" class="dock-menu-item" role="menuitem" onclick={() => closeWindow(window)}>
          <span>Close Window</span>
        </button>
        <button type="button" class="dock-menu-item is-danger" role="menuitem" onclick={() => forceQuit(app)}>
          <span>Force Quit</span>
        </button>
      {:else if isRunning}
        <button type="button" class="dock-menu-item" role="menuitem" onclick={() => open(app, true)}>
          <span>Open New Window</span>
        </button>
        <button type="button" class="dock-menu-item is-danger" role="menuitem" onclick={() => forceQuit(app)}>
          <span>Force Quit</span>
        </button>
      {:else}
        <button type="button" class="dock-menu-item" role="menuitem" onclick={() => open(app, true)}>
          <span>Open</span>
        </button>
      {/if}
      <button type="button" class="dock-menu-item" role="menuitem" onclick={() => unpin(app)}>
        <span>Unpin from Dock</span>
      </button>
    </div>
  {/if}
</section>
