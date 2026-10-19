export type ProjectStatus = "running" | "stopped" | "starting" | "stopping" | "disconnected" | "degraded" | "failed" | "unknown";

export interface ProjectRecord {
  id: string;
  name: string;
  path: string;
  volume_uuid: string;
  document_root: string;
  php_version: string;
  local_domain: string;
  status: ProjectStatus;
  created_at: string;
  updated_at: string;
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

export interface RuntimeOverview {
  php_versions: string[];
  nginx_port: number;
  nginx_running: boolean;
  nginx_port_conflict?: boolean;
  mariadb_port: number;
  mariadb_socket: string;
  mariadb_alive: boolean;
  mariadb_version?: string;
  mariadb_latency_ms?: number;
  php_fpm_running?: boolean;
}

export type ViewMode = "studio" | "projects" | "services" | "storage" | "deployments" | "mcp" | "settings";
