use crate::filter::is_excluded_from_deployment;
use crate::ledger::{compute_sha256, DeploymentLedger, FileChecksumEntry};
use localforge_core::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FileChangeType {
    Added,
    Modified,
    Deleted,
    Unchanged,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentFileDiff {
    pub relative_path: String,
    pub change_type: FileChangeType,
    pub sha256: Option<String>,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentDiff {
    pub target_id: String,
    pub changes: Vec<DeploymentFileDiff>,
    pub total_upload_bytes: u64,
    pub to_add_count: usize,
    pub to_modify_count: usize,
    pub to_delete_count: usize,
}

/// Compares current working tree against the last recorded deployment ledger.
pub fn calculate_deployment_diff(
    project_root: &Path,
    ledger: &DeploymentLedger,
) -> Result<DeploymentDiff> {
    let mut changes = Vec::new();
    let mut seen_rel_paths = HashSet::new();
    let mut total_upload_bytes = 0u64;

    let mut to_add_count = 0;
    let mut to_modify_count = 0;
    let mut to_delete_count = 0;

    // Scan disk
    for entry in WalkDir::new(project_root)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() {
            if let Ok(rel) = path.strip_prefix(project_root) {
                if is_excluded_from_deployment(rel) {
                    continue;
                }

                let rel_str = rel.to_string_lossy().to_string();
                seen_rel_paths.insert(rel_str.clone());

                let file_size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                let current_sha = compute_sha256(path)?;

                if let Some(recorded) = ledger.files.get(&rel_str) {
                    if recorded.sha256 == current_sha {
                        changes.push(DeploymentFileDiff {
                            relative_path: rel_str,
                            change_type: FileChangeType::Unchanged,
                            sha256: Some(current_sha),
                            size_bytes: file_size,
                        });
                    } else {
                        to_modify_count += 1;
                        total_upload_bytes += file_size;
                        changes.push(DeploymentFileDiff {
                            relative_path: rel_str,
                            change_type: FileChangeType::Modified,
                            sha256: Some(current_sha),
                            size_bytes: file_size,
                        });
                    }
                } else {
                    to_add_count += 1;
                    total_upload_bytes += file_size;
                    changes.push(DeploymentFileDiff {
                        relative_path: rel_str,
                        change_type: FileChangeType::Added,
                        sha256: Some(current_sha),
                        size_bytes: file_size,
                    });
                }
            }
        }
    }

    // Detect deleted files
    for (recorded_rel, entry) in &ledger.files {
        if !seen_rel_paths.contains(recorded_rel) {
            to_delete_count += 1;
            changes.push(DeploymentFileDiff {
                relative_path: recorded_rel.clone(),
                change_type: FileChangeType::Deleted,
                sha256: None,
                size_bytes: entry.size_bytes,
            });
        }
    }

    Ok(DeploymentDiff {
        target_id: ledger.target_id.clone(),
        changes,
        total_upload_bytes,
        to_add_count,
        to_modify_count,
        to_delete_count,
    })
}

/// Updates a deployment ledger with the successfully transferred changes.
pub fn apply_diff_to_ledger(ledger: &mut DeploymentLedger, diff: &DeploymentDiff) {
    for c in &diff.changes {
        match c.change_type {
            FileChangeType::Added | FileChangeType::Modified => {
                if let Some(sha) = &c.sha256 {
                    ledger.files.insert(
                        c.relative_path.clone(),
                        FileChecksumEntry {
                            relative_path: c.relative_path.clone(),
                            sha256: sha.clone(),
                            size_bytes: c.size_bytes,
                        },
                    );
                }
            }
            FileChangeType::Deleted => {
                ledger.files.remove(&c.relative_path);
            }
            FileChangeType::Unchanged => {}
        }
    }
}
