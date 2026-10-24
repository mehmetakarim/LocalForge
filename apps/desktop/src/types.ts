export type ProjectStatus = "running" | "stopped" | "starting" | "stopping" | "disconnected" | "degraded" | "failed" | "unknown";

export interface ProjectRecord {
  id: string;
  name: string;
  path: string;
  volume_uuid: string;
  document_root: string;
  php_version: string;
  local_domain: string;
  loopback_port?: number;
  status: ProjectStatus;
  created_at: string;
  updated_at: string;
}

export type DomainResolutionStatus = "Resolved" | "NxDomain" | "PortConflict" | "GatewayDown";
export type GatewayPortStatus = "Available" | "OccupiedByUs" | "OccupiedByOther";

export interface ProjectRoutingProfile {
  preferred_domain: string;
  dns_independent_domain: string;
  loopback_endpoint: string;
  legacy_domain: string;
  preferred_url: string;
  dns_independent_url: string;
  loopback_url: string;
  legacy_url: string;
  allocated_port?: number;
  domain_resolution: DomainResolutionStatus;
  gateway_port_status: GatewayPortStatus;
  is_accessible: boolean;
  recommended_url: string;
  guidance_tr: string;
}

export interface VolumeInfo {
  uuid: string;
  name: string;
  mount_point: string;
  filesystem: string;
  is_removable: boolean;
  is_internal: boolean;
  total_bytes: number;
  available_bytes: number;
}

export interface PhpVersionDetail {
  version: string;
  major_minor: string;
  binary_path: string;
  is_fpm_verified: boolean;
  architecture: string;
  is_supported: boolean;
}

export interface RuntimeOverview {
  php_versions: string[];
  php_details?: PhpVersionDetail[];
  nginx_port: number;
  nginx_running: boolean;
  nginx_port_conflict?: boolean;
  mariadb_port: number;
  mariadb_socket: string;
  mariadb_alive: boolean;
  mariadb_version?: string;
  mariadb_latency_ms?: number;
  mariadb_datadir_initialized?: boolean;
  php_fpm_running?: boolean;
}

export type ViewMode = "studio" | "projects" | "services" | "storage" | "deployments" | "mcp" | "settings";

export interface BrowserReadinessReport {
  is_ready: boolean;
  target_url: string | null;
  dns_resolved: boolean;
  port_assigned: boolean;
  port: number | null;
  listener_verified: boolean;
  vhost_configured: boolean;
  runtime_ready: boolean;
  http_status: number | null;
  failure_reason_tr: string | null;
  layer_failures: string[];
}
