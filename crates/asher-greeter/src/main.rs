mod auth_launch;
mod session_list;

use auth_launch::{AuthLaunchRequest, authenticate_and_launch, read_password_stdin};
use clap::{Parser, Subcommand};
use session_list::{SessionEntry, discover_sessions};
use std::{
    path::PathBuf,
    process::{Command, ExitCode},
};

#[derive(Debug, Parser)]
#[command(name = "asher-greeter", about = "Asher session selector")]
struct Args {
    #[command(subcommand)]
    command: GreeterCommand,
    #[arg(long = "session-dir", global = true)]
    session_dirs: Vec<PathBuf>,
}

#[derive(Debug, Subcommand)]
enum GreeterCommand {
    List,
    Launch {
        session: String,
        #[arg(long)]
        dry_run: bool,
    },
    AuthLaunch {
        user: String,
        session: String,
        #[arg(long, default_value = "asher-greeter")]
        pam_service: String,
        #[arg(long)]
        password_stdin: bool,
        #[arg(long)]
        dry_run: bool,
    },
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(error) => {
            eprintln!("asher-greeter: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<ExitCode, Box<dyn std::error::Error>> {
    let args = Args::parse();
    let sessions = discover_sessions(&session_dirs(args.session_dirs))?;
    match args.command {
        GreeterCommand::List => {
            print_sessions(&sessions)?;
            Ok(ExitCode::SUCCESS)
        }
        GreeterCommand::Launch { session, dry_run } => launch_session(&sessions, &session, dry_run),
        GreeterCommand::AuthLaunch {
            user,
            session,
            pam_service,
            password_stdin,
            dry_run,
        } => auth_launch_session(
            &sessions,
            &user,
            &session,
            &pam_service,
            password_stdin,
            dry_run,
        ),
    }
}

fn print_sessions(sessions: &[SessionEntry]) -> Result<(), Box<dyn std::error::Error>> {
    for session in sessions {
        println!("{}\t{}\t{}", session.id, session.name, session.exec);
    }
    Ok(())
}

fn launch_session(
    sessions: &[SessionEntry],
    selected: &str,
    dry_run: bool,
) -> Result<ExitCode, Box<dyn std::error::Error>> {
    let session = sessions
        .iter()
        .find(|session| session.id == selected || session.name.eq_ignore_ascii_case(selected))
        .ok_or_else(|| format!("unknown session {selected}"))?;
    if dry_run {
        println!("{}", session.exec);
        return Ok(ExitCode::SUCCESS);
    }

    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;

        let error = Command::new("sh").arg("-lc").arg(&session.exec).exec();
        Err(Box::new(error))
    }

    #[cfg(not(unix))]
    {
        let status = Command::new("sh").arg("-lc").arg(&session.exec).status()?;
        if status.success() {
            Ok(ExitCode::SUCCESS)
        } else {
            Err(format!("session exited with {status}").into())
        }
    }
}

fn auth_launch_session(
    sessions: &[SessionEntry],
    user: &str,
    selected: &str,
    pam_service: &str,
    password_stdin: bool,
    dry_run: bool,
) -> Result<ExitCode, Box<dyn std::error::Error>> {
    let session = sessions
        .iter()
        .find(|session| session.id == selected || session.name.eq_ignore_ascii_case(selected))
        .ok_or_else(|| format!("unknown session {selected}"))?;
    let password = if password_stdin {
        read_password_stdin()?
    } else if dry_run {
        String::new()
    } else {
        return Err("pass the password on stdin with --password-stdin".into());
    };

    authenticate_and_launch(AuthLaunchRequest {
        user,
        password,
        session,
        service: pam_service,
        dry_run,
    })
    .map_err(Into::into)
}

fn session_dirs(configured: Vec<PathBuf>) -> Vec<PathBuf> {
    if !configured.is_empty() {
        return configured;
    }

    [
        "/usr/share/wayland-sessions",
        "/usr/local/share/wayland-sessions",
        "/usr/share/xsessions",
        "/usr/local/share/xsessions",
    ]
    .into_iter()
    .map(PathBuf::from)
    .collect()
}
