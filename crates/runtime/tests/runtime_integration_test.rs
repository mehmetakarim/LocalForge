use localforge_core::{ProjectId, ProjectStatus, VolumeId};
use localforge_platform::{MacOSProcessManager, MacOSVolumeManager, ProcessManager, VolumeManager};
use localforge_runtime::{
    health::probe_http_health, NginxSupervisor, PhpFpmSupervisor, ProjectRuntimeService,
};
use localforge_storage::{MetadataStore, ProjectRecord, VolumeBarrier};
use std::path::Path;
use std::sync::Arc;
use tempfile::tempdir;

#[test]
fn test_r1_1_01_fpm_pool_config() {
    let dir = tempdir().unwrap();
    let pm = Arc::new(MacOSProcessManager::new());
    let sup = PhpFpmSupervisor::new(dir.path().to_path_buf(), pm);
    let pid = ProjectId::from_string("proj_site1");
    let proj_dir = Path::new("/Volumes/Mac Harici Disk/Projects/site1");

    let conf = sup.generate_pool_config(&pid, proj_dir);
    assert!(conf.contains("[proj_site1]"));
    assert!(conf.contains("proj_site1.sock"));
    assert!(conf.contains("pm = ondemand"));
    assert!(conf.contains("listen.mode = 0660"));
}

#[test]
fn test_r1_1_02_nginx_vhost_validation() {
    let dir = tempdir().unwrap();
    let pm = Arc::new(MacOSProcessManager::new());
    let sup = NginxSupervisor::new(dir.path().join("conf"), dir.path().join("run"), 18080, pm);

    let project = ProjectRecord {
        id: ProjectId::from_string("proj_valid"),
        name: "Valid Site".into(),
        path: dir.path().to_path_buf(),
        volume_uuid: VolumeId::new("UUID-TEST"),
        document_root: "".into(),
        php_version: "8.5".into(),
        local_domain: "valid.test".into(),
        status: ProjectStatus::Stopped,
        created_at: "2026-10-18T00:00:00Z".into(),
        updated_at: "2026-10-18T00:00:00Z".into(),
    };

    let sock = dir.path().join("run").join("proj_valid.sock");
    let vhost = sup.generate_vhost_config(&project, &sock);
    assert!(vhost.contains("server_name valid.test"));
    assert!(vhost.contains("fastcgi_pass unix:"));

    // Validate generated base configuration passes `nginx -t` if nginx binary is installed
    if NginxSupervisor::find_nginx_binary().is_ok() {
        let res = sup.validate_configuration();
        assert!(
            res.is_ok(),
            "Base Nginx config must pass nginx -t validation: {:?}",
            res
        );
    }
}

#[test]
fn test_r1_1_03_real_php_fpm_and_nginx_http_execution() {
    // Check if php-fpm and nginx exist on the test host
    let fpm_exists = PhpFpmSupervisor::find_php_fpm_binary("any").is_ok();
    let nginx_exists = NginxSupervisor::find_nginx_binary().is_ok();

    if !fpm_exists || !nginx_exists {
        eprintln!("Skipping real daemon test: php-fpm or nginx missing on host");
        return;
    }

    let dir = tempdir().unwrap();
    let project_dir = dir.path().join("dynamic_php_site");
    std::fs::create_dir_all(&project_dir).unwrap();

    // Create index.php returning dynamic JSON payload
    let index_php = project_dir.join("index.php");
    let php_code = r#"<?php
header('Content-Type: application/json');
echo json_encode([
    'application' => 'LocalForge',
    'runtime' => PHP_VERSION,
    'execution' => 'php-fpm',
    'timestamp' => time()
]);
"#;
    std::fs::write(&index_php, php_code).unwrap();

    let store = Arc::new(MetadataStore::open_in_memory().unwrap());
    let volume_manager = Arc::new(MacOSVolumeManager::new());
    let volume_barrier = Arc::new(VolumeBarrier::new(volume_manager.clone()));
    let process_manager = Arc::new(MacOSProcessManager::new());

    // Resolve current volume for project_dir
    let vol_info = volume_manager
        .find_volume_for_path(&project_dir)
        .unwrap()
        .unwrap();

    let project = ProjectRecord {
        id: ProjectId::from_string("proj_e2e_dyn"),
        name: "E2E Dynamic Site".into(),
        path: project_dir.clone(),
        volume_uuid: vol_info.uuid,
        document_root: "".into(),
        php_version: "8.5".into(),
        local_domain: "e2edyn.test".into(),
        status: ProjectStatus::Stopped,
        created_at: "2026-10-18T00:00:00Z".into(),
        updated_at: "2026-10-18T00:00:00Z".into(),
    };
    store.insert_project(&project).unwrap();

    let test_port = 18092;
    let runtime_base = dir.path().join("localforge_runtime");
    let service = ProjectRuntimeService::create_default(
        runtime_base,
        store.clone(),
        volume_manager.clone(),
        volume_barrier.clone(),
        process_manager.clone(),
        test_port,
    );

    // 1. Start project
    let start_status = service
        .start_project(&project.id)
        .expect("Failed to start project runtime");
    assert_eq!(start_status.status, ProjectStatus::Running);
    assert!(start_status.php_fpm_pid.is_some());
    assert!(start_status.socket_path.is_some());
    assert!(start_status.nginx_pid.is_some());

    let fpm_pid = start_status.php_fpm_pid.unwrap();
    let sock_path = start_status.socket_path.unwrap();

    // Verify FPM process is genuinely in OS process table
    let fpm_identity = process_manager.inspect_process(fpm_pid).unwrap();
    assert!(
        fpm_identity.is_some(),
        "PHP-FPM process must exist in OS process table"
    );

    // Verify Unix domain socket exists and is connectable
    assert!(sock_path.exists(), "PHP-FPM socket must exist");

    // 2. Issue real HTTP request to Nginx gateway with Host header
    let (code, body) =
        probe_http_health(test_port, "e2edyn.test", "/").expect("HTTP probe to Nginx must succeed");
    assert_eq!(
        code, 200,
        "HTTP response status must be 200 OK. Raw response: {body}"
    );

    // 3. Verify dynamic PHP output in body
    assert!(
        body.contains("\"application\":\"LocalForge\""),
        "Body must contain application name"
    );
    assert!(
        body.contains("\"execution\":\"php-fpm\""),
        "Body must confirm php-fpm execution"
    );
    assert!(
        body.contains("\"runtime\":"),
        "Body must confirm PHP runtime version"
    );

    // 4. Stop project cleanly
    let stop_status = service
        .stop_project(&project.id)
        .expect("Failed to stop project runtime");
    assert_eq!(stop_status.status, ProjectStatus::Stopped);

    // Verify PHP-FPM process has terminated and socket is removed
    std::thread::sleep(std::time::Duration::from_millis(500));
    assert!(
        !sock_path.exists(),
        "PHP-FPM socket must be removed on stop"
    );
}

#[test]
fn test_r1_1_06_multi_project_isolation() {
    let fpm_exists = PhpFpmSupervisor::find_php_fpm_binary("any").is_ok();
    let nginx_exists = NginxSupervisor::find_nginx_binary().is_ok();

    if !fpm_exists || !nginx_exists {
        return;
    }

    let dir = tempdir().unwrap();
    let site_a_dir = dir.path().join("site_a");
    let site_b_dir = dir.path().join("site_b");
    std::fs::create_dir_all(&site_a_dir).unwrap();
    std::fs::create_dir_all(&site_b_dir).unwrap();

    std::fs::write(site_a_dir.join("index.php"), "<?php echo 'SITE_A_OUTPUT';").unwrap();
    std::fs::write(site_b_dir.join("index.php"), "<?php echo 'SITE_B_OUTPUT';").unwrap();

    let store = Arc::new(MetadataStore::open_in_memory().unwrap());
    let volume_manager = Arc::new(MacOSVolumeManager::new());
    let volume_barrier = Arc::new(VolumeBarrier::new(volume_manager.clone()));
    let process_manager = Arc::new(MacOSProcessManager::new());

    let vol_info = volume_manager
        .find_volume_for_path(&site_a_dir)
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

    let test_port = 18094;
    let runtime_base = dir.path().join("localforge_runtime_multi");
    let service = ProjectRuntimeService::create_default(
        runtime_base,
        store.clone(),
        volume_manager.clone(),
        volume_barrier.clone(),
        process_manager.clone(),
        test_port,
    );

    // Start both projects
    let status_a = service.start_project(&proj_a.id).unwrap();
    let status_b = service.start_project(&proj_b.id).unwrap();

    let sock_a = status_a.socket_path.unwrap();
    let sock_b = status_b.socket_path.unwrap();
    let fpm_a_pid = status_a.php_fpm_pid.unwrap();
    let fpm_b_pid = status_b.php_fpm_pid.unwrap();

    assert!(sock_a.exists());
    assert!(sock_b.exists());

    // Allow brief moment for nginx worker reload
    std::thread::sleep(std::time::Duration::from_millis(150));

    // Both should respond to their respective domain requests
    let (code_a, body_a) = probe_http_health(test_port, "alpha.test", "/").unwrap();
    assert_eq!(code_a, 200, "Site A must return 200: {body_a}");
    assert!(
        body_a.contains("SITE_A_OUTPUT"),
        "Site A output mismatch: {body_a}"
    );

    let (code_b, body_b) = probe_http_health(test_port, "beta.test", "/").unwrap();
    assert_eq!(code_b, 200, "Site B must return 200: {body_b}");
    assert!(
        body_b.contains("SITE_B_OUTPUT"),
        "Site B output mismatch: {body_b}"
    );

    // STOP Project A ONLY!
    service.stop_project(&proj_a.id).unwrap();

    // Verify Project A is stopped and its socket is gone
    std::thread::sleep(std::time::Duration::from_millis(500));
    assert!(!sock_a.exists(), "Project A socket must be removed");
    assert!(
        process_manager
            .inspect_process(fpm_a_pid)
            .unwrap()
            .is_none(),
        "Project A FPM must be terminated"
    );

    // CRITICAL: Verify Project B is STILL ALIVE and its socket STILL EXISTS!
    assert!(sock_b.exists(), "Project B socket must remain alive");
    assert!(
        process_manager
            .inspect_process(fpm_b_pid)
            .unwrap()
            .is_some(),
        "Project B FPM must remain running"
    );

    // CRITICAL: Shared Nginx gateway must still serve Project B!
    let (_, body_b_again) = probe_http_health(test_port, "beta.test", "/").unwrap();
    assert!(
        body_b_again.contains("SITE_B_OUTPUT"),
        "Project B must still be served through shared Nginx gateway"
    );

    // Stop Project B
    service.stop_project(&proj_b.id).unwrap();
    assert!(!sock_b.exists());
}

#[test]
fn test_r1_1a_01_probe_classification_distinguishes_routing_from_infrastructure() {
    use localforge_runtime::{classify_probe_result, RuntimeError};

    // 1. Success HTTP 200 -> Running
    let assess_200 = classify_probe_result(Ok((200, "OK".into())));
    assert_eq!(assess_200.project_status(), ProjectStatus::Running);
    assert!(assess_200.message().contains("200"));

    // 2. HTTP 404 (Route not found / missing root index) -> Running with routing diagnostic
    let assess_404 = classify_probe_result(Ok((404, "Not Found".into())));
    assert_eq!(assess_404.project_status(), ProjectStatus::Running);
    assert!(assess_404.message().contains("HTTP 404"));
    assert!(assess_404
        .message()
        .contains("without default root handler"));

    // 3. HTTP 403 -> Running with permission / index diagnostic
    let assess_403 = classify_probe_result(Ok((403, "Forbidden".into())));
    assert_eq!(assess_403.project_status(), ProjectStatus::Running);
    assert!(assess_403.message().contains("HTTP 403"));

    // 4. HTTP 500 (Application script error) -> Degraded (infrastructure alive, app code crashed)
    let assess_500 = classify_probe_result(Ok((500, "Internal Server Error".into())));
    assert_eq!(assess_500.project_status(), ProjectStatus::Degraded);
    assert!(assess_500.message().contains("HTTP 500"));
    assert!(assess_500.message().contains("Inspect PHP error log"));

    // 5. HTTP 502 (FastCGI upstream / Bad Gateway) -> Degraded (NEVER Running!)
    let assess_502 = classify_probe_result(Ok((502, "Bad Gateway".into())));
    assert_eq!(assess_502.project_status(), ProjectStatus::Degraded);
    assert!(assess_502.message().contains("HTTP 502"));
    assert!(assess_502
        .message()
        .contains("FastCGI communication failed"));

    // 6. HTTP 504 (FastCGI timeout) -> Degraded (NEVER Running!)
    let assess_504 = classify_probe_result(Ok((504, "Gateway Timeout".into())));
    assert_eq!(assess_504.project_status(), ProjectStatus::Degraded);
    assert!(assess_504.message().contains("HTTP 504"));

    // 7. Transport connection failure -> Degraded (NEVER Running!)
    let transport_err = Err(RuntimeError::ServiceUnhealthy {
        service: "nginx".into(),
        reason: "Connection refused to 127.0.0.1:8080".into(),
    });
    let assess_transport = classify_probe_result(transport_err);
    assert_eq!(assess_transport.project_status(), ProjectStatus::Degraded);
    assert!(assess_transport
        .message()
        .contains("Gateway transport error"));
}

#[test]
fn test_r1_1a_02_loopback_url_and_domain_separation() {
    let dir = tempdir().unwrap();
    let store = Arc::new(MetadataStore::open_in_memory().unwrap());
    let volume_manager = Arc::new(MacOSVolumeManager::new());
    let volume_barrier = Arc::new(VolumeBarrier::new(volume_manager.clone()));
    let process_manager = Arc::new(MacOSProcessManager::new());

    let vol_info = volume_manager
        .find_volume_for_path(dir.path())
        .unwrap()
        .unwrap();

    let project = ProjectRecord {
        id: ProjectId::from_string("proj_url_test"),
        name: "URL Test".into(),
        path: dir.path().to_path_buf(),
        volume_uuid: vol_info.uuid,
        document_root: "".into(),
        php_version: "8.5".into(),
        local_domain: "urltest.test".into(),
        status: ProjectStatus::Stopped,
        created_at: "2026-10-18T00:00:00Z".into(),
        updated_at: "2026-10-18T00:00:00Z".into(),
    };
    store.insert_project(&project).unwrap();

    let service = ProjectRuntimeService::create_default(
        dir.path().join("runtime"),
        store.clone(),
        volume_manager,
        volume_barrier,
        process_manager,
        18099,
    );

    let status = service.get_runtime_status(&project.id).unwrap();
    // Must report loopback IP with port, NEVER unresolvable domain
    assert_eq!(status.local_url, "http://127.0.0.1:18099");
    assert_eq!(status.local_domain, "urltest.test");
}

#[test]
fn test_r1_1a_03_stale_or_dead_fpm_socket_never_reports_running() {
    let dir = tempdir().unwrap();
    let store = Arc::new(MetadataStore::open_in_memory().unwrap());
    let volume_manager = Arc::new(MacOSVolumeManager::new());
    let volume_barrier = Arc::new(VolumeBarrier::new(volume_manager.clone()));
    let process_manager = Arc::new(MacOSProcessManager::new());

    let vol_info = volume_manager
        .find_volume_for_path(dir.path())
        .unwrap()
        .unwrap();

    // Insert a project with status Running in DB, but with no live process
    let project = ProjectRecord {
        id: ProjectId::from_string("proj_crashed"),
        name: "Crashed Site".into(),
        path: dir.path().to_path_buf(),
        volume_uuid: vol_info.uuid,
        document_root: "".into(),
        php_version: "8.5".into(),
        local_domain: "crashed.test".into(),
        status: ProjectStatus::Running,
        created_at: "2026-10-18T00:00:00Z".into(),
        updated_at: "2026-10-18T00:00:00Z".into(),
    };
    store.insert_project(&project).unwrap();

    let service = ProjectRuntimeService::create_default(
        dir.path().join("runtime"),
        store.clone(),
        volume_manager,
        volume_barrier,
        process_manager,
        18098,
    );

    let status = service.get_runtime_status(&project.id).unwrap();
    // Must be FAILED, NOT RUNNING!
    assert_eq!(status.status, ProjectStatus::Failed);
    assert!(status.message.contains("terminated unexpectedly"));

    // DB must be synced to Failed
    let refreshed = store.get_project(&project.id).unwrap().unwrap();
    assert_eq!(refreshed.status, ProjectStatus::Failed);
}

#[test]
fn test_r1_3a_v1_01_sync_all_projects_surfaces_failures_and_syncs_sqlite() {
    let dir = tempdir().unwrap();
    let store = Arc::new(MetadataStore::open_in_memory().unwrap());
    let volume_manager = Arc::new(MacOSVolumeManager::new());
    let volume_barrier = Arc::new(VolumeBarrier::new(volume_manager.clone()));
    let process_manager = Arc::new(MacOSProcessManager::new());

    // Project 1: On a nonexistent/unmounted external volume
    let proj_disconnected = ProjectRecord {
        id: ProjectId::from_string("proj_unmounted"),
        name: "Unmounted Site".into(),
        path: Path::new("/Volumes/NonexistentDisk/site").to_path_buf(),
        volume_uuid: VolumeId::new("NONEXISTENT-UUID-999"),
        document_root: "".into(),
        php_version: "8.4".into(),
        local_domain: "unmounted.test".into(),
        status: ProjectStatus::Running, // Stale persisted status!
        created_at: "2026-10-18T00:00:00Z".into(),
        updated_at: "2026-10-18T00:00:00Z".into(),
    };
    store.insert_project(&proj_disconnected).unwrap();

    // Project 2: On a valid volume, but daemon is crashed
    let vol_info = volume_manager
        .find_volume_for_path(dir.path())
        .unwrap()
        .unwrap();

    let proj_crashed = ProjectRecord {
        id: ProjectId::from_string("proj_crashed_sync"),
        name: "Crashed Site Sync".into(),
        path: dir.path().to_path_buf(),
        volume_uuid: vol_info.uuid,
        document_root: "".into(),
        php_version: "8.4".into(),
        local_domain: "crashed-sync.test".into(),
        status: ProjectStatus::Running, // Stale persisted status!
        created_at: "2026-10-18T00:00:00Z".into(),
        updated_at: "2026-10-18T00:00:00Z".into(),
    };
    store.insert_project(&proj_crashed).unwrap();

    let service = ProjectRuntimeService::create_default(
        dir.path().join("runtime"),
        store.clone(),
        volume_manager,
        volume_barrier,
        process_manager,
        18097,
    );

    // Perform sync
    let sync_results = service.sync_all_projects_runtime_status().unwrap();
    assert_eq!(sync_results.len(), 2);

    let res_disconnected = sync_results
        .iter()
        .find(|s| s.project_id == proj_disconnected.id)
        .unwrap();
    assert_eq!(res_disconnected.status, ProjectStatus::Disconnected);
    assert!(res_disconnected
        .message
        .contains("External storage volume unmounted"));

    let res_crashed = sync_results
        .iter()
        .find(|s| s.project_id == proj_crashed.id)
        .unwrap();
    assert_eq!(res_crashed.status, ProjectStatus::Failed);
    assert!(res_crashed.message.contains("terminated unexpectedly"));

    // Verify persisted SQLite states: stale Running status MUST NOT remain!
    let db_p1 = store.get_project(&proj_disconnected.id).unwrap().unwrap();
    assert_eq!(db_p1.status, ProjectStatus::Disconnected);

    let db_p2 = store.get_project(&proj_crashed.id).unwrap().unwrap();
    assert_eq!(db_p2.status, ProjectStatus::Failed);
}
