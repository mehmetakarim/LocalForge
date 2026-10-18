use anyhow::Result;
use localforge_deployment::{
    calculate_deployment_diff, DeploymentEngine, DeploymentLedger, DeploymentPreview,
};
use localforge_platform::{MacOSVolumeManager, VolumeManager};
use localforge_project_manager::ProjectService;
use localforge_runtime::{health::is_port_available, PhpFpmSupervisor};
use localforge_storage::{sanitize_project_path, MetadataStore, VolumeBarrier};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    #[allow(dead_code)]
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    params: Option<Value>,
}

#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: &'static str,
    id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<Value>,
}

fn open_metadata_store() -> MetadataStore {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    let db_path = PathBuf::from(home).join("Library/Application Support/LocalForge/localforge.db");
    MetadataStore::open(&db_path).unwrap_or_else(|_| MetadataStore::open_in_memory().unwrap())
}

fn get_tools_manifest() -> Value {
    json!([
        {
            "name": "localforge_list_projects",
            "description": "Lists all local PHP projects registered in LocalForge, including workspace paths, status, PHP versions, and volume information.",
            "inputSchema": {
                "type": "object",
                "properties": {},
                "additionalProperties": false
            }
        },
        {
            "name": "localforge_project_status",
            "description": "Inspects status, PHP-FPM socket, domain, and document root for a specific project.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "project": {
                        "type": "string",
                        "description": "Project ID or project name"
                    }
                },
                "required": ["project"],
                "additionalProperties": false
            }
        },
        {
            "name": "localforge_runtime_diagnostics",
            "description": "Provides runtime health diagnostics including MariaDB port (3306), Nginx port (8080), installed PHP versions, and mounted external storage volumes.",
            "inputSchema": {
                "type": "object",
                "properties": {},
                "additionalProperties": false
            }
        },
        {
            "name": "localforge_deploy_preview",
            "description": "Generates a read-only cryptographic deployment diff preview for a project against a remote target, calculating added, modified, deleted files and excluding sensitive files (.env, keys, git). Notice: Real deployments require out-of-band human authorization; MCP cannot self-authorize or execute production deployments.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "project_path": {
                        "type": "string",
                        "description": "Absolute path to project directory"
                    },
                    "target": {
                        "type": "string",
                        "description": "Deployment target, e.g. production (default: production)"
                    }
                },
                "required": ["project_path"],
                "additionalProperties": false
            }
        },
        {
            "name": "localforge_deploy_execute",
            "description": "DEPRECATED / FAIL-CLOSED IN R0: Automated deployment execution via MCP is disabled to prevent AI self-authorization. Only dry-run simulations are permitted via MCP. Real deployments require application-mediated human authorization.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "project_path": {
                        "type": "string",
                        "description": "Absolute path to project directory"
                    },
                    "target": {
                        "type": "string",
                        "description": "Deployment target (default: production)"
                    },
                    "dry_run": {
                        "type": "boolean",
                        "description": "If true, simulates execution without modifying remote files or ledger. Non-dry-run is rejected."
                    }
                },
                "required": ["project_path"],
                "additionalProperties": false
            }
        }
    ])
}

fn handle_tool_call(
    name: &str,
    arguments: &Value,
    store: &MetadataStore,
    volume_manager: &dyn VolumeManager,
    project_service: &ProjectService,
) -> (Value, bool) {
    match name {
        "localforge_list_projects" => match project_service.sync_all_projects_status() {
            Ok(projects) => {
                let formatted: Vec<_> = projects
                    .into_iter()
                    .map(|p| {
                        json!({
                            "id": p.id.0,
                            "name": p.name,
                            "path": p.path.to_string_lossy(),
                            "volume_uuid": p.volume_uuid.0,
                            "document_root": p.document_root,
                            "php_version": p.php_version,
                            "local_domain": p.local_domain,
                            "status": p.status.to_string(),
                        })
                    })
                    .collect();
                (json!({ "projects": formatted }), false)
            }
            Err(e) => (json!({ "error": e.to_string() }), true),
        },
        "localforge_project_status" => {
            let query = arguments
                .get("project")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            match store.list_projects() {
                Ok(projects) => {
                    let found = projects
                        .into_iter()
                        .find(|p| p.id.0 == query || p.name.eq_ignore_ascii_case(query));

                    match found {
                        Some(p) => (
                            json!({
                                "id": p.id.0,
                                "name": p.name,
                                "path": p.path.to_string_lossy(),
                                "volume_uuid": p.volume_uuid.0,
                                "document_root": p.document_root,
                                "php_version": p.php_version,
                                "local_domain": p.local_domain,
                                "status": format!("{:?}", p.status),
                                "php_fpm_socket": p.path.join(".localforge/run/php-fpm.sock").to_string_lossy(),
                                "local_url": format!("http://{}:8080", p.local_domain)
                            }),
                            false,
                        ),
                        None => (
                            json!({ "error": format!("Project '{}' not found", query) }),
                            true,
                        ),
                    }
                }
                Err(e) => (json!({ "error": e.to_string() }), true),
            }
        }
        "localforge_runtime_diagnostics" => {
            let mariadb_busy = !is_port_available(3306);
            let nginx_busy = !is_port_available(8080);
            let php_versions = PhpFpmSupervisor::discover_installed_versions();
            let volumes = volume_manager.list_volumes().unwrap_or_default();

            (
                json!({
                    "mariadb_loopback_3306": if mariadb_busy { "active" } else { "idle" },
                    "nginx_loopback_8080": if nginx_busy { "active" } else { "idle" },
                    "installed_php_versions": php_versions,
                    "storage_volumes": volumes.into_iter().map(|v| {
                        json!({
                            "name": v.name,
                            "uuid": v.uuid.0,
                            "mount_point": v.mount_point.to_string_lossy(),
                            "is_external": !v.is_internal,
                            "is_mounted": v.mount_point.exists()
                        })
                    }).collect::<Vec<_>>()
                }),
                false,
            )
        }
        "localforge_deploy_preview" => {
            let path_str = arguments
                .get("project_path")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            let target = arguments
                .get("target")
                .and_then(|v| v.as_str())
                .unwrap_or("production");

            let path = Path::new(path_str);
            let canonical = match dunce::canonicalize(path) {
                Ok(c) => c,
                Err(e) => return (json!({ "error": format!("Path error: {}", e) }), true),
            };

            let safe_root = match sanitize_project_path(&canonical, &canonical) {
                Ok(r) => r,
                Err(e) => {
                    return (
                        json!({ "error": format!("Security barrier error: {}", e) }),
                        true,
                    )
                }
            };

            let ledger = match DeploymentLedger::load(&safe_root, target) {
                Ok(l) => l,
                Err(e) => return (json!({ "error": format!("Ledger error: {}", e) }), true),
            };

            let diff = match calculate_deployment_diff(&safe_root, &ledger) {
                Ok(d) => d,
                Err(e) => return (json!({ "error": format!("Diff error: {}", e) }), true),
            };

            let preview = DeploymentPreview::new(target.to_string(), diff);

            // Persist preview for token gate verification
            let preview_dir = safe_root.join(".localforge").join("deployments");
            if let Err(e) = std::fs::create_dir_all(&preview_dir) {
                return (
                    json!({ "error": format!("Failed to create preview dir: {}", e) }),
                    true,
                );
            }
            let preview_file = preview_dir.join(format!(".preview_{}.json", target));
            if let Err(e) = std::fs::write(
                &preview_file,
                serde_json::to_string_pretty(&preview).unwrap(),
            ) {
                return (
                    json!({ "error": format!("Failed to write preview file: {}", e) }),
                    true,
                );
            }

            (
                json!({
                    "preview_id": preview.preview_id,
                    "target": preview.target_id,
                    "to_add_count": preview.diff.to_add_count,
                    "to_modify_count": preview.diff.to_modify_count,
                    "to_delete_count": preview.diff.to_delete_count,
                    "total_upload_bytes": preview.diff.total_upload_bytes,
                    "requires_human_approval": true,
                    "approval_status": "PENDING_OUT_OF_BAND_HUMAN_APPROVAL",
                    "security_notice": "Approval credentials are never exposed via MCP. Production deployments must be approved directly by a human operator in LocalForge Desktop or verified CLI.",
                    "changes": preview.diff.changes.iter().map(|c| {
                        json!({
                            "relative_path": c.relative_path,
                            "change_type": format!("{:?}", c.change_type),
                            "size_bytes": c.size_bytes
                        })
                    }).collect::<Vec<_>>()
                }),
                false,
            )
        }
        "localforge_deploy_execute" => {
            let dry_run = arguments
                .get("dry_run")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            // R0-02 SECURITY BOUNDARY:
            // MCP automated deployment execution is strictly rejected for production transfers.
            // AI agents are not permitted to self-authorize or trigger remote file modifications.
            if !dry_run {
                return (
                    json!({
                        "error": "Deployment execution rejected: Automated MCP production deployment is disabled in Recovery Phase R0. AI agents cannot self-authorize production transfers. Real deployments strictly require out-of-band application-mediated human authorization.",
                        "error_code": "UNAUTHORIZED_MCP_DEPLOYMENT_EXECUTION",
                        "requires_human_approval": true
                    }),
                    true,
                );
            }

            let path_str = arguments
                .get("project_path")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            let target = arguments
                .get("target")
                .and_then(|v| v.as_str())
                .unwrap_or("production");

            let path = Path::new(path_str);
            let canonical = match dunce::canonicalize(path) {
                Ok(c) => c,
                Err(e) => return (json!({ "error": format!("Path error: {}", e) }), true),
            };

            let safe_root = match sanitize_project_path(&canonical, &canonical) {
                Ok(r) => r,
                Err(e) => {
                    return (
                        json!({ "error": format!("Security barrier error: {}", e) }),
                        true,
                    )
                }
            };

            let preview_file = safe_root
                .join(".localforge")
                .join("deployments")
                .join(format!(".preview_{}.json", target));

            if !preview_file.exists() {
                return (
                    json!({ "error": "No active preview found for this target. Please generate a preview first." }),
                    true,
                );
            }

            let preview_data = match std::fs::read_to_string(&preview_file) {
                Ok(d) => d,
                Err(e) => {
                    return (
                        json!({ "error": format!("Failed to read preview: {}", e) }),
                        true,
                    )
                }
            };

            let preview: DeploymentPreview = match serde_json::from_str(&preview_data) {
                Ok(p) => p,
                Err(e) => {
                    return (
                        json!({ "error": format!("Failed to parse preview: {}", e) }),
                        true,
                    )
                }
            };

            let mut ledger = match DeploymentLedger::load(&safe_root, target) {
                Ok(l) => l,
                Err(e) => {
                    return (
                        json!({ "error": format!("Failed to load ledger: {}", e) }),
                        true,
                    )
                }
            };

            // For dry_run simulation, invoke engine in dry_run mode
            match DeploymentEngine::execute_deployment(
                &safe_root,
                &preview,
                &preview.preview_id,
                &mut ledger,
                true,
            ) {
                Ok(res) => (
                    json!({
                        "status": "simulation_only",
                        "preview_id": res.preview_id,
                        "target_id": res.target_id,
                        "simulated_transfers_count": res.simulated_files_to_transfer.len(),
                        "simulated_deletions_count": res.simulated_files_to_delete.len(),
                        "transferred_count": 0,
                        "deleted_count": 0,
                        "total_bytes": res.total_bytes,
                        "completed_at": res.completed_at,
                        "is_dry_run": true,
                        "is_simulation": true,
                        "message": "Dry-run simulation completed. No remote transfers occurred and no ledger state was mutated."
                    }),
                    false,
                ),
                Err(e) => (
                    json!({ "error": format!("Deployment simulation failed: {}", e) }),
                    true,
                ),
            }
        }
        _ => (json!({ "error": format!("Unknown tool: {}", name) }), true),
    }
}

fn main() -> Result<()> {
    let store = Arc::new(open_metadata_store());
    let volume_manager = Arc::new(MacOSVolumeManager::new());
    let volume_barrier = Arc::new(VolumeBarrier::new(volume_manager.clone()));
    let project_service = Arc::new(ProjectService::new(
        store.clone(),
        volume_manager.clone(),
        volume_barrier,
    ));

    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let reader = stdin.lock();

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };

        if line.trim().is_empty() {
            continue;
        }

        let request: JsonRpcRequest = match serde_json::from_str(&line) {
            Ok(req) => req,
            Err(e) => {
                let err_res = JsonRpcResponse {
                    jsonrpc: "2.0",
                    id: Value::Null,
                    result: None,
                    error: Some(json!({
                        "code": -32700,
                        "message": format!("Parse error: {}", e)
                    })),
                };
                let _ = writeln!(stdout, "{}", serde_json::to_string(&err_res)?);
                let _ = stdout.flush();
                continue;
            }
        };

        let req_id = request.id.unwrap_or(Value::Null);

        match request.method.as_str() {
            "initialize" => {
                let res = JsonRpcResponse {
                    jsonrpc: "2.0",
                    id: req_id,
                    result: Some(json!({
                        "protocolVersion": "2024-11-05",
                        "serverInfo": {
                            "name": "localforge-mcp",
                            "version": "0.1.0"
                        },
                        "capabilities": {
                            "tools": {}
                        }
                    })),
                    error: None,
                };
                writeln!(stdout, "{}", serde_json::to_string(&res)?)?;
                stdout.flush()?;
            }
            "notifications/initialized" => {
                // MCP notification after initialize, no reply required
            }
            "ping" => {
                let res = JsonRpcResponse {
                    jsonrpc: "2.0",
                    id: req_id,
                    result: Some(json!({})),
                    error: None,
                };
                writeln!(stdout, "{}", serde_json::to_string(&res)?)?;
                stdout.flush()?;
            }
            "tools/list" => {
                let res = JsonRpcResponse {
                    jsonrpc: "2.0",
                    id: req_id,
                    result: Some(json!({
                        "tools": get_tools_manifest()
                    })),
                    error: None,
                };
                writeln!(stdout, "{}", serde_json::to_string(&res)?)?;
                stdout.flush()?;
            }
            "tools/call" => {
                let params = request.params.unwrap_or(Value::Null);
                let tool_name = params
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();
                let tool_args = params.get("arguments").cloned().unwrap_or(json!({}));

                let (output, is_error) = handle_tool_call(
                    tool_name,
                    &tool_args,
                    &store,
                    volume_manager.as_ref(),
                    &project_service,
                );

                let res = JsonRpcResponse {
                    jsonrpc: "2.0",
                    id: req_id,
                    result: Some(json!({
                        "content": [
                            {
                                "type": "text",
                                "text": serde_json::to_string_pretty(&output).unwrap_or_default()
                            }
                        ],
                        "isError": is_error
                    })),
                    error: None,
                };
                writeln!(stdout, "{}", serde_json::to_string(&res)?)?;
                stdout.flush()?;
            }
            _ => {
                let res = JsonRpcResponse {
                    jsonrpc: "2.0",
                    id: req_id,
                    result: None,
                    error: Some(json!({
                        "code": -32601,
                        "message": format!("Method not found: {}", request.method)
                    })),
                };
                writeln!(stdout, "{}", serde_json::to_string(&res)?)?;
                stdout.flush()?;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn setup_test_context() -> (MetadataStore, Arc<MacOSVolumeManager>, ProjectService) {
        let store = MetadataStore::open_in_memory().unwrap();
        let volume_manager = Arc::new(MacOSVolumeManager::new());
        let volume_barrier = Arc::new(VolumeBarrier::new(volume_manager.clone()));
        let project_service = ProjectService::new(
            Arc::new(MetadataStore::open_in_memory().unwrap()),
            volume_manager.clone(),
            volume_barrier,
        );
        (store, volume_manager, project_service)
    }

    #[test]
    fn test_mcp_preview_does_not_expose_approval_token() {
        let dir = tempdir().unwrap();
        let proj = dir.path();
        std::fs::write(proj.join("index.php"), "<?php echo 'hello';").unwrap();
        std::fs::write(proj.join(".env"), "SECRET=val").unwrap();

        let (store, vm, ps) = setup_test_context();
        let args = json!({
            "project_path": proj.to_string_lossy(),
            "target": "production"
        });

        let (output, is_err) =
            handle_tool_call("localforge_deploy_preview", &args, &store, vm.as_ref(), &ps);
        assert!(!is_err);

        // Crucial security requirement: approval_token MUST NOT be exposed in output
        assert!(
            output.get("approval_token").is_none(),
            "MCP preview must never return approval_token"
        );
        assert_eq!(output["requires_human_approval"], true);
        assert_eq!(
            output["approval_status"],
            "PENDING_OUT_OF_BAND_HUMAN_APPROVAL"
        );
    }

    #[test]
    fn test_mcp_execute_non_dry_run_is_rejected() {
        let dir = tempdir().unwrap();
        let proj = dir.path();
        std::fs::write(proj.join("index.php"), "<?php echo 'hello';").unwrap();

        let (store, vm, ps) = setup_test_context();

        // 1. Generate preview first
        let preview_args = json!({ "project_path": proj.to_string_lossy() });
        let (preview_out, is_err) = handle_tool_call(
            "localforge_deploy_preview",
            &preview_args,
            &store,
            vm.as_ref(),
            &ps,
        );
        assert!(!is_err);
        let preview_id = preview_out["preview_id"].as_str().unwrap();

        // 2. Attempt execute with dry_run = false (even if attacker supplies preview_id as token)
        let exec_args = json!({
            "project_path": proj.to_string_lossy(),
            "approval_token": preview_id,
            "dry_run": false
        });

        let (exec_out, is_err) = handle_tool_call(
            "localforge_deploy_execute",
            &exec_args,
            &store,
            vm.as_ref(),
            &ps,
        );
        assert!(
            is_err,
            "Non-dry-run deployment execution via MCP must be rejected"
        );
        assert_eq!(
            exec_out["error_code"],
            "UNAUTHORIZED_MCP_DEPLOYMENT_EXECUTION"
        );

        // 3. Confirm no ledger created on disk
        let ledger_file = proj.join(".localforge/deployments/production.json");
        assert!(
            !ledger_file.exists(),
            "Ledger must not be written on disk after rejected execution"
        );
    }

    #[test]
    fn test_mcp_execute_dry_run_simulation_reports_zero_transfers() {
        let dir = tempdir().unwrap();
        let proj = dir.path();
        std::fs::write(proj.join("index.php"), "<?php echo 'hello';").unwrap();

        let (store, vm, ps) = setup_test_context();

        // 1. Generate preview
        let preview_args = json!({ "project_path": proj.to_string_lossy() });
        let (_, is_err) = handle_tool_call(
            "localforge_deploy_preview",
            &preview_args,
            &store,
            vm.as_ref(),
            &ps,
        );
        assert!(!is_err);

        // 2. Run simulation via dry_run = true
        let exec_args = json!({
            "project_path": proj.to_string_lossy(),
            "dry_run": true
        });

        let (exec_out, is_err) = handle_tool_call(
            "localforge_deploy_execute",
            &exec_args,
            &store,
            vm.as_ref(),
            &ps,
        );
        assert!(!is_err);
        assert_eq!(exec_out["status"], "simulation_only");
        assert_eq!(exec_out["transferred_count"], 0);
        assert_eq!(exec_out["deleted_count"], 0);
        assert_eq!(exec_out["simulated_transfers_count"], 1);
        assert_eq!(exec_out["is_dry_run"], true);
        assert_eq!(exec_out["is_simulation"], true);

        // 3. Confirm no ledger created on disk
        let ledger_file = proj.join(".localforge/deployments/production.json");
        assert!(
            !ledger_file.exists(),
            "Ledger must not be written on disk during simulation"
        );
    }
}
