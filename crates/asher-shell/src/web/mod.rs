use crate::{
    apps::{AppEntry, spawn_command},
    chrome::ShellChrome,
    control::ShellControlServer,
    dock::{self, DockApp, dock_app_matches_window},
    ipc::{
        ShellModel, activate_window, minimize_window, move_window_to_workspace, set_debug_overlay,
        set_workspace_profile, switch_relative_workspace, switch_workspace,
    },
    services::{
        notifications::NotificationService,
        system_status::{SystemStatus, set_audio_volume, set_brightness, toggle_audio_mute},
        tray::TrayService,
    },
    theme::ShellPalette,
};
mod actions;
mod appearance;
mod chrome_visibility;
mod command_actions;
mod icons;
mod init;
mod model;
mod palette;
mod settings_command;
mod surface;
mod surface_layout;
mod surface_sizing;
mod sync;
mod wallpaper;
use actions::{
    QuickSettingsPage, SessionCommand, WebShellAction, profile_id, window_id, workspace_id,
};
use asher_config::AsherConfig;
use chrome_visibility::ChromeVisibility;
use settings_command::settings_command;
use std::{
    cell::RefCell,
    error::Error,
    process::Child,
    rc::Rc,
    sync::mpsc::{self, Receiver},
    thread,
    time::{Duration, Instant},
};
use surface::WebSurfaces;
use tracing::{debug, warn};

const MODEL_REFRESH: Duration = Duration::from_millis(500);
const STATUS_REFRESH: Duration = Duration::from_secs(1);
const CONFIG_REFRESH: Duration = Duration::from_secs(2);
const ACTION_TICK: Duration = Duration::from_millis(16);
const MAINTENANCE_TICK: Duration = Duration::from_millis(100);

pub fn run(config: AsherConfig) -> Result<(), Box<dyn Error>> {
    let (actions_tx, actions_rx) = mpsc::channel();
    let shell = Rc::new(RefCell::new(WebShell::new(config, actions_tx, actions_rx)?));
    shell.borrow_mut().sync_surfaces();

    let mut last_maintenance = Instant::now();
    loop {
        shell.borrow_mut().tick_actions();
        if last_maintenance.elapsed() >= MAINTENANCE_TICK {
            shell.borrow_mut().tick();
            last_maintenance = Instant::now();
        }
        thread::sleep(ACTION_TICK);
    }
}

struct WebShell {
    config: AsherConfig,
    palette: ShellPalette,
    wallpaper_uri: Option<String>,
    glass_blur_wallpaper_uri: Option<String>,
    model: ShellModel,
    status: SystemStatus,
    chrome: ShellChrome,
    tray: TrayService,
    notifications: NotificationService,
    dock_apps: Vec<DockApp>,
    applications: Vec<AppEntry>,
    surfaces: WebSurfaces,
    actions_rx: Receiver<WebShellAction>,
    control: Option<ShellControlServer>,
    app_processes: Vec<Child>,
    launcher_command: String,
    overview_visible: bool,
    quick_visible: bool,
    date_visible: bool,
    chrome_visibility: ChromeVisibility,
    dock_menu_open: bool,
    dock_menu_command: Option<String>,
    dock_menu_x: Option<i32>,
    last_model_refresh: Instant,
    last_status_refresh: Instant,
    last_config_refresh: Instant,
    last_snapshot: String,
}

impl WebShell {
    fn tick_actions(&mut self) {
        self.handle_control_requests();
        let mut handled_action = false;
        while let Ok(action) = self.actions_rx.try_recv() {
            handled_action = true;
            self.handle_action(action);
        }

        if handled_action || self.overview_visible {
            self.sync_chrome();
            self.sync_surfaces();
        }
        self.surfaces.tick();
    }

    fn tick(&mut self) {
        self.tick_actions();

        self.app_processes
            .retain_mut(|child| matches!(child.try_wait(), Ok(None)));
        self.tray.refresh();
        self.notifications.refresh();
        self.refresh_model();
        self.refresh_status();
        self.refresh_config();
        self.sync_chrome();
        self.sync_surfaces();
    }

    fn handle_control_requests(&mut self) {
        let Some(control) = &self.control else {
            return;
        };

        match control.drain() {
            Ok(requests) => {
                for request in requests {
                    match request {
                        asher_ipc::ShellControlRequest::LaunchDefaultApp { app } => {
                            self.launch_default_app(app)
                        }
                        asher_ipc::ShellControlRequest::OpenLauncher => self.open_launcher(),
                        asher_ipc::ShellControlRequest::ToggleOverview => self.toggle_overview(),
                        asher_ipc::ShellControlRequest::CloseTransientPopovers => {
                            self.close_transient_popovers()
                        }
                    }
                }
            }
            Err(error) => warn!(%error, "failed to read shell control request"),
        }
    }

    fn handle_action(&mut self, action: WebShellAction) {
        match action {
            WebShellAction::OpenLauncher => self.open_launcher(),
            WebShellAction::LaunchDefaultApp { app } => self.launch_default_app(app),
            WebShellAction::ToggleOverview => self.toggle_overview(),
            WebShellAction::ToggleQuickSettings => self.toggle_quick_settings(),
            WebShellAction::ToggleDateCenter => self.toggle_date_center(),
            WebShellAction::WorkspaceSwitch { workspace } => {
                self.apply_model_result(switch_workspace(workspace_id(workspace)));
                self.hide_chrome();
            }
            WebShellAction::WorkspaceRelative { offset } => {
                self.apply_model_result(switch_relative_workspace(offset))
            }
            WebShellAction::WorkspaceNew => self.new_workspace_from_overview(),
            WebShellAction::WorkspaceSetProfile { profile } => {
                self.set_active_workspace_profile(profile)
            }
            WebShellAction::WindowActivate { window } => self.activate_task_window(window),
            WebShellAction::WindowMove { window, workspace } => self.apply_model_result(
                move_window_to_workspace(window_id(window), workspace_id(workspace)),
            ),
            WebShellAction::DockLaunch { command } => self.activate_dock_command(command),
            WebShellAction::DockMenuOpen { command, x } => self.open_dock_menu(command, x),
            WebShellAction::DockMenuClose => self.close_dock_menu(),
            WebShellAction::DockPin {
                label,
                command,
                icon,
            } => self.pin_dock_app(label, command, icon),
            WebShellAction::DockUnpin { command } => self.unpin_dock_app(&command),
            WebShellAction::DockReorder { commands } => self.reorder_dock_apps(commands),
            WebShellAction::AppLaunch { command } => {
                self.hide_chrome();
                self.launch(command);
            }
            WebShellAction::TrayActivate { index } => self.activate_tray(index, false),
            WebShellAction::TrayMenu { index } => self.activate_tray(index, true),
            WebShellAction::QuickOpenSettings { page } => self.open_settings_page(page),
            WebShellAction::QuickSetVolume { percent } => {
                if let Err(error) = set_audio_volume(percent) {
                    warn!(%error, "failed to set audio volume");
                }
                self.refresh_status_now();
            }
            WebShellAction::QuickToggleMute => {
                if let Err(error) = toggle_audio_mute() {
                    warn!(%error, "failed to toggle audio mute");
                }
                self.refresh_status_now();
            }
            WebShellAction::QuickSetBrightness { percent } => {
                if let Err(error) = set_brightness(percent) {
                    warn!(%error, "failed to set brightness");
                }
                self.refresh_status_now();
            }
            WebShellAction::QuickToggleDebugOverlay => {
                self.apply_model_result(set_debug_overlay(!self.model.debug_overlay))
            }
            WebShellAction::SessionCommand { command } => self.run_session_command(command),
            WebShellAction::ReloadConfig => self.reload_config_from_command(),
            WebShellAction::OpenLogsFolder => self.open_logs_folder(),
            WebShellAction::ToggleSafeMode => self.toggle_safe_mode(),
            WebShellAction::NotificationClose { notification } => {
                self.notifications.close(notification);
            }
            WebShellAction::NotificationClearAll => {
                self.notifications.clear_all();
            }
            WebShellAction::NotificationDoNotDisturb { enabled } => {
                self.notifications.set_do_not_disturb(enabled);
            }
            WebShellAction::NotificationAction {
                notification,
                action,
            } => {
                self.notifications.invoke(notification, action);
            }
        }
    }

    fn open_settings_page(&mut self, page: QuickSettingsPage) {
        self.hide_chrome();
        let command = settings_command(&self.config.default_apps.settings, page.as_settings_arg());
        if !command.trim().is_empty() {
            self.launch(command);
        }
    }

    fn open_launcher(&mut self) {
        self.hide_chrome();
        if self.launcher_command.trim().is_empty() {
            return;
        }
        match spawn_command(
            &self.launcher_command,
            self.model.xwayland_display.as_deref(),
        ) {
            Ok(child) => {
                debug!(pid = child.id(), command = %self.launcher_command, "launched app launcher");
                self.app_processes.push(child);
            }
            Err(error) => {
                warn!(%error, command = %self.launcher_command, "failed to launch app launcher")
            }
        }
    }

    fn launch_default_app(&mut self, app: asher_ipc::DefaultAppKind) {
        if app == asher_ipc::DefaultAppKind::Settings {
            self.open_settings_page(QuickSettingsPage::Appearance);
            return;
        }
        let command = match app {
            asher_ipc::DefaultAppKind::Terminal => self.config.default_apps.terminal.clone(),
            asher_ipc::DefaultAppKind::FileManager => self.config.default_apps.file_manager.clone(),
            asher_ipc::DefaultAppKind::Browser => self.config.default_apps.browser.clone(),
            asher_ipc::DefaultAppKind::Settings => String::new(),
        };
        self.hide_chrome();
        if !command.trim().is_empty() {
            self.launch(command);
        }
    }

    fn run_session_command(&mut self, command: SessionCommand) {
        let command = match command {
            SessionCommand::Lock => self.config.session.lock_command.clone(),
            SessionCommand::Suspend => self.config.session.suspend_command.clone(),
            SessionCommand::Reboot => self.config.session.reboot_command.clone(),
            SessionCommand::PowerOff => self.config.session.poweroff_command.clone(),
        };
        self.hide_chrome();
        match spawn_command(&command, self.model.xwayland_display.as_deref()) {
            Ok(child) => {
                debug!(pid = child.id(), command, "started session command");
                self.app_processes.push(child);
            }
            Err(error) => warn!(%error, command, "failed to start session command"),
        }
    }

    fn toggle_overview(&mut self) {
        self.quick_visible = false;
        self.date_visible = false;
        self.overview_visible = !self.overview_visible;
        self.sync_chrome();
        self.sync_surfaces();
        self.surfaces.quick.set_visible(false);
        self.surfaces.date.set_visible(false);
        self.surfaces.overview.set_visible(self.overview_visible);
    }

    fn toggle_quick_settings(&mut self) {
        self.date_visible = false;
        self.overview_visible = false;
        self.quick_visible = !self.quick_visible;
        self.refresh_status_now();
        self.sync_chrome();
        self.sync_surfaces();
        self.surfaces.quick.set_visible(self.quick_visible);
        self.surfaces.date.set_visible(false);
        self.surfaces.overview.set_visible(false);
    }
    fn toggle_date_center(&mut self) {
        self.quick_visible = false;
        self.overview_visible = false;
        self.date_visible = !self.date_visible;
        self.sync_chrome();
        self.sync_surfaces();
        self.surfaces.date.set_visible(self.date_visible);
        self.surfaces.quick.set_visible(false);
        self.surfaces.overview.set_visible(false);
    }

    fn close_transient_popovers(&mut self) {
        self.quick_visible = false;
        self.date_visible = false;
        self.close_dock_menu();
        self.sync_chrome();
        self.sync_surfaces();
        self.surfaces.quick.set_visible(false);
        self.surfaces.date.set_visible(false);
    }

    fn new_workspace_from_overview(&mut self) {
        let previous = self.model.active_workspace.clone();
        self.apply_model_result(switch_relative_workspace(1));
        if self.model.active_workspace != previous {
            self.hide_chrome();
        }
    }

    fn set_active_workspace_profile(&mut self, profile: String) {
        let profile = profile_id(profile);
        if profile == self.model.active_profile {
            self.hide_chrome();
            return;
        }
        self.apply_model_result(set_workspace_profile(
            self.model.active_workspace.clone(),
            profile,
        ));
        self.hide_chrome();
    }

    fn hide_chrome(&mut self) {
        self.overview_visible = false;
        self.quick_visible = false;
        self.date_visible = false;
        self.close_dock_menu();
        self.surfaces.overview.set_visible(false);
        self.surfaces.quick.set_visible(false);
        self.surfaces.date.set_visible(false);
    }
    fn activate_task_window(&mut self, window: u64) {
        let id = window_id(window);
        let result = if self
            .model
            .windows
            .iter()
            .any(|summary| summary.id == id && summary.is_active && summary.is_visible)
        {
            minimize_window(id)
        } else {
            activate_window(id)
        };
        self.apply_model_result(result);
        self.hide_chrome();
    }

    fn activate_dock_command(&mut self, command: String) {
        self.close_dock_menu();
        let Some(app) = self
            .dock_apps
            .iter()
            .find(|app| app.command == command)
            .cloned()
        else {
            self.launch(command);
            return;
        };

        if let Some(window) = self.dock_window_for(&app) {
            self.activate_task_window(window.0);
        } else {
            self.launch(app.command);
        }
    }

    fn dock_window_for(&self, app: &DockApp) -> Option<asher_layout::WindowId> {
        self.model
            .windows
            .iter()
            .find(|window| window.is_active && dock_app_matches_window(app, window))
            .or_else(|| {
                self.model
                    .windows
                    .iter()
                    .find(|window| window.is_visible && dock_app_matches_window(app, window))
            })
            .or_else(|| {
                self.model
                    .windows
                    .iter()
                    .find(|window| dock_app_matches_window(app, window))
            })
            .map(|window| window.id)
    }

    fn pin_dock_app(&mut self, label: String, command: String, icon: Option<String>) {
        let mut config = self.config.clone();
        if dock::pin_app(&mut config, &self.dock_apps, label, command, icon) {
            self.save_shell_config(config);
        }
    }

    fn unpin_dock_app(&mut self, command: &str) {
        let mut config = self.config.clone();
        if dock::unpin_app(&mut config, &self.dock_apps, command) {
            self.save_shell_config(config);
        }
    }

    fn reorder_dock_apps(&mut self, commands: Vec<String>) {
        let mut config = self.config.clone();
        if dock::reorder_apps(&mut config, &self.dock_apps, commands) {
            self.save_shell_config(config);
        }
    }

    fn launch(&mut self, command: String) {
        match spawn_command(&command, self.model.xwayland_display.as_deref()) {
            Ok(child) => {
                debug!(pid = child.id(), command, "launched dock app");
                self.app_processes.push(child);
            }
            Err(error) => warn!(%error, command, "failed to launch dock app"),
        }
    }

    fn open_dock_menu(&mut self, command: String, x: Option<i32>) {
        if self.dock_menu_open
            && self.dock_menu_command.as_deref() == Some(command.as_str())
            && self.dock_menu_x == x
        {
            return;
        }
        self.dock_menu_open = true;
        self.dock_menu_command = Some(command);
        self.dock_menu_x = x;
        self.surfaces.set_dock_menu_x(x);
        self.sync_surfaces();
        self.surfaces.set_dock_menu_visible(true);
    }

    fn close_dock_menu(&mut self) {
        if !self.dock_menu_open {
            return;
        }
        self.dock_menu_open = false;
        self.surfaces.set_dock_menu_visible(false);
    }

    fn activate_tray(&self, index: usize, menu: bool) {
        let Some(item) = self.tray.snapshot().items.get(index) else {
            return;
        };
        if menu {
            self.tray.context_menu(item, 0, 0);
        } else {
            self.tray.activate(item, 0, 0);
        }
    }
}
