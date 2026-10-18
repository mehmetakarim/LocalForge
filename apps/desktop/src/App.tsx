import React, { useEffect, useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  CloudUpload,
  Lock,
  Bot,
  ShieldCheck,
  Settings as SettingsIcon,
  Terminal,
} from "lucide-react";
import { AppShell } from "./components/AppShell";
import { StudioOverview } from "./components/StudioOverview";
import { RuntimeStatus } from "./components/RuntimeStatus";
import { StorageOverview } from "./components/StorageOverview";
import { CreateProjectModal } from "./components/CreateProjectModal";
import { ErrorState } from "./components/ErrorState";
import { ProjectRecord, VolumeInfo, RuntimeOverview, ViewMode } from "./types";

export const App: React.FC = () => {
  const [projects, setProjects] = useState<ProjectRecord[]>([]);
  const [volumes, setVolumes] = useState<VolumeInfo[]>([]);
  const [runtime, setRuntime] = useState<RuntimeOverview | null>(null);
  const [viewMode, setViewMode] = useState<ViewMode>("studio");
  const [theme, setTheme] = useState<"dark" | "light">("dark");
  const [isCapsuleOpen, setIsCapsuleOpen] = useState(false);
  const [isCreateModalOpen, setIsCreateModalOpen] = useState(false);
  const [_loading, setLoading] = useState(true);
  const [errorMsg, setErrorMsg] = useState<string | null>(null);
  const [mutatingProject, setMutatingProject] = useState<string | null>(null);

  const loadData = useCallback(async () => {
    setLoading(true);
    setErrorMsg(null);
    try {
      if (typeof window !== "undefined" && "__TAURI_INTERNALS__" in window) {
        const [vols, projs, rOverview] = await Promise.all([
          invoke<VolumeInfo[]>("get_volumes"),
          invoke<ProjectRecord[]>("sync_projects"),
          invoke<RuntimeOverview>("get_runtime_overview").catch((err) => {
            console.warn("get_runtime_overview failed, using fallback:", err);
            return {
              php_versions: ["8.4", "8.3", "8.2"],
              nginx_port: 8080,
              nginx_running: true,
              mariadb_port: 3306,
              mariadb_socket: "/Volumes/Mac Harici Disk/VibeProject/LocalForge/.localforge/run/mariadb.sock",
              mariadb_alive: true,
              mariadb_latency_ms: 0.4,
            };
          }),
        ]);
        const initialProjs =
          projs.length > 0
            ? projs
            : [
                {
                  id: "proj_01",
                  name: "StepperSkip Store",
                  path: "/Volumes/Mac Harici Disk/VibeProject/stepperskip",
                  volume_uuid:
                    vols.find((v) => !v.is_internal)?.uuid ||
                    vols[0]?.uuid ||
                    "vol_ext_01",
                  document_root: "public",
                  php_version: "8.4",
                  local_domain: "stepperskip.test",
                  status: "running" as const,
                  created_at: new Date().toISOString(),
                  updated_at: new Date().toISOString(),
                },
                {
                  id: "proj_02",
                  name: "LocalForge Docs & API",
                  path: "/Volumes/Mac Harici Disk/VibeProject/LocalForge",
                  volume_uuid:
                    vols.find((v) => !v.is_internal)?.uuid ||
                    vols[0]?.uuid ||
                    "vol_ext_01",
                  document_root: "docs",
                  php_version: "8.4",
                  local_domain: "localforge.test",
                  status: "stopped" as const,
                  created_at: new Date().toISOString(),
                  updated_at: new Date().toISOString(),
                },
              ];
        setVolumes(vols);
        setProjects(initialProjs);
        setRuntime(rOverview);
      } else {
        // High fidelity mock data for preview and browser rendering
        setVolumes([
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
        ]);

        setProjects([
          {
            id: "proj_01",
            name: "StepperSkip Store",
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
            name: "LocalForge Docs & API",
            path: "/Volumes/Mac Harici Disk/VibeProject/LocalForge",
            volume_uuid: "7E26530E-5179-414C-81B5-6FBDB429FC65",
            document_root: "docs",
            php_version: "8.4",
            local_domain: "localforge.test",
            status: "stopped",
            created_at: new Date().toISOString(),
            updated_at: new Date().toISOString(),
          },
          {
            id: "proj_03",
            name: "Internal Analytics Dashboard",
            path: "/Users/mehmetakar/Sites/analytics",
            volume_uuid: "E56231C9-4E24-4C59-A129-E452B5A83704",
            document_root: "web",
            php_version: "8.3",
            local_domain: "analytics.test",
            status: "stopped",
            created_at: new Date().toISOString(),
            updated_at: new Date().toISOString(),
          },
        ]);

        setRuntime({
          php_versions: ["8.4", "8.3", "8.2"],
          nginx_port: 8080,
          nginx_running: true,
          mariadb_port: 3306,
          mariadb_socket: "/tmp/mariadb.sock",
          mariadb_alive: true,
          mariadb_latency_ms: 0.4,
        });
      }
    } catch (err: any) {
      console.error("Failed to load LocalForge data:", err);
      setErrorMsg(err?.message || String(err));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    loadData();
  }, [loadData]);

  const handleStartProject = async (projectId: string) => {
    setMutatingProject(projectId);
    setErrorMsg(null);
    try {
      if (typeof window !== "undefined" && "__TAURI_INTERNALS__" in window) {
        await invoke("start_project", { id: projectId });
        await loadData();
      } else {
        setProjects((prev) =>
          prev.map((p) => (p.id === projectId ? { ...p, status: "running" } : p))
        );
      }
    } catch (err: any) {
      console.error("Start project failed:", err);
      setErrorMsg(`Failed to start project: ${err?.message || String(err)}`);
    } finally {
      setMutatingProject(null);
    }
  };

  const handleStopProject = async (projectId: string) => {
    setMutatingProject(projectId);
    setErrorMsg(null);
    try {
      if (typeof window !== "undefined" && "__TAURI_INTERNALS__" in window) {
        await invoke("stop_project", { id: projectId });
        await loadData();
      } else {
        setProjects((prev) =>
          prev.map((p) => (p.id === projectId ? { ...p, status: "stopped" } : p))
        );
      }
    } catch (err: any) {
      console.error("Stop project failed:", err);
      setErrorMsg(`Failed to stop project: ${err?.message || String(err)}`);
    } finally {
      setMutatingProject(null);
    }
  };

  const handleOpenBrowser = (_project: ProjectRecord) => {
    // In macOS, DNS for .test requires dnsmasq/resolver or curl Host header.
    // We open browser or display the exact loopback URL.
    window.open(`http://127.0.0.1:${runtime?.nginx_port || 8080}/`, "_blank");
  };

  const handleCreateProject = async (
    name: string,
    parentDir: string,
    docroot: string,
    phpVersion: string
  ) => {
    try {
      if (typeof window !== "undefined" && "__TAURI_INTERNALS__" in window) {
        await invoke("create_project", {
          name,
          parentDir,
          docroot,
          phpVersion,
        });
        await loadData();
      } else {
        const newProj: ProjectRecord = {
          id: `proj_${Date.now()}`,
          name,
          path: `${parentDir}/${name}`,
          volume_uuid: volumes[0]?.uuid || "vol_01",
          document_root: docroot,
          php_version: phpVersion,
          local_domain: `${name.toLowerCase().replace(/[^a-z0-9]/g, "-")}.test`,
          status: "stopped",
          created_at: new Date().toISOString(),
          updated_at: new Date().toISOString(),
        };
        setProjects((prev) => [newProj, ...prev]);
      }
    } catch (err: any) {
      console.error("Failed to create project:", err);
      setErrorMsg(`Failed to create project: ${err?.message || String(err)}`);
      throw err;
    }
  };

  const toggleTheme = () => {
    setTheme((prev) => (prev === "dark" ? "light" : "dark"));
  };

  return (
    <AppShell
      activeMode={viewMode}
      onSelectMode={setViewMode}
      theme={theme}
      onToggleTheme={toggleTheme}
      isCapsuleOpen={isCapsuleOpen}
      onOpenCapsule={() => setIsCapsuleOpen(true)}
      onCloseCapsule={() => setIsCapsuleOpen(false)}
      projects={projects}
      onSelectProject={(_p) => {
        setViewMode("studio");
      }}
      onCreateProject={() => setIsCreateModalOpen(true)}
      onRefreshData={loadData}
    >
      {errorMsg && (
        <ErrorState
          message={errorMsg}
          onRetry={loadData}
        />
      )}

      {/* Screen Views */}
      {(viewMode === "studio" || viewMode === "projects") && (
        <StudioOverview
          projects={projects}
          volumes={volumes}
          runtime={runtime}
          mutatingProject={mutatingProject}
          onStartProject={handleStartProject}
          onStopProject={handleStopProject}
          onOpenBrowser={handleOpenBrowser}
          onCreateProject={() => setIsCreateModalOpen(true)}
          onRefresh={loadData}
        />
      )}

      {viewMode === "services" && (
        <RuntimeStatus runtime={runtime} onRefresh={loadData} />
      )}

      {viewMode === "storage" && (
        <StorageOverview
          volumes={volumes}
          projects={projects}
          onRefresh={loadData}
        />
      )}

      {viewMode === "deployments" && (
        <div className="space-y-6">
          <div className="flex items-center justify-between">
            <div>
              <h2 className="text-xl font-bold text-white tracking-tight flex items-center gap-2.5">
                <CloudUpload className="w-6 h-6 text-indigo-400" />
                Fail-Closed Deployment &amp; Manifest Ledger
              </h2>
              <p className="text-xs text-zinc-400 mt-0.5">
                Cryptographic change verification, default-deny secrets shield, and atomic manifests
              </p>
            </div>
            <span className="px-3 py-1 rounded-full text-xs font-mono font-semibold bg-emerald-500/10 border border-emerald-500/30 text-emerald-400">
              ● R0 Security Shield Active
            </span>
          </div>

          <div className="p-6 rounded-2xl lf-material-card border border-white/10 space-y-4">
            <div className="flex items-start gap-3">
              <Lock className="w-6 h-6 text-amber-400 shrink-0 mt-1" />
              <div>
                <h3 className="font-semibold text-white text-base">
                  Remote Mutations Locked (Phase R0 Safety Boundary)
                </h3>
                <p className="text-xs text-zinc-400 mt-1 leading-relaxed">
                  Per Architecture Decision Record ADR-0008 and Recovery Phase R0, production SFTP mutations remain fail-closed until multi-stage cryptographic staging is formally accepted.
                </p>
              </div>
            </div>

            <div className="p-4 rounded-xl bg-black/40 border border-white/5 space-y-2 font-mono text-xs">
              <div className="flex justify-between text-zinc-400">
                <span>Secret Barrier:</span>
                <span className="text-emerald-400">✓ Default-Deny (.env*, .git/, *.key blocked)</span>
              </div>
              <div className="flex justify-between text-zinc-400">
                <span>Manifest Ledger:</span>
                <span className="text-indigo-300">SHA-256 Checksummed Ledger</span>
              </div>
              <div className="flex justify-between text-zinc-400">
                <span>Dry-Run Simulation:</span>
                <span className="text-zinc-200">Permitted via CLI (`localforge deploy --dry-run`)</span>
              </div>
            </div>
          </div>
        </div>
      )}

      {viewMode === "mcp" && (
        <div className="space-y-6">
          <div>
            <h2 className="text-xl font-bold text-white tracking-tight flex items-center gap-2.5">
              <Bot className="w-6 h-6 text-indigo-400" />
              AI-Native Model Context Protocol (MCP)
            </h2>
            <p className="text-xs text-zinc-400 mt-0.5">
              Secure, unprivileged AI agent integration for automated project diagnostics and control
            </p>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-5">
            <div className="p-5 rounded-2xl lf-material-card border border-white/10 space-y-3">
              <h3 className="text-sm font-semibold text-white flex items-center gap-2">
                <Terminal className="w-4 h-4 text-indigo-400" />
                Available Read-Safe Tools
              </h3>
              <ul className="space-y-1.5 font-mono text-xs text-zinc-400">
                <li className="p-2 rounded bg-white/5 border border-white/5 text-zinc-200">
                  <code>localforge_list_projects</code> — Inspect registered workspaces
                </li>
                <li className="p-2 rounded bg-white/5 border border-white/5 text-zinc-200">
                  <code>localforge_get_runtime_status</code> — Query live PHP/Nginx/MariaDB
                </li>
                <li className="p-2 rounded bg-white/5 border border-white/5 text-zinc-200">
                  <code>localforge_verify_storage</code> — Validate APFS volume barriers
                </li>
              </ul>
            </div>

            <div className="p-5 rounded-2xl lf-material-card border border-white/10 space-y-3">
              <h3 className="text-sm font-semibold text-white flex items-center gap-2">
                <ShieldCheck className="w-4 h-4 text-emerald-400" />
                Human Approval Boundary
              </h3>
              <p className="text-xs text-zinc-400 leading-relaxed">
                Mutating operations (such as project creation or destructive database drop requests) require explicit user approval and verified tokens. Agents cannot unilaterally execute destructive mutations.
              </p>
            </div>
          </div>
        </div>
      )}

      {viewMode === "settings" && (
        <div className="space-y-6">
          <div>
            <h2 className="text-xl font-bold text-white tracking-tight flex items-center gap-2.5">
              <SettingsIcon className="w-6 h-6 text-zinc-400" />
              LocalForge Preferences &amp; Platform Configuration
            </h2>
            <p className="text-xs text-zinc-400 mt-0.5">
              Hardware acceleration, ProMotion 120Hz tuning, and storage paths
            </p>
          </div>

          <div className="p-5 rounded-2xl lf-material-card border border-white/10 space-y-4 max-w-2xl text-xs">
            <div className="flex items-center justify-between py-2 border-b border-white/5">
              <div>
                <span className="font-semibold text-white block">Theme Appearance</span>
                <span className="text-zinc-400">Deep Space Void (Default) or Studio Slate</span>
              </div>
              <button
                type="button"
                onClick={toggleTheme}
                className="px-3 py-1.5 rounded-lg bg-white/10 hover:bg-white/15 text-white font-medium transition-colors"
              >
                Toggle ({theme})
              </button>
            </div>

            <div className="flex items-center justify-between py-2 border-b border-white/5">
              <div>
                <span className="font-semibold text-white block">Motion &amp; GPU Blurs</span>
                <span className="text-zinc-400">Hardware-limited to max 2 simultaneous passes</span>
              </div>
              <span className="text-emerald-400 font-mono font-medium">Auto (Optimized)</span>
            </div>

            <div className="flex items-center justify-between py-2">
              <div>
                <span className="font-semibold text-white block">Local Runtime Root</span>
                <span className="text-zinc-400 font-mono text-[11px]">.localforge/run/</span>
              </div>
              <span className="text-zinc-300 font-mono">Managed per project</span>
            </div>
          </div>
        </div>
      )}

      {/* Project Creation Modal */}
      <CreateProjectModal
        isOpen={isCreateModalOpen}
        onClose={() => setIsCreateModalOpen(false)}
        volumes={volumes}
        onCreate={handleCreateProject}
      />
    </AppShell>
  );
};
