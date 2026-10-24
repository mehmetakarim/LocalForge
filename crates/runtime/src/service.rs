use crate::error::{Result, RuntimeError};
use crate::health::probe_http_health;
use crate::mariadb::{DatabaseHealthStatus, MariaDbSupervisor};
use crate::nginx::NginxSupervisor;
use crate::php::PhpFpmSupervisor;
use localforge_core::{BrowserReadinessReport, ProjectId, ProjectStatus};
use localforge_platform::{ProcessManager, VolumeManager};
use localforge_storage::{MetadataStore, VolumeBarrier};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectRuntimeStatus {
    pub project_id: ProjectId,
    pub name: String,
    pub status: ProjectStatus,
    pub php_version: String,
    pub php_fpm_pid: Option<u32>,
    pub socket_path: Option<PathBuf>,
    pub nginx_pid: Option<u32>,
    pub gateway_port: u16,
    pub loopback_port: Option<u16>,
    pub local_domain: String,
    pub local_url: String,
    pub message: String,
}

/// Detailed health assessment of an HTTP probe on the loopback gateway.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GatewayHealthAssessment {
    /// FastCGI and application executed successfully (HTTP 200..=399).
    RunningHealthy { status_code: u16, message: String },
    /// Gateway and FastCGI are alive, but application route was not found or client error (HTTP 400..=499).
    /// Common for generic PHP applications without a default '/' index.
    RunningWithClientStatus { status_code: u16, message: String },
    /// PHP-FPM executed, but PHP application raised a 500 Internal Server Error (syntax/fatal error).
    DegradedApplicationError { status_code: u16, message: String },
    /// Nginx gateway received request, but FastCGI upstream communication failed (502 Bad Gateway / 504 Timeout).
    DegradedFastCgiFailure { status_code: u16, message: String },
    /// Service Unavailable (503).
    DegradedServiceUnavailable { status_code: u16, message: String },
    /// Other HTTP server error.
    DegradedHttpError { status_code: u16, message: String },
    /// Loopback TCP connection failed, timed out, or response was malformed.
    TransportError { message: String },
}

impl GatewayHealthAssessment {
    pub fn project_status(&self) -> ProjectStatus {
        match self {
            Self::RunningHealthy { .. } | Self::RunningWithClientStatus { .. } => {
                ProjectStatus::Running
            }
            Self::DegradedApplicationError { .. }
            | Self::DegradedFastCgiFailure { .. }
            | Self::DegradedServiceUnavailable { .. }
            | Self::DegradedHttpError { .. }
            | Self::TransportError { .. } => ProjectStatus::Degraded,
        }
    }

    pub fn message(&self) -> &str {
        match self {
            Self::RunningHealthy { message, .. }
            | Self::RunningWithClientStatus { message, .. }
            | Self::DegradedApplicationError { message, .. }
            | Self::DegradedFastCgiFailure { message, .. }
            | Self::DegradedServiceUnavailable { message, .. }
            | Self::DegradedHttpError { message, .. }
            | Self::TransportError { message } => message.as_str(),
        }
    }
}

pub fn classify_probe_result(
    probe_result: std::result::Result<(u16, String), RuntimeError>,
) -> GatewayHealthAssessment {
    match probe_result {
        Ok((code, _)) => match code {
            200..=399 => GatewayHealthAssessment::RunningHealthy {
                status_code: code,
                message: format!("Runtime active and verified (HTTP {code})."),
            },
            404 => GatewayHealthAssessment::RunningWithClientStatus {
                status_code: 404,
                message: "Runtime active and verified (HTTP 404 at root route; application is running without default root handler).".into(),
            },
            403 => GatewayHealthAssessment::RunningWithClientStatus {
                status_code: 403,
                message: "Runtime active and verified (HTTP 403 at root route; check directory permissions or index configuration).".into(),
            },
            400..=499 => GatewayHealthAssessment::RunningWithClientStatus {
                status_code: code,
                message: format!("Runtime active and verified (HTTP {code} client response)."),
            },
            500 => GatewayHealthAssessment::DegradedApplicationError {
                status_code: 500,
                message: "Infrastructure operational, but PHP application returned server error (HTTP 500). Inspect PHP error log.".into(),
            },
            502 => GatewayHealthAssessment::DegradedFastCgiFailure {
                status_code: 502,
                message: "Gateway upstream error: FastCGI communication failed (HTTP 502 Bad Gateway). PHP-FPM socket may be unresponsive or worker crashed.".into(),
            },
            504 => GatewayHealthAssessment::DegradedFastCgiFailure {
                status_code: 504,
                message: "Gateway upstream error: FastCGI request timed out (HTTP 504 Gateway Timeout).".into(),
            },
            503 => GatewayHealthAssessment::DegradedServiceUnavailable {
                status_code: 503,
                message: "Gateway service temporarily unavailable (HTTP 503).".into(),
            },
            other => GatewayHealthAssessment::DegradedHttpError {
                status_code: other,
                message: format!("Gateway returned unexpected server error (HTTP {other})."),
            },
        },
        Err(e) => GatewayHealthAssessment::TransportError {
            message: format!("Gateway transport error: failed to establish loopback HTTP connection: {e}"),
        },
    }
}

pub struct ProjectRuntimeService {
    metadata_store: std::sync::RwLock<Arc<MetadataStore>>,
    volume_manager: Arc<dyn VolumeManager>,
    volume_barrier: Arc<VolumeBarrier>,
    process_manager: Arc<dyn ProcessManager>,
    php_supervisor: Arc<PhpFpmSupervisor>,
    nginx_supervisor: Arc<NginxSupervisor>,
    mariadb_supervisor: Arc<MariaDbSupervisor>,
    gateway_port: u16,
}

impl ProjectRuntimeService {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        metadata_store: Arc<MetadataStore>,
        volume_manager: Arc<dyn VolumeManager>,
        volume_barrier: Arc<VolumeBarrier>,
        process_manager: Arc<dyn ProcessManager>,
        php_supervisor: Arc<PhpFpmSupervisor>,
        nginx_supervisor: Arc<NginxSupervisor>,
        mariadb_supervisor: Arc<MariaDbSupervisor>,
        gateway_port: u16,
    ) -> Self {
        Self {
            metadata_store: std::sync::RwLock::new(metadata_store),
            volume_manager,
            volume_barrier,
            process_manager,
            php_supervisor,
            nginx_supervisor,
            mariadb_supervisor,
            gateway_port,
        }
    }

    /// Reconnects the runtime service to a newly recovered persistent metadata store.
    pub fn reconnect_metadata_store(&self, store: Arc<MetadataStore>) {
        let mut lock = self.metadata_store.write().unwrap();
        *lock = store;
    }

    /// Returns the currently active metadata store reference.
    pub fn metadata_store(&self) -> Arc<MetadataStore> {
        self.metadata_store.read().unwrap().clone()
    }

    /// Convenience constructor initializing default paths under base runtime directory.
    pub fn create_default(
        base_dir: PathBuf,
        metadata_store: Arc<MetadataStore>,
        volume_manager: Arc<dyn VolumeManager>,
        volume_barrier: Arc<VolumeBarrier>,
        process_manager: Arc<dyn ProcessManager>,
        gateway_port: u16,
    ) -> Self {
        let run_dir = base_dir.join("run");
        let conf_dir = base_dir.join("nginx");
        let mariadb_dir = base_dir.join("mariadb");
        let _ = std::fs::create_dir_all(&run_dir);
        let _ = std::fs::create_dir_all(&conf_dir);
        let _ = std::fs::create_dir_all(&mariadb_dir);

        let php_supervisor = Arc::new(PhpFpmSupervisor::new(
            run_dir.clone(),
            process_manager.clone(),
        ));
        let nginx_supervisor = Arc::new(NginxSupervisor::new(
            conf_dir,
            run_dir,
            gateway_port,
            process_manager.clone(),
        ));
        let mariadb_supervisor = Arc::new(MariaDbSupervisor::new(
            mariadb_dir,
            3306,
            process_manager.clone(),
        ));

        Self::new(
            metadata_store,
            volume_manager,
            volume_barrier,
            process_manager,
            php_supervisor,
            nginx_supervisor,
            mariadb_supervisor,
            gateway_port,
        )
    }

    /// Ensures the isolated MariaDB database daemon is started, healthy, and verified.
    pub fn ensure_database_running(&self) -> Result<DatabaseHealthStatus> {
        let _ = self.mariadb_supervisor.start()?;
        self.mariadb_supervisor.check_health()
    }

    /// Returns verified database health status.
    pub fn get_database_status(&self) -> Result<DatabaseHealthStatus> {
        self.mariadb_supervisor.check_health()
    }

    /// Returns reference to the MariaDbSupervisor.
    pub fn mariadb_supervisor(&self) -> Arc<MariaDbSupervisor> {
        self.mariadb_supervisor.clone()
    }

    /// Starts development runtime for the given project.
    pub fn start_project(&self, project_id: &ProjectId) -> Result<ProjectRuntimeStatus> {
        let store = self.metadata_store();
        let project = store
            .get_project(project_id)?
            .ok_or_else(|| RuntimeError::ProjectNotFound(project_id.0.clone()))?;

        // 1. External volume barrier check
        if let Err(e) = self
            .volume_barrier
            .verify_volume_available(&project.path, &project.volume_uuid)
        {
            let _ = store.update_project_status(&project.id, ProjectStatus::Disconnected);
            return Err(RuntimeError::VolumeDisconnected(e.to_string()));
        }

        // 2. Validate requested PHP runtime version
        if let Err(e) = self
            .php_supervisor
            .validate_php_version(&project.php_version)
        {
            let _ = store.update_project_status(&project.id, ProjectStatus::Degraded);
            return Err(e);
        }

        // 3. Mark project STARTING
        let _ = store.update_project_status(&project.id, ProjectStatus::Starting);

        // 3b. Allocate or revalidate persistent loopback port (if V2 schema active)
        let allocator = crate::routing::PortAllocator::default();
        if let Ok(crate::routing::PortAllocationResult::Conflict { port, details, .. }) =
            allocator.allocate_port_for_project(&store, &project.id, self.process_manager.as_ref())
        {
            let _ = store.update_project_status(&project.id, ProjectStatus::Failed);
            return Err(RuntimeError::ServiceUnhealthy {
                service: format!("port-{}", port),
                reason: details,
            });
        }
        let project = store.get_project(project_id)?.unwrap_or(project);

        // 4. Start isolated PHP-FPM pool
        let socket_path =
            match self
                .php_supervisor
                .start_pool(&project.id, &project.php_version, &project.path)
            {
                Ok(sock) => sock,
                Err(e) => {
                    let _ = store.update_project_status(&project.id, ProjectStatus::Failed);
                    return Err(e);
                }
            };

        // 5. Register virtual host with shared Nginx gateway (multi-port listener)
        if let Err(e) =
            self.nginx_supervisor
                .update_vhost_state(&project, Some(&socket_path), true, true)
        {
            let _ = self.php_supervisor.stop_pool(&project.id);
            let _ = store.update_project_status(&project.id, ProjectStatus::Failed);
            return Err(e);
        }

        // 6. Independent process and socket health verification before probing
        let fpm_alive = self.php_supervisor.is_pool_alive(&project.id);
        let socket_alive = crate::health::is_unix_socket_alive(&socket_path);
        let nginx_alive = self.nginx_supervisor.is_gateway_alive();

        if !fpm_alive || !socket_alive {
            let _ = store.update_project_status(&project.id, ProjectStatus::Failed);
            return Err(RuntimeError::ServiceUnhealthy {
                service: format!("php-fpm-{}", project.id),
                reason: "PHP-FPM worker or FastCGI socket failed immediately upon startup".into(),
            });
        }

        if !nginx_alive {
            let _ = store.update_project_status(&project.id, ProjectStatus::Failed);
            return Err(RuntimeError::ServiceUnhealthy {
                service: "nginx".into(),
                reason: "Nginx gateway master process is unresponsive or failed to launch".into(),
            });
        }

        // 7. Real HTTP Health Probe (allow 50ms for worker reload)
        std::thread::sleep(std::time::Duration::from_millis(50));
        let probe_port = project.loopback_port.unwrap_or(self.gateway_port);
        let local_url = project
            .loopback_port
            .map(|p| format!("http://127.0.0.1:{p}"))
            .unwrap_or_default();

        let mut probe_result = probe_http_health(probe_port, &project.local_domain, "/");
        if probe_result.is_err() {
            let loopback_host = format!("{}:{}", project.local_domain, probe_port);
            let retry = probe_http_health(probe_port, &loopback_host, "/");
            if retry.is_ok() {
                probe_result = retry;
            } else {
                // Also retry with 127.0.0.1 host
                let direct_retry = probe_http_health(probe_port, "127.0.0.1", "/");
                if direct_retry.is_ok() {
                    probe_result = direct_retry;
                }
            }
        }

        let assessment = classify_probe_result(probe_result);
        let final_status = assessment.project_status();
        let message = assessment.message().to_string();

        // 8. Update verified status in metadata store (never false RUNNING)
        let _ = store.update_project_status(&project.id, final_status);

        let fpm_id = self.php_supervisor.get_pool_identity(&project.id);
        let nginx_id = self.nginx_supervisor.get_gateway_identity();

        Ok(ProjectRuntimeStatus {
            project_id: project.id,
            name: project.name,
            status: final_status,
            php_version: project.php_version,
            php_fpm_pid: fpm_id.map(|i| i.pid),
            socket_path: Some(socket_path),
            nginx_pid: nginx_id.map(|i| i.pid),
            gateway_port: self.gateway_port,
            loopback_port: project.loopback_port,
            local_domain: project.local_domain,
            local_url,
            message,
        })
    }

    /// Stops development runtime for the given project cleanly.
    pub fn stop_project(&self, project_id: &ProjectId) -> Result<ProjectRuntimeStatus> {
        let store = self.metadata_store();
        let project = store
            .get_project(project_id)?
            .ok_or_else(|| RuntimeError::ProjectNotFound(project_id.0.clone()))?;

        // 1. Mark project STOPPING
        let _ = store.update_project_status(&project.id, ProjectStatus::Stopping);

        // 2. Stop independent PHP-FPM pool (Project A stopped, Project B untouched)
        self.php_supervisor.stop_pool(&project.id)?;

        // 3. Update virtual host to stopped state in Nginx gateway (or remove if needed)
        let _ = self
            .nginx_supervisor
            .update_vhost_state(&project, None, false, true);

        // 4. Mark project STOPPED in metadata store
        let _ = store.update_project_status(&project.id, ProjectStatus::Stopped);

        let local_url = project
            .loopback_port
            .map(|p| format!("http://127.0.0.1:{p}"))
            .unwrap_or_default();

        Ok(ProjectRuntimeStatus {
            project_id: project.id,
            name: project.name,
            status: ProjectStatus::Stopped,
            php_version: project.php_version,
            php_fpm_pid: None,
            socket_path: None,
            nginx_pid: self.nginx_supervisor.get_gateway_identity().map(|i| i.pid),
            gateway_port: self.gateway_port,
            loopback_port: project.loopback_port,
            local_domain: project.local_domain.clone(),
            local_url,
            message: "Project runtime stopped cleanly.".into(),
        })
    }

    /// Stops the shared Nginx gateway master process.
    pub fn stop_gateway(&self) -> Result<()> {
        self.nginx_supervisor.stop()
    }

    /// Computes full typed routing profile for a project across Scenarios A through F.
    pub fn get_project_routing_profile(
        &self,
        project_id: &ProjectId,
    ) -> Result<localforge_core::ProjectRoutingProfile> {
        let store = self.metadata_store();
        let project = store
            .get_project(project_id)?
            .ok_or_else(|| RuntimeError::ProjectNotFound(project_id.0.clone()))?;

        let is_volume_mounted = self
            .volume_barrier
            .verify_volume_available(&project.path, &project.volume_uuid)
            .is_ok();

        let gateway_running = self.nginx_supervisor.is_gateway_alive();
        let fpm_running = self.php_supervisor.is_pool_alive(&project.id);

        Ok(crate::routing::ProjectRoutingEngine::compute_profile(
            &project,
            project.loopback_port,
            gateway_running,
            fpm_running,
            is_volume_mounted,
            false, // Domain resolution checked separately by client/host probe
            None,
        ))
    }

    /// Evaluates the 6 discrete readiness layers required before safely opening an external browser:
    /// 1. DNS resolution for .localforge domain
    /// 2. Assigned dedicated project loopback port (18081..=18999)
    /// 3. Verified LocalForge listener ownership on port
    /// 4. Nginx / vhost configuration existence and syntax
    /// 5. Project runtime readiness (PHP-FPM pool & socket alive)
    /// 6. Correct project HTTP response (200/redirects, not 502/503)
    pub fn verify_browser_readiness(
        &self,
        project_id: &ProjectId,
    ) -> Result<BrowserReadinessReport> {
        let store = self.metadata_store();
        let project = store
            .get_project(project_id)?
            .ok_or_else(|| RuntimeError::ProjectNotFound(project_id.0.clone()))?;

        let slug = crate::routing::ProjectRoutingEngine::slugify(&project.name);
        let preferred_domain = format!("{slug}.localforge");

        let mut layer_failures = Vec::new();

        // Layer 1: DNS Resolution
        let dns_resolved = {
            use std::net::ToSocketAddrs;
            let target = format!("{preferred_domain}:80");
            target
                .to_socket_addrs()
                .map(|mut addrs| addrs.any(|a| a.ip().is_loopback()))
                .unwrap_or(false)
        };
        if !dns_resolved {
            layer_failures.push(format!(
                "DNS Çözümleme: '{preferred_domain}' yerel alan adı çözümlenemiyor (mDNS / .localforge çözücü aktif değil)."
            ));
        }

        // Layer 2: Assigned Project Port
        let port_assigned = match project.loopback_port {
            Some(p) => (18081..=18999).contains(&p),
            None => false,
        };
        if !port_assigned {
            layer_failures.push(
                "Port Tahsisi: Projeye atanmış geçerli bir ayrık loopback portu bulunmuyor (18081..=18999 aralığında olmalıdır).".into()
            );
        }

        // Layer 3: Verified LocalForge Listener Ownership
        let mut listener_verified = false;
        if let Some(port) = project.loopback_port {
            if port_assigned {
                match self.nginx_supervisor.check_listener_health(port) {
                    crate::nginx::GatewayHealth::Running { pid: _, .. } => {
                        listener_verified = true;
                    }
                    crate::nginx::GatewayHealth::PortConflict {
                        occupying_pid,
                        details,
                        ..
                    } => {
                        layer_failures.push(format!(
                            "Port Çakışması / Ağ Dinleyicisi: Port {port} LocalForge dışındaki bir süreç (PID {:?}) tarafından işgal edilmiş. Detay: {details}",
                            occupying_pid
                        ));
                    }
                    crate::nginx::GatewayHealth::Stopped => {
                        layer_failures.push(format!(
                            "Ağ Dinleyicisi: Port {port} üzerinde dinleyen aktif bir Nginx ağ geçidi bulunmuyor."
                        ));
                    }
                }
            }
        }

        // Layer 4: Nginx / VHost Configuration
        let vhost_configured = {
            let conf_file = self
                .nginx_supervisor
                .conf_dir()
                .join("conf.d")
                .join(format!("{}.conf", project.id.0));
            if conf_file.exists() {
                if let Ok(content) = std::fs::read_to_string(&conf_file) {
                    content.contains(&project.local_domain)
                        && project
                            .loopback_port
                            .is_some_and(|p| content.contains(&format!("listen 127.0.0.1:{p}")))
                } else {
                    false
                }
            } else {
                false
            }
        };
        if !vhost_configured {
            layer_failures.push(
                "VirtualHost Yapılandırması: Nginx sanal ana bilgisayar yapılandırması mevcut değil veya port eşleşmiyor.".into()
            );
        }

        // Layer 5: Project Runtime Readiness
        let is_running = project.status == ProjectStatus::Running;
        let fpm_alive = self.php_supervisor.is_pool_alive(&project.id);
        let sock_path = self.php_supervisor.socket_path(&project.id);
        let socket_alive = crate::health::is_unix_socket_alive(&sock_path);

        let runtime_ready = is_running && fpm_alive && socket_alive;
        if !runtime_ready {
            if project.status == ProjectStatus::Stopped {
                layer_failures.push(
                    "Proje Durumu: Proje şu anda durdurulmuş (Stopped). Tarayıcıda açmak için önce projeyi başlatınız.".into()
                );
            } else if !fpm_alive || !socket_alive {
                layer_failures.push(
                    "Çalışma Zamanı: PHP-FPM servis havuzu veya FastCGI soketi aktif değil.".into(),
                );
            } else {
                layer_failures.push(format!(
                    "Proje Durumu: Proje çalışma zamanı hazır değil (Mevcut durum: {}).",
                    project.status
                ));
            }
        }

        // Layer 6: Project Identity and HTTP Readiness Probe
        let mut http_status = None;
        let mut http_probe_ok = false;
        if port_assigned && listener_verified && vhost_configured && runtime_ready {
            let port = project.loopback_port.unwrap();

            // Step 1: Probe the LocalForge-owned identity-aware health endpoint using the project's domain
            let identity_probe =
                probe_http_health(port, &project.local_domain, "/__localforge_health");
            let mut identity_verified = match identity_probe {
                Ok((200, ref body)) => {
                    let expected_id = format!("\"project_id\":\"{}\"", project.id.0);
                    body.contains(&expected_id) && body.contains("\"status\":\"ready\"")
                }
                _ => false,
            };

            // If local_domain probe failed, retry with localhost candidate domain
            if !identity_verified {
                let localhost_domain = format!("{}.localforge.localhost", slug);
                let retry_identity =
                    probe_http_health(port, &localhost_domain, "/__localforge_health");
                if let Ok((200, ref body)) = retry_identity {
                    let expected_id = format!("\"project_id\":\"{}\"", project.id.0);
                    if body.contains(&expected_id) && body.contains("\"status\":\"ready\"") {
                        identity_verified = true;
                    }
                }
            }

            if !identity_verified {
                layer_failures.push(format!(
                    "Proje Kimliği Doğrulanamadı: Port {port} üzerindeki uç nokta bu projeye ({}) ait doğrulanmış kimlik yanıtı vermedi.",
                    project.id.0
                ));
            } else {
                // Step 2: Project identity is proven! Now probe the application root route
                let app_probe = probe_http_health(port, &project.local_domain, "/");
                let app_probe = if app_probe.is_err() {
                    let localhost_domain = format!("{}.localforge.localhost", slug);
                    probe_http_health(port, &localhost_domain, "/")
                } else {
                    app_probe
                };

                match app_probe {
                    Ok((code, body)) => {
                        http_status = Some(code);

                        // Reject any response originating from gateway fallback or default_server
                        if body.contains("LocalForge Gateway: Host mismatch") {
                            layer_failures.push(
                                "Yönlendirme Uyuşmazlığı: Nginx ağ geçidi 'Host mismatch' (alan adı uyuşmazlığı) bildirdi.".into(),
                            );
                        } else if body.contains("LocalForge Gateway: Proje Başlatılmamış") {
                            layer_failures.push(
                                "Proje Durumu: Nginx ağ geçidi projenin durdurulmuş olduğunu bildirdi (HTTP 503).".into(),
                            );
                        } else if body.contains("LocalForge Gateway: Depolama Sürücüsü") {
                            layer_failures.push(
                                "Depolama Hatası: Proje depolama sürücüsü bağlı değil (HTTP 502)."
                                    .into(),
                            );
                        } else if (200..=399).contains(&code) {
                            // Legitimate HTTP 200 OK or 3xx redirect
                            http_probe_ok = true;
                        } else if (400..=499).contains(&code) {
                            // Legitimate application client response (e.g. 401 Unauthorized, 403 Forbidden, 404 Not Found from app routing)
                            // Accepted because identity_verified has already established that this endpoint belongs to the project!
                            http_probe_ok = true;
                        } else if code == 502 {
                            layer_failures.push(format!(
                                "HTTP Yanıtı: Nginx 502 Bad Gateway döndürdü (PHP-FPM upstream bağlantısı kurulamıyor). Detay: {body}"
                            ));
                        } else if code == 503 {
                            layer_failures.push(
                                "HTTP Yanıtı: Nginx 503 Service Unavailable döndürdü (Proje durdurulmuş veya soket yanıt vermiyor).".into(),
                            );
                        } else {
                            layer_failures.push(format!(
                                "HTTP Yanıtı: Sunucu beklenmeyen hata kodu döndürdü (HTTP {code})."
                            ));
                        }
                    }
                    Err(e) => {
                        layer_failures.push(format!("HTTP Sağlık Probu Başarısız: {e}"));
                    }
                }
            }
        }

        let is_ready = port_assigned
            && listener_verified
            && vhost_configured
            && runtime_ready
            && http_probe_ok;

        let target_url = if is_ready {
            let port = project.loopback_port.unwrap();
            if dns_resolved {
                Some(format!("http://{preferred_domain}:{port}"))
            } else {
                Some(format!("http://127.0.0.1:{port}"))
            }
        } else {
            None
        };

        let failure_reason_tr = if is_ready {
            None
        } else {
            Some(layer_failures.join(" | "))
        };

        Ok(BrowserReadinessReport {
            is_ready,
            target_url,
            dns_resolved,
            port_assigned,
            port: project.loopback_port,
            listener_verified,
            vhost_configured,
            runtime_ready,
            http_status,
            failure_reason_tr,
            layer_failures,
        })
    }

    /// Returns verified runtime health status for a project.
    pub fn get_runtime_status(&self, project_id: &ProjectId) -> Result<ProjectRuntimeStatus> {
        let store = self.metadata_store();
        let project = store
            .get_project(project_id)?
            .ok_or_else(|| RuntimeError::ProjectNotFound(project_id.0.clone()))?;

        // Check volume
        let volume_ok = self
            .volume_barrier
            .verify_volume_available(&project.path, &project.volume_uuid)
            .is_ok();

        if !volume_ok {
            let _ = store.update_project_status(&project.id, ProjectStatus::Disconnected);

            let local_url = project
                .loopback_port
                .map(|p| format!("http://127.0.0.1:{p}/"))
                .unwrap_or_else(|| format!("http://127.0.0.1:{}/", self.gateway_port));

            return Ok(ProjectRuntimeStatus {
                project_id: project.id,
                name: project.name,
                status: ProjectStatus::Disconnected,
                php_version: project.php_version,
                php_fpm_pid: None,
                socket_path: None,
                nginx_pid: None,
                gateway_port: self.gateway_port,
                loopback_port: project.loopback_port,
                local_domain: project.local_domain.clone(),
                local_url,
                message: "External storage volume unmounted or missing.".into(),
            });
        }

        let fpm_alive = self.php_supervisor.is_pool_alive(&project.id);
        let sock_path = self.php_supervisor.socket_path(&project.id);
        let sock_alive = crate::health::is_unix_socket_alive(&sock_path);
        let nginx_alive = self.nginx_supervisor.is_gateway_alive();

        let fpm_id = self.php_supervisor.get_pool_identity(&project.id);
        let nginx_id = self.nginx_supervisor.get_gateway_identity();

        let (verified_status, message) = if !fpm_alive || !sock_alive {
            if project.status == ProjectStatus::Running
                || project.status == ProjectStatus::Degraded
                || project.status == ProjectStatus::Starting
            {
                (
                    ProjectStatus::Failed,
                    "PHP-FPM process or FastCGI socket terminated unexpectedly.".to_string(),
                )
            } else {
                (ProjectStatus::Stopped, "Runtime stopped.".to_string())
            }
        } else if !nginx_alive {
            (
                ProjectStatus::Degraded,
                "PHP-FPM active, but Nginx gateway is unresponsive.".to_string(),
            )
        } else {
            // Both PHP-FPM process, socket, and Nginx are alive.
            // Probe HTTP loopback to classify actual operational health.
            let mut probe_result = probe_http_health(self.gateway_port, &project.local_domain, "/");
            if probe_result.is_err() {
                let loopback_host = format!("{}:{}", project.local_domain, self.gateway_port);
                let retry = probe_http_health(self.gateway_port, &loopback_host, "/");
                if retry.is_ok() {
                    probe_result = retry;
                }
            }
            let assessment = classify_probe_result(probe_result);
            (
                assessment.project_status(),
                assessment.message().to_string(),
            )
        };

        // Sync verified status to SQLite if it diverged
        if verified_status != project.status {
            let _ = store.update_project_status(&project.id, verified_status);
        }

        let local_url = project
            .loopback_port
            .map(|p| format!("http://127.0.0.1:{p}"))
            .unwrap_or_default();

        Ok(ProjectRuntimeStatus {
            project_id: project.id.clone(),
            name: project.name,
            status: verified_status,
            php_version: project.php_version,
            php_fpm_pid: fpm_id.map(|i| i.pid),
            socket_path: if fpm_alive && sock_alive {
                Some(sock_path)
            } else {
                None
            },
            nginx_pid: nginx_id.map(|i| i.pid),
            gateway_port: self.gateway_port,
            loopback_port: project.loopback_port,
            local_domain: project.local_domain.clone(),
            local_url,
            message,
        })
    }

    /// Queries runtime status for all registered projects.
    /// Propagates genuine synchronization failures and prevents stale persisted statuses
    /// from appearing as freshly verified by setting UNKNOWN state on probe errors.
    pub fn sync_all_projects_runtime_status(&self) -> Result<Vec<ProjectRuntimeStatus>> {
        let store = self.metadata_store();
        let projects = store.list_projects()?;
        let mut list = Vec::new();
        for p in projects {
            match self.get_runtime_status(&p.id) {
                Ok(st) => list.push(st),
                Err(e) => {
                    tracing::error!("Runtime status sync failed for project {}: {}", p.id, e);
                    let _ = store.update_project_status(&p.id, ProjectStatus::Unknown);
                    return Err(e);
                }
            }
        }
        Ok(list)
    }

    /// Exposes supervisor references when direct lower-level operations are needed.
    pub fn php_supervisor(&self) -> &PhpFpmSupervisor {
        &self.php_supervisor
    }

    pub fn nginx_supervisor(&self) -> &NginxSupervisor {
        &self.nginx_supervisor
    }

    pub fn volume_manager(&self) -> &dyn VolumeManager {
        &*self.volume_manager
    }

    pub fn process_manager(&self) -> &dyn ProcessManager {
        &*self.process_manager
    }
}
