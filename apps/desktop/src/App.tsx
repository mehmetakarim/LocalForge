import React, { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Plus, RefreshCw, Layers, ShieldCheck } from "lucide-react";
import { ProjectList } from "./components/ProjectList";
import { DriveStatusBadge } from "./components/DriveStatusBadge";
import { CreateProjectModal } from "./components/CreateProjectModal";
import { ProjectRecord, VolumeInfo } from "./types";

export const App: React.FC = () => {
  const [projects, setProjects] = useState<ProjectRecord[]>([]);
  const [volumes, setVolumes] = useState<VolumeInfo[]>([]);
  const [loading, setLoading] = useState(true);
  const [errorMsg, setErrorMsg] = useState<string | null>(null);
  const [isModalOpen, setIsModalOpen] = useState(false);

  const loadData = async () => {
    setLoading(true);
    setErrorMsg(null);
    try {
      // Check if Tauri is available
      if (typeof window !== "undefined" && "__TAURI_INTERNALS__" in window) {
        const vols = await invoke<VolumeInfo[]>("get_volumes");
        const projs = await invoke<ProjectRecord[]>("sync_projects");
        setVolumes(vols);
        setProjects(projs);
      } else {
        // Fallback mock data for webview dev server preview
        setVolumes([
          {
            uuid: "E289C823-7F01-44B1-888A-9A4B8F349E01",
            name: "Mac Harici Disk",
            mount_point: "/Volumes/Mac Harici Disk",
            filesystem: "apfs",
            is_removable: true,
            is_internal: false,
            total_bytes: 1000204886016,
            available_bytes: 144955146240,
          },
          {
            uuid: "ROOT-MAC-DISK-UUID",
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
            volume_uuid: "E289C823-7F01-44B1-888A-9A4B8F349E01",
            document_root: "public",
            php_version: "8.2",
            local_domain: "stepperskip.test",
            status: "stopped",
            created_at: new Date().toISOString(),
            updated_at: new Date().toISOString(),
          },
        ]);
      }
    } catch (err: any) {
      console.error("Failed to load LocalForge data:", err);
      setErrorMsg(err?.message || String(err));
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadData();
  }, []);

  const handleToggleStatus = async (id: string, currentStatus: string) => {
    try {
      const newStatus = currentStatus === "running" ? "stopped" : "running";
      if (typeof window !== "undefined" && "__TAURI_INTERNALS__" in window) {
        await invoke("update_project_status", { id, status: newStatus });
      }
      setProjects((prev) =>
        prev.map((p) => (p.id === id ? { ...p, status: newStatus as any } : p))
      );
    } catch (err) {
      console.error("Failed to update project status:", err);
    }
  };

  const handleCreateProject = async (
    name: string,
    parentDir: string,
    docroot: string,
    phpVersion: string
  ) => {
    if (typeof window !== "undefined" && "__TAURI_INTERNALS__" in window) {
      const created = await invoke<ProjectRecord>("create_project", {
        name,
        parentDir,
        docroot,
        phpVersion,
      });
      setProjects((prev) => [...prev, created]);
    } else {
      const newMock: ProjectRecord = {
        id: `proj_${Date.now()}`,
        name,
        path: `${parentDir}/${name}`,
        volume_uuid: "MOCK-UUID",
        document_root: docroot,
        php_version: phpVersion,
        local_domain: `${name.toLowerCase()}.test`,
        status: "stopped",
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
      };
      setProjects((prev) => [...prev, newMock]);
    }
  };

  return (
    <div className="min-h-screen flex flex-col bg-slate-950 text-slate-100 font-sans">
      {/* Top Navbar */}
      <header className="h-14 border-b border-slate-800/80 px-6 flex items-center justify-between bg-slate-900/50 backdrop-blur-md sticky top-0 z-40">
        <div className="flex items-center gap-3">
          <div className="w-8 h-8 rounded-lg bg-gradient-to-br from-indigo-500 to-indigo-700 flex items-center justify-center shadow-md shadow-indigo-500/20">
            <Layers className="w-4 h-4 text-white" />
          </div>
          <div>
            <span className="font-bold text-sm tracking-tight text-white">LocalForge</span>
            <span className="ml-2 text-[10px] font-semibold tracking-wider text-indigo-400 uppercase bg-indigo-950/60 px-1.5 py-0.5 rounded border border-indigo-800/50">
              macOS arm64
            </span>
          </div>
        </div>

        <div className="flex items-center gap-3">
          <DriveStatusBadge volumes={volumes} />

          <button
            onClick={loadData}
            title="Refresh Projects and Volumes"
            className="p-2 rounded-lg bg-slate-900 hover:bg-slate-800 text-slate-400 hover:text-white border border-slate-800 transition-colors"
          >
            <RefreshCw className={`w-4 h-4 ${loading ? "animate-spin" : ""}`} />
          </button>

          <button
            onClick={() => setIsModalOpen(true)}
            className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-indigo-600 hover:bg-indigo-500 text-white text-xs font-semibold shadow-sm shadow-indigo-600/30 transition-all"
          >
            <Plus className="w-4 h-4" />
            <span>New Project</span>
          </button>
        </div>
      </header>

      {/* Main Content */}
      <main className="flex-1 max-w-7xl w-full mx-auto p-6">
        {errorMsg && (
          <div className="mb-4 p-4 rounded-xl bg-rose-950/40 border border-rose-800 text-rose-200 text-xs">
            <span className="font-bold">Initialization Notice: </span>
            {errorMsg}
          </div>
        )}

        <div className="mb-6 flex items-center justify-between">
          <div>
            <h1 className="text-xl font-bold text-white tracking-tight">Active Workspaces</h1>
            <p className="text-xs text-slate-400 mt-0.5">
              Manage your local PHP projects, external storage drives, and loopback domains.
            </p>
          </div>
          <div className="flex items-center gap-2 text-xs text-slate-400 bg-slate-900/80 px-3 py-1.5 rounded-lg border border-slate-800">
            <ShieldCheck className="w-4 h-4 text-emerald-400" />
            <span>Zero-Root Loopback Isolation</span>
          </div>
        </div>

        <ProjectList projects={projects} onToggleStatus={handleToggleStatus} />
      </main>

      {/* Footer */}
      <footer className="h-10 border-t border-slate-900 px-6 flex items-center justify-between text-[11px] text-slate-500 bg-slate-950">
        <div>LocalForge Core v0.1.0 (Phase 1 Desktop Foundation)</div>
        <div>Apache-2.0 / MIT Dual License</div>
      </footer>

      {/* New Project Modal */}
      <CreateProjectModal
        isOpen={isModalOpen}
        onClose={() => setIsModalOpen(false)}
        volumes={volumes}
        onCreate={handleCreateProject}
      />
    </div>
  );
};
