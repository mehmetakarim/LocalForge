import { RuntimeOverview } from "../types";

export type ServiceStatus = "running" | "conflict" | "stopped" | "unknown";

export interface ServicesDerivedStates {
  nginxStatus: ServiceStatus;
  phpFpmStatus: ServiceStatus;
  mariadbStatus: ServiceStatus;
}

export function deriveServiceStates(
  runtime: RuntimeOverview | null
): ServicesDerivedStates {
  const nginxStatus: ServiceStatus =
    runtime == null
      ? "unknown"
      : runtime.nginx_running
      ? "running"
      : runtime.nginx_port_conflict
      ? "conflict"
      : "stopped";

  const phpFpmStatus: ServiceStatus =
    runtime == null ||
    runtime.php_fpm_running === undefined ||
    runtime.php_fpm_running === null
      ? "unknown"
      : runtime.php_fpm_running
      ? "running"
      : "stopped";

  const mariadbStatus: ServiceStatus =
    runtime == null ||
    runtime.mariadb_alive === undefined ||
    runtime.mariadb_alive === null
      ? "unknown"
      : runtime.mariadb_alive
      ? "running"
      : "stopped";

  return { nginxStatus, phpFpmStatus, mariadbStatus };
}
