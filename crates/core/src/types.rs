use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;
use uuid::Uuid;

/// Unique identifier for a LocalForge project.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProjectId(pub String);

impl ProjectId {
    pub fn new() -> Self {
        Self(format!("proj_{}", Uuid::new_v4().simple()))
    }

    pub fn from_string(s: impl Into<String>) -> Self {
        Self(s.into())
    }
}

impl Default for ProjectId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ProjectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Persistent hardware / filesystem UUID of a storage volume.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VolumeId(pub String);

impl VolumeId {
    pub fn new(uuid_str: impl Into<String>) -> Self {
        Self(uuid_str.into())
    }
}

impl fmt::Display for VolumeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Operational status of a registered project.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProjectStatus {
    Running,
    Stopped,
    Starting,
    Stopping,
    Degraded,
    Failed,
    Disconnected,
    Unknown,
}

impl fmt::Display for ProjectStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Running => write!(f, "running"),
            Self::Stopped => write!(f, "stopped"),
            Self::Starting => write!(f, "starting"),
            Self::Stopping => write!(f, "stopping"),
            Self::Degraded => write!(f, "degraded"),
            Self::Failed => write!(f, "failed"),
            Self::Disconnected => write!(f, "disconnected"),
            Self::Unknown => write!(f, "unknown"),
        }
    }
}

/// Storage volume information inspected from the host operating system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeInfo {
    pub uuid: VolumeId,
    pub name: String,
    pub mount_point: PathBuf,
    pub filesystem: String,
    pub is_removable: bool,
    pub is_internal: bool,
    pub total_bytes: u64,
    pub available_bytes: u64,
}
