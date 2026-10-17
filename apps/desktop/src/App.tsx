import React, { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  Plus,
  RefreshCw,
  Search,
  HardDrive,
  Layers,
  Cpu,
  ShieldCheck,
  Zap,
  Sparkles,
  Command,
} from "lucide-react";
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
  const [activeTab, setActiveTab] = useState<"all" | "external" | "running">("all");
  const [searchQuery, setSearchQuery] = useState("");

  const loadData = async () => {
    setLoading(true);
    setErrorMsg(null);
    try {
      if (typeof window !== "undefined" && "__TAURI_INTERNALS__" in window) {
        const vols = await invoke<VolumeInfo[]>("get_volumes");
        const projs = await invoke<ProjectRecord[]>("sync_projects");
        setVolumes(vols);
        setProjects(projs);
      } else {
        // High fidelity mock data for preview/browser testing
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
      const newProj: ProjectRecord = {
        id: `proj_${Date.now()}`,
        name,
        path: `${parentDir}/${name}`,
        volume_uuid: volumes.find((v) => v.mount_point === parentDir)?.uuid || "UUID-DEV",
        document_root: docroot,
        php_version: phpVersion,
        local_domain: `${name.toLowerCase().replace(/[^a-z0-9]/g, "")}.test`,
        status: "stopped",
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
      };
      setProjects((prev) => [...prev, newProj]);
    }
  };

  // Filter projects by active tab & search query
  const filteredProjects = projects.filter((p) => {
    const matchesSearch =
      p.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      p.local_domain.toLowerCase().includes(searchQuery.toLowerCase()) ||
      p.path.toLowerCase().includes(searchQuery.toLowerCase());

    if (!matchesSearch) return false;

    if (activeTab === "external") {
      return p.path.startsWith("/Volumes");
    }
    if (activeTab === "running") {
      return p.status === "running";
    }
    return true;
  });

  const runningCount = projects.filter((p) => p.status === "running").length;
  const externalCount = projects.filter((p) => p.path.startsWith("/Volumes")).length;

  return (
    <div className="flex h-screen w-screen overflow-hidden bg-[#0d0f13] text-white">
      {/* macOS Translucent Left Sidebar */}
      <aside className="w-72 flex-shrink-0 flex flex-col justify-between border-r border-white/[0.08] bg-neutral-950/80 backdrop-blur-3xl select-none">
        <div>
          {/* Window Drag Title Bar Header */}
          <div data-tauri-drag-region className="h-12 pl-20 pr-4 flex items-center justify-between border-b border-white/[0.06]">
            <span className="text-[11px] font-semibold tracking-wider text-neutral-400 uppercase">
              LocalForge
            </span>
            <span className="px-1.5 py-0.5 rounded text-[9px] font-bold bg-blue-500/20 text-blue-300 border border-blue-500/30">
              macOS arm64
            </span>
          </div>

          {/* Brand Mark */}
          <div className="p-4 flex items-center gap-3">
            <div className="w-9 h-9 rounded-xl bg-gradient-to-tr from-blue-600 via-indigo-600 to-cyan-400 flex items-center justify-center text-white shadow-lg shadow-blue-500/25">
              <Sparkles className="w-5 h-5" />
            </div>
            <div>
              <div className="font-bold text-sm tracking-tight text-white">LocalForge</div>
              <div className="text-[11px] text-neutral-400">AI-Ready Dev & Deploy</div>
            </div>
          </div>

          {/* Navigation Items */}
          <nav className="px-3 space-y-1">
            <button
              onClick={() => setActiveTab("all")}
              className={`w-full flex items-center justify-between px-3 py-2 rounded-xl text-xs font-medium transition-all ${
                activeTab === "all"
                  ? "bg-blue-600 text-white shadow-sm shadow-blue-600/30"
                  : "text-neutral-400 hover:text-white hover:bg-white/[0.05]"
              }`}
            >
              <div className="flex items-center gap-2.5">
                <Layers className="w-4 h-4" />
                <span>Tüm Çalışma Alanları</span>
              </div>
              <span className={`text-[10px] font-semibold px-1.5 py-0.2 rounded-md ${activeTab === "all" ? "bg-white/20" : "bg-neutral-800 text-neutral-300"}`}>
                {projects.length}
              </span>
            </button>

            <button
              onClick={() => setActiveTab("external")}
              className={`w-full flex items-center justify-between px-3 py-2 rounded-xl text-xs font-medium transition-all ${
                activeTab === "external"
                  ? "bg-blue-600 text-white shadow-sm shadow-blue-600/30"
                  : "text-neutral-400 hover:text-white hover:bg-white/[0.05]"
              }`}
            >
              <div className="flex items-center gap-2.5">
                <HardDrive className="w-4 h-4 text-amber-400" />
                <span>Harici SSD Projeleri</span>
              </div>
              <span className={`text-[10px] font-semibold px-1.5 py-0.2 rounded-md ${activeTab === "external" ? "bg-white/20" : "bg-neutral-800 text-neutral-300"}`}>
                {externalCount}
              </span>
            </button>

            <button
              onClick={() => setActiveTab("running")}
              className={`w-full flex items-center justify-between px-3 py-2 rounded-xl text-xs font-medium transition-all ${
                activeTab === "running"
                  ? "bg-blue-600 text-white shadow-sm shadow-blue-600/30"
                  : "text-neutral-400 hover:text-white hover:bg-white/[0.05]"
              }`}
            >
              <div className="flex items-center gap-2.5">
                <Zap className="w-4 h-4 text-emerald-400" />
                <span>Aktif Servisler</span>
              </div>
              {runningCount > 0 && (
                <span className="w-2 h-2 rounded-full bg-emerald-400 animate-pulse" />
              )}
            </button>
          </nav>

          {/* Runtime Daemons Status */}
          <div className="mt-6 px-4">
            <div className="text-[10px] font-bold uppercase tracking-wider text-neutral-400 mb-2.5">
              Yerel Servis Durumu
            </div>
            <div className="p-3 rounded-xl bg-neutral-900/50 border border-white/[0.06] space-y-2">
              <div className="flex items-center justify-between text-xs">
                <div className="flex items-center gap-2">
                  <span className="w-1.5 h-1.5 rounded-full bg-emerald-400" />
                  <span className="text-neutral-300 font-medium">PHP-FPM</span>
                </div>
                <span className="text-[10px] text-neutral-400 font-mono">v8.4 Ondemand</span>
              </div>

              <div className="flex items-center justify-between text-xs">
                <div className="flex items-center gap-2">
                  <span className="w-1.5 h-1.5 rounded-full bg-emerald-400" />
                  <span className="text-neutral-300 font-medium">Nginx Proxy</span>
                </div>
                <span className="text-[10px] text-neutral-400 font-mono">127.0.0.1:8080</span>
              </div>

              <div className="flex items-center justify-between text-xs">
                <div className="flex items-center gap-2">
                  <span className="w-1.5 h-1.5 rounded-full bg-emerald-400" />
                  <span className="text-neutral-300 font-medium">MariaDB</span>
                </div>
                <span className="text-[10px] text-neutral-400 font-mono">127.0.0.1:3306</span>
              </div>
            </div>
          </div>
        </div>

        {/* Sidebar Bottom: Storage Volumes */}
        <div className="p-4 border-t border-white/[0.06]">
          <DriveStatusBadge volumes={volumes} />
        </div>
      </aside>

      {/* Main Content Workspace */}
      <div className="flex-1 flex flex-col h-full overflow-hidden">
        {/* macOS Unified Titlebar & Toolbar */}
        <header data-tauri-drag-region className="h-14 px-6 border-b border-white/[0.08] bg-neutral-950/60 backdrop-blur-2xl flex items-center justify-between gap-4 select-none">
          {/* Search bar with Apple styling */}
          <div className="relative w-72">
            <Search className="w-4 h-4 text-neutral-400 absolute left-3 top-1/2 -translate-y-1/2" />
            <input
              type="text"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              placeholder="Çalışma alanı ara..."
              className="w-full pl-9 pr-8 py-1.5 bg-neutral-900/60 border border-white/[0.08] rounded-xl text-xs text-white placeholder-neutral-400 focus:outline-none focus:border-blue-500 transition-all"
            />
            <div className="absolute right-2.5 top-1/2 -translate-y-1/2 flex items-center gap-0.5 text-[9px] text-neutral-400 bg-white/10 px-1.5 py-0.5 rounded">
              <Command className="w-2.5 h-2.5" />
              <span>K</span>
            </div>
          </div>

          {/* Action CTAs */}
          <div className="flex items-center gap-2.5">
            <button
              onClick={loadData}
              disabled={loading}
              className="p-2 rounded-xl text-neutral-400 hover:text-white apple-button-ghost transition-all"
              title="Yenile"
            >
              <RefreshCw className={`w-4 h-4 ${loading ? "animate-spin" : ""}`} />
            </button>

            <button
              onClick={() => setIsModalOpen(true)}
              className="px-4 py-2 rounded-xl text-xs font-semibold text-white apple-button-primary flex items-center gap-1.5"
            >
              <Plus className="w-3.5 h-3.5 stroke-[2.5]" />
              <span>Yeni Proje</span>
            </button>
          </div>
        </header>

        {/* Scrollable Main Area */}
        <main className="flex-1 overflow-y-auto p-8">
          {errorMsg && (
            <div className="mb-6 p-4 rounded-2xl bg-rose-950/40 border border-rose-800/80 text-rose-200 text-xs flex items-center justify-between">
              <span>{errorMsg}</span>
              <button onClick={() => setErrorMsg(null)} className="text-rose-400 hover:text-rose-200">Kapat</button>
            </div>
          )}

          {/* Apple Hardware / Performance Summary Banners */}
          <div className="grid grid-cols-1 md:grid-cols-4 gap-4 mb-8">
            <div className="apple-card p-4 rounded-2xl">
              <div className="flex items-center justify-between text-neutral-400 text-xs">
                <span>Kayıtlı Projeler</span>
                <Layers className="w-4 h-4 text-blue-400" />
              </div>
              <div className="text-2xl font-bold text-white mt-2 tracking-tight">
                {projects.length}
              </div>
              <div className="text-[11px] text-emerald-400 mt-1 flex items-center gap-1 font-medium">
                <span className="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse" />
                {runningCount} aktif çalışıyor
              </div>
            </div>

            <div className="apple-card p-4 rounded-2xl">
              <div className="flex items-center justify-between text-neutral-400 text-xs">
                <span>Harici APFS Alanı</span>
                <HardDrive className="w-4 h-4 text-amber-400" />
              </div>
              <div className="text-2xl font-bold text-white mt-2 tracking-tight">
                {volumes.find((v) => !v.is_internal || v.mount_point.startsWith("/Volumes"))?.name || "Mac Harici Disk"}
              </div>
              <div className="text-[11px] text-neutral-400 mt-1">
                Donanım UUID Korumalı
              </div>
            </div>

            <div className="apple-card p-4 rounded-2xl">
              <div className="flex items-center justify-between text-neutral-400 text-xs">
                <span>Yalıtılmış Runtimes</span>
                <Cpu className="w-4 h-4 text-indigo-400" />
              </div>
              <div className="text-2xl font-bold text-white mt-2 tracking-tight">
                PHP 8.4 + FPM
              </div>
              <div className="text-[11px] text-neutral-400 mt-1">
                Loopback 127.0.0.1:8080
              </div>
            </div>

            <div className="apple-card p-4 rounded-2xl">
              <div className="flex items-center justify-between text-neutral-400 text-xs">
                <span>Akıllı Dağıtım & MCP</span>
                <ShieldCheck className="w-4 h-4 text-emerald-400" />
              </div>
              <div className="text-2xl font-bold text-white mt-2 tracking-tight">
                Onay Kapısı Devrede
              </div>
              <div className="text-[11px] text-neutral-400 mt-1 font-mono">
                SHA-256 Defteri Hazır
              </div>
            </div>
          </div>

          {/* Section Title & View Switcher */}
          <div className="flex items-center justify-between mb-5">
            <div>
              <h2 className="text-lg font-bold text-white tracking-tight">
                {activeTab === "all"
                  ? "Tüm Projeler"
                  : activeTab === "external"
                  ? "Harici SSD Projeleri"
                  : "Aktif Çalışan Servisler"}
              </h2>
              <p className="text-xs text-neutral-400">
                {filteredProjects.length} çalışma alanı listelendi
              </p>
            </div>
          </div>

          {/* Project Cards Grid */}
          <ProjectList
            projects={filteredProjects}
            onToggleStatus={handleToggleStatus}
            onOpenCreate={() => setIsModalOpen(true)}
          />
        </main>
      </div>

      {/* Apple Styled Create Project Modal Sheet */}
      <CreateProjectModal
        isOpen={isModalOpen}
        onClose={() => setIsModalOpen(false)}
        volumes={volumes}
        onCreate={handleCreateProject}
      />
    </div>
  );
};
