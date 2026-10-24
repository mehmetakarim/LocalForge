use crate::error::{Result, RuntimeError};
use crate::health::is_port_available;
use localforge_core::{
    DomainResolutionStatus, GatewayPortStatus, ProjectId, ProjectRoutingProfile,
};
use localforge_platform::ProcessManager;
use localforge_storage::{MetadataStore, ProjectRecord};
use std::sync::Mutex;

/// Result of an attempt to allocate a persistent loopback port for a project.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PortAllocationResult {
    /// Port was already assigned to this project and verified free on OS.
    ReusedExisting { port: u16 },
    /// New port successfully allocated and persisted in SQLite.
    NewlyAllocated { port: u16 },
    /// Assigned port is currently occupied by a foreign process on the host.
    Conflict {
        port: u16,
        occupying_pid: Option<u32>,
        details: String,
    },
}

/// Deterministic, persistent project port allocation engine.
/// Enforces bounded ranges (18081..18999), thread-safe allocation,
/// database uniqueness, foreign process detection, and no killing foreign PIDs.
pub struct PortAllocator {
    range_start: u16,
    range_end: u16,
    lock: Mutex<()>,
}

impl Default for PortAllocator {
    fn default() -> Self {
        Self::new(18081, 18999)
    }
}

impl PortAllocator {
    pub fn new(range_start: u16, range_end: u16) -> Self {
        Self {
            range_start,
            range_end,
            lock: Mutex::new(()),
        }
    }

    /// Allocates or revalidates a dedicated loopback port for the given project.
    /// Thread-safe via Mutex to prevent race conditions during concurrent startup.
    pub fn allocate_port_for_project(
        &self,
        store: &MetadataStore,
        project_id: &ProjectId,
        process_manager: &dyn ProcessManager,
    ) -> Result<PortAllocationResult> {
        let _guard = self.lock.lock().unwrap();

        let project = store
            .get_project(project_id)?
            .ok_or_else(|| RuntimeError::ProjectNotFound(project_id.0.clone()))?;

        // 1. If project already has an assigned port in SQLite, check its OS status
        if let Some(port) = project.loopback_port {
            if is_port_available(port) {
                return Ok(PortAllocationResult::ReusedExisting { port });
            }

            // Port is occupied on the OS! Inspect foreign PID without terminating it
            let foreign_pid = find_pid_listening_on_port(port, process_manager);
            let details = format!(
                "Port {port} yerel makinede başka bir süreç (PID: {:?}) tarafından kullanılıyor. Güvenlik politikası gereği yabancı süreç kapatılmadı.",
                foreign_pid
            );

            return Ok(PortAllocationResult::Conflict {
                port,
                occupying_pid: foreign_pid,
                details,
            });
        }

        // 2. Scan bounded range 18081..18999 for next free port not reserved in DB and free on OS
        for port in self.range_start..=self.range_end {
            let in_db = store
                .is_port_allocated_in_db(port)
                .map_err(|e| RuntimeError::ConfigGenerationFailed(e.to_string()))?;

            if !in_db && is_port_available(port) {
                // Atomically assign port in database
                store
                    .update_project_loopback_port(project_id, Some(port))
                    .map_err(|e| RuntimeError::ConfigGenerationFailed(e.to_string()))?;

                return Ok(PortAllocationResult::NewlyAllocated { port });
            }
        }

        Err(RuntimeError::ConfigGenerationFailed(format!(
            "No free loopback ports available in bounded range {}..{}",
            self.range_start, self.range_end
        )))
    }

    /// Explicit non-destructive port conflict recovery.
    /// Finds the next free port and updates the project record without terminating the conflicting process.
    pub fn reassign_conflicting_port(
        &self,
        store: &MetadataStore,
        project_id: &ProjectId,
    ) -> Result<u16> {
        let _guard = self.lock.lock().unwrap();

        for port in self.range_start..=self.range_end {
            let in_db = store
                .is_port_allocated_in_db(port)
                .map_err(|e| RuntimeError::ConfigGenerationFailed(e.to_string()))?;

            if !in_db && is_port_available(port) {
                store
                    .update_project_loopback_port(project_id, Some(port))
                    .map_err(|e| RuntimeError::ConfigGenerationFailed(e.to_string()))?;

                return Ok(port);
            }
        }

        Err(RuntimeError::ConfigGenerationFailed(
            "Port conflict recovery failed: no alternative port available in range".into(),
        ))
    }
}

/// Helper to inspect which PID owns a local TCP port using `lsof`.
fn find_pid_listening_on_port(port: u16, _process_manager: &dyn ProcessManager) -> Option<u32> {
    use std::process::Command;

    let output = Command::new("lsof")
        .arg(format!("-iTCP:{port}"))
        .arg("-sTCP:LISTEN")
        .arg("-t")
        .output()
        .ok()?;

    if output.status.success() {
        let text = String::from_utf8_lossy(&output.stdout);
        if let Some(first_line) = text.lines().next() {
            if let Ok(pid) = first_line.trim().parse::<u32>() {
                return Some(pid);
            }
        }
    }
    None
}

/// Hybrid Domain Resolution & Routing Profile Engine.
/// Derives preferred branded domain, .localhost subdomain, and loopback fallback,
/// classifying into Scenarios A through F with Turkish user guidance.
pub struct ProjectRoutingEngine;

impl ProjectRoutingEngine {
    /// Generates canonical URL slug from project name.
    pub fn slugify(name: &str) -> String {
        name.to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '-' })
            .collect::<String>()
            .split('-')
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("-")
    }

    /// Computes full typed routing profile across Scenarios A through F.
    pub fn compute_profile(
        project: &ProjectRecord,
        allocated_port: Option<u16>,
        gateway_running: bool,
        fpm_running: bool,
        volume_mounted: bool,
        domain_resolves: bool,
        port_conflict: Option<(u32, String)>,
    ) -> ProjectRoutingProfile {
        let slug = Self::slugify(&project.name);
        let preferred_domain = format!("{slug}.localforge");
        let legacy_domain = project.local_domain.clone();

        let (localhost_domain, loopback_endpoint) = if let Some(port) = allocated_port {
            (
                format!("{slug}.localforge.localhost:{port}"),
                format!("127.0.0.1:{port}"),
            )
        } else {
            (format!("{slug}.localforge.localhost"), "".to_string())
        };

        // Scenario F: Storage volume disconnected
        if !volume_mounted {
            return ProjectRoutingProfile {
                preferred_domain,
                localhost_domain,
                loopback_endpoint,
                legacy_domain,
                allocated_port,
                resolution_status: DomainResolutionStatus::Untested,
                gateway_status: GatewayPortStatus::VolumeDisconnected,
                effective_url: "".into(),
                scenario: "Scenario F: Storage Volume Disconnected".into(),
                guidance_tr: "Harici APFS depolama sürücüsü bağlı değil. VolumeBarrier koruması devrede; dâhili diske yazma engellendi.".into(),
            };
        }

        // Scenario D: Port conflict
        if let Some((occ_pid, details)) = port_conflict {
            return ProjectRoutingProfile {
                preferred_domain,
                localhost_domain,
                loopback_endpoint,
                legacy_domain,
                allocated_port,
                resolution_status: DomainResolutionStatus::Untested,
                gateway_status: match allocated_port {
                    Some(port) => GatewayPortStatus::PortConflict {
                        port,
                        occupying_pid: Some(occ_pid),
                        details,
                    },
                    None => GatewayPortStatus::Unassigned,
                },
                effective_url: "".into(),
                scenario: "Scenario D: Port Conflict".into(),
                guidance_tr: format!("Atanan port ({:?}) yabancı süreç (PID: {occ_pid}) tarafından kullanılıyor. Port çakışması ekranından yeni port tahsis edilebilir.", allocated_port),
            };
        }

        // Scenario E: PHP-FPM unavailable
        if !fpm_running && gateway_running {
            return ProjectRoutingProfile {
                preferred_domain,
                localhost_domain,
                loopback_endpoint,
                legacy_domain,
                allocated_port,
                resolution_status: if domain_resolves { DomainResolutionStatus::Resolved } else { DomainResolutionStatus::NxDomain },
                gateway_status: match allocated_port {
                    Some(port) => GatewayPortStatus::Stopped { port },
                    None => GatewayPortStatus::Unassigned,
                },
                effective_url: "".into(),
                scenario: "Scenario E: PHP-FPM Runtime Unavailable".into(),
                guidance_tr: format!("PHP-FPM çalışma zamanı (PHP {}) aktif değil veya ikili bulunamadı. Proje başlatılamadı.", project.php_version),
            };
        }

        // If stopped
        if !gateway_running || project.status != localforge_core::ProjectStatus::Running {
            return ProjectRoutingProfile {
                preferred_domain,
                localhost_domain,
                loopback_endpoint,
                legacy_domain,
                allocated_port,
                resolution_status: if domain_resolves {
                    DomainResolutionStatus::Resolved
                } else {
                    DomainResolutionStatus::NxDomain
                },
                gateway_status: match allocated_port {
                    Some(port) => GatewayPortStatus::Stopped { port },
                    None => GatewayPortStatus::Unassigned,
                },
                effective_url: "".into(),
                scenario: if allocated_port.is_none() {
                    "Scenario: Loopback Port Unassigned".into()
                } else {
                    "Project Stopped".into()
                },
                guidance_tr: if allocated_port.is_none() {
                    "Bu projeye henüz ayrık bir loopback portu tahsis edilmemiştir. Port tahsis etmek için projeyi başlatınız.".into()
                } else {
                    "Proje durdurulmuş durumda. Tarayıcıda açmak için Başlat butonuna tıklayınız."
                        .into()
                },
            };
        }

        // Scenario: Unassigned port on running project
        let Some(port) = allocated_port else {
            return ProjectRoutingProfile {
                preferred_domain: preferred_domain.clone(),
                localhost_domain,
                loopback_endpoint: "".into(),
                legacy_domain,
                allocated_port: None,
                resolution_status: if domain_resolves { DomainResolutionStatus::Resolved } else { DomainResolutionStatus::NxDomain },
                gateway_status: GatewayPortStatus::Unassigned,
                effective_url: "".into(),
                scenario: "Scenario: Loopback Port Unassigned".into(),
                guidance_tr: "Bu projeye henüz ayrık bir loopback portu tahsis edilmemiştir. Port tahsis etmek için projeyi durdurup yeniden başlatınız.".into(),
            };
        };

        // Scenario A: Branded Local Domain Resolved
        if domain_resolves {
            return ProjectRoutingProfile {
                preferred_domain: preferred_domain.clone(),
                localhost_domain,
                loopback_endpoint,
                legacy_domain,
                allocated_port: Some(port),
                resolution_status: DomainResolutionStatus::Resolved,
                gateway_status: GatewayPortStatus::Active { port },
                effective_url: format!("http://{preferred_domain}"),
                scenario: "Scenario A: Branded Local Domain Resolved".into(),
                guidance_tr: format!("http://{preferred_domain} yerel adı başarıyla çözümlendi ve ağ geçidine yönlendirildi."),
            };
        }

        // Scenario B: DNS-Independent Loopback Fallback (Standard default when no /etc/hosts exists)
        ProjectRoutingProfile {
            preferred_domain: preferred_domain.clone(),
            localhost_domain,
            loopback_endpoint: format!("127.0.0.1:{port}"),
            legacy_domain,
            allocated_port: Some(port),
            resolution_status: DomainResolutionStatus::NxDomain,
            gateway_status: GatewayPortStatus::Active { port },
            effective_url: format!("http://127.0.0.1:{port}/"),
            scenario: "Scenario B: DNS-Independent Loopback Fallback".into(),
            guidance_tr: format!("Yerel {preferred_domain} adı için DNS yapılandırılmamış (NXDOMAIN). Projeniz bağımsız loopback portu (http://127.0.0.1:{port}/) üzerinden açılacaktır."),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use localforge_core::{ProjectId, ProjectStatus, VolumeId};
    use localforge_platform::MacOSProcessManager;
    use std::path::PathBuf;

    #[test]
    fn test_slugify() {
        assert_eq!(ProjectRoutingEngine::slugify("Projetest"), "projetest");
        assert_eq!(
            ProjectRoutingEngine::slugify("My Cool Project!"),
            "my-cool-project"
        );
        assert_eq!(
            ProjectRoutingEngine::slugify("Site_Alpha.2"),
            "site-alpha-2"
        );
    }

    #[test]
    fn test_port_allocator_deterministic_and_unique() {
        let store = MetadataStore::open_in_memory().unwrap();
        store.migrate_to_v2_loopback_ports().unwrap();

        let proj1 = ProjectRecord {
            id: ProjectId::from_string("proj_alloc_1"),
            name: "Project One".into(),
            path: PathBuf::from("/path/p1"),
            volume_uuid: VolumeId::new("V1"),
            document_root: "".into(),
            php_version: "8.5".into(),
            local_domain: "p1.test".into(),
            status: ProjectStatus::Stopped,
            created_at: "2026-10-22T00:00:00Z".into(),
            updated_at: "2026-10-22T00:00:00Z".into(),
            loopback_port: None,
        };
        store.insert_project(&proj1).unwrap();

        let proj2 = ProjectRecord {
            id: ProjectId::from_string("proj_alloc_2"),
            name: "Project Two".into(),
            path: PathBuf::from("/path/p2"),
            volume_uuid: VolumeId::new("V2"),
            document_root: "".into(),
            php_version: "8.4".into(),
            local_domain: "p2.test".into(),
            status: ProjectStatus::Stopped,
            created_at: "2026-10-22T00:00:00Z".into(),
            updated_at: "2026-10-22T00:00:00Z".into(),
            loopback_port: None,
        };
        store.insert_project(&proj2).unwrap();

        let pm = MacOSProcessManager::new();
        let allocator = PortAllocator::new(18081, 18090);

        // Allocate for project 1
        let res1 = allocator
            .allocate_port_for_project(&store, &proj1.id, &pm)
            .unwrap();
        let port1 = match res1 {
            PortAllocationResult::NewlyAllocated { port } => port,
            _ => panic!("Expected newly allocated port"),
        };
        assert!(port1 >= 18081);

        // Allocate for project 2
        let res2 = allocator
            .allocate_port_for_project(&store, &proj2.id, &pm)
            .unwrap();
        let port2 = match res2 {
            PortAllocationResult::NewlyAllocated { port } => port,
            _ => panic!("Expected newly allocated port"),
        };
        assert!(port2 >= 18081);
        assert_ne!(port1, port2);

        // Reallocate for project 1 (stable persistence across restarts)
        let res1_repeat = allocator
            .allocate_port_for_project(&store, &proj1.id, &pm)
            .unwrap();
        match res1_repeat {
            PortAllocationResult::ReusedExisting { port } => assert_eq!(port, port1),
            _ => panic!("Expected reused existing port"),
        }
    }

    #[test]
    fn test_routing_scenarios() {
        let proj = ProjectRecord {
            id: ProjectId::from_string("proj_scen"),
            name: "Projetest".into(),
            path: PathBuf::from("/path"),
            volume_uuid: VolumeId::new("V"),
            document_root: "".into(),
            php_version: "8.4".into(),
            local_domain: "projetest.test".into(),
            status: ProjectStatus::Running,
            created_at: "2026-10-22T00:00:00Z".into(),
            updated_at: "2026-10-22T00:00:00Z".into(),
            loopback_port: Some(18082),
        };

        // Scenario A: Branded Local Domain Resolved
        let profile_a =
            ProjectRoutingEngine::compute_profile(&proj, Some(18082), true, true, true, true, None);
        assert_eq!(profile_a.effective_url, "http://projetest.localforge");
        assert_eq!(
            profile_a.resolution_status,
            DomainResolutionStatus::Resolved
        );
        assert!(profile_a.scenario.contains("Scenario A"));

        // Scenario B: DNS-Independent Loopback Fallback (NXDOMAIN)
        let profile_b = ProjectRoutingEngine::compute_profile(
            &proj,
            Some(18082),
            true,
            true,
            true,
            false,
            None,
        );
        assert_eq!(profile_b.effective_url, "http://127.0.0.1:18082/");
        assert_eq!(
            profile_b.resolution_status,
            DomainResolutionStatus::NxDomain
        );
        assert!(profile_b.scenario.contains("Scenario B"));

        // Scenario D: Port conflict
        let profile_d = ProjectRoutingEngine::compute_profile(
            &proj,
            Some(18082),
            true,
            true,
            true,
            false,
            Some((58491, "Conflict".into())),
        );
        assert_eq!(profile_d.effective_url, "");
        assert!(profile_d.scenario.contains("Scenario D"));

        // Scenario E: PHP-FPM unavailable
        let profile_e = ProjectRoutingEngine::compute_profile(
            &proj,
            Some(18082),
            true,
            false,
            true,
            false,
            None,
        );
        assert_eq!(profile_e.effective_url, "");
        assert!(profile_e.scenario.contains("Scenario E"));

        // Scenario F: Storage disconnected
        let profile_f = ProjectRoutingEngine::compute_profile(
            &proj,
            Some(18082),
            true,
            true,
            false,
            false,
            None,
        );
        assert_eq!(profile_f.effective_url, "");
        assert!(profile_f.scenario.contains("Scenario F"));

        // Scenario Unassigned: No loopback port allocated
        let profile_unassigned =
            ProjectRoutingEngine::compute_profile(&proj, None, true, true, true, false, None);
        assert_eq!(profile_unassigned.effective_url, "");
        assert_eq!(profile_unassigned.loopback_endpoint, "");
        assert_eq!(profile_unassigned.allocated_port, None);
        assert_eq!(
            profile_unassigned.gateway_status,
            GatewayPortStatus::Unassigned
        );
        assert!(profile_unassigned.scenario.contains("Unassigned"));
    }
}
