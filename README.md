# Asher

Asher is a Linux desktop environment built around a custom Wayland compositor, Kestrel, and a modular shell.

This repository currently contains the initial Rust workspace and the first nested Kestrel prototype from `asher.md`.

## Build

```sh
cargo build
```

The default shell UI uses the shared Fenestra CEF runtime for Svelte/TypeScript/Tailwind chrome.

For X11 application support, install `xwayland-satellite` and `Xwayland` from the distribution package manager. Asher starts satellite automatically when `compositor.xwayland = true`.

The real DRM/KMS session backend is built behind an explicit feature while it is under active development:

```sh
cargo build -p kestrel --features session-backend
```

That feature requires libseat development files. On Fedora install `libseat-devel`, `systemd-devel`, `mesa-libgbm-devel`, `mesa-libEGL-devel`, `mesa-libGLES-devel`, `libxkbcommon-devel`, `libudev-devel`, `libinput-devel`, `xwayland-satellite`, `xorg-x11-server-Xwayland`, `xdg-desktop-portal`, `xdg-desktop-portal-gtk`, `xdg-desktop-portal-gnome`, `gnome-keyring`, and a PolicyKit agent such as `lxpolkit` or `xfce-polkit`. On Arch-based systems install `seatd`; on Debian/Ubuntu-style systems install `libseat-dev`. For a complete login session, install `dbus-run-session`, `dbus-update-activation-environment`, a Secret Service provider such as `gnome-keyring-daemon`, and a PolicyKit authentication agent.

Build the web shell assets with Bun before compiling `asher-shell` when the web UI has changed. The build emits a single HTML file that is embedded into the shell binary:

```sh
cd crates/asher-shell/web
bun install
bun run build
```

During shell development, `asherctl dev apply` performs that web build, rebuilds `asher-shell`, asks the live compositor to restart only the shell process, and reloads config:

```sh
cargo run -p asherctl -- dev setup
sudo target/debug/asherctl dev install-session
cargo run -p asherctl -- dev apply
cargo run -p asherctl -- dev watch
```

`dev setup` installs Bun dependencies, builds the shell web bundle, and builds Kestrel, `asher-session`, `asher-shell`, `asher-settings`, and `asherctl`. `dev install-session` installs the display-manager entry, portal preferences, and the `asher-greeter` PAM policy; by default the entry points at `target/debug` binaries, so a rebuild changes what the next Asher login uses. The dev login entry starts `asher-session --session --guard`, so an early Kestrel crash returns to the display manager instead of leaving a broken login. Set `ASHER_FALLBACK_SESSION` or pass `--fallback-session` when a guarded session should launch another desktop after a startup failure. `dev watch` watches the shell web UI, shell Rust code, settings app code, default and user config, and Kestrel/session source groups. Shell changes are applied through the shell restart IPC path. Kestrel changes are built with `--features session-backend` and reported, but the compositor is not restarted automatically because replacing it ends the running app session.

## Test

```sh
cargo test
```

## Run Kestrel Nested

```sh
cargo run -p kestrel -- --nested
```

Kestrel prints the `WAYLAND_DISPLAY` socket name. Launch a Wayland client against that socket from another terminal:

```sh
WAYLAND_DISPLAY=<printed-socket> ghostty
```

The nested prototype accepts xdg-shell toplevel clients, advertises a `wl_output`, supports `wlr-layer-shell` shell surfaces, starts and restarts `asher-shell` when the shell binary is available, starts `xwayland-satellite` for X11 apps when configured and installed, draws `/home/kristof/Pictures/bg.jpg` as the default compositor background, draws active-workspace windows in a nested window, forwards keyboard and pointer input, supports workspace slide transitions, respects xdg-decoration client-side and server-side mode requests, supports normal clipboard focus, primary selection, xdg activation, xdg toplevel icons, named cursor-shape requests, viewporter, fractional-scale preferences, presentation-time feedback, text-input v3 focus tracking, and `ext-background-effect-v1`, and draws compositor-side blurred titlebars with right-side traffic-light controls only for server-decorated windows. Layer-shell surfaces are arranged by layer order around normal application windows: background, bottom, app windows, top, then overlay. Kestrel reports the shell process, XWayland status, active workspace/profile, and live effect state through IPC, shows a wallpaper-backed loading progress overlay until the top panel layer is ready, cleans stale shell-control sockets, and stops its child processes when the compositor exits. If the shell crashes repeatedly inside the configured recovery window, Kestrel restarts it in runtime safe mode with expensive effects disabled.

The shell uses a Fenestra web UI for the user-facing panel, dock, sidebar, overview, quick settings, and notification/date center. Settings is a separate `asher-settings` Fenestra app that reuses the shell web bundle in settings mode. Rust still owns Wayland IPC, workspace/window actions, tray hosting, notifications, app launching, session commands, config reloads, and surface lifetime; the web layer renders the chrome and sends typed actions back to Rust. The user-facing shell chrome is web-only, so there is no separate native fallback UI to keep in sync.

Hidden shell popovers and the overview are launched lazily and evicted after a short idle period to keep the resident memory footprint down. Set `ASHER_SHELL_PREWARM=1` while developing when first-open latency matters more than startup memory.

The built-in default starts in a panel workspace with a bottom taskbar; sidebar chrome is available through workspace profiles instead of being the first-run default. Workspaces switch from panel, dock, or sidebar vertical scroll, `Super+Left/Right`, `Super+Up/Down`, `Super+scroll`, or direct numeric shortcuts. Kestrel keeps one trailing empty dynamic workspace once windows exist and does not keep creating empty workspaces when scrolling past it. Pressing and releasing `Super` opens the web overview with real workspace previews, active windows, searchable discovered desktop apps, workspace profiles, and command results for launcher, quick settings, notifications, shell mode, settings pages, do-not-disturb, session commands, diagnostics, config reload, logs, and safe mode. The clock opens a notification and calendar center. The panel status area opens quick settings, renders live network, audio, power, volume, brightness, do-not-disturb, launcher, overview, notifications, settings, and session controls when the backing services are available, and hides unavailable hardware controls instead of showing disabled placeholders. StatusNotifier/AppIndicator tray items registered on the session bus are hosted in the panel; tray icons come from the item icon theme name when available, left click activates the item, and right click asks the item to open its context menu. The shell owns `org.freedesktop.Notifications`, supports body text, static icons, action buttons, replacement IDs, timeouts, close requests, toast default actions, suppresses non-critical popups while do-not-disturb is enabled, and emits `NotificationClosed` and `ActionInvoked` signals for app notifications. Kestrel inserts downsampled rounded blurred wallpaper material under the dock, sidebar, and popover shell surfaces when blur is enabled; full-width panel and overview surfaces use normal translucent material to keep frame cost low. Wayland apps that draw their own header bars keep them by default; Kestrel draws its traffic-light frame only for clients that explicitly request server-side decorations. The panel and dock tint their material from the configured wallpaper, and the dock/taskbar render pinned apps as real icon-theme images with hover lift and running/active indicators. Clicking a pinned dock or taskbar app focuses or restores its matching running window before launching a new process; clicking its active visible window minimizes it. Hold and drag pinned dock or taskbar apps to reorder them. Right-clicking an overview app pins or unpins it, and right-clicking a dock item unpins it. Chromium-family browsers launched by the shell use a Asher-specific profile under `${XDG_STATE_HOME:-$HOME/.local/state}/asher` and force Wayland Ozone, so nested sessions do not hand off browser windows to an existing host desktop browser process.

Settings can be opened from quick settings, overview search, or directly with:

```sh
asher-settings
```

The Settings app writes the same config file used by Kestrel and can change shell mode, glass effects, animation, performance mode, dock icon size, dock hover lift, overview button visibility, pinned app order, workspace startup behavior, wallpaper, display scale, compositor backend, XWayland, default apps, session commands, and recovery behavior. Glass colors are derived from the configured wallpaper, with compatibility for older material config values. Wallpaper browsing uses the desktop portal-backed native chooser when available.

To inspect the advertised Wayland globals:

```sh
WAYLAND_DISPLAY=<printed-socket> wayland-info
```

## Run Kestrel Headless

```sh
ASHER_IPC_SOCKET=/tmp/asher-headless.sock cargo run -p kestrel -- --headless --socket asher-headless
```

The headless backend binds a Wayland socket and runs the compositor protocol, layout, frame-callback, and IPC loops without opening a host window or starting the shell. `asherctl status` reports `Shell: NotStarted` for this backend. It is intended for protocol and automation smoke tests:

```sh
WAYLAND_DISPLAY=asher-headless wayland-info
ASHER_IPC_SOCKET=/tmp/asher-headless.sock asherctl status
```

## Session Launcher

`asher-session` is the display-manager entry point from `data/sessions/asher.desktop`.
The installed desktop entry launches `asher-session --session --guard`, sets the Asher desktop environment variables, and starts Kestrel as a real Wayland session. When run manually without an explicit backend, `asher-session` defaults to nested inside an existing Wayland session and to the session backend outside one. When `dbus-run-session` is available, the session runs Kestrel under a private D-Bus session so shell services and launched apps do not attach to the host desktop session while testing nested. Kestrel exports `WAYLAND_DISPLAY`, `DISPLAY`, and desktop identifiers to D-Bus activation/systemd user environments and starts `gnome-keyring-daemon --components=secrets` when it is installed on the private bus. If Kestrel is started directly for development, it wraps `asher-shell` in its own private D-Bus session when possible. Set `ASHER_USE_HOST_DBUS=1` only when intentionally debugging against the host session bus.

```sh
cargo run -p asher-session -- --nested --socket asher-dev
cargo run -p asher-session -- --desktop-entry
cargo run -p asher-session -- --session --dry-run
cargo run -p asher-greeter -- list
cargo run -p asher-greeter -- launch asher --dry-run
cargo run -p asher-greeter -- auth-launch kristof asher --password-stdin --dry-run
```

The DRM/KMS session backend behind `kestrel --features session-backend` opens the active seat through libseat, selects the primary DRM card with udev, discovers the connected output graph, creates a GBM/KMS surface for each connected output, renders the full scene on the primary scanout, queues a compositor-clear frame on secondary scanouts, forwards libinput keyboard/pointer events, starts the shell and XWayland satellite, advertises linux-dmabuf formats accepted by the renderer, refreshes the output graph when udev reports connector changes on the active DRM device, and keeps the private D-Bus session behavior used by the nested launcher. Fullscreen Wayland clients can be scanned out directly on the primary plane when there are no visible shell/effect layers and KMS accepts the client framebuffer; otherwise Kestrel falls back to normal composition. Install `data/xdg-desktop-portal/asher-portals.conf` to `/usr/share/xdg-desktop-portal/asher-portals.conf` or `/etc/xdg/xdg-desktop-portal/asher-portals.conf` so the xdg-desktop-portal broker chooses Asher's GTK/GNOME backend preferences. Output scale is configurable per connector and can be changed live through IPC. Viewport-aware scene rendering on non-primary outputs and cursor/overlay plane assignment are the remaining KMS work.

`asher-greeter` discovers installed `.desktop` sessions and has an authenticated launch path for a future Asher login manager. `auth-launch` authenticates through the `asher-greeter` PAM service, opens a PAM session, drops to the selected user, starts the selected desktop entry, waits for it to exit, and closes the PAM session. The lock-screen-style graphical greeter still needs to be layered on top of this helper.

Current compositor shortcuts:

```txt
Super+1..9          Switch workspace by numeric workspace id
Super+Left/Right    Switch to the previous or next workspace
Super+Up/Down       Switch to the previous or next workspace
Super+scroll        Switch to the previous or next workspace
Super               Open the window overview when released by itself
Super+Space         Open the configured launcher, Vicinae by default
Super+Return        Open the configured terminal, Ghostty by default
Super+E             Open the configured file manager
Escape              Close the active overview, quick settings, date center, or menu surface
Super+Shift+1..9    Move the active window to a workspace
Super+Shift+R       Restart Asher Shell without ending the compositor session
Super+Shift+Backspace
                    Ignore user config and reload the built-in default profile
Alt+Tab             Cycle windows in the active workspace
Alt+Shift+Tab       Cycle windows in reverse
Super+Tab           Cycle windows in the active workspace
Super+Shift+Tab     Cycle windows in reverse
Super+Q             Close the active window
F3                  Toggle Kestrel debug overlay
```

Window titlebars can be dragged, resized from edges/corners with matching cursor feedback, closed, minimized, and maximized with the right-side traffic-light controls. Normal app windows animate when opened, restored, and minimized when animations are enabled.

## CLI

```sh
cargo run -p asherctl -- status
cargo run -p asherctl -- status --json
cargo run -p asherctl -- config path
cargo run -p asherctl -- config validate
cargo run -p asherctl -- logs
cargo run -p asherctl -- doctor
cargo run -p asherctl -- dev setup
cargo run -p asherctl -- dev install-session
cargo run -p asherctl -- dev apply
cargo run -p asherctl -- dev watch
cargo run -p asherctl -- dev apply kestrel
cargo run -p asherctl -- recovery status
cargo run -p asherctl -- recovery backups
cargo run -p asherctl -- recovery rollback
cargo run -p asherctl -- recovery defaults
cargo run -p asherctl -- reload
cargo run -p asherctl -- effects blur off
cargo run -p asherctl -- debug overlay on
cargo run -p asherctl -- safe-mode set on
cargo run -p asherctl -- shell restart
cargo run -p asherctl -- profile list
cargo run -p asherctl -- workspace list
cargo run -p asherctl -- workspace switch 2
cargo run -p asherctl -- workspace profile 2 browser-dev
cargo run -p asherctl -- workspace style dock
cargo run -p asherctl -- workspace style panel
cargo run -p asherctl -- output list
cargo run -p asherctl -- output scale 1.25
cargo run -p asher-greeter -- auth-launch kristof asher --password-stdin
cargo run -p asherctl -- window list
cargo run -p asherctl -- window focus 1
cargo run -p asherctl -- window move 1 2
cargo run -p asherctl -- window minimize 1
cargo run -p asherctl -- window maximize 1
cargo run -p asherctl -- window close 1
```

Configuration is loaded from `~/.config/asher/config.toml` when present and falls back to built-in defaults.
At startup, Kestrel, `asher-session`, and `asher-shell` fall back to defaults if the user config cannot be parsed or validated, so a broken config does not prevent the session from starting. `asherctl config validate` and live reload remain strict.
When Kestrel is running, `asherctl status`, workspace commands, profile commands, window commands, config reload, and live setting toggles use the live IPC socket.
When `recovery.backup_before_apply` is enabled, config writes create timestamped backups under `~/.config/asher/backups`. `asherctl recovery rollback` restores the latest backup and asks a running Kestrel instance to reload.

The background image can be overridden with `compositor.background_image` in the config file. Set it to `null` to fall back to the solid compositor clear color.

The appearance section controls the default shell profile and panel/dock presentation:

```toml
[appearance]
material_mode = "glass"
shell_mode = "panel"
dock_icon_size = 40
dock_magnification = true
taskbar_launcher = true
```

Display defaults can be configured globally or per connector:

```toml
[display]
default_scale = 1.0

[display."eDP-1"]
scale = 1.25
```

Dock pins can be edited without hand-writing config:

```sh
cargo run -p asherctl -- dock list
cargo run -p asherctl -- dock pin google-chrome-stable --label Browser --icon google-chrome
cargo run -p asherctl -- dock unpin Browser
```

In the web overview, right-clicking an app pins or unpins it. Right-clicking a dock item unpins it, and dragging pinned apps in the dock or panel changes their order. The first dock customization materializes the built-in defaults into the user config, so removing the last pin leaves an intentionally empty dock instead of silently restoring the defaults.

The same pins are stored as `dock.pinned` entries:

```toml
[[dock.pinned]]
label = "Terminal"
command = "ghostty"
icon = "com.mitchellh.ghostty"
```

Set `dock.customized = true` with no `dock.pinned` entries to keep the dock empty.
