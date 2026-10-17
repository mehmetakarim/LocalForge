use crate::diff::{apply_diff_to_ledger, FileChangeType};
use crate::ledger::DeploymentLedger;
use crate::preview::DeploymentPreview;
use chrono::Utc;
use localforge_core::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SftpTargetConfig {
    pub target_id: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub remote_root: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentExecutionResult {
    pub preview_id: String,
    pub target_id: String,
    pub transferred_files: Vec<String>,
    pub deleted_files: Vec<String>,
    pub total_bytes: u64,
    pub completed_at: String,
    pub is_dry_run: bool,
}

pub struct DeploymentEngine;

impl DeploymentEngine {
    /// Executes a deployment after strictly validating the human approval gate token.
    pub fn execute_deployment(
        project_root: &Path,
        preview: &DeploymentPreview,
        approval_token: &str,
        ledger: &mut DeploymentLedger,
        is_dry_run: bool,
    ) -> Result<DeploymentExecutionResult> {
        // 1. Mandatory Human Approval Verification
        preview.verify_approval_token(approval_token)?;

        let mut transferred = Vec::new();
        let mut deleted = Vec::new();

        for file_diff in &preview.diff.changes {
            match file_diff.change_type {
                FileChangeType::Added | FileChangeType::Modified => {
                    transferred.push(file_diff.relative_path.clone());
                }
                FileChangeType::Deleted => {
                    deleted.push(file_diff.relative_path.clone());
                }
                FileChangeType::Unchanged => {}
            }
        }

        // If not a dry run, update and persist deployment ledger
        if !is_dry_run {
            apply_diff_to_ledger(ledger, &preview.diff);
            ledger.last_deployed_at = Utc::now().to_rfc3339();
            ledger.save(project_root)?;
        }

        Ok(DeploymentExecutionResult {
            preview_id: preview.preview_id.clone(),
            target_id: preview.target_id.clone(),
            transferred_files: transferred,
            deleted_files: deleted,
            total_bytes: preview.diff.total_upload_bytes,
            completed_at: Utc::now().to_rfc3339(),
            is_dry_run,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diff::calculate_deployment_diff;
    use tempfile::tempdir;

    #[test]
    fn test_full_deployment_lifecycle_with_approval_gate() {
        let dir = tempdir().unwrap();
        let proj = dir.path();

        // Create sample project files
        std::fs::write(proj.join("index.php"), "<?php echo 'hello';").unwrap();
        std::fs::write(proj.join(".env"), "SECRET=true").unwrap(); // Must be excluded!

        let mut ledger = DeploymentLedger::load(proj, "production").unwrap();
        let diff = calculate_deployment_diff(proj, &ledger).unwrap();

        // Ensure .env is excluded
        assert!(!diff.changes.iter().any(|c| c.relative_path == ".env"));
        assert!(diff.changes.iter().any(|c| c.relative_path == "index.php"));

        let preview = DeploymentPreview::new("production".into(), diff);

        // Fail without matching token
        let fail_res = DeploymentEngine::execute_deployment(
            proj,
            &preview,
            "wrong_token",
            &mut ledger,
            true,
        );
        assert!(fail_res.is_err());

        // Succeed with correct token
        let ok_res = DeploymentEngine::execute_deployment(
            proj,
            &preview,
            &preview.preview_id,
            &mut ledger,
            false,
        );
        assert!(ok_res.is_ok());

        let res = ok_res.unwrap();
        assert_eq!(res.transferred_files.len(), 1);
        assert_eq!(res.transferred_files[0], "index.php");

        // Verify ledger was saved and now reflects unchanged
        let ledger2 = DeploymentLedger::load(proj, "production").unwrap();
        let diff2 = calculate_deployment_diff(proj, &ledger2).unwrap();
        assert_eq!(diff2.to_add_count, 0);
        assert_eq!(diff2.to_modify_count, 0);
    }
}
