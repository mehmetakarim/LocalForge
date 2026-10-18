use crate::diff::FileChangeType;
use crate::ledger::DeploymentLedger;
use crate::preview::DeploymentPreview;
use chrono::Utc;
use localforge_core::{LocalForgeError, Result};
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
    pub simulated_files_to_transfer: Vec<String>,
    pub simulated_files_to_delete: Vec<String>,
    pub total_bytes: u64,
    pub completed_at: String,
    pub is_dry_run: bool,
    pub is_simulation: bool,
}

pub struct DeploymentEngine;

impl DeploymentEngine {
    /// Executes a deployment after validating human approval.
    ///
    /// SAFETY / RECOVERY R0 POLICY:
    /// In this release, non-dry-run deployments are strictly rejected with `LocalForgeError::DeploymentNotImplemented`
    /// to prevent unverified ledger mutations and fake file transfer reports.
    ///
    /// Dry-run operations perform read-only simulation:
    /// - They do NOT report files as transferred (transferred_files is empty).
    /// - They do NOT mutate the deployment ledger.
    /// - They do NOT write any ledger state to disk.
    /// - They explicitly label output with `is_simulation: true`.
    pub fn execute_deployment(
        _project_root: &Path,
        preview: &DeploymentPreview,
        approval_token: &str,
        _ledger: &mut DeploymentLedger,
        is_dry_run: bool,
    ) -> Result<DeploymentExecutionResult> {
        // 1. Mandatory Human Approval Verification
        preview.verify_approval_token(approval_token)?;

        // 2. Reject non-dry-run execution (Fail-Closed)
        if !is_dry_run {
            return Err(LocalForgeError::DeploymentNotImplemented);
        }

        // 3. Dry-run read-only simulation
        let mut simulated_transfer = Vec::new();
        let mut simulated_deleted = Vec::new();

        for file_diff in &preview.diff.changes {
            match file_diff.change_type {
                FileChangeType::Added | FileChangeType::Modified => {
                    simulated_transfer.push(file_diff.relative_path.clone());
                }
                FileChangeType::Deleted => {
                    simulated_deleted.push(file_diff.relative_path.clone());
                }
                FileChangeType::Unchanged => {}
            }
        }

        // DO NOT update or persist deployment ledger during simulation
        Ok(DeploymentExecutionResult {
            preview_id: preview.preview_id.clone(),
            target_id: preview.target_id.clone(),
            transferred_files: Vec::new(), // Zero actual transfers
            deleted_files: Vec::new(),     // Zero actual deletions
            simulated_files_to_transfer: simulated_transfer,
            simulated_files_to_delete: simulated_deleted,
            total_bytes: preview.diff.total_upload_bytes,
            completed_at: Utc::now().to_rfc3339(),
            is_dry_run: true,
            is_simulation: true,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diff::calculate_deployment_diff;
    use tempfile::tempdir;

    #[test]
    fn test_non_dry_run_fails_safely_with_deployment_not_implemented() {
        let dir = tempdir().unwrap();
        let proj = dir.path();

        std::fs::write(proj.join("index.php"), "<?php echo 'hello';").unwrap();

        let mut ledger = DeploymentLedger::load(proj, "production").unwrap();
        let diff = calculate_deployment_diff(proj, &ledger).unwrap();
        let preview = DeploymentPreview::new("production".into(), diff);

        let initial_last_deployed = ledger.last_deployed_at.clone();

        // Attempt non-dry-run deployment with valid token
        let res = DeploymentEngine::execute_deployment(
            proj,
            &preview,
            &preview.preview_id,
            &mut ledger,
            false,
        );

        // 1. Must fail with DeploymentNotImplemented
        assert!(res.is_err());
        match res.unwrap_err() {
            LocalForgeError::DeploymentNotImplemented => {}
            other => panic!("Expected DeploymentNotImplemented, got: {:?}", other),
        }

        // 2. Ledger in-memory state must not be mutated
        assert_eq!(ledger.last_deployed_at, initial_last_deployed);

        // 3. Ledger file on disk must NOT exist
        let ledger_file = proj
            .join(".localforge")
            .join("deployments")
            .join("production.json");
        assert!(
            !ledger_file.exists(),
            "Ledger file must not be created on failed deployment"
        );
    }

    #[test]
    fn test_dry_run_simulation_does_not_mutate_ledger_and_reports_zero_actual_transfers() {
        let dir = tempdir().unwrap();
        let proj = dir.path();

        std::fs::write(proj.join("index.php"), "<?php echo 'hello';").unwrap();

        let mut ledger = DeploymentLedger::load(proj, "production").unwrap();
        let diff = calculate_deployment_diff(proj, &ledger).unwrap();
        let preview = DeploymentPreview::new("production".into(), diff);

        let initial_last_deployed = ledger.last_deployed_at.clone();

        // Execute dry-run simulation
        let res = DeploymentEngine::execute_deployment(
            proj,
            &preview,
            &preview.preview_id,
            &mut ledger,
            true,
        )
        .expect("Dry-run simulation should succeed with valid token");

        // 1. Result must be explicitly labeled as simulation
        assert!(res.is_dry_run);
        assert!(res.is_simulation);

        // 2. Zero actual transferred or deleted files reported
        assert!(
            res.transferred_files.is_empty(),
            "Must not report transferred files when no transfer occurred"
        );
        assert!(
            res.deleted_files.is_empty(),
            "Must not report deleted files when no deletion occurred"
        );

        // 3. Simulated files to transfer are accurately listed
        assert_eq!(
            res.simulated_files_to_transfer,
            vec!["index.php".to_string()]
        );

        // 4. Ledger last_deployed_at is not modified
        assert_eq!(ledger.last_deployed_at, initial_last_deployed);

        // 5. No ledger file written to disk
        let ledger_file = proj
            .join(".localforge")
            .join("deployments")
            .join("production.json");
        assert!(
            !ledger_file.exists(),
            "Dry-run must not create or modify ledger on disk"
        );
    }

    #[test]
    fn test_approval_token_gate_rejection() {
        let dir = tempdir().unwrap();
        let proj = dir.path();

        std::fs::write(proj.join("index.php"), "<?php echo 'hello';").unwrap();

        let mut ledger = DeploymentLedger::load(proj, "production").unwrap();
        let diff = calculate_deployment_diff(proj, &ledger).unwrap();
        let preview = DeploymentPreview::new("production".into(), diff);

        // Fail without matching token even for dry-run
        let fail_res = DeploymentEngine::execute_deployment(
            proj,
            &preview,
            "invalid_wrong_token",
            &mut ledger,
            true,
        );
        assert!(fail_res.is_err());
    }
}
