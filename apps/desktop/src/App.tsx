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
import { ProjectsWorkspace } from "./components/ProjectsWorkspace";
import { ServicesRuntime } from "./components/ServicesRuntime";
import { StorageVolumes } from "./components/StorageVolumes";
import { CreateProjectModal } from "./components/CreateProjectModal";
import { WelcomeSplash } from "./components/WelcomeSplash";
import { ErrorState } from "./components/ErrorState";
import { ProjectRecord, VolumeInfo, RuntimeOverview, ViewMode } from "./types";
import { getVisualFixture } from "./fixtures/visualFixtures";

export const App: React.FC = () => {
  const [projects, setProjects] = useState<ProjectRecord[]>([]);
  const [volumes, setVolumes] = useState<VolumeInfo[]>([]);
  const [runtime, setRuntime] = useState<RuntimeOverview | null>(null);
  const [viewMode, setViewMode] = useState<ViewMode>("studio");
  const [isCapsuleOpen, setIsCapsuleOpen] = useState(false);
  const [isCreateModalOpen, setIsCreateModalOpen] = useState(false);
  const [isWelcomeOpen, setIsWelcomeOpen] = useState(false);
  const [isFixtureMode, setIsFixtureMode] = useState(false);
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

      const welcomeParam = isDev
        ? new URLSearchParams(window.location.search).get("welcome")
        : null;

      const modeParam = isDev
        ? (new URLSearchParams(window.location.search).get("mode") as ViewMode)
        : null;

      if (modeParam && ["studio", "projects", "services", "storage", "deployments", "mcp", "settings"].includes(modeParam)) {
        setViewMode(modeParam);
      }

      if (welcomeParam === "true") {
        setIsWelcomeOpen(true);
      }

      const activeFixture = isDev && fixtureParam ? getVisualFixture(fixtureParam) : null;

      if (activeFixture) {
        setIsFixtureMode(true);
        setVolumes(activeFixture.volumes);
        setProjects(activeFixture.projects);
        setRuntime(activeFixture.runtime);
        return;
      }

      setIsFixtureMode(false);

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
      } else if (isFixtureMode) {
        setProjects((prev) =>
          prev.map((p) => (p.id === projectId ? { ...p, status: "running" } : p))
        );
      } else {
        setErrorMsg("LocalForge masaüstü ortamında (Tauri) çalıştırılmalıdır.");
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
      } else if (isFixtureMode) {
        setProjects((prev) =>
          prev.map((p) => (p.id === projectId ? { ...p, status: "stopped" } : p))
        );
      } else {
        setErrorMsg("LocalForge masaüstü ortamında (Tauri) çalıştırılmalıdır.");
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
      } else if (isFixtureMode) {
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
      } else {
        setErrorMsg("LocalForge masaüstü ortamında (Tauri) çalıştırılmalıdır.");
        return;
      }
      setIsCreateModalOpen(false);
    } catch (err: any) {
      console.error("Create project failed:", err);
      setErrorMsg(`Failed to create project: ${err?.message || String(err)}`);
    }
  };

  return (
    <AppShell
      activeMode={viewMode}
      onSelectMode={setViewMode}
      projects={projects}
      volumes={volumes}
      runtime={runtime}
      isCapsuleOpen={isCapsuleOpen}
      onOpenCapsule={() => setIsCapsuleOpen(true)}
      onCloseCapsule={() => setIsCapsuleOpen(false)}
      onSelectProject={(_p) => {}}
      onCreateProject={() => setIsCreateModalOpen(true)}
      onRefreshData={loadData}
      onOpenWelcome={() => setIsWelcomeOpen(true)}
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
          <ProjectsWorkspace
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
          <ServicesRuntime runtime={runtime} onRefresh={loadData} />
        )}

        {viewMode === "storage" && (
          <StorageVolumes
            volumes={volumes}
            projects={projects}
            onRefresh={loadData}
          />
        )}

        {viewMode === "deployments" && (
          <div className="w-full max-w-3xl py-16 text-center space-y-4">
            <div className="w-16 h-16 rounded-3xl bg-[#695FA4]/20 border border-[#B7ADDF]/30 text-[#B7ADDF] flex items-center justify-center mx-auto">
              <CloudUpload className="w-8 h-8" />
            </div>
            <h2 className="font-['Plus_Jakarta_Sans',sans-serif] text-xl font-bold text-[#FFFBFF]">
              Akıllı Güvenli Dağıtım
            </h2>
            <p className="text-xs text-[#D4CCF1] max-w-md mx-auto leading-relaxed">
              Git diff değil; SHA-256 sağlama defteri tabanlı güvenli dağıtım. Üretim dağıtımları insan onay kapısıyla korunur.
            </p>
            <div className="inline-flex items-center gap-2 px-3 py-1.5 rounded-full bg-[#F59E0B]/12 border border-[#F59E0B]/25 text-xs text-[#F59E0B]">
              <Lock className="w-3.5 h-3.5" />
              <span>SFTP Dağıtım Kapısı Devrede (Salt-Okunur Önizleme / Dry-Run Only)</span>
            </div>
          </div>
        )}

        {viewMode === "mcp" && (
          <div className="w-full max-w-3xl py-16 text-center space-y-4">
            <div className="w-16 h-16 rounded-3xl bg-[#695FA4]/20 border border-[#B7ADDF]/30 text-[#B7ADDF] flex items-center justify-center mx-auto">
              <Bot className="w-8 h-8" />
            </div>
            <h2 className="font-['Plus_Jakarta_Sans',sans-serif] text-xl font-bold text-[#FFFBFF]">
              AI / Model Context Protocol (MCP)
            </h2>
            <p className="text-xs text-[#D4CCF1] max-w-md mx-auto leading-relaxed">
              Claude Code, Gemini ve Cursor için yerel MCP sunucusu. Salt okunur tanılama ve insan onaylı işlem güvenliği.
            </p>
            <div className="inline-flex items-center gap-2 px-3 py-1.5 rounded-full bg-[#10B981]/12 border border-[#10B981]/25 text-xs text-[#10B981] font-mono">
              <ShieldCheck className="w-3.5 h-3.5" />
              <span>localforge-mcp-server: Dinleniyor (stdio)</span>
            </div>
          </div>
        )}

        {viewMode === "settings" && (
          <div className="w-full max-w-3xl py-16 text-center space-y-4">
            <div className="w-16 h-16 rounded-3xl bg-[#2B2450]/60 border border-[rgba(183,173,223,0.2)] text-[#D4CCF1] flex items-center justify-center mx-auto">
              <SettingsIcon className="w-8 h-8" />
            </div>
            <h2 className="font-['Plus_Jakarta_Sans',sans-serif] text-xl font-bold text-[#FFFBFF]">
              Sistem Tercihleri
            </h2>
            <p className="text-xs text-[#D4CCF1] max-w-md mx-auto leading-relaxed">
              Port yapılandırmaları, APFS bağlama güvenlik sınırları ve loopback proxy ayarları.
            </p>
            <div className="p-4 rounded-2xl bg-[#191337]/90 border border-[rgba(183,173,223,0.14)] text-left font-mono text-xs max-w-md mx-auto space-y-2 text-[#D4CCF1]">
              <div className="flex justify-between">
                <span className="text-[#B7ADDF]">Nginx Loopback:</span>
                <span className="text-[#FFFBFF]">127.0.0.1:8080</span>
              </div>
              <div className="flex justify-between">
                <span className="text-[#B7ADDF]">MariaDB Port:</span>
                <span className="text-[#FFFBFF]">3306</span>
              </div>
              <div className="flex justify-between">
                <span className="text-[#B7ADDF]">Desteklenen PHP:</span>
                <span className="text-[#FFFBFF]">8.4, 8.3, 8.2</span>
              </div>
              <div className="flex justify-between">
                <span className="text-[#B7ADDF]">Harici Çalışma Alanı:</span>
                <span className="text-[#10B981]">/Volumes/Mac Harici Disk</span>
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

      {/* Welcome Splash Experience */}
      <WelcomeSplash
        isOpen={isWelcomeOpen}
        onClose={() => setIsWelcomeOpen(false)}
        onCreateProject={() => {
          setIsWelcomeOpen(false);
          setIsCreateModalOpen(true);
        }}
        onOpenFolder={() => {
          setIsWelcomeOpen(false);
          setIsCreateModalOpen(true);
        }}
        onDefineExternalDisk={() => {
          setIsWelcomeOpen(false);
          setViewMode("storage");
        }}
      />
    </AppShell>
  );
};
