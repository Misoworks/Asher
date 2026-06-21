use crate::session_list::SessionEntry;
use pam_client2::{Context, ConversationHandler, ErrorCode, Flag};
use std::{
    ffi::{CStr, CString},
    io::{self, Read},
    os::unix::process::CommandExt,
    process::{Command, ExitCode, ExitStatus},
};
use thiserror::Error;
use users::{get_user_by_name, get_user_groups, os::unix::UserExt};

pub struct AuthLaunchRequest<'a> {
    pub user: &'a str,
    pub password: String,
    pub session: &'a SessionEntry,
    pub service: &'a str,
    pub dry_run: bool,
}

#[derive(Debug, Error)]
pub enum AuthLaunchError {
    #[error("unknown user {0}")]
    UnknownUser(String),
    #[error("PAM rejected the login: {0}")]
    Pam(String),
    #[error("failed to launch session: {0}")]
    Launch(#[from] io::Error),
}

pub fn read_password_stdin() -> io::Result<String> {
    let mut password = String::new();
    io::stdin().read_to_string(&mut password)?;
    while password.ends_with(['\n', '\r']) {
        password.pop();
    }
    Ok(password)
}

pub fn authenticate_and_launch(
    request: AuthLaunchRequest<'_>,
) -> Result<ExitCode, AuthLaunchError> {
    if request.dry_run {
        println!("{}: {}", request.user, request.session.exec);
        return Ok(ExitCode::SUCCESS);
    }

    let user = get_user_by_name(request.user)
        .ok_or_else(|| AuthLaunchError::UnknownUser(request.user.to_string()))?;
    let mut context = Context::new(
        request.service,
        Some(request.user),
        PasswordConversation::new(request.user, request.password),
    )
    .map_err(pam_error)?;

    context.authenticate(Flag::NONE).map_err(pam_error)?;
    context.acct_mgmt(Flag::NONE).map_err(pam_error)?;

    let session = context.open_session(Flag::NONE).map_err(pam_error)?;
    let pam_env = session.envlist();
    let groups = get_user_groups(user.name(), user.primary_group_id())
        .unwrap_or_default()
        .into_iter()
        .map(|group| group.gid() as libc::gid_t)
        .collect::<Vec<_>>();
    let uid = user.uid() as libc::uid_t;
    let gid = user.primary_group_id() as libc::gid_t;
    let mut command = Command::new("sh");
    command
        .arg("-lc")
        .arg(&request.session.exec)
        .env_clear()
        .envs(pam_env.iter_tuples())
        .env("USER", user.name())
        .env("LOGNAME", user.name())
        .env("HOME", user.home_dir())
        .env("SHELL", user.shell())
        .env("DESKTOP_SESSION", &request.session.id)
        .env("XDG_CURRENT_DESKTOP", "Asher")
        .env("XDG_SESSION_DESKTOP", &request.session.id)
        .env("XDG_SESSION_TYPE", "wayland")
        .current_dir(user.home_dir());

    unsafe {
        command.pre_exec(move || {
            if libc::setsid() < 0 {
                return Err(io::Error::last_os_error());
            }
            if libc::setgroups(groups.len(), groups.as_ptr()) != 0 {
                return Err(io::Error::last_os_error());
            }
            if libc::setgid(gid) != 0 {
                return Err(io::Error::last_os_error());
            }
            if libc::setuid(uid) != 0 {
                return Err(io::Error::last_os_error());
            }
            Ok(())
        });
    }

    let status = command.status()?;
    drop(session);
    Ok(exit_code(status))
}

fn exit_code(status: ExitStatus) -> ExitCode {
    match status.code() {
        Some(code) => ExitCode::from(code.clamp(0, 255) as u8),
        None => ExitCode::FAILURE,
    }
}

fn pam_error(error: pam_client2::Error) -> AuthLaunchError {
    AuthLaunchError::Pam(error.to_string())
}

struct PasswordConversation {
    username: String,
    password: String,
}

impl PasswordConversation {
    fn new(username: &str, password: String) -> Self {
        Self {
            username: username.to_string(),
            password,
        }
    }
}

impl ConversationHandler for PasswordConversation {
    fn prompt_echo_on(&mut self, _prompt: &CStr) -> Result<CString, ErrorCode> {
        CString::new(self.username.clone()).map_err(|_| ErrorCode::BUF_ERR)
    }

    fn prompt_echo_off(&mut self, _prompt: &CStr) -> Result<CString, ErrorCode> {
        CString::new(self.password.clone()).map_err(|_| ErrorCode::BUF_ERR)
    }

    fn text_info(&mut self, _msg: &CStr) {}

    fn error_msg(&mut self, _msg: &CStr) {}
}
