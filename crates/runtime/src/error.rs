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

    #[error("Project '{0}' not found in registry")]
    ProjectNotFound(String),

    #[error("Storage volume disconnected or unavailable: {0}")]
    VolumeDisconnected(String),

    #[error("Process identity mismatch / PID reuse detected: {0}")]
    ProcessIdentityMismatch(String),

    #[error("Process mismatch: expected '{expected}', found '{found}'")]
    ProcessMismatch { expected: String, found: String },

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Database destructive operation unauthorized: {0}")]
    DatabaseUnauthorized(String),

    #[error("Database destructive operation disabled / unavailable: {0}")]
    DatabaseDeletionUnavailable(String),

    #[error("Runtime configuration error: {0}")]
    ConfigurationError(String),

    #[error("Database backup creation failed: {0}")]
    BackupError(String),

    #[error("Database restore failed: {0}")]
    RestoreError(String),

    #[error("Credential security violation: {0}")]
    CredentialError(String),

    #[error("IO error in runtime supervisor: {0}")]
    Io(#[from] std::io::Error),

    #[error("Core error: {0}")]
    Core(#[from] localforge_core::LocalForgeError),
}

pub type Result<T> = std::result::Result<T, RuntimeError>;
