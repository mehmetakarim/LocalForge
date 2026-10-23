use std::path::PathBuf;
use thiserror::Error;

/// Canonical error enum for all LocalForge domain operations.
#[derive(Error, Debug)]
pub enum LocalForgeError {
    #[error("Path traversal security violation: path '{path}' escapes project boundary '{root}'")]
    PathTraversalViolation { path: PathBuf, root: PathBuf },

    #[error("Target volume '{volume_uuid}' at '{mount_path}' is disconnected or unavailable")]
    VolumeDisconnected {
        volume_uuid: String,
        mount_path: PathBuf,
    },

    #[error(
        "Target volume UUID mismatch: expected '{expected}', found '{actual}' at '{mount_path}'"
    )]
    VolumeMismatch {
        expected: String,
        actual: String,
        mount_path: PathBuf,
    },

    #[error("Refusing to create directory on unmounted external volume: '{path}'")]
    UnmountedExternalPathProtection { path: PathBuf },

    #[error("Project manifest not found at: '{path}'")]
    ManifestNotFound { path: PathBuf },

    #[error("Failed to parse project manifest: {0}")]
    ManifestParseError(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Platform error: {0}")]
    PlatformError(String),

    #[error("Remote deployment transport is not implemented in this release: production transfers are disabled to prevent unverified mutations")]
    DeploymentNotImplemented,

    #[error("Unauthorized deployment execution: automated MCP or agent self-approval is rejected. Production deployments require native application-mediated human authorization")]
    UnauthorizedDeploymentExecution,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(String),
}

pub type Result<T> = std::result::Result<T, LocalForgeError>;
