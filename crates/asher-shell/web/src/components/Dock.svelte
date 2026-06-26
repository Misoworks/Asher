<script lang="ts">
  import AppButton from "./AppButton.svelte";
  import DebugMeter from "./DebugMeter.svelte";
  import { moveDockCommand, sameOrder } from "../lib/dock_reorder";
  import { workspaceWheelOffset } from "../lib/workspace_wheel";
  import { sendAction } from "../shell/bridge";
  import type { DockApp, ShellSnapshot } from "../shell/model";

  let { snapshot }: { snapshot: ShellSnapshot } = $props();
  let order = $state<string[]>([]);
  let orderSignature = $state("");
  let draggedCommand = $state<string | null>(null);

  const appByCommand = $derived(new Map(snapshot.dockApps.map((app) => [app.command, app])));
  const orderedApps = $derived(
    (order.length > 0 ? order : snapshot.dockApps.map((app) => app.command))
      .map((command) => appByCommand.get(command))
      .filter((app): app is DockApp => Boolean(app)),
  );

  $effect(() => {
    const commands = snapshot.dockApps.map((app) => app.command);
    const signature = commands.join("\0");
    if (!draggedCommand && signature !== orderSignature) {
      order = commands;
      orderSignature = signature;
    }
  });

  function launch(command: string) {
    sendAction({ type: "dock-menu-close" });
    sendAction({ type: "dock-launch", command });
  }

  function openMenu(command: string, x: number) {
    sendAction({ type: "dock-menu-open", command, x });
  }

  function workspaceScroll(event: WheelEvent) {
    const offset = workspaceWheelOffset(event);
    if (offset === 0) return;
    sendAction({ type: "workspace-relative", offset });
  }

  function startReorder(command: string) {
    draggedCommand = command;
    order = snapshot.dockApps.map((app) => app.command);
    sendAction({ type: "dock-menu-close" });
  }

  function previewReorder(target: string, after: boolean) {
    if (!draggedCommand) return;
    order = moveDockCommand(order, draggedCommand, target, after);
  }

  function commitReorder() {
    if (!draggedCommand) return;
    const current = snapshot.dockApps.map((app) => app.command);
    if (!sameOrder(order, current)) {
      sendAction({ type: "dock-reorder", commands: order });
    }
    draggedCommand = null;
  }

  function endReorder() {
    if (!draggedCommand) return;
    draggedCommand = null;
    order = snapshot.dockApps.map((app) => app.command);
  }
</script>

<section class="dock-shell">
  <nav class="shell-dock" aria-label="Pinned applications" onwheel={workspaceScroll}>
    {#each orderedApps as app (app.command)}
      <AppButton
        {app}
        variant="dock"
        onlaunch={launch}
        onmenu={openMenu}
        onreorderstart={startReorder}
        onreorderover={previewReorder}
        onreorderdrop={commitReorder}
        onreorderend={endReorder}
      />
    {/each}
  </nav>
  {#if snapshot.debugOverlay}
    <DebugMeter surface="DOCK" />
  {/if}
</section>
