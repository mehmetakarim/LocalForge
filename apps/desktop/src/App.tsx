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
import { ImportProjectModal } from "./components/ImportProjectModal";
import { RoutingAlertModal } from "./components/RoutingAlertModal";
import { WelcomeSplash } from "./components/WelcomeSplash";
import { ErrorState } from "./components/ErrorState";
import { ProjectRecord, VolumeInfo, RuntimeOverview, ViewMode } from "./types";
import { getVisualFixture } from "./fixtures/visualFixtures";
import { computeProjectRouting } from "./utils/browserRouting";

export const App: React.FC = () => {
  const [projects, setProjects] = useState<ProjectRecord[]>([]);
  const [volumes, setVolumes] = useState<VolumeInfo[]>([]);
  const [runtime, setRuntime] = useState<RuntimeOverview | null>(null);
  const [viewMode, setViewMode] = useState<ViewMode>("studio");
  const [isCapsuleOpen, setIsCapsuleOpen] = useState(false);
  const [isCreateModalOpen, setIsCreateModalOpen] = useState(false);
  const [isImportModalOpen, setIsImportModalOpen] = useState(false);
  const [isWelcomeOpen, setIsWelcomeOpen] = useState(false);
  const [routingAlert, setRoutingAlert] = useState<{
    project: ProjectRecord;
    port: number;
    url: string;
    curlCommand: string;
  } | null>(null);
  const [isFixtureMode, setIsFixtureMode] = useState(false);
  const [_loading, setLoading] = useState(true);
  const [errorMsg, setErrorMsg] = useState<string | null>(null);
  const [mutatingProject, setMutatingProject] = useState<string | null>(null);
  const [zoomLevel, setZoomLevel] = useState<number>(100);
  const [isInspectorOpen, setIsInspectorOpen] = useState(false);

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

  const handleUpdatePhpVersion = async (projectId: string, newVersion: string) => {
    try {
      if (typeof window !== "undefined" && "__TAURI_INTERNALS__" in window) {
        await invoke("update_project_php_version", { id: projectId, phpVersion: newVersion });
        await loadData();
      } else if (isFixtureMode) {
        setProjects((prev) =>
          prev.map((p) => (p.id === projectId ? { ...p, php_version: newVersion } : p))
        );
      } else {
        throw new Error("LocalForge masaüstü ortamında (Tauri) çalıştırılmalıdır.");
      }
    } catch (err: any) {
      console.error("Update PHP version failed:", err);
      throw err;
    }
  };

  const handleStartService = async (service: string) => {
    setErrorMsg(null);
    try {
      if (typeof window !== "undefined" && "__TAURI_INTERNALS__" in window) {
        const overview = await invoke<RuntimeOverview>("start_service", { service });
        setRuntime(overview);
      }
    } catch (err: any) {
      console.error(`Start ${service} failed:`, err);
      setErrorMsg(`Servis başlatılamadı: ${err?.message || String(err)}`);
    }
  };

  const handleStopService = async (service: string) => {
    setErrorMsg(null);
    try {
      if (typeof window !== "undefined" && "__TAURI_INTERNALS__" in window) {
        const overview = await invoke<RuntimeOverview>("stop_service", { service });
        setRuntime(overview);
      }
    } catch (err: any) {
      console.error(`Stop ${service} failed:`, err);
      setErrorMsg(`Servis durdurulamadı: ${err?.message || String(err)}`);
    }
  };

  const handleOpenBrowser = async (project: ProjectRecord) => {
    const port = runtime?.nginx_port || 8080;
    const { projectUrl, curlCommand } = computeProjectRouting(project.local_domain, port);

    let isResolvable = false;
    try {
      if (typeof window !== "undefined" && "__TAURI_INTERNALS__" in window) {
        isResolvable = await invoke<boolean>("check_domain_resolution", {
          domain: project.local_domain,
        });
      } else {
        isResolvable = false;
      }
    } catch (err) {
      console.error("Domain resolution check failed:", err);
      isResolvable = false;
    }

    if (isResolvable) {
      window.open(projectUrl, "_blank");
    } else {
      setRoutingAlert({
        project,
        port,
        url: projectUrl,
        curlCommand,
      });
    }
  };

  const handleImportProject = async (path: string) => {
    try {
      if (typeof window !== "undefined" && "__TAURI_INTERNALS__" in window) {
        await invoke("import_project", { path });
        await loadData();
      } else if (isFixtureMode) {
        const name = path.split("/").filter(Boolean).pop() || "imported-project";
        const newProj: ProjectRecord = {
          id: `proj_${Date.now()}`,
          name,
          path,
          volume_uuid: volumes[0]?.uuid || "vol_01",
          document_root: "public",
          php_version: "8.4",
          local_domain: `${name.toLowerCase().replace(/[^a-z0-9]/g, "-")}.test`,
          status: "stopped",
          created_at: new Date().toISOString(),
          updated_at: new Date().toISOString(),
        };
        setProjects((prev) => [newProj, ...prev]);
      } else {
        throw new Error("LocalForge masaüstü ortamında (Tauri) çalıştırılmalıdır.");
      }
      setIsImportModalOpen(false);
    } catch (err: any) {
      console.error("Import project failed:", err);
      throw err;
    }
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
      isInspectorOpen={isInspectorOpen}
    >
      {/* Non-Layout-Shifting Floating HUD Error Notification */}
      {errorMsg && (
        <aside
          aria-label="Sistem Hata Bildirimi"
          className="fixed top-16 right-8 z-50 max-w-md w-full animate-in slide-in-from-top-3 duration-200"
        >
          <ErrorState
            message={errorMsg}
            onRetry={() => setErrorMsg(null)}
            onDismiss={() => setErrorMsg(null)}
          />
        </aside>
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
            onUpdatePhpVersion={handleUpdatePhpVersion}
            onInspectorStateChange={setIsInspectorOpen}
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
            onUpdatePhpVersion={handleUpdatePhpVersion}
            onInspectorStateChange={setIsInspectorOpen}
          />
        )}

        {viewMode === "services" && (
          <ServicesRuntime
            runtime={runtime}
            onRefresh={loadData}
            onStartService={handleStartService}
            onStopService={handleStopService}
          />
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
              Git diff değil; bağımsız SHA-256 sağlama defteri tabanlı güvenli dağıtım. Üretim dağıtımları insan onay kapısıyla korunur.
            </p>
            <div className="inline-flex items-center gap-2 px-3 py-1.5 rounded-full bg-[#F59E0B]/12 border border-[#F59E0B]/25 text-xs text-[#F59E0B]">
              <Lock className="w-3.5 h-3.5" />
              <span>Canlı Dağıtım Motoru Devre Dışı (Phase R1 Güvenlik Kuralı: DeploymentNotImplemented)</span>
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
              <span>localforge-mcp-server: Stdio Arabirimi Hazır (Phase R1 Salt-Okunur İstemci Modu)</span>
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
              Doğrulanmış port yapılandırmaları, APFS bağlama güvenlik sınırları ve loopback proxy ayarları.
            </p>
            <div className="p-4 rounded-2xl bg-[#191337]/90 border border-[rgba(183,173,223,0.14)] text-left font-mono text-xs max-w-md mx-auto space-y-2 text-[#D4CCF1]">
              <div className="flex justify-between">
                <span className="text-[#B7ADDF]">Nginx Loopback:</span>
                <span className="text-[#FFFBFF]">
                  {runtime?.nginx_port ? `127.0.0.1:${runtime.nginx_port}` : "Bilinmiyor (Yoklanmadı)"}
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-[#B7ADDF]">MariaDB Port:</span>
                <span className="text-[#FFFBFF]">
                  {runtime?.mariadb_port ? String(runtime.mariadb_port) : "Bilinmiyor"}
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-[#B7ADDF]">Desteklenen PHP:</span>
                <span className="text-[#FFFBFF]">
                  {runtime?.php_versions && runtime.php_versions.length > 0
                    ? runtime.php_versions.join(", ")
                    : "Tespit Edilmedi"}
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-[#B7ADDF]">Harici Çalışma Alanı:</span>
                <span className="text-[#10B981]">
                  {volumes.find((v) => !v.is_internal || v.is_removable)?.mount_point || "Bağlı Harici Birim Yok"}
                </span>
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
        runtime={runtime}
      />

      {/* Project Import Modal */}
      <ImportProjectModal
        isOpen={isImportModalOpen}
        onClose={() => setIsImportModalOpen(false)}
        onImport={handleImportProject}
        volumes={volumes}
      />

      {/* Safe Domain Routing Alert Modal */}
      <RoutingAlertModal
        isOpen={Boolean(routingAlert)}
        onClose={() => setRoutingAlert(null)}
        project={routingAlert?.project || null}
        port={routingAlert?.port || 8080}
        url={routingAlert?.url || ""}
        curlCommand={routingAlert?.curlCommand || ""}
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
          setIsImportModalOpen(true);
        }}
        onDefineExternalDisk={() => {
          setIsWelcomeOpen(false);
          setViewMode("storage");
        }}
      />
    </AppShell>
  );
};
