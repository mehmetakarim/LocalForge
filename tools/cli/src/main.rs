use anyhow::{anyhow, Context, Result};
use clap::{Args, Parser, Subcommand};
use localforge_core::ProjectStatus;
use localforge_deployment::{
    calculate_deployment_diff, DeploymentEngine, DeploymentLedger, DeploymentPreview,
};
use localforge_platform::{MacOSVolumeManager, VolumeManager};
use localforge_project_manager::ProjectService;
use localforge_runtime::{health::is_port_available, PhpFpmSupervisor};
use localforge_storage::{sanitize_project_path, MetadataStore, VolumeBarrier};
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Parser)]
#[command(name = "localforge")]
#[command(about = "LocalForge — AI-ready local development & deployment environment", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List all registered projects
    List,
    /// Inspect runtime and project health status
    Status {
        /// Project identifier or name (optional)
        project: Option<String>,
    },
    /// Start development runtime for a project
    Start {
        /// Project identifier or name
        project: String,
    },
    /// Stop development runtime for a project
    Stop {
        /// Project identifier or name
        project: String,
    },
    /// Inspect mounted volumes (internal & external SSDs)
    Volumes,
    /// Smart, auditable deployment management
    Deploy(DeployArgs),
}

#[derive(Args)]
struct DeployArgs {
    #[command(subcommand)]
    action: DeployCommands,
}

#[derive(Subcommand)]
enum DeployCommands {
    /// Generate a cryptographic diff preview and approval token
    Preview {
        /// Path to the project root directory
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Deployment target identifier
        #[arg(short, long, default_value = "production")]
        target: String,
    },
    /// Apply changes to the remote target (strictly requires human approval token)
    Apply {
        /// Path to the project root directory
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Deployment target identifier
        #[arg(short, long, default_value = "production")]
        target: String,
        /// Mandatory human approval token from `deploy preview`
        #[arg(short, long)]
        token: String,
        /// Perform dry-run without writing to remote or ledger
        #[arg(long)]
        dry_run: bool,
    },
}

fn open_metadata_store() -> MetadataStore {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    let db_path = PathBuf::from(home).join("Library/Application Support/LocalForge/localforge.db");
    MetadataStore::open(&db_path).unwrap_or_else(|_| MetadataStore::open_in_memory().unwrap())
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();

    let store = Arc::new(open_metadata_store());
    let volume_manager = Arc::new(MacOSVolumeManager::new());
    let volume_barrier = Arc::new(VolumeBarrier::new(volume_manager.clone()));
    let project_service = Arc::new(ProjectService::new(
        store.clone(),
        volume_manager.clone(),
        volume_barrier.clone(),
    ));

    match cli.command {
        Commands::List => {
            let projects = project_service.sync_all_projects_status()?;
            if projects.is_empty() {
                println!("No projects registered yet.");
                println!("Use the desktop application or import a project to get started.");
                return Ok(());
            }

            println!(
                "{:<20} {:<32} {:<14} {:<24} PATH",
                "PROJECT ID", "NAME", "STATUS", "PHP VERSION"
            );
            println!("{:-<100}", "");
            for p in projects {
                let status_str = match p.status {
                    ProjectStatus::Running => "\x1b[32m● Running\x1b[0m",
                    ProjectStatus::Stopped => "\x1b[90m○ Stopped\x1b[0m",
                    ProjectStatus::Disconnected => "\x1b[31m▲ Disconnected\x1b[0m",
                    ProjectStatus::Degraded => "\x1b[33m⚠ Degraded\x1b[0m",
                };
                println!(
                    "{:<20} {:<32} {:<23} {:<24} {}",
                    p.id.0,
                    p.name,
                    status_str,
                    p.php_version,
                    p.path.display()
                );
            }
        }
        Commands::Status { project } => {
            println!("=== LocalForge System Diagnostics ===");
            let mariadb_busy = !is_port_available(3306);
            let nginx_busy = !is_port_available(8080);
            println!(
                "MariaDB loopback (127.0.0.1:3306): {}",
                if mariadb_busy {
                    "\x1b[32mActive / Bound\x1b[0m"
                } else {
                    "\x1b[90mIdle\x1b[0m"
                }
            );
            println!(
                "Nginx Proxy (127.0.0.1:8080):      {}",
                if nginx_busy {
                    "\x1b[32mActive / Bound\x1b[0m"
                } else {
                    "\x1b[90mIdle\x1b[0m"
                }
            );

            let php_versions = PhpFpmSupervisor::discover_installed_versions();
            let php_list = if php_versions.is_empty() {
                "None discovered".into()
            } else {
                php_versions.join(", ")
            };
            println!("Installed PHP Runtimes:            {}", php_list);

            if let Some(proj_query) = project {
                println!("\n=== Project Details: {} ===", proj_query);
                let projects = store.list_projects()?;
                let target = projects
                    .into_iter()
                    .find(|p| p.id.0 == proj_query || p.name.eq_ignore_ascii_case(&proj_query));

                match target {
                    Some(p) => {
                        println!("ID:            {}", p.id.0);
                        println!("Name:          {}", p.name);
                        println!("Path:          {}", p.path.display());
                        println!("Volume UUID:   {}", p.volume_uuid.0);
                        println!("Document Root: {}", p.document_root);
                        println!("PHP Version:   {}", p.php_version);
                        println!("Local Domain:  {}", p.local_domain);
                        println!("Status:        {:?}", p.status);
                    }
                    None => {
                        eprintln!("Error: Project '{}' not found in registry.", proj_query);
                    }
                }
            }
        }
        Commands::Start { project } => {
            let projects = store.list_projects()?;
            let target = projects
                .into_iter()
                .find(|p| p.id.0 == project || p.name.eq_ignore_ascii_case(&project))
                .ok_or_else(|| anyhow!("Project '{}' not found", project))?;

            // Check volume barrier
            volume_barrier
                .verify_volume_available(&target.path, &target.volume_uuid)
                .context("External storage volume verification failed")?;

            store.update_project_status(&target.id, ProjectStatus::Running)?;
            println!("\x1b[32m✓ Started runtime for project '{}'\x1b[0m", target.name);
            println!("Local URL: http://{}:8080", target.local_domain);
        }
        Commands::Stop { project } => {
            let projects = store.list_projects()?;
            let target = projects
                .into_iter()
                .find(|p| p.id.0 == project || p.name.eq_ignore_ascii_case(&project))
                .ok_or_else(|| anyhow!("Project '{}' not found", project))?;

            store.update_project_status(&target.id, ProjectStatus::Stopped)?;
            println!("\x1b[33m○ Stopped runtime for project '{}'\x1b[0m", target.name);
        }
        Commands::Volumes => {
            println!("=== LocalForge Storage Volumes ===");
            let volumes = volume_manager.list_volumes()?;
            for v in volumes {
                let kind = if !v.is_internal {
                    "\x1b[34m[External SSD]\x1b[0m"
                } else {
                    "\x1b[36m[Internal SSD]\x1b[0m"
                };
                let status = if v.mount_point.exists() {
                    "\x1b[32mMounted\x1b[0m"
                } else {
                    "\x1b[31mUnmounted\x1b[0m"
                };
                println!(
                    "{} {:<20} (UUID: {}) at {} - {}",
                    kind,
                    v.name,
                    v.uuid.0,
                    v.mount_point.display(),
                    status
                );
            }
        }
        Commands::Deploy(deploy_args) => match deploy_args.action {
            DeployCommands::Preview { path, target } => {
                let canonical_path = dunce::canonicalize(&path)
                    .with_context(|| format!("Project directory '{}' does not exist", path.display()))?;
                let safe_root = sanitize_project_path(&canonical_path, &canonical_path)?;

                let ledger = DeploymentLedger::load(&safe_root, &target)?;
                let diff = calculate_deployment_diff(&safe_root, &ledger)?;
                let preview = DeploymentPreview::new(target.clone(), diff);

                // Save preview file for validation
                let preview_dir = safe_root.join(".localforge").join("deployments");
                std::fs::create_dir_all(&preview_dir)?;
                let preview_file = preview_dir.join(format!(".preview_{}.json", target));
                std::fs::write(&preview_file, serde_json::to_string_pretty(&preview)?)?;

                println!("\n=== LocalForge Deployment Preview ===");
                println!("Target:             {}", preview.target_id);
                println!("Files to Add:       {}", preview.diff.to_add_count);
                println!("Files to Modify:    {}", preview.diff.to_modify_count);
                println!("Files to Delete:    {}", preview.diff.to_delete_count);
                println!("Transfer Size:      {} bytes", preview.diff.total_upload_bytes);

                if !preview.diff.changes.is_empty() {
                    println!("\nChanges:");
                    for c in &preview.diff.changes {
                        println!("  {:?} {}", c.change_type, c.relative_path);
                    }
                }

                println!("\n\x1b[1;33m[!] MANDATORY HUMAN APPROVAL TOKEN REQUIRED\x1b[0m");
                println!("Approval Token:     \x1b[1;32m{}\x1b[0m", preview.preview_id);
                println!(
                    "\nTo execute this deployment, run:\n  localforge deploy apply --target {} --token {} {}",
                    preview.target_id,
                    preview.preview_id,
                    safe_root.display()
                );
            }
            DeployCommands::Apply {
                path,
                target,
                token,
                dry_run,
            } => {
                let canonical_path = dunce::canonicalize(&path)
                    .with_context(|| format!("Project directory '{}' does not exist", path.display()))?;
                let safe_root = sanitize_project_path(&canonical_path, &canonical_path)?;

                let preview_file = safe_root
                    .join(".localforge")
                    .join("deployments")
                    .join(format!(".preview_{}.json", target));

                if !preview_file.exists() {
                    return Err(anyhow!(
                        "No active preview found for target '{}'. Please run 'localforge deploy preview' first.",
                        target
                    ));
                }

                let preview_data = std::fs::read_to_string(&preview_file)?;
                let preview: DeploymentPreview = serde_json::from_str(&preview_data)?;

                let mut ledger = DeploymentLedger::load(&safe_root, &target)?;

                println!(
                    "Executing deployment for target '{}' (Dry-run: {})...",
                    target, dry_run
                );
                let result = DeploymentEngine::execute_deployment(
                    &safe_root, &preview, &token, &mut ledger, dry_run,
                )?;

                println!("\x1b[32m✓ Deployment execution completed successfully!\x1b[0m");
                println!("Transferred files:  {}", result.transferred_files.len());
                println!("Deleted files:      {}", result.deleted_files.len());
                println!("Total data size:    {} bytes", result.total_bytes);
                println!("Completed at:       {}", result.completed_at);

                if !dry_run {
                    let _ = std::fs::remove_file(preview_file);
                }
            }
        },
    }

    Ok(())
}
