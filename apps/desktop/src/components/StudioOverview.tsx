import React, { useState } from "react";
import {
  Plus,
  RefreshCw,
  Search,
} from "lucide-react";
import { ProjectRecord, VolumeInfo, RuntimeOverview } from "../types";
import { SpatialCard } from "./SpatialCard";
import { ServiceHealthPill } from "./ServiceHealthPill";
import { EmptyState } from "./EmptyState";

export interface StudioOverviewProps {
  projects: ProjectRecord[];
  volumes: VolumeInfo[];
  runtime: RuntimeOverview | null;
  mutatingProject: string | null;
  onStartProject: (projectId: string) => void;
  onStopProject: (projectId: string) => void;
  onOpenBrowser: (project: ProjectRecord) => void;
  onCreateProject: () => void;
  onRefresh: () => void;
}

export const StudioOverview: React.FC<StudioOverviewProps> = ({
  projects,
  volumes,
  runtime,
  mutatingProject,
  onStartProject,
  onStopProject,
  onOpenBrowser,
  onCreateProject,
  onRefresh,
}) => {
  const [filter, setFilter] = useState<"all" | "running" | "external">("all");
  const [search, setSearch] = useState("");

  const runningCount = projects.filter((p) => p.status === "running").length;
  const externalProjects = projects.filter((p) => {
    const vol = volumes.find((v) => v.uuid === p.volume_uuid);
    return vol ? vol.is_removable || !vol.is_internal : false;
  });

  const filteredProjects = projects.filter((p) => {
    const matchesSearch =
      p.name.toLowerCase().includes(search.toLowerCase()) ||
      p.local_domain.toLowerCase().includes(search.toLowerCase()) ||
      p.php_version.includes(search);

    if (!matchesSearch) return false;

    if (filter === "running") return p.status === "running";
    if (filter === "external") {
      const vol = volumes.find((v) => v.uuid === p.volume_uuid);
      return vol ? vol.is_removable || !vol.is_internal : false;
    }
    return true;
  });

  return (
    <div className="space-y-7 pb-20">
      {/* Top Atmospheric Header & Macro Metrics */}
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
        <div>
          <div className="flex items-center gap-2 mb-1">
            <span className="px-2.5 py-0.5 rounded-full text-[10px] font-mono uppercase font-bold tracking-wider bg-indigo-500/15 border border-indigo-400/30 text-indigo-300">
              macOS Apple Silicon
            </span>
            <span className="text-zinc-500 text-xs">•</span>
            <span className="text-xs text-zinc-400 font-mono">Loopback 127.0.0.1</span>
          </div>
          <h1 className="text-2xl md:text-3xl font-extrabold text-white tracking-tight">
            Spatial Studio
          </h1>
          <p className="text-xs md:text-sm text-zinc-400 mt-0.5">
            AI-native local PHP orchestration, unprivileged runtimes &amp; resilient APFS storage
          </p>
        </div>

        {/* Global Action Deck */}
        <div className="flex items-center gap-3">
          <button
            type="button"
            onClick={onRefresh}
            className="flex items-center gap-1.5 px-3 py-2 rounded-xl bg-white/5 hover:bg-white/10 border border-white/10 text-xs font-medium text-zinc-300 hover:text-white transition-colors"
            title="Resync projects & runtimes"
          >
            <RefreshCw className="w-3.5 h-3.5 text-indigo-400" />
            <span className="hidden sm:inline">Refresh</span>
          </button>

          <button
            type="button"
            onClick={onCreateProject}
            className="flex items-center gap-1.5 px-4 py-2 rounded-xl bg-indigo-600 hover:bg-indigo-500 text-white font-medium text-xs transition-all shadow-[0_0_20px_rgba(79,110,247,0.45)] border border-indigo-400/50"
          >
            <Plus className="w-4 h-4" />
            <span>Create Project</span>
          </button>
        </div>
      </div>

      {/* Infrastructure Telemetry Ribbon */}
      <div
        style={{
          boxShadow:
            "0 12px 28px -6px rgba(0, 0, 0, 0.45), inset 0 1px 0 0 rgba(255, 255, 255, 0.15)",
        }}
        className="p-3.5 rounded-2xl lf-material-card border border-white/10 flex flex-wrap items-center justify-between gap-3"
      >
        <div className="flex flex-wrap items-center gap-2">
          <ServiceHealthPill
            name="Nginx Gateway"
            status={runtime?.nginx_running ? "running" : "stopped"}
            port={runtime?.nginx_port || 8080}
            onRefresh={onRefresh}
          />
          <ServiceHealthPill
            name="PHP-FPM Pools"
            status="running"
            version={runtime?.php_versions ? runtime.php_versions.join(", ") : "8.4"}
            onRefresh={onRefresh}
          />
          <ServiceHealthPill
            name="MariaDB 3306"
            status={runtime?.mariadb_alive ? "running" : "stopped"}
            port={runtime?.mariadb_port || 3306}
            socketPath={runtime?.mariadb_socket}
            latencyMs={runtime?.mariadb_latency_ms}
            onRefresh={onRefresh}
          />
        </div>

        {/* Quick Stats Counter */}
        <div className="flex items-center gap-4 text-xs font-mono text-zinc-400 px-2">
          <div className="flex items-center gap-1.5">
            <span className="text-zinc-500">Active:</span>
            <span className="text-emerald-400 font-bold">{runningCount}</span>
            <span className="text-zinc-600">/</span>
            <span className="text-zinc-300">{projects.length}</span>
          </div>
          <div className="hidden sm:flex items-center gap-1.5">
            <span className="text-zinc-500">External:</span>
            <span className="text-amber-400 font-bold">{externalProjects.length}</span>
          </div>
        </div>
      </div>

      {/* Project Navigation & Filter Bar */}
      <div className="flex flex-col sm:flex-row items-center justify-between gap-3">
        {/* Filter Pills */}
        <div className="flex items-center gap-1.5 p-1 rounded-xl bg-white/5 border border-white/10 text-xs">
          <button
            type="button"
            onClick={() => setFilter("all")}
            className={`px-3 py-1 rounded-lg font-medium transition-colors ${
              filter === "all"
                ? "bg-indigo-600 text-white shadow"
                : "text-zinc-400 hover:text-zinc-200"
            }`}
          >
            All Workspaces ({projects.length})
          </button>
          <button
            type="button"
            onClick={() => setFilter("running")}
            className={`px-3 py-1 rounded-lg font-medium transition-colors ${
              filter === "running"
                ? "bg-indigo-600 text-white shadow"
                : "text-zinc-400 hover:text-zinc-200"
            }`}
          >
            Running ({runningCount})
          </button>
          <button
            type="button"
            onClick={() => setFilter("external")}
            className={`px-3 py-1 rounded-lg font-medium transition-colors ${
              filter === "external"
                ? "bg-indigo-600 text-white shadow"
                : "text-zinc-400 hover:text-zinc-200"
            }`}
          >
            External SSD ({externalProjects.length})
          </button>
        </div>

        {/* Search input */}
        <div className="relative w-full sm:w-64">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-zinc-500" />
          <input
            type="text"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            placeholder="Filter projects..."
            className="w-full pl-9 pr-3 py-1.5 rounded-xl bg-black/40 border border-white/10 text-xs text-white placeholder-zinc-500 outline-none focus:border-indigo-400/60 transition-colors"
          />
        </div>
      </div>

      {/* Spatial Card Grid */}
      {filteredProjects.length === 0 ? (
        <EmptyState
          title={search ? "No Matching Projects" : "No Projects Registered Yet"}
          description={
            search
              ? `No workspaces found matching "${search}". Try checking your spelling or clearing filters.`
              : "Create your first local PHP workspace on your external drive or internal SSD to begin serving."
          }
          actionLabel={search ? undefined : "Create New Project"}
          onAction={search ? undefined : onCreateProject}
        />
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-5">
          {filteredProjects.map((project) => {
            const vol = volumes.find((v) => v.uuid === project.volume_uuid);
            const isExternal = vol ? vol.is_removable || !vol.is_internal : false;
            const volumeLabel = vol ? vol.name : "Internal Drive";

            return (
              <SpatialCard
                key={project.id}
                id={project.id}
                title={project.name}
                status={project.status}
                phpVersion={project.php_version}
                localDomain={project.local_domain}
                volumeName={volumeLabel}
                isExternalVolume={isExternal}
                documentRoot={project.document_root}
                activePort={runtime?.nginx_port || 8080}
                isMutating={mutatingProject === project.id}
                onStart={() => onStartProject(project.id)}
                onStop={() => onStopProject(project.id)}
                onOpenBrowser={() => onOpenBrowser(project)}
              />
            );
          })}
        </div>
      )}
    </div>
  );
};
