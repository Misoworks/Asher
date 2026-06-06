use serde::Serialize;
use staccato_config::{ConfigPaths, ConfigSource, load_config};
use staccato_ipc::{IpcRequest, IpcResponse, ShellStatus, XwaylandStatus, send_request};
use std::{
    env, fs,
    path::{Path, PathBuf},
};

pub(crate) fn doctor_checks() -> Vec<DoctorCheck> {
    let mut checks = Vec::new();
    match ConfigPaths::discover() {
        Ok(paths) => {
            checks.push(check_config());
            checks.push(check_logs_dir(&paths));
            checks.push(check_binary("baton"));
            checks.push(check_binary("staccato-shell"));
            checks.push(check_session_file());
            checks.push(check_ipc());
            checks.push(check_xwayland());
            checks.extend(check_session_services());
            checks.extend(check_portal_services());
            checks.push(check_session_backend_dependencies());
            checks.push(DoctorCheck::ok(
                "session-backend",
                "DRM/KMS backend has GBM/GLES modeset rendering with active-output hotplug refresh",
            ));
        }
        Err(error) => checks.push(DoctorCheck::fail("xdg-paths", error.to_string())),
    }
    checks
}

fn check_config() -> DoctorCheck {
    match load_config() {
        Ok(loaded) => {
            let source = match loaded.source {
                ConfigSource::User(path) => path.display().to_string(),
                ConfigSource::Defaults => "built-in defaults".to_string(),
            };
            DoctorCheck::ok("config", format!("valid ({source})"))
        }
        Err(error) => DoctorCheck::fail("config", error.to_string()),
    }
}

fn check_logs_dir(paths: &ConfigPaths) -> DoctorCheck {
    match fs::create_dir_all(paths.logs_dir()) {
        Ok(()) => DoctorCheck::ok("logs", paths.logs_dir().display().to_string()),
        Err(error) => DoctorCheck::fail("logs", format!("{}: {error}", paths.logs_dir().display())),
    }
}

fn check_binary(name: &str) -> DoctorCheck {
    if sibling_binary(name).is_some() || binary_in_path(name).is_some() {
        DoctorCheck::ok(format!("binary:{name}"), "found")
    } else {
        DoctorCheck::fail(
            format!("binary:{name}"),
            "not found beside staccatoctl or in PATH",
        )
    }
}

fn check_session_file() -> DoctorCheck {
    let installed = Path::new("/usr/share/wayland-sessions/staccato.desktop");
    if installed.exists() || Path::new("data/sessions/staccato.desktop").exists() {
        DoctorCheck::ok("session-file", "found")
    } else {
        DoctorCheck::warning(
            "session-file",
            "not installed in /usr/share/wayland-sessions",
        )
    }
}

fn check_ipc() -> DoctorCheck {
    match send_request(&IpcRequest::Status) {
        Ok(IpcResponse::Status(status)) => {
            let message = format!(
                "Baton reachable, workspace {}, shell {:?}",
                status.active_workspace.0, status.shell
            );
            match status.shell {
                ShellStatus::Running => DoctorCheck::ok("ipc", message),
                ShellStatus::Restarting => DoctorCheck::warning("ipc", message),
                ShellStatus::NotStarted | ShellStatus::Failed => {
                    DoctorCheck::warning("ipc", message)
                }
            }
        }
        Ok(response) => DoctorCheck::fail("ipc", format!("unexpected response: {response:?}")),
        Err(error) => DoctorCheck::warning("ipc", format!("Baton is not reachable: {error}")),
    }
}

fn check_xwayland() -> DoctorCheck {
    let enabled = load_config().is_ok_and(|loaded| loaded.config.compositor.xwayland);
    if !enabled {
        return DoctorCheck::ok("xwayland", "disabled in config");
    }

    if let Ok(IpcResponse::Status(status)) = send_request(&IpcRequest::Status) {
        return match status.xwayland {
            XwaylandStatus::Running => DoctorCheck::ok(
                "xwayland",
                format!(
                    "xwayland-satellite running on {}",
                    status
                        .xwayland_display
                        .unwrap_or_else(|| "unknown display".to_string())
                ),
            ),
            XwaylandStatus::Restarting => {
                DoctorCheck::warning("xwayland", "xwayland-satellite is restarting")
            }
            XwaylandStatus::Disabled => DoctorCheck::warning("xwayland", "disabled at runtime"),
            XwaylandStatus::Unavailable => {
                DoctorCheck::warning("xwayland", "xwayland-satellite is unavailable")
            }
            XwaylandStatus::Failed => {
                DoctorCheck::warning("xwayland", "xwayland-satellite failed to start")
            }
        };
    }

    if binary_in_path("xwayland-satellite").is_some() {
        DoctorCheck::ok("xwayland", "xwayland-satellite found")
    } else {
        DoctorCheck::warning("xwayland", "install xwayland-satellite for X11 app support")
    }
}

fn check_session_backend_dependencies() -> DoctorCheck {
    if binary_in_path("pkg-config").is_none() {
        return DoctorCheck::warning(
            "session-deps",
            "pkg-config is needed to build Baton with --features session-backend",
        );
    }

    match std::process::Command::new("pkg-config")
        .arg("--exists")
        .arg("libseat")
        .status()
    {
        Ok(status) if status.success() => DoctorCheck::ok("session-deps", "libseat found"),
        Ok(_) => DoctorCheck::warning(
            "session-deps",
            "libseat development files are missing; install seatd/libseat to build the DRM/KMS session backend",
        ),
        Err(error) => DoctorCheck::warning("session-deps", error.to_string()),
    }
}

fn check_session_services() -> Vec<DoctorCheck> {
    let mut checks = Vec::new();
    if binary_in_path("dbus-run-session").is_some() {
        checks.push(DoctorCheck::ok("dbus-session", "dbus-run-session found"));
    } else {
        checks.push(DoctorCheck::warning(
            "dbus-session",
            "install dbus-run-session for a private Staccato session bus",
        ));
    }

    if binary_in_path("dbus-update-activation-environment").is_some() {
        checks.push(DoctorCheck::ok(
            "dbus-activation",
            "dbus-update-activation-environment found",
        ));
    } else {
        checks.push(DoctorCheck::warning(
            "dbus-activation",
            "install dbus-update-activation-environment so activated services inherit the session display",
        ));
    }

    if binary_in_path("gnome-keyring-daemon").is_some() {
        checks.push(DoctorCheck::ok(
            "secret-service",
            "gnome-keyring-daemon found",
        ));
    } else {
        checks.push(DoctorCheck::warning(
            "secret-service",
            "install gnome-keyring-daemon or another Secret Service provider to avoid org.freedesktop.secrets timeouts",
        ));
    }

    checks.push(check_known_binary(
        "polkit-agent",
        "polkit-gnome-authentication-agent-1",
        &[
            "/usr/libexec/polkit-gnome-authentication-agent-1",
            "/usr/lib/polkit-gnome/polkit-gnome-authentication-agent-1",
            "/usr/libexec/polkit-kde-authentication-agent-1",
            "/usr/lib64/libexec/polkit-kde-authentication-agent-1",
            "/usr/bin/lxpolkit",
            "/usr/libexec/lxqt-policykit-agent",
            "/usr/bin/lxqt-policykit-agent",
            "/usr/lib/mate-polkit/polkit-mate-authentication-agent-1",
            "/usr/libexec/xfce-polkit",
        ],
        "install a PolicyKit authentication agent for GUI privilege prompts",
    ));
    checks
}

fn check_portal_services() -> Vec<DoctorCheck> {
    vec![
        check_known_binary(
            "portal-broker",
            "xdg-desktop-portal",
            &["/usr/libexec/xdg-desktop-portal"],
            "install xdg-desktop-portal so sandboxed apps can reach desktop services",
        ),
        check_known_binary(
            "portal-backend:gtk",
            "xdg-desktop-portal-gtk",
            &["/usr/libexec/xdg-desktop-portal-gtk"],
            "install xdg-desktop-portal-gtk for file chooser, print, app chooser, and settings portals",
        ),
        check_known_binary(
            "portal-backend:gnome",
            "xdg-desktop-portal-gnome",
            &["/usr/libexec/xdg-desktop-portal-gnome"],
            "install xdg-desktop-portal-gnome for screenshot/screencast and non-GTK portal fallbacks",
        ),
        check_known_binary(
            "portal-backend:secret",
            "gnome-keyring-daemon",
            &[
                "/usr/bin/gnome-keyring-daemon",
                "/usr/libexec/gnome-keyring-daemon",
            ],
            "install gnome-keyring-daemon for Secret Service portal support",
        ),
        check_portal_config(),
    ]
}

fn check_known_binary(
    check_name: &'static str,
    binary: &'static str,
    known_paths: &[&str],
    missing: &'static str,
) -> DoctorCheck {
    match known_binary(binary, known_paths) {
        Some(path) => DoctorCheck::ok(check_name, path.display().to_string()),
        None => DoctorCheck::warning(check_name, missing),
    }
}

fn check_portal_config() -> DoctorCheck {
    match portal_config_paths().into_iter().find(|path| path.exists()) {
        Some(path) => DoctorCheck::ok("portal-config", path.display().to_string()),
        None => DoctorCheck::warning(
            "portal-config",
            "install data/xdg-desktop-portal/staccato-portals.conf into xdg-desktop-portal's data or config path",
        ),
    }
}

fn portal_config_paths() -> Vec<PathBuf> {
    vec![
        PathBuf::from("data/xdg-desktop-portal/staccato-portals.conf"),
        PathBuf::from("/etc/xdg/xdg-desktop-portal/staccato-portals.conf"),
        PathBuf::from("/usr/local/share/xdg-desktop-portal/staccato-portals.conf"),
        PathBuf::from("/usr/share/xdg-desktop-portal/staccato-portals.conf"),
    ]
}

fn sibling_binary(name: &str) -> Option<PathBuf> {
    let mut path = env::current_exe().ok()?;
    path.set_file_name(name);
    path.exists().then_some(path)
}

fn known_binary(name: &str, paths: &[&str]) -> Option<PathBuf> {
    binary_in_path(name).or_else(|| {
        paths
            .iter()
            .map(Path::new)
            .find(|path| path.exists())
            .map(Path::to_path_buf)
    })
}

fn binary_in_path(name: &str) -> Option<PathBuf> {
    env::var_os("PATH")
        .into_iter()
        .flat_map(|paths| env::split_paths(&paths).collect::<Vec<_>>())
        .map(|dir| dir.join(name))
        .find(|path| path.exists())
}

#[derive(Debug, Serialize)]
pub(crate) struct DoctorCheck {
    pub(crate) name: String,
    pub(crate) severity: Severity,
    pub(crate) message: String,
}

impl DoctorCheck {
    fn ok(name: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            severity: Severity::Ok,
            message: message.into(),
        }
    }

    fn warning(name: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            severity: Severity::Warning,
            message: message.into(),
        }
    }

    fn fail(name: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            severity: Severity::Fail,
            message: message.into(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Severity {
    Ok,
    Warning,
    Fail,
}

impl Severity {
    pub(crate) const fn label(&self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Warning => "warn",
            Self::Fail => "fail",
        }
    }
}
