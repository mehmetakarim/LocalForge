use localforge_core::{ProjectId, ProjectStatus, VolumeId};
use localforge_platform::{MacOSProcessManager, MacOSVolumeManager, ProcessManager, VolumeManager};
use localforge_runtime::{
    health::probe_http_health, MariaDbSupervisor, NginxSupervisor, PhpFpmSupervisor,
    ProjectRuntimeService,
};
use localforge_storage::{MetadataStore, ProjectRecord, VolumeBarrier};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tempfile::tempdir;

static SERIAL_TEST_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn test_r1_2_01_mariadb_initialization_and_verified_health() {
    let _lock = SERIAL_TEST_LOCK.lock().unwrap();
    let binaries_found = MariaDbSupervisor::discover_binaries().is_ok();
    if !binaries_found {
        eprintln!("Skipping MariaDB daemon test: binaries not found on host");
        return;
    }

    let dir = tempdir().unwrap();
    let pm = Arc::new(MacOSProcessManager::new());
    let db_port = localforge_runtime::find_available_port(13320, 50).unwrap_or(13320);
    let sup = MariaDbSupervisor::new(dir.path().to_path_buf(), db_port, pm.clone());

    // 1. Initialize data directory
    let init_res = sup.ensure_datadir_initialized();
    assert!(
        init_res.is_ok(),
        "Failed to initialize MariaDB data directory: {:?}",
        init_res
    );
    assert!(sup.data_dir().join("mysql").is_dir());
    assert!(sup.config_file().exists());

    // 2. Start MariaDB daemon
    let start_res = sup.start();
    assert!(
        start_res.is_ok(),
        "Failed to start MariaDB: {:?}",
        start_res
    );
    let identity = start_res.unwrap();

    // Verify Darwin process identity
    assert!(identity.pid > 0);
    assert_eq!(identity.uid, unsafe { libc::getuid() });
    let verified = pm.verify_process_identity(
        identity.pid,
        identity.executable_path(),
        identity.start_time_secs,
        identity.start_time_usecs,
    );
    assert!(
        verified,
        "Process identity must match live OS process table"
    );

    // 3. Verify Unix domain socket is active
    assert!(
        sup.socket_path().exists(),
        "Socket file must exist after start"
    );

    // 4. Perform authenticated health check
    let health = sup.check_health().expect("Health check must execute");
    assert!(health.is_alive, "MariaDB must report is_alive: true");
    assert!(health.version.is_some(), "Version must be reported");
    assert!(health.latency_ms.is_some(), "Latency ms must be measured");
    assert!(health.message.contains("verified alive and responsive"));

    // 5. Clean stop
    let stop_res = sup.stop();
    assert!(stop_res.is_ok(), "Stop must succeed: {:?}", stop_res);

    // Verify socket removed and process gone
    assert!(
        !sup.socket_path().exists(),
        "Socket must be cleaned up on stop"
    );
}

#[test]
fn test_r1_2_02_database_user_provisioning_and_safety() {
    let _lock = SERIAL_TEST_LOCK.lock().unwrap();
    let binaries_found = MariaDbSupervisor::discover_binaries().is_ok();
    if !binaries_found {
        eprintln!("Skipping MariaDB provisioning test: binaries not found on host");
        return;
    }

    let dir = tempdir().unwrap();
    let pm = Arc::new(MacOSProcessManager::new());
    let db_port = localforge_runtime::find_available_port(13370, 50).unwrap_or(13370);
    let sup = MariaDbSupervisor::new(dir.path().to_path_buf(), db_port, pm);

    sup.start().expect("Failed to start MariaDB");

    // 1. Create project database and isolated user
    let db_name = "test_proj_db";
    let user = "test_user";
    let pass = "secret_pass_123";

    let create_res = sup.create_project_database(db_name, user, pass);
    assert!(
        create_res.is_ok(),
        "Failed to create project database: {:?}",
        create_res
    );

    // 2. Execute SQL on created database
    let create_table_sql = format!(
        "USE `{db_name}`; CREATE TABLE users (id INT AUTO_INCREMENT PRIMARY KEY, name VARCHAR(50)); INSERT INTO users (name) VALUES ('Alice'), ('Bob');"
    );
    let exec_res = sup.execute_query(&create_table_sql);
    assert!(exec_res.is_ok(), "Failed to insert rows: {:?}", exec_res);

    let select_sql = format!("USE `{db_name}`; SELECT COUNT(*) FROM users;");
    let count_out = sup.execute_query(&select_sql).unwrap();
    assert!(count_out.contains('2'), "Count must be 2. Got: {count_out}");

    // 3. Drop safety: invalid confirmation token must fail closed
    let bad_drop = sup.drop_project_database(db_name, "wrong_token");
    assert!(
        bad_drop.is_err(),
        "Dropping database with invalid confirmation must fail"
    );

    // 4. Drop with valid confirmation must succeed
    let good_drop = sup.drop_project_database(db_name, db_name);
    assert!(
        good_drop.is_ok(),
        "Dropping database with valid confirmation must succeed"
    );

    sup.stop().expect("Failed to stop MariaDB");
}

#[test]
fn test_r1_2_03_database_backup_and_recovery() {
    let _lock = SERIAL_TEST_LOCK.lock().unwrap();
    let binaries_found = MariaDbSupervisor::discover_binaries().is_ok();
    if !binaries_found {
        eprintln!("Skipping MariaDB backup test: binaries not found on host");
        return;
    }

    let dir = tempdir().unwrap();
    let pm = Arc::new(MacOSProcessManager::new());
    let db_port = localforge_runtime::find_available_port(13420, 50).unwrap_or(13420);
    let sup = MariaDbSupervisor::new(dir.path().to_path_buf(), db_port, pm);

    sup.start().expect("Failed to start MariaDB");

    let source_db = "backup_src_db";
    sup.create_project_database(source_db, "backup_user", "pass123")
        .unwrap();

    let init_sql = format!(
        "USE `{source_db}`; CREATE TABLE logs (id INT AUTO_INCREMENT PRIMARY KEY, msg VARCHAR(100)); INSERT INTO logs (msg) VALUES ('first log'), ('second log');"
    );
    sup.execute_query(&init_sql).unwrap();

    // 1. Dump database to backup file
    let backup_file = dir
        .path()
        .join(".localforge")
        .join("backups")
        .join("backup_src.sql");
    let dump_res = sup.dump_database(source_db, &backup_file);
    assert!(dump_res.is_ok(), "Dump must succeed: {:?}", dump_res);
    assert!(backup_file.exists(), "Backup file must be created on disk");

    // Verify 0600 file permissions on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::metadata(&backup_file).unwrap().permissions();
        assert_eq!(
            perms.mode() & 0o777,
            0o600,
            "Backup archive must enforce 0600 file permissions"
        );
    }

    // 2. Restore into a new database
    let target_db = "backup_restored_db";
    let restore_res = sup.restore_database(target_db, &backup_file);
    assert!(
        restore_res.is_ok(),
        "Restore must succeed: {:?}",
        restore_res
    );

    // 3. Verify data records in restored database
    let verify_sql = format!("USE `{target_db}`; SELECT msg FROM logs ORDER BY id;");
    let rows = sup.execute_query(&verify_sql).unwrap();
    assert!(rows.contains("first log"), "Must contain first log");
    assert!(rows.contains("second log"), "Must contain second log");

    sup.stop().expect("Failed to stop MariaDB");
}

#[test]
fn test_r1_2_04_php_pdo_end_to_end_on_external_apfs() {
    let _lock = SERIAL_TEST_LOCK.lock().unwrap();
    let fpm_exists = PhpFpmSupervisor::find_php_fpm_binary("any").is_ok();
    let nginx_exists = NginxSupervisor::find_nginx_binary().is_ok();
    let mariadb_exists = MariaDbSupervisor::discover_binaries().is_ok();

    if !fpm_exists || !nginx_exists || !mariadb_exists {
        eprintln!("Skipping E2E PHP+MariaDB test: required runtimes not all installed on host");
        return;
    }

    // 1. External APFS directory under scratch on external volume
    let external_scratch =
        PathBuf::from("/Volumes/Mac Harici Disk/VibeProject/LocalForge/scratch/test_pdo_e2e");
    let _ = std::fs::remove_dir_all(&external_scratch);
    std::fs::create_dir_all(&external_scratch)
        .expect("Failed to create external scratch directory");

    let pm = Arc::new(MacOSProcessManager::new());
    let internal_db_dir = tempdir().unwrap();
    let db_port = localforge_runtime::find_available_port(13470, 50).unwrap_or(13470);
    let mariadb_sup =
        MariaDbSupervisor::new(internal_db_dir.path().to_path_buf(), db_port, pm.clone());
    mariadb_sup
        .start()
        .expect("Failed to start MariaDB for E2E test");

    let db_name = "pdo_site_db";
    let db_user = "pdo_user";
    let db_pass = "pdo_pass_2026";
    mariadb_sup
        .create_project_database(db_name, db_user, db_pass)
        .expect("Failed to create E2E project database");

    // 2. Author index.php connecting over MariaDB socket via PDO
    let sock_path = mariadb_sup.socket_path().to_string_lossy().to_string();
    let index_php = external_scratch.join("index.php");
    let php_code = format!(
        r#"<?php
header('Content-Type: application/json');
try {{
    $pdo = new PDO("mysql:unix_socket={sock_path};dbname={db_name};charset=utf8mb4", "{db_user}", "{db_pass}", [
        PDO::ATTR_ERRMODE => PDO::ERRMODE_EXCEPTION
    ]);

    $pdo->exec("CREATE TABLE IF NOT EXISTS inventory (id INT AUTO_INCREMENT PRIMARY KEY, sku VARCHAR(50), quantity INT)");
    
    // Check if table empty
    $count = $pdo->query("SELECT COUNT(*) FROM inventory")->fetchColumn();
    if ($count == 0) {{
        $stmt = $pdo->prepare("INSERT INTO inventory (sku, quantity) VALUES (?, ?)");
        $stmt->execute(['PROD-A100', 42]);
        $stmt->execute(['PROD-B200', 108]);
    }}

    $rows = $pdo->query("SELECT sku, quantity FROM inventory ORDER BY id")->fetchAll(PDO::FETCH_ASSOC);

    echo json_encode([
        'status' => 'success',
        'database' => '{db_name}',
        'rows' => $rows,
        'count' => count($rows)
    ]);
}} catch (Exception $e) {{
    http_response_code(500);
    echo json_encode(['status' => 'error', 'message' => $e->getMessage()]);
}}
"#
    );
    std::fs::write(&index_php, php_code).expect("Failed to write index.php");

    let store = Arc::new(MetadataStore::open_in_memory().unwrap());
    let volume_manager = Arc::new(MacOSVolumeManager::new());
    let volume_barrier = Arc::new(VolumeBarrier::new(volume_manager.clone()));

    let vol_info = volume_manager
        .find_volume_for_path(&external_scratch)
        .unwrap()
        .expect("Must resolve external volume");

    let project = ProjectRecord {
        id: ProjectId::from_string("proj_pdo_e2e"),
        name: "PDO E2E External".into(),
        path: external_scratch.clone(),
        volume_uuid: vol_info.uuid,
        document_root: "".into(),
        php_version: "8.5".into(),
        local_domain: "pdoe2e.test".into(),
        status: ProjectStatus::Stopped,
        created_at: "2026-10-18T00:00:00Z".into(),
        updated_at: "2026-10-18T00:00:00Z".into(),
    };
    store.insert_project(&project).unwrap();

    let test_port = localforge_runtime::find_available_port(18220, 50).unwrap_or(18220);
    let runtime_base = tempdir().unwrap();
    let nginx_sup = Arc::new(NginxSupervisor::new(
        runtime_base.path().join("nginx"),
        runtime_base.path().join("run"),
        test_port,
        pm.clone(),
    ));
    let service = ProjectRuntimeService::new(
        store.clone(),
        volume_manager.clone(),
        volume_barrier.clone(),
        pm.clone(),
        Arc::new(PhpFpmSupervisor::new(
            runtime_base.path().join("run"),
            pm.clone(),
        )),
        nginx_sup.clone(),
        Arc::new(mariadb_sup),
        test_port,
    );

    // 3. Start project and probe via HTTP loopback
    let start_status = service
        .start_project(&project.id)
        .expect("Failed to start project");
    assert_eq!(start_status.status, ProjectStatus::Running);

    let (code, body) =
        probe_http_health(test_port, "pdoe2e.test", "/").expect("Probe must succeed");
    assert_eq!(code, 200, "Response must be 200 OK. Got: {body}");
    assert!(
        body.contains("\"status\":\"success\""),
        "Must return status success"
    );
    assert!(
        body.contains("PROD-A100"),
        "Must return inserted product row"
    );
    assert!(
        body.contains("PROD-B200"),
        "Must return inserted product row"
    );

    // 4. Persistence Test across PHP-FPM pool restart
    service
        .stop_project(&project.id)
        .expect("Failed to stop project");
    std::thread::sleep(std::time::Duration::from_millis(100));
    service
        .start_project(&project.id)
        .expect("Failed to restart project");

    let (code2, body2) =
        probe_http_health(test_port, "pdoe2e.test", "/").expect("Probe must succeed");
    assert_eq!(code2, 200);
    assert!(
        body2.contains("\"count\":2"),
        "Data count must remain 2 after FPM restart"
    );

    // Clean up
    let _ = service.stop_project(&project.id);
    let _ = service.mariadb_supervisor().stop();
    let _ = nginx_sup.stop();
    let _ = std::fs::remove_dir_all(&external_scratch);
}

#[test]
fn test_r1_2_05_multi_project_concurrency_with_mariadb() {
    let _lock = SERIAL_TEST_LOCK.lock().unwrap();
    let fpm_exists = PhpFpmSupervisor::find_php_fpm_binary("any").is_ok();
    let nginx_exists = NginxSupervisor::find_nginx_binary().is_ok();
    let mariadb_exists = MariaDbSupervisor::discover_binaries().is_ok();

    if !fpm_exists || !nginx_exists || !mariadb_exists {
        eprintln!("Skipping multi-project concurrency test: missing runtimes");
        return;
    }

    let dir = tempdir().unwrap();
    let pm = Arc::new(MacOSProcessManager::new());
    let db_port = localforge_runtime::find_available_port(13530, 50).unwrap_or(13530);
    let mariadb_sup = MariaDbSupervisor::new(dir.path().join("mariadb"), db_port, pm.clone());
    mariadb_sup.start().expect("Failed to start MariaDB");

    // Setup DBs for Project A and Project B
    mariadb_sup
        .create_project_database("db_alpha", "user_alpha", "alpha_pass")
        .unwrap();
    mariadb_sup
        .create_project_database("db_beta", "user_beta", "beta_pass")
        .unwrap();

    let sock_path = mariadb_sup.socket_path().to_string_lossy().to_string();

    // Create Site A files
    let site_a_dir = dir.path().join("site_alpha");
    std::fs::create_dir_all(&site_a_dir).unwrap();
    let index_a = site_a_dir.join("index.php");
    std::fs::write(
        &index_a,
        format!(
            r#"<?php
header('Content-Type: application/json');
$pdo = new PDO("mysql:unix_socket={sock_path};dbname=db_alpha", "user_alpha", "alpha_pass");
echo json_encode(['site' => 'alpha', 'php' => PHP_VERSION]);
"#
        ),
    )
    .unwrap();

    // Create Site B files
    let site_b_dir = dir.path().join("site_beta");
    std::fs::create_dir_all(&site_b_dir).unwrap();
    let index_b = site_b_dir.join("index.php");
    std::fs::write(
        &index_b,
        format!(
            r#"<?php
header('Content-Type: application/json');
$pdo = new PDO("mysql:unix_socket={sock_path};dbname=db_beta", "user_beta", "beta_pass");
echo json_encode(['site' => 'beta', 'php' => PHP_VERSION]);
"#
        ),
    )
    .unwrap();

    let store = Arc::new(MetadataStore::open_in_memory().unwrap());
    let volume_manager = Arc::new(MacOSVolumeManager::new());
    let volume_barrier = Arc::new(VolumeBarrier::new(volume_manager.clone()));
    let vol_info = volume_manager
        .find_volume_for_path(dir.path())
        .unwrap()
        .unwrap();

    let proj_a = ProjectRecord {
        id: ProjectId::from_string("proj_alpha"),
        name: "Site Alpha".into(),
        path: site_a_dir,
        volume_uuid: vol_info.uuid.clone(),
        document_root: "".into(),
        php_version: "8.5".into(),
        local_domain: "alpha.test".into(),
        status: ProjectStatus::Stopped,
        created_at: "2026-10-18T00:00:00Z".into(),
        updated_at: "2026-10-18T00:00:00Z".into(),
    };
    let proj_b = ProjectRecord {
        id: ProjectId::from_string("proj_beta"),
        name: "Site Beta".into(),
        path: site_b_dir,
        volume_uuid: vol_info.uuid,
        document_root: "".into(),
        php_version: "8.5".into(),
        local_domain: "beta.test".into(),
        status: ProjectStatus::Stopped,
        created_at: "2026-10-18T00:00:00Z".into(),
        updated_at: "2026-10-18T00:00:00Z".into(),
    };
    store.insert_project(&proj_a).unwrap();
    store.insert_project(&proj_b).unwrap();

    let test_port = localforge_runtime::find_available_port(18320, 50).unwrap_or(18320);
    let nginx_sup = Arc::new(NginxSupervisor::new(
        dir.path().join("nginx"),
        dir.path().join("run"),
        test_port,
        pm.clone(),
    ));
    let service = ProjectRuntimeService::new(
        store.clone(),
        volume_manager.clone(),
        volume_barrier.clone(),
        pm.clone(),
        Arc::new(PhpFpmSupervisor::new(dir.path().join("run"), pm.clone())),
        nginx_sup.clone(),
        Arc::new(mariadb_sup),
        test_port,
    );

    // 1. Start both projects concurrently
    let status_a = service
        .start_project(&proj_a.id)
        .expect("Site Alpha start failed");
    let status_b = service
        .start_project(&proj_b.id)
        .expect("Site Beta start failed");

    assert_eq!(status_a.status, ProjectStatus::Running);
    assert_eq!(status_b.status, ProjectStatus::Running);
    assert_ne!(
        status_a.php_fpm_pid, status_b.php_fpm_pid,
        "Pools must run under separate PIDs"
    );
    assert_ne!(
        status_a.socket_path, status_b.socket_path,
        "Pools must bind separate sockets"
    );

    // 2. Query both concurrently via shared Nginx gateway
    let (code_a, body_a) = probe_http_health(test_port, "alpha.test", "/").unwrap();
    let (code_b, body_b) = probe_http_health(test_port, "beta.test", "/").unwrap();

    assert_eq!(code_a, 200);
    assert_eq!(code_b, 200);
    assert!(body_a.contains("\"site\":\"alpha\""));
    assert!(body_b.contains("\"site\":\"beta\""));

    // 3. Stopping Project A does NOT stop Project B, nor does it stop MariaDB
    service.stop_project(&proj_a.id).expect("Stop Alpha failed");
    let (code_b_after, body_b_after) = probe_http_health(test_port, "beta.test", "/").unwrap();
    assert_eq!(
        code_b_after, 200,
        "Site Beta must remain responsive after Alpha stopped"
    );
    assert!(body_b_after.contains("\"site\":\"beta\""));

    let db_status = service.get_database_status().unwrap();
    assert!(
        db_status.is_alive,
        "MariaDB must remain active when Project A is stopped"
    );

    // Clean up
    let _ = service.stop_project(&proj_b.id);
    let _ = service.mariadb_supervisor().stop();
    let _ = nginx_sup.stop();
}

#[test]
fn test_r1_2_06_storage_resilience_and_volume_barrier() {
    let volume_manager = Arc::new(MacOSVolumeManager::new());
    let barrier = VolumeBarrier::new(volume_manager.clone());

    // 1. Verify volume barrier on fake/unmounted volume UUID
    let fake_uuid = VolumeId::new("00000000-0000-0000-0000-000000000000");
    let fake_path = Path::new("/Volumes/NonExistentDisk_12345/project");

    let verify_res = barrier.verify_volume_available(fake_path, &fake_uuid);
    assert!(
        verify_res.is_err(),
        "VolumeBarrier must reject unmounted external path"
    );

    // 2. Safe create dir all must refuse creating directories under /Volumes/ for unmounted disks
    let safe_create_res = barrier.safe_create_dir_all(fake_path, Some(&fake_uuid));
    assert!(
        safe_create_res.is_err(),
        "safe_create_dir_all must refuse to create folders on unmounted external volumes"
    );

    // 3. Safe create dir all on internal temporary disk succeeds
    let temp_dir = tempdir().unwrap();
    let valid_internal_sub = temp_dir.path().join("safe_internal_subfolder");
    let internal_res = barrier.safe_create_dir_all(&valid_internal_sub, None);
    assert!(
        internal_res.is_ok(),
        "Internal disk folder creation must succeed"
    );
    assert!(valid_internal_sub.exists());
}
