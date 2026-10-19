import React, { useEffect, useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  CloudUpload,
  Lock,
  Bot,
  ShieldCheck,
  Settings as SettingsIcon,
} from "lucide-react";
import { AppShell } from "./components/AppShell";
import { StudioOverview } from "./components/StudioOverview";
import { RuntimeStatus } from "./components/RuntimeStatus";
import { StorageOverview } from "./components/StorageOverview";
import { CreateProjectModal } from "./components/CreateProjectModal";
import { ErrorState } from "./components/ErrorState";
import { ProjectRecord, VolumeInfo, RuntimeOverview, ViewMode } from "./types";
import { getVisualFixture } from "./fixtures/visualFixtures";

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
  const [zoomLevel, setZoomLevel] = useState<number>(100);

  const loadData = useCallback(async () => {
    setLoading(true);
    setErrorMsg(null);
    try {
      const isTauri = typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
      const isDev = Boolean(import.meta.env.DEV);

      // STRICT PRODUCTION GUARD:
      // In production builds, query parameters are NEVER evaluated for fixtures.
      // Fixtures can only be requested in isolated development/testing mode.
      const fixtureParam = isDev
        ? new URLSearchParams(window.location.search).get("fixture")
        : null;

      const activeFixture = isDev && fixtureParam ? getVisualFixture(fixtureParam) : null;

      if (activeFixture) {
        setVolumes(activeFixture.volumes);
        setProjects(activeFixture.projects);
        setRuntime(activeFixture.runtime);
        return;
      }

      if (isTauri) {
        // STRICT PRODUCTION PATH — Zero Fabricated Data
        const [vols, projs, rOverview] = await Promise.all([
          invoke<VolumeInfo[]>("get_volumes"),
          invoke<ProjectRecord[]>("sync_projects"),
          invoke<RuntimeOverview>("get_runtime_overview").catch((err) => {
            console.warn("get_runtime_overview failed:", err);
            return null;
          }),
        ]);

        setVolumes(vols);
        setProjects(projs); // When 0 records, strictly renders genuine Empty State
        setRuntime(rOverview);
      } else {
        // Running outside Tauri in production or without fixture:
        // STRICT FAIL-CLOSED: Never invent fake projects.
        setVolumes([]);
        setProjects([]);
        setRuntime(null);
        if (!isDev) {
          setErrorMsg("LocalForge masaüstü ortamında (Tauri) çalıştırılmalıdır.");
        }
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
      setIsCreateModalOpen(false);
    } catch (err: any) {
      console.error("Create project failed:", err);
      setErrorMsg(`Failed to create project: ${err?.message || String(err)}`);
    }
  };

  const handleToggleTheme = () => {
    const nextTheme = theme === "dark" ? "light" : "dark";
    setTheme(nextTheme);
    document.documentElement.setAttribute("data-theme", nextTheme);
  };

  return (
    <AppShell
      activeMode={viewMode}
      onSelectMode={setViewMode}
      theme={theme}
      onToggleTheme={handleToggleTheme}
      isCapsuleOpen={isCapsuleOpen}
      onOpenCapsule={() => setIsCapsuleOpen(true)}
      onCloseCapsule={() => setIsCapsuleOpen(false)}
      projects={projects}
      onSelectProject={(_p) => {}}
      onCreateProject={() => setIsCreateModalOpen(true)}
      onRefreshData={loadData}
      zoomLevel={zoomLevel}
      onZoomIn={() => setZoomLevel((z) => Math.min(125, z + 5))}
      onZoomOut={() => setZoomLevel((z) => Math.max(75, z - 5))}
      onZoomReset={() => setZoomLevel(100)}
    >
      {errorMsg && (
        <div className="w-full max-w-xl mx-auto mb-4">
          <ErrorState message={errorMsg} onRetry={() => setErrorMsg(null)} />
        </div>
      )}

      {/* Main Mode Views */}
      <div
        className="w-full flex-1 flex flex-col items-center justify-center transition-transform duration-200"
        style={{ transform: `scale(${zoomLevel / 100})`, transformOrigin: "center top" }}
      >
        {viewMode === "studio" && (
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

        {viewMode === "projects" && (
          <div className="w-full max-w-5xl py-8">
            <h2 className="text-2xl font-bold text-white mb-6">Tüm Projeler</h2>
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
          </div>
        )}

        {viewMode === "services" && (
          <div className="w-full max-w-5xl py-8">
            <RuntimeStatus runtime={runtime} onRefresh={loadData} />
          </div>
        )}

        {viewMode === "storage" && (
          <div className="w-full max-w-5xl py-8">
            <StorageOverview
              volumes={volumes}
              projects={projects}
              onRefresh={loadData}
            />
          </div>
        )}

        {viewMode === "deployments" && (
          <div className="w-full max-w-3xl py-16 text-center space-y-4">
            <div className="w-16 h-16 rounded-3xl bg-indigo-500/10 border border-indigo-400/30 text-indigo-400 flex items-center justify-center mx-auto">
              <CloudUpload className="w-8 h-8" />
            </div>
            <h2 className="text-xl font-bold text-white">Akıllı Güvenli Dağıtım</h2>
            <p className="text-xs text-zinc-400 max-w-md mx-auto">
              Git diff değil; SHA-256 sağlama defteri tabanlı güvenli dağıtım. Üretim dağıtımları onay kapısıyla korunur.
            </p>
            <div className="inline-flex items-center gap-2 px-3 py-1.5 rounded-full bg-white/5 border border-white/10 text-xs text-zinc-400">
              <Lock className="w-3.5 h-3.5 text-amber-400" />
              <span>SFTP Dağıtım Kapısı Devrede (Salt-Okunur Önizleme)</span>
            </div>
          </div>
        )}

        {viewMode === "mcp" && (
          <div className="w-full max-w-3xl py-16 text-center space-y-4">
            <div className="w-16 h-16 rounded-3xl bg-purple-500/10 border border-purple-400/30 text-purple-400 flex items-center justify-center mx-auto">
              <Bot className="w-8 h-8" />
            </div>
            <h2 className="text-xl font-bold text-white">AI / Model Context Protocol</h2>
            <p className="text-xs text-zinc-400 max-w-md mx-auto">
              Claude Code, Gemini ve Cursor için yerel MCP sunucusu. Salt okunur tanılama ve insan onaylı işlem güvenliği.
            </p>
            <div className="inline-flex items-center gap-2 px-3 py-1.5 rounded-full bg-emerald-500/10 border border-emerald-500/30 text-xs text-emerald-400 font-mono">
              <ShieldCheck className="w-3.5 h-3.5" />
              <span>localforge-mcp-server: Dinleniyor (stdio)</span>
            </div>
          </div>
        )}

        {viewMode === "settings" && (
          <div className="w-full max-w-3xl py-16 text-center space-y-4">
            <div className="w-16 h-16 rounded-3xl bg-white/5 border border-white/10 text-zinc-300 flex items-center justify-center mx-auto">
              <SettingsIcon className="w-8 h-8" />
            </div>
            <h2 className="text-xl font-bold text-white">Sistem Tercihleri</h2>
            <p className="text-xs text-zinc-400 max-w-md mx-auto">
              Port yapılandırmaları, APFS bağlama güvenlik sınırları ve loopback proxy ayarları.
            </p>
            <div className="p-4 rounded-2xl bg-black/40 border border-white/5 text-left font-mono text-xs max-w-md mx-auto space-y-2">
              <div className="flex justify-between">
                <span className="text-zinc-500">Nginx Loopback:</span>
                <span className="text-zinc-200">127.0.0.1:8080</span>
              </div>
              <div className="flex justify-between">
                <span className="text-zinc-500">MariaDB Port:</span>
                <span className="text-zinc-200">3306</span>
              </div>
              <div className="flex justify-between">
                <span className="text-zinc-500">Desteklenen PHP:</span>
                <span className="text-zinc-200">8.4, 8.3, 8.2</span>
              </div>
            </div>
          </div>
        )}
      </div>

      {/* Project Creation Modal */}
      <CreateProjectModal
        isOpen={isCreateModalOpen}
        onClose={() => setIsCreateModalOpen(false)}
        onCreate={handleCreateProject}
        volumes={volumes}
      />
    </AppShell>
  );
};
