use thiserror::Error;

#[derive(Error, Debug)]
pub enum RuntimeError {
    #[error("Runtime binary '{binary}' (version '{version}') not found on host")]
    RuntimeNotFound { binary: String, version: String },

    #[error("Failed to spawn process '{process}': {reason}")]
    ProcessSpawnFailed { process: String, reason: String },

    #[error("Port conflict detected on port {port}: {details}")]
    PortConflict { port: u16, details: String },

    #[error("Configuration generation failed: {0}")]
    ConfigGenerationFailed(String),

    #[error("Service '{service}' unhealthy or unresponsive: {reason}")]
    ServiceUnhealthy { service: String, reason: String },

    #[error("IO error in runtime supervisor: {0}")]
    Io(#[from] std::io::Error),

    #[error("Core error: {0}")]
    Core(#[from] localforge_core::LocalForgeError),
}

pub type Result<T> = std::result::Result<T, RuntimeError>;
