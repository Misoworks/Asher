#[cfg(feature = "session-backend")]
mod device;
#[cfg(feature = "session-backend")]
mod frame;
#[cfg(feature = "session-backend")]
mod runtime;
#[cfg(feature = "session-backend")]
mod scanout;

use asher_config::AsherConfig;
use thiserror::Error;

pub struct DrmOptions {
    pub config: AsherConfig,
    pub socket_name: Option<String>,
}

#[cfg(feature = "session-backend")]
pub fn run(options: DrmOptions) -> Result<(), DrmError> {
    runtime::run(options)
}

#[cfg(not(feature = "session-backend"))]
pub fn run(options: DrmOptions) -> Result<(), DrmError> {
    let _config = options.config;
    let _socket_name = options.socket_name;
    Err(DrmError::Unsupported(
        "DRM/KMS session backend requires building Kestrel with --features session-backend"
            .to_string(),
    ))
}

#[derive(Debug, Error)]
pub enum DrmError {
    #[error("{0}")]
    Unsupported(String),
}
