use asher_ipc::{IpcRequest, IpcResponse, send_request};
use asher_layout::{ProfileId, WorkspaceId};
use std::{error::Error, io};

#[derive(Debug, Clone)]
pub struct SettingsShellModel {
    pub active_workspace: WorkspaceId,
}

pub fn load_model() -> Result<SettingsShellModel, Box<dyn Error>> {
    let status = match send_request(&IpcRequest::Status)? {
        IpcResponse::Status(status) => status,
        IpcResponse::Error { message } => return Err(message.into()),
        response => return Err(unexpected_response(response).into()),
    };
    Ok(SettingsShellModel {
        active_workspace: status.active_workspace,
    })
}

pub fn reload_config() -> Result<(), Box<dyn Error>> {
    send_accepted(IpcRequest::Reload)
}

pub fn set_workspace_profile(
    workspace: WorkspaceId,
    profile: ProfileId,
) -> Result<(), Box<dyn Error>> {
    send_accepted(IpcRequest::SetWorkspaceProfile { workspace, profile })
}

fn send_accepted(request: IpcRequest) -> Result<(), Box<dyn Error>> {
    match send_request(&request)? {
        IpcResponse::Accepted => Ok(()),
        IpcResponse::Error { message } => Err(message.into()),
        response => Err(unexpected_response(response).into()),
    }
}

fn unexpected_response(response: IpcResponse) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        format!("unexpected ipc response: {response:?}"),
    )
}
