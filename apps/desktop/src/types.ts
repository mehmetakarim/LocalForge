export type ProjectStatus = "running" | "stopped" | "disconnected" | "degraded";

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
