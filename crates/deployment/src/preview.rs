use crate::diff::DeploymentDiff;
use chrono::Utc;
use localforge_core::{LocalForgeError, Result};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentPreview {
    pub preview_id: String,
    pub target_id: String,
    pub diff: DeploymentDiff,
    pub created_at: String,
    pub requires_human_approval: bool,
}

impl DeploymentPreview {
    pub fn new(target_id: String, diff: DeploymentDiff) -> Self {
        Self {
            preview_id: format!("prev_{}", Uuid::new_v4().simple()),
            target_id,
            diff,
            created_at: Utc::now().to_rfc3339(),
            requires_human_approval: true,
        }
    }

    /// Verifies that the human approval token matches this preview before executing transfers.
    pub fn verify_approval_token(&self, token: &str) -> Result<()> {
        if token.trim() == self.preview_id {
            Ok(())
        } else {
            Err(LocalForgeError::Serialization(
                "Human approval gate failed: invalid or mismatched preview approval token".into(),
            ))
        }
    }
}
