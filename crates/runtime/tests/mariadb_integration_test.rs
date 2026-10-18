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

    // 3. Drop safety: self-authorization using database name as token MUST fail closed
    let self_auth = sup.drop_project_database(db_name, db_name);
    assert!(
        self_auth.is_err(),
        "Self-authorization using database name as token must be rejected"
    );

    // 4. Request deletion handle (returns audit request_id, NOT authorization decision)
    let req = sup
        .request_database_deletion_approval(db_name, "integration_test_ui")
        .expect("Failed to request deletion handle");
    assert!(req.request_id.starts_with("del_req_"));

    // 5. Caller attempting to self-authorize using request_id must be rejected
    let self_req_drop = sup.drop_project_database(db_name, &req.request_id);
    assert!(
        self_req_drop.is_err(),
        "Self-authorization using request_id must be rejected"
    );

    // 6. Database deletion is explicitly disabled in Phase R1.2B pending trusted desktop human approval
    let disabled_drop = sup.drop_project_database(db_name, "arbitrary_token");
    assert!(
        disabled_drop.is_err(),
        "Database deletion must be explicitly disabled in R1.2B"
    );
    assert!(matches!(
        disabled_drop.unwrap_err(),
        localforge_runtime::RuntimeError::DatabaseDeletionUnavailable(_)
    ));

    // Database remains intact (not deleted)
    let check_db = sup
        .execute_query(&format!("SHOW DATABASES LIKE '{db_name}';"))
        .unwrap();
    assert!(
        check_db.contains(db_name),
        "Database must remain intact in R1.2B"
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

    // 2. Overwrite Protection: dumping to existing file without permission must fail
    let overwrite_res = sup.dump_database(source_db, &backup_file);
    assert!(
        overwrite_res.is_err(),
        "Overwriting existing backup file must fail closed"
    );

    // 3. Non-empty restore protection: attempting to restore into existing nonempty source_db must fail
    let nonempty_restore = sup.restore_database(source_db, &backup_file);
    assert!(
        nonempty_restore.is_err(),
        "Restoring into existing non-empty database must fail closed"
    );

    // 4. Restore into a new database
    let target_db = "backup_restored_db";
    let restore_res = sup.restore_database(target_db, &backup_file);
    assert!(
        restore_res.is_ok(),
        "Restore must succeed: {:?}",
        restore_res
    );

    // 5. Verify data records in restored database
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

#[test]
fn test_r1_2a_07_credential_security_and_injection_adversarial() {
    let _lock = SERIAL_TEST_LOCK.lock().unwrap();
    let binaries_found = MariaDbSupervisor::discover_binaries().is_ok();
    if !binaries_found {
        eprintln!("Skipping MariaDB adversarial credential test: binaries not found on host");
        return;
    }

    let dir = tempdir().unwrap();
    let pm = Arc::new(MacOSProcessManager::new());
    let db_port = localforge_runtime::find_available_port(13590, 50).unwrap_or(13590);
    let sup = MariaDbSupervisor::new(dir.path().to_path_buf(), db_port, pm);
    sup.start().expect("Failed to start MariaDB");

    // 1. Password with null byte must fail closed
    let null_byte_res = sup.create_project_database("db_null", "user_null", "bad\0pass");
    assert!(null_byte_res.is_err());
    assert!(null_byte_res.unwrap_err().to_string().contains("null byte"));

    // 2. Empty password must fail closed
    let empty_pass_res = sup.create_project_database("db_empty", "user_empty", "");
    assert!(empty_pass_res.is_err());

    // 3. SQL injection payload in password: must be provisioned safely as literal
    let inj_db = "db_inj_test";
    let inj_user = "user_inj";
    let inj_pass = "p'ass'; DROP DATABASE mysql; --";
    let inj_res = sup.create_project_database(inj_db, inj_user, inj_pass);
    assert!(
        inj_res.is_ok(),
        "SQL injection password must be safely escaped: {:?}",
        inj_res
    );

    // Verify mysql database was NOT dropped
    let check_mysql = sup.execute_query("SHOW DATABASES LIKE 'mysql';").unwrap();
    assert!(
        check_mysql.contains("mysql"),
        "System database must not be dropped"
    );

    // 4. Special characters, quotes, backslashes and unicode in password
    let unicode_db = "db_unicode_test";
    let unicode_user = "user_unicode";
    let unicode_pass = "P@ss\\w'o\"rd🚀2026_Şçğüö;--";
    let unicode_res = sup.create_project_database(unicode_db, unicode_user, unicode_pass);
    assert!(
        unicode_res.is_ok(),
        "Unicode and special char password must be provisioned: {:?}",
        unicode_res
    );

    sup.stop().expect("Failed to stop MariaDB");
}

#[test]
fn test_r1_2a_08_backup_symlink_defense_and_atomic_permissions() {
    let _lock = SERIAL_TEST_LOCK.lock().unwrap();
    let binaries_found = MariaDbSupervisor::discover_binaries().is_ok();
    if !binaries_found {
        eprintln!("Skipping MariaDB backup symlink test: binaries not found on host");
        return;
    }

    let dir = tempdir().unwrap();
    let pm = Arc::new(MacOSProcessManager::new());
    let db_port = localforge_runtime::find_available_port(13640, 50).unwrap_or(13640);
    let sup = MariaDbSupervisor::new(dir.path().to_path_buf(), db_port, pm);
    sup.start().expect("Failed to start MariaDB");

    let db_name = "db_symlink_test";
    sup.create_project_database(db_name, "u_sym", "p_sym")
        .unwrap();

    // Create a sensitive file
    let sensitive_file = dir.path().join("sensitive.txt");
    std::fs::write(&sensitive_file, "TOP_SECRET").unwrap();

    let symlink_target = dir.path().join("symlink_backup.sql");
    #[cfg(unix)]
    std::os::unix::fs::symlink(&sensitive_file, &symlink_target).unwrap();

    // Dump targeting symlink must be rejected
    let sym_res = sup.dump_database(db_name, &symlink_target);
    assert!(sym_res.is_err(), "Dumping to symlink must fail closed");

    // Verify sensitive file was not overwritten
    let content = std::fs::read_to_string(&sensitive_file).unwrap();
    assert_eq!(content, "TOP_SECRET");

    sup.stop().expect("Failed to stop MariaDB");
}

#[test]
fn test_r1_2a_09_restore_invalid_and_corrupt_files() {
    let _lock = SERIAL_TEST_LOCK.lock().unwrap();
    let binaries_found = MariaDbSupervisor::discover_binaries().is_ok();
    if !binaries_found {
        eprintln!("Skipping MariaDB restore invalid test: binaries not found on host");
        return;
    }

    let dir = tempdir().unwrap();
    let pm = Arc::new(MacOSProcessManager::new());
    let db_port = localforge_runtime::find_available_port(13690, 50).unwrap_or(13690);
    let sup = MariaDbSupervisor::new(dir.path().to_path_buf(), db_port, pm);
    sup.start().expect("Failed to start MariaDB");

    // 1. Nonexistent file must fail closed
    let non_existent = dir.path().join("does_not_exist.sql");
    assert!(sup.restore_database("target_1", &non_existent).is_err());

    // 2. Empty file must fail closed
    let empty_file = dir.path().join("empty.sql");
    std::fs::write(&empty_file, "").unwrap();
    assert!(sup.restore_database("target_2", &empty_file).is_err());

    // 3. Binary corrupted file (not SQL) must fail closed
    let corrupt_file = dir.path().join("corrupt.sql");
    std::fs::write(&corrupt_file, b"\x00\x01\x02\x03\xFF\xFE\xFD").unwrap();
    assert!(sup.restore_database("target_3", &corrupt_file).is_err());

    sup.stop().expect("Failed to stop MariaDB");
}

#[test]
fn test_r1_2a_10_datadir_ownership_and_unknown_directory_guard() {
    let _lock = SERIAL_TEST_LOCK.lock().unwrap();

    let dir = tempdir().unwrap();
    let data_dir = dir.path().join("data");
    std::fs::create_dir_all(&data_dir).unwrap();

    // Populate data_dir with unexpected non-MariaDB files
    let alien_file = data_dir.join("some_random_file.bin");
    std::fs::write(&alien_file, "unexpected alien contents").unwrap();

    let pm = Arc::new(MacOSProcessManager::new());
    let sup = MariaDbSupervisor::new(dir.path().to_path_buf(), 13740, pm);

    // Initializing datadir must fail closed due to unexpected contents without mysql/ schema
    let init_res = sup.ensure_datadir_initialized();
    assert!(
        init_res.is_err(),
        "Must refuse to overwrite unknown existing data directory"
    );
    assert!(init_res
        .unwrap_err()
        .to_string()
        .contains("Refusing to overwrite unknown data directory"));
}

#[test]
fn test_r1_2a_11_deletion_authorization_tampering_and_expiry() {
    let _lock = SERIAL_TEST_LOCK.lock().unwrap();

    let dir = tempdir().unwrap();
    let sup = MariaDbSupervisor::new_simple(dir.path().to_path_buf(), 13790);

    // 1. Self-approval using database name as token must be rejected
    let self_auth = sup.drop_project_database("secret_prod_db", "secret_prod_db");
    assert!(self_auth.is_err());
    assert!(self_auth
        .unwrap_err()
        .to_string()
        .contains("Self-authorization rejected: using database name as confirmation token is strictly prohibited"));

    // 2. Request deletion handle for db_a
    let req_a = sup
        .request_database_deletion_approval("db_a", "ui_admin")
        .unwrap();
    assert!(req_a.request_id.starts_with("del_req_db_a_"));

    // 3. Attempting to self-authorize using request_id must be rejected
    let req_auth = sup.drop_project_database("db_a", &req_a.request_id);
    assert!(req_auth.is_err());
    assert!(req_auth
        .unwrap_err()
        .to_string()
        .contains("Self-authorization rejected: a deletion request identifier is not a trusted human approval decision"));

    // 4. Any arbitrary token fails closed because deletion is explicitly disabled in R1.2B
    let disabled = sup.drop_project_database("db_a", "arbitrary_token");
    assert!(disabled.is_err());
    assert!(matches!(
        disabled.unwrap_err(),
        localforge_runtime::RuntimeError::DatabaseDeletionUnavailable(_)
    ));
}

#[test]
fn test_r1_2b_12_restore_fail_closed_occupied_destination_and_interrupted_restores() {
    let _lock = SERIAL_TEST_LOCK.lock().unwrap();
    let binaries_found = MariaDbSupervisor::discover_binaries().is_ok();
    if !binaries_found {
        eprintln!("Skipping MariaDB restore fail-closed test: binaries not found");
        return;
    }

    let dir = tempdir().unwrap();
    let pm = Arc::new(MacOSProcessManager::new());
    let db_port = localforge_runtime::find_available_port(13820, 50).unwrap_or(13820);
    let sup = MariaDbSupervisor::new(dir.path().to_path_buf(), db_port, pm);

    sup.start().expect("Failed to start MariaDB");

    // 1. Setup occupied destination database
    let occupied_db = "db_occupied_dest";
    sup.create_project_database(occupied_db, "occ_user", "pass123")
        .unwrap();
    sup.execute_query(&format!(
        "USE `{occupied_db}`; CREATE TABLE existing_t (id INT PRIMARY KEY); INSERT INTO existing_t VALUES (42);"
    ))
    .unwrap();

    // 2. Create a valid SQL dump file
    let valid_dump = dir.path().join("valid_dump.sql");
    std::fs::write(
        &valid_dump,
        "-- MariaDB dump\nCREATE TABLE new_t (id INT PRIMARY KEY);\nINSERT INTO new_t VALUES (100);\n",
    )
    .unwrap();

    // 3. Restore into occupied destination MUST fail closed without modifying destination
    let occupied_err = sup.restore_database(occupied_db, &valid_dump);
    assert!(
        occupied_err.is_err(),
        "Must fail closed when destination is non-empty"
    );
    let err_str = occupied_err.unwrap_err().to_string();
    assert!(
        err_str.contains("contains 1 tables. Refusing to restore into non-empty database"),
        "Unexpected error: {err_str}"
    );

    // Verify existing data was untouched
    let check_existing = sup
        .execute_query(&format!(
            "USE `{occupied_db}`; SELECT COUNT(*) FROM existing_t;"
        ))
        .unwrap();
    assert!(
        check_existing.contains('1'),
        "Original table must remain intact"
    );

    // 4. Test interrupted / corrupted dump restore into empty database
    let partial_db = "db_partial_dest";
    let partial_dump = dir.path().join("partial_broken.sql");
    std::fs::write(
        &partial_dump,
        "-- MariaDB dump\nCREATE TABLE part1 (id INT PRIMARY KEY);\nINSERT INTO part1 VALUES (1);\nINVALID_SQL_SYNTAX_ERROR_ABORT;\nCREATE TABLE part2 (id INT PRIMARY KEY);\n",
    )
    .unwrap();

    let partial_err = sup.restore_database(partial_db, &partial_dump);
    assert!(
        partial_err.is_err(),
        "Must fail closed on malformed / interrupted dump"
    );
    let part_err_str = partial_err.unwrap_err().to_string();
    assert!(
        part_err_str.contains("Database restore failed partially"),
        "Must report partial restore accurately: {part_err_str}"
    );
    assert!(
        part_err_str.contains("Warning: 1 tables were created before failure"),
        "Must record exact table count created: {part_err_str}"
    );
    assert!(
        part_err_str.contains("Note: MariaDB DDL operations execute with implicit commits"),
        "Must document DDL implicit commit limitation: {part_err_str}"
    );

    sup.stop().expect("Failed to stop MariaDB");
}

#[test]
fn test_r1_2b_13_application_support_storage_safety() {
    let _lock = SERIAL_TEST_LOCK.lock().unwrap();

    // 1. Verify Application Support resolution returns non-empty path
    let app_support = localforge_runtime::resolve_application_support_dir();
    assert!(app_support.is_ok(), "Directory resolution must succeed");
    let p = app_support.unwrap();
    assert!(
        p.ends_with("Library/Application Support/LocalForge"),
        "Path must end in Library/Application Support/LocalForge, got: {:?}",
        p
    );

    // 2. Verify MariaDB base dir is under Application Support
    let mariadb_dir = localforge_runtime::resolve_mariadb_base_dir().unwrap();
    assert_eq!(mariadb_dir, p.join("mariadb"));

    // 3. Verify MariaDbSupervisor::create_default resolves correctly
    let pm = Arc::new(MacOSProcessManager::new());
    let default_sup = MariaDbSupervisor::create_default(pm, 13890);
    assert!(
        default_sup.is_ok(),
        "create_default must succeed with valid Application Support: {:?}",
        default_sup.err()
    );
    let sup = default_sup.unwrap();
    assert_eq!(sup.data_dir(), mariadb_dir.join("data"));
    assert_eq!(sup.run_dir(), mariadb_dir.join("run"));
}
