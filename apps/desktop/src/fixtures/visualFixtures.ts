import { ProjectRecord, VolumeInfo, RuntimeOverview } from "../types";

export interface FixtureData {
  volumes: VolumeInfo[];
  projects: ProjectRecord[];
  runtime: RuntimeOverview | null;
}

/**
 * Returns visual test fixtures ONLY in development or explicit test environments.
 * CRITICAL SECURITY & DATA INTEGRITY GUARD:
 * In production builds (import.meta.env.PROD === true), strictly returns null.
 * Production URL query parameters can NEVER activate fixture data.
 */
export function getVisualFixture(
  key: string | null,
  overrideEnv?: { PROD?: boolean; DEV?: boolean }
): FixtureData | null {
  const isProd = overrideEnv !== undefined
    ? Boolean(overrideEnv.PROD)
    : (typeof import.meta !== "undefined" && import.meta.env ? Boolean(import.meta.env.PROD) : true);

  const isDev = overrideEnv !== undefined
    ? Boolean(overrideEnv.DEV)
    : (typeof import.meta !== "undefined" && import.meta.env ? Boolean(import.meta.env.DEV) : false);

  // STRICT PRODUCTION SAFETY GATE
  if (isProd || !isDev) {
    return null;
  }

  if (!key) return null;

  const mockVolumes: VolumeInfo[] = [
    {
      uuid: "7E26530E-5179-414C-81B5-6FBDB429FC65",
      name: "Mac Harici Disk",
      mount_point: "/Volumes/Mac Harici Disk",
      filesystem: "apfs",
      is_removable: true,
      is_internal: false,
      total_bytes: 1000204886016,
      available_bytes: 144955146240,
    },
    {
      uuid: "E56231C9-4E24-4C59-A129-E452B5A83704",
      name: "Macintosh HD",
      mount_point: "/",
      filesystem: "apfs",
      is_removable: false,
      is_internal: true,
      total_bytes: 494384128000,
      available_bytes: 120455146240,
    },
  ];

  if (key === "empty") {
    return {
      volumes: [mockVolumes[0]],
      projects: [],
      runtime: {
        php_versions: ["8.4"],
        nginx_port: 8080,
        nginx_running: true,
        mariadb_port: 3306,
        mariadb_socket: "/Volumes/Mac Harici Disk/VibeProject/LocalForge/.localforge/run/mariadb.sock",
        mariadb_alive: true,
        mariadb_latency_ms: 0.4,
        php_fpm_running: false,
      },
    };
  }

  if (key === "6") {
    return {
      volumes: [mockVolumes[0]],
      projects: [
        {
          id: "p1",
          name: "stepperskip",
          path: "/Volumes/Mac Harici Disk/stepperskip",
          volume_uuid: "7E26530E-5179-414C-81B5-6FBDB429FC65",
          document_root: "public",
          php_version: "8.4",
          local_domain: "stepperskip.test",
          status: "running",
          created_at: new Date().toISOString(),
          updated_at: new Date().toISOString(),
        },
        {
          id: "p2",
          name: "portal-api",
          path: "/Volumes/Mac Harici Disk/portal",
          volume_uuid: "7E26530E-5179-414C-81B5-6FBDB429FC65",
          document_root: "src/public",
          php_version: "8.4",
          local_domain: "portal.test",
          status: "running",
          created_at: new Date().toISOString(),
          updated_at: new Date().toISOString(),
        },
        {
          id: "p3",
          name: "docs-site",
          path: "/Volumes/Mac Harici Disk/docs",
          volume_uuid: "7E26530E-5179-414C-81B5-6FBDB429FC65",
          document_root: "docs",
          php_version: "8.3",
          local_domain: "docs.test",
          status: "stopped",
          created_at: new Date().toISOString(),
          updated_at: new Date().toISOString(),
        },
        {
          id: "p4",
          name: "admin-v2",
          path: "/Volumes/Mac Harici Disk/admin",
          volume_uuid: "7E26530E-5179-414C-81B5-6FBDB429FC65",
          document_root: "public",
          php_version: "8.4",
          local_domain: "admin.test",
          status: "running",
          created_at: new Date().toISOString(),
          updated_at: new Date().toISOString(),
        },
        {
          id: "p5",
          name: "billing-svc",
          path: "/Volumes/Mac Harici Disk/billing",
          volume_uuid: "7E26530E-5179-414C-81B5-6FBDB429FC65",
          document_root: "public",
          php_version: "8.3",
          local_domain: "billing.test",
          status: "stopped",
          created_at: new Date().toISOString(),
          updated_at: new Date().toISOString(),
        },
        {
          id: "p6",
          name: "analytics",
          path: "/Volumes/Mac Harici Disk/analytics",
          volume_uuid: "7E26530E-5179-414C-81B5-6FBDB429FC65",
          document_root: "web",
          php_version: "8.4",
          local_domain: "analytics.test",
          status: "running",
          created_at: new Date().toISOString(),
          updated_at: new Date().toISOString(),
        },
      ],
      runtime: {
        php_versions: ["8.4", "8.3"],
        nginx_port: 8080,
        nginx_running: true,
        mariadb_port: 3306,
        mariadb_socket: "/Volumes/Mac Harici Disk/VibeProject/LocalForge/.localforge/run/mariadb.sock",
        mariadb_alive: true,
        mariadb_latency_ms: 0.4,
        php_fpm_running: true,
      },
    };
  }

  if (key === "8" || key === "more") {
    // 8 projects fixture to verify >6 pagination navigation without hiding projects
    const base6 = getVisualFixture("6")!;
    const extraProjects: ProjectRecord[] = [
      {
        id: "p7",
        name: "shop-service",
        path: "/Volumes/Mac Harici Disk/shop",
        volume_uuid: "7E26530E-5179-414C-81B5-6FBDB429FC65",
        document_root: "public",
        php_version: "8.4",
        local_domain: "shop.test",
        status: "stopped",
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
      },
      {
        id: "p8",
        name: "mobile-backend",
        path: "/Volumes/Mac Harici Disk/mobile",
        volume_uuid: "7E26530E-5179-414C-81B5-6FBDB429FC65",
        document_root: "api",
        php_version: "8.3",
        local_domain: "mobile.test",
        status: "running",
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
      },
    ];

    return {
      volumes: base6.volumes,
      projects: [...base6.projects, ...extraProjects],
      runtime: base6.runtime,
    };
  }

  if (key === "2" || key === "baseline" || key === "default") {
    return {
      volumes: mockVolumes,
      projects: [
        {
          id: "proj_01",
          name: "stepperskip",
          path: "/Volumes/Mac Harici Disk/VibeProject/stepperskip",
          volume_uuid: "7E26530E-5179-414C-81B5-6FBDB429FC65",
          document_root: "public",
          php_version: "8.4",
          local_domain: "stepperskip.test",
          status: "running",
          created_at: new Date().toISOString(),
          updated_at: new Date().toISOString(),
        },
        {
          id: "proj_02",
          name: "client-portal",
          path: "/Users/mehmetakar/Sites/portal",
          volume_uuid: "E56231C9-4E24-4C59-A129-E452B5A83704",
          document_root: "src/public",
          php_version: "8.3",
          local_domain: "portal.test",
          status: "stopped",
          created_at: new Date().toISOString(),
          updated_at: new Date().toISOString(),
        },
      ],
      runtime: {
        php_versions: ["8.4", "8.3"],
        nginx_port: 8080,
        nginx_running: true,
        mariadb_port: 3306,
        mariadb_socket: "/tmp/mariadb.sock",
        mariadb_alive: true,
        mariadb_latency_ms: 0.38,
        php_fpm_running: true,
      },
    };
  }

  return null;
}
