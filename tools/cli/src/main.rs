use anyhow::{anyhow, Context, Result};
use clap::{Args, Parser, Subcommand};
use localforge_core::ProjectStatus;
use localforge_deployment::{
    calculate_deployment_diff, DeploymentEngine, DeploymentLedger, DeploymentPreview,
};
use localforge_platform::{MacOSProcessManager, MacOSVolumeManager, VolumeManager};
use localforge_project_manager::ProjectService;
use localforge_runtime::{health::is_port_available, PhpFpmSupervisor, ProjectRuntimeService};
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
    /// Import an existing project directory
    Import {
        /// Path to the project root directory
        path: PathBuf,
    },
    /// Inspect mounted volumes (internal & external SSDs)
    Volumes,
    /// Smart, auditable deployment management
    Deploy(DeployArgs),
    /// Manage database storage and controlled migrations
    Storage(StorageArgs),
}

#[derive(Args)]
struct StorageArgs {
    #[command(subcommand)]
    action: StorageCommands,
}

#[derive(Subcommand)]
enum StorageCommands {
    /// Safe, auditable V2 loopback port schema migration and port allocation
    MigrateV2 {
        /// Target database path (defaults to canonical Application Support path)
        #[arg(long)]
        db_path: Option<PathBuf>,

        /// Path to verified SQLite backup file (mandatory for live execution)
        #[arg(long)]
        backup_path: Option<PathBuf>,

        /// Candidate port for Project A (r1_acceptance_project)
        #[arg(long, default_value_t = 18081)]
        port_project_a: u16,

        /// Candidate port for Project B (Projetest)
        #[arg(long, default_value_t = 18082)]
        port_project_b: u16,

        /// Explicit execution confirmation (required to disable default read-only dry-run)
        #[arg(long)]
        execute: bool,

        /// Mandatory Product Owner human approval token required for live execution
        #[arg(long)]
        approval_token: Option<String>,
    },
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
    let base = localforge_runtime::resolve_application_support_dir()
        .unwrap_or_else(|_| PathBuf::from("Library/Application Support/LocalForge"));
    let _ = std::fs::create_dir_all(&base);
    let db_path = base.join("localforge.db");
    MetadataStore::open(&db_path).unwrap_or_else(|_| MetadataStore::open_in_memory().unwrap())
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();

    if let Commands::Storage(storage_args) = cli.command {
        return handle_storage_command(storage_args).await;
    }

    let base = localforge_runtime::resolve_application_support_dir()
        .unwrap_or_else(|_| PathBuf::from("Library/Application Support/LocalForge"));
    let runtime_base = base.join("runtime");

    let store = Arc::new(open_metadata_store());
    let volume_manager = Arc::new(MacOSVolumeManager::new());
    let volume_barrier = Arc::new(VolumeBarrier::new(volume_manager.clone()));
    let process_manager = Arc::new(MacOSProcessManager::new());

    let project_service = Arc::new(ProjectService::new(
        store.clone(),
        volume_manager.clone(),
        volume_barrier.clone(),
    ));

    let runtime_service = Arc::new(ProjectRuntimeService::create_default(
        runtime_base,
        store.clone(),
        volume_manager.clone(),
        volume_barrier.clone(),
        process_manager.clone(),
        8080,
    ));

    match cli.command {
        Commands::List => {
            let statuses = runtime_service.sync_all_projects_runtime_status()?;
            if statuses.is_empty() {
                println!("No projects registered yet.");
                println!("Use the desktop application or import a project to get started.");
                return Ok(());
            }

            println!(
                "{:<20} {:<32} {:<18} {:<14} GATEWAY & VIRTUAL HOST",
                "PROJECT ID", "NAME", "STATUS", "PHP VERSION"
            );
            println!("{:-<110}", "");
            for p in statuses {
                let status_str = match p.status {
                    ProjectStatus::Running => "\x1b[32m● Running\x1b[0m",
                    ProjectStatus::Stopped => "\x1b[90m○ Stopped\x1b[0m",
                    ProjectStatus::Starting => "\x1b[36m◌ Starting\x1b[0m",
                    ProjectStatus::Stopping => "\x1b[33m◌ Stopping\x1b[0m",
                    ProjectStatus::Disconnected => "\x1b[31m▲ Disconnected\x1b[0m",
                    ProjectStatus::Degraded => "\x1b[33m⚠ Degraded\x1b[0m",
                    ProjectStatus::Failed => "\x1b[31m✖ Failed\x1b[0m",
                    ProjectStatus::Unknown => "\x1b[90m? Unknown\x1b[0m",
                };
                let host_info = format!("{} (Host: {})", p.local_url, p.local_domain);
                println!(
                    "{:<20} {:<32} {:<27} {:<14} {}",
                    p.project_id.0, p.name, status_str, p.php_version, host_info
                );
            }
        }
        Commands::Status { project } => {
            println!("=== LocalForge System Diagnostics ===");
            let db_status = runtime_service.get_database_status().unwrap_or_else(|_| {
                localforge_runtime::DatabaseHealthStatus {
                    is_alive: false,
                    socket_path: PathBuf::from("unavailable"),
                    port: 3306,
                    version: None,
                    latency_ms: None,
                    message: "Database supervisor not started".into(),
                }
            });
            if db_status.is_alive {
                println!(
                    "MariaDB Service:                   \x1b[32m● Alive\x1b[0m ({} - {} ms, socket: {})",
                    db_status.version.as_deref().unwrap_or("unknown"),
                    db_status.latency_ms.unwrap_or(0),
                    db_status.socket_path.display()
                );
            } else {
                let port_busy = !is_port_available(db_status.port);
                if port_busy {
                    println!(
                        "MariaDB Service:                   \x1b[33m○ External Port {} Bound (LocalForge socket idle)\x1b[0m",
                        db_status.port
                    );
                } else {
                    println!("MariaDB Service:                   \x1b[90m○ Inactive / Idle\x1b[0m");
                }
            }
            let nginx_busy = !is_port_available(8080);
            println!(
                "Nginx Gateway (127.0.0.1:8080):    {}",
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
                        let st = runtime_service.get_runtime_status(&p.id)?;
                        println!("ID:            {}", p.id.0);
                        println!("Name:          {}", p.name);
                        println!("Path:          {}", p.path.display());
                        println!("Volume UUID:   {}", p.volume_uuid.0);
                        println!("Document Root: {}", p.document_root);
                        println!("PHP Version:   {}", p.php_version);
                        println!("Status:        {:?}", st.status);
                        println!("PHP-FPM PID:   {:?}", st.php_fpm_pid);
                        println!("Socket Path:   {:?}", st.socket_path);
                        println!("Nginx PID:     {:?}", st.nginx_pid);
                        println!("Gateway Port:  {}", st.gateway_port);
                        println!("Loopback URL:  {}", st.local_url);
                        println!(
                            "Virtual Host:  {} (system .test resolver pending)",
                            st.local_domain
                        );
                        println!(
                            "Access Test:   curl -H 'Host: {}' {}/",
                            st.local_domain, st.local_url
                        );
                        println!("Diagnostics:   {}", st.message);
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
                .ok_or_else(|| anyhow!("Project '{}' not found in registry", project))?;

            println!(
                "Starting development runtime for project '{}'...",
                target.name
            );
            let st = runtime_service
                .start_project(&target.id)
                .context("Failed to start project runtime")?;

            match st.status {
                ProjectStatus::Running => {
                    println!(
                        "\x1b[32m✓ Successfully started runtime for project '{}' (RUNNING)\x1b[0m",
                        target.name
                    );
                }
                ProjectStatus::Degraded => {
                    println!(
                        "\x1b[33m⚠ Runtime started with warnings for project '{}' (DEGRADED)\x1b[0m",
                        target.name
                    );
                }
                other => {
                    println!(
                        "\x1b[31m✖ Runtime failed verification for project '{}' ({:?})\x1b[0m",
                        target.name, other
                    );
                }
            }

            if let Some(pid) = st.php_fpm_pid {
                println!("  PHP-FPM PID:      {}", pid);
            }
            if let Some(sock) = st.socket_path {
                println!("  FastCGI Socket:   {}", sock.display());
            }
            if let Some(pid) = st.nginx_pid {
                println!("  Nginx Master PID: {}", pid);
            }
            println!("  Loopback Gateway: {}", st.local_url);
            println!(
                "  Virtual Host:     {} (system .test resolver pending)",
                st.local_domain
            );
            println!(
                "  Access Command:   curl -H 'Host: {}' {}/",
                st.local_domain, st.local_url
            );
            println!("  Diagnostics:      {}", st.message);
        }
        Commands::Stop { project } => {
            let projects = store.list_projects()?;
            let target = projects
                .into_iter()
                .find(|p| p.id.0 == project || p.name.eq_ignore_ascii_case(&project))
                .ok_or_else(|| anyhow!("Project '{}' not found in registry", project))?;

            println!(
                "Stopping development runtime for project '{}'...",
                target.name
            );
            let st = runtime_service
                .stop_project(&target.id)
                .context("Failed to stop project runtime")?;

            println!(
                "\x1b[33m○ Successfully stopped runtime for project '{}'\x1b[0m",
                target.name
            );
            println!("  Status:           {:?}", st.status);
            println!("  Diagnostics:      {}", st.message);
        }

        Commands::Import { path } => {
            let canonical = dunce::canonicalize(&path)
                .with_context(|| format!("Failed to find project path '{}'", path.display()))?;
            let imported = project_service
                .import_project(&canonical)
                .context("Failed to import project")?;

            println!(
                "\x1b[32m✓ Successfully imported project '{}' (ID: {})\x1b[0m",
                imported.name, imported.id.0
            );
            println!("  Path:        {}", imported.path.display());
            println!("  PHP Version: {}", imported.php_version);
            println!("  Domain:      {}", imported.local_domain);
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
                let canonical_path = dunce::canonicalize(&path).with_context(|| {
                    format!("Project directory '{}' does not exist", path.display())
                })?;
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
                println!(
                    "Transfer Size:      {} bytes",
                    preview.diff.total_upload_bytes
                );

                if !preview.diff.changes.is_empty() {
                    println!("\nChanges:");
                    for c in &preview.diff.changes {
                        println!("  {:?} {}", c.change_type, c.relative_path);
                    }
                }

                println!("\n\x1b[1;33m[!] MANDATORY HUMAN APPROVAL TOKEN REQUIRED\x1b[0m");
                println!(
                    "Approval Token:     \x1b[1;32m{}\x1b[0m",
                    preview.preview_id
                );
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
                let canonical_path = dunce::canonicalize(&path).with_context(|| {
                    format!("Project directory '{}' does not exist", path.display())
                })?;
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
                    &safe_root,
                    &preview,
                    &token,
                    &mut ledger,
                    dry_run,
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
        Commands::Storage(_) => {
            unreachable!("Commands::Storage is handled early before runtime initialization")
        }
    }

    Ok(())
}

async fn handle_storage_command(args: StorageArgs) -> Result<()> {
    match args.action {
        StorageCommands::MigrateV2 {
            db_path,
            backup_path,
            port_project_a,
            port_project_b,
            execute,
            approval_token,
        } => {
            let canonical_app_support = localforge_runtime::resolve_application_support_dir()?;
            let canonical_db_path =
                db_path.unwrap_or_else(|| canonical_app_support.join("localforge.db"));

            if !canonical_db_path.exists() {
                return Err(anyhow!(
                    "Target database '{}' does not exist",
                    canonical_db_path.display()
                ));
            }

            // 1. Ownership & Symlink Verifications
            use std::os::unix::fs::MetadataExt;
            let db_meta = std::fs::symlink_metadata(&canonical_db_path)?;
            if db_meta.file_type().is_symlink() {
                return Err(anyhow!(
                    "Security violation: database path cannot be a symbolic link"
                ));
            }
            let current_uid = unsafe { libc::geteuid() };
            if db_meta.uid() != current_uid {
                return Err(anyhow!(
                    "Security violation: database UID {} does not match current user UID {}",
                    db_meta.uid(),
                    current_uid
                ));
            }
            if let Some(parent) = canonical_db_path.parent() {
                let parent_meta = std::fs::symlink_metadata(parent)?;
                if parent_meta.file_type().is_symlink() {
                    return Err(anyhow!(
                        "Security violation: parent directory cannot be a symbolic link"
                    ));
                }
                if parent_meta.uid() != current_uid {
                    return Err(anyhow!(
                        "Security violation: parent directory UID {} does not match current user UID {}",
                        parent_meta.uid(),
                        current_uid
                    ));
                }
            }

            // 2. SQLite Quiescence Verification
            let current_pid = std::process::id();
            if let Ok(output) = std::process::Command::new("pgrep")
                .arg("-il")
                .arg("localforge")
                .output()
            {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines() {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if let Some(pid_str) = parts.first() {
                        if let Ok(pid) = pid_str.parse::<u32>() {
                            if pid != current_pid {
                                return Err(anyhow!(
                                    "SQLite quiescence check failed: active LocalForge process detected (PID: {})",
                                    pid
                                ));
                            }
                        }
                    }
                }
            }

            // 3. Port Bounds & Uniqueness
            if !(18081..=18999).contains(&port_project_a) {
                return Err(anyhow!(
                    "Port A ({}) is outside permitted range 18081..=18999",
                    port_project_a
                ));
            }
            if !(18081..=18999).contains(&port_project_b) {
                return Err(anyhow!(
                    "Port B ({}) is outside permitted range 18081..=18999",
                    port_project_b
                ));
            }
            if port_project_a == port_project_b {
                return Err(anyhow!(
                    "Port A and Port B cannot be identical ({})",
                    port_project_a
                ));
            }
            let a_free = is_port_available(port_project_a);
            let b_free = is_port_available(port_project_b);

            // 4. Read-Only Dry-Run Mode (Default)
            if !execute {
                println!(
                    "\x1b[1;36m=== LOCALFORGE GATE U5.F: V2 MIGRATION DRY-RUN AUDIT ===\x1b[0m"
                );
                println!("Target Database:      {}", canonical_db_path.display());
                println!("Database Size:        {} bytes", db_meta.len());
                println!("Database Owner UID:   {}", db_meta.uid());
                println!("Current User UID:     {}", current_uid);

                // Open STRICTLY in read-only mode with immutable=1 to prevent any writes to db, wal, or shm
                let uri_path = format!("file:{}?immutable=1", canonical_db_path.display());
                let conn = rusqlite::Connection::open_with_flags(
                    &uri_path,
                    rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY
                        | rusqlite::OpenFlags::SQLITE_OPEN_URI,
                )?;

                let integrity: String =
                    conn.query_row("PRAGMA integrity_check;", [], |r| r.get(0))?;
                if integrity != "ok" {
                    return Err(anyhow!("Database integrity check failed: {}", integrity));
                }
                println!("Integrity Check:      \x1b[1;32m{}\x1b[0m", integrity);

                let has_port_col: i64 = conn.prepare("SELECT count(*) FROM pragma_table_info('projects') WHERE name = 'loopback_port';")?
                    .query_row([], |r| r.get(0))?;
                println!(
                    "Schema Status:        {}",
                    if has_port_col > 0 {
                        "Schema V2 (loopback_port present)"
                    } else {
                        "Schema V1 (loopback_port absent)"
                    }
                );

                let mut stmt = conn.prepare(
                    "SELECT id, name, php_version, local_domain FROM projects ORDER BY name ASC;",
                )?;
                let projects = stmt.query_map([], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                    ))
                })?;

                println!("\nProject Inspection:");
                for p in projects {
                    let (id, name, php_version, domain) = p?;
                    let proposed_port = if name.contains("r1_acceptance") {
                        port_project_a
                    } else {
                        port_project_b
                    };
                    let port_free = if name.contains("r1_acceptance") {
                        a_free
                    } else {
                        b_free
                    };
                    println!("  - [{}] {}", id, name);
                    println!("    PHP Version:     {}", php_version);
                    println!("    Local Domain:    {}", domain);
                    println!(
                        "    Proposed Port:   {} (Available on host: {})",
                        proposed_port,
                        if port_free {
                            "\x1b[32mYES\x1b[0m"
                        } else {
                            "\x1b[31mNO\x1b[0m"
                        }
                    );
                    if name == "Projetest" {
                        if php_version != "8.4" {
                            return Err(anyhow!(
                                "PO-U5-01 VIOLATION: Projetest PHP version is '{}', expected '8.4'",
                                php_version
                            ));
                        }
                        println!(
                            "    PO Compliance:   \x1b[1;32mPO-U5-01 PRESERVED (PHP 8.4)\x1b[0m"
                        );
                    }
                }

                println!(
                    "\n\x1b[1;33m[!] DRY-RUN COMPLETE: Zero modifications made to database.\x1b[0m"
                );
                println!("To execute live migration, explicit authorization is required:\n  localforge storage migrate-v2 --execute --approval-token <TOKEN> --backup-path <PATH>");
                return Ok(());
            }

            // 5. Live Execution Mode
            let token = approval_token.as_deref().unwrap_or("").trim();
            if token.is_empty() {
                return Err(anyhow!(
                    "Execution rejected: --approval-token is mandatory when --execute is specified"
                ));
            }

            let bpath = backup_path.ok_or_else(|| {
                anyhow!("Execution rejected: --backup-path is mandatory for live execution")
            })?;
            if !bpath.exists() {
                return Err(anyhow!("Backup file '{}' does not exist", bpath.display()));
            }
            let bmeta = std::fs::symlink_metadata(&bpath)?;
            if bmeta.file_type().is_symlink() {
                return Err(anyhow!(
                    "Security violation: backup file cannot be a symbolic link"
                ));
            }
            if bmeta.uid() != current_uid {
                return Err(anyhow!(
                    "Security violation: backup file UID does not match current user"
                ));
            }

            // Verify backup file integrity and Schema V1 in read-only mode
            let b_uri = format!("file:{}?immutable=1", bpath.display());
            let bconn = rusqlite::Connection::open_with_flags(
                &b_uri,
                rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_URI,
            )?;
            let b_integ: String = bconn.query_row("PRAGMA integrity_check;", [], |r| r.get(0))?;
            if b_integ != "ok" {
                return Err(anyhow!("Backup integrity check failed: {}", b_integ));
            }
            let b_count: i64 =
                bconn.query_row("SELECT count(*) FROM projects;", [], |r| r.get(0))?;
            if b_count != 2 {
                return Err(anyhow!(
                    "Backup project count mismatch: expected 2, found {}",
                    b_count
                ));
            }
            let b_has_port: i64 = bconn.query_row(
                "SELECT count(*) FROM pragma_table_info('projects') WHERE name = 'loopback_port';",
                [],
                |r| r.get(0),
            )?;
            if b_has_port != 0 {
                return Err(anyhow!(
                    "Backup is not clean Schema V1 (already has loopback_port column)"
                ));
            }

            if !a_free {
                return Err(anyhow!("Port A ({}) is currently occupied on the host. Aborting without modifying database.", port_project_a));
            }
            if !b_free {
                return Err(anyhow!("Port B ({}) is currently occupied on the host. Aborting without modifying database.", port_project_b));
            }

            println!("\x1b[1;32m[✓] All preconditions verified: Approval Token present, Quiescence ok, Backup verified V1, Ports free.\x1b[0m");

            let rollback = |reason: &str| -> Result<()> {
                eprintln!("\x1b[1;31m[!] Live migration failed: {}. Initiating auditable rollback...\x1b[0m", reason);
                let allowlist = ["localforge.db", "localforge.db-wal", "localforge.db-shm"];
                let quarantine_dir = canonical_app_support.join("quarantine").join(format!(
                    "quarantine_{}",
                    chrono::Utc::now().format("%Y%m%d_%H%M%S")
                ));
                std::fs::create_dir_all(&quarantine_dir)?;
                let qmeta = std::fs::metadata(&quarantine_dir)?;
                if qmeta.uid() != current_uid {
                    return Err(anyhow!("Quarantine directory ownership failure"));
                }

                for name in &allowlist {
                    let fpath = canonical_app_support.join(name);
                    if fpath.exists() {
                        let fmeta = std::fs::symlink_metadata(&fpath)?;
                        if !fmeta.file_type().is_symlink() {
                            std::fs::rename(&fpath, quarantine_dir.join(name))?;
                        }
                    }
                }

                std::fs::copy(&bpath, &canonical_db_path)?;
                eprintln!(
                    "\x1b[1;32m[✓] Rollback completed: state restored from backup '{}'.\x1b[0m",
                    bpath.display()
                );
                Ok(())
            };

            // Open live database via canonical MetadataStore
            let store = MetadataStore::open(&canonical_db_path)?;

            let projects = store.list_projects()?;
            let p1 = projects
                .iter()
                .find(|p| p.name == "r1_acceptance_project")
                .ok_or_else(|| anyhow!("Project 'r1_acceptance_project' not found in database"))?;
            let p2 = projects
                .iter()
                .find(|p| p.name == "Projetest")
                .ok_or_else(|| anyhow!("Project 'Projetest' not found in database"))?;

            if p2.php_version != "8.4" {
                return Err(anyhow!(
                    "PO-U5-01 VIOLATION: Projetest PHP version is '{}', must be '8.4'",
                    p2.php_version
                ));
            }

            println!("Executing canonical MetadataStore::migrate_to_v2_loopback_ports()...");
            let report = match store.migrate_to_v2_loopback_ports() {
                Ok(r) => r,
                Err(e) => {
                    let _ = rollback(&format!("Migration failed: {e}"));
                    return Err(anyhow!("Migration failed: {e}"));
                }
            };
            println!(
                "Migration report: Target version {}, applied: {}",
                report.target_version, report.applied
            );

            println!(
                "Assigning loopback ports via MetadataStore::update_project_loopback_port()..."
            );
            if let Err(e) = store.update_project_loopback_port(&p1.id, Some(port_project_a)) {
                let _ = rollback(&format!("Port A assignment failed: {e}"));
                return Err(anyhow!("Port A assignment failed: {e}"));
            }
            if let Err(e) = store.update_project_loopback_port(&p2.id, Some(port_project_b)) {
                let _ = rollback(&format!("Port B assignment failed: {e}"));
                return Err(anyhow!("Port B assignment failed: {e}"));
            }

            // Post-migration verification
            let p1_after = store
                .get_project(&p1.id)?
                .ok_or_else(|| anyhow!("Project A missing after migration"))?;
            let p2_after = store
                .get_project(&p2.id)?
                .ok_or_else(|| anyhow!("Project B missing after migration"))?;

            if p1_after.loopback_port != Some(port_project_a) {
                let _ = rollback("Project A loopback_port verification mismatch");
                return Err(anyhow!("Project A port verification failed"));
            }
            if p2_after.loopback_port != Some(port_project_b) {
                let _ = rollback("Project B loopback_port verification mismatch");
                return Err(anyhow!("Project B port verification failed"));
            }
            if p2_after.php_version != "8.4" {
                let _ = rollback("PO-U5-01 VIOLATION: Projetest PHP version mutated");
                return Err(anyhow!(
                    "PO-U5-01 VIOLATION: Projetest PHP version mutated to {}",
                    p2_after.php_version
                ));
            }

            let check_conn = rusqlite::Connection::open(&canonical_db_path)?;
            if localforge_storage::validate_loopback_port_index(&check_conn)?
                != localforge_storage::IndexValidationStatus::Valid
            {
                let _ = rollback("Post-migration unique index validation failed");
                return Err(anyhow!("Unique index validation failed"));
            }

            println!("\x1b[1;32m=== LIVE MIGRATION AND PORT ALLOCATION COMPLETED SUCCESSFULLY ===\x1b[0m");
            println!("  - Project A ({}): Port {}", p1_after.name, port_project_a);
            println!(
                "  - Project B ({}): Port {} (PHP 8.4 strictly preserved)",
                p2_after.name, port_project_b
            );
            println!("  - Unique Index:    VALIDATED (idx_projects_loopback_port)");
            println!("  - Schema Version:  2");

            Ok(())
        }
    }
}
