import React, { useState } from "react";
import { Search, Plus, ExternalLink, Play, Square, Eye, HardDrive } from "lucide-react";
import { ProjectRecord, VolumeInfo, RuntimeOverview } from "../types";
import { ProjectInspector } from "./ProjectInspector";

export interface ProjectsWorkspaceProps {
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

export const ProjectsWorkspace: React.FC<ProjectsWorkspaceProps> = ({
  projects,
  volumes,
  runtime,
  mutatingProject,
  onStartProject,
  onStopProject,
  onOpenBrowser,
  onCreateProject,
  onRefresh: _onRefresh,
}) => {
  const [searchQuery, setSearchQuery] = useState("");
  const [activeFilter, setActiveFilter] = useState<"all" | "running" | "stopped" | "external">("all");
  const [selectedProjectId, setSelectedProjectId] = useState<string | null>(null);

  const selectedProject = projects.find((p) => p.id === selectedProjectId) || null;
  const selectedVolume = selectedProject
    ? volumes.find((v) => v.uuid === selectedProject.volume_uuid) || null
    : null;

  // Filter projects
  const filteredProjects = projects.filter((project) => {
    const matchesSearch =
      project.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      project.local_domain.toLowerCase().includes(searchQuery.toLowerCase()) ||
      project.path.toLowerCase().includes(searchQuery.toLowerCase());

    if (!matchesSearch) return false;

    const vol = volumes.find((v) => v.uuid === project.volume_uuid);
    const isExternal = vol ? !vol.is_internal || vol.is_removable : false;

    switch (activeFilter) {
      case "running":
        return project.status === "running";
      case "stopped":
        return project.status === "stopped";
      case "external":
        return isExternal;
      case "all":
      default:
        return true;
    }
  });

  const runningCount = projects.filter((p) => p.status === "running").length;
  const stoppedCount = projects.filter((p) => p.status === "stopped").length;
  const externalCount = projects.filter((p) => {
    const vol = volumes.find((v) => v.uuid === p.volume_uuid);
    return vol ? !vol.is_internal || vol.is_removable : false;
  }).length;

  return (
    <div className="w-full max-w-6xl py-8 px-6 flex flex-col gap-6">
      {/* Workspace Header */}
      <div className="flex items-end justify-between border-b border-[rgba(183,173,223,0.12)] pb-5">
        <div>
          <h1 className="font-['Plus_Jakarta_Sans',sans-serif] font-extrabold text-3xl tracking-[-0.03em] text-[#FFFBFF]">
            Projeler
          </h1>
          <p className="text-sm text-[#D4CCF1] mt-1 font-['Figtree',sans-serif]">
            Yerel ve harici APFS birimlerinde kayıtlı tüm geliştirme ortamları ({projects.length} proje).
          </p>
        </div>

        <button
          type="button"
          onClick={onCreateProject}
          className="flex items-center gap-2 px-4 py-2 rounded-xl bg-gradient-to-r from-[#695FA4] to-[#2B2450] border border-[#B7ADDF] text-xs font-semibold text-[#FFFBFF] hover:brightness-110 shadow-[0_4px_16px_rgba(105,95,164,0.35)] transition-all cursor-pointer"
        >
          <Plus className="w-4 h-4 text-[#10B981]" />
          <span>Yeni Proje Başlat</span>
        </button>
      </div>

      {/* Filter and Search Bar */}
      <div className="flex items-center justify-between gap-4 flex-wrap">
        {/* Search Input */}
        <div className="flex items-center gap-2.5 px-3.5 py-2 rounded-xl bg-[#2B2450]/65 border border-[rgba(183,173,223,0.16)] text-xs text-[#FFFBFF] w-80">
          <Search className="w-4 h-4 text-[#B7ADDF]" />
          <input
            type="text"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            placeholder="Proje adı veya yerel etki alanı ara..."
            className="bg-transparent outline-none flex-1 placeholder-[#D4CCF1]/50 text-xs text-[#FFFBFF]"
          />
        </div>

        {/* Filter Pills */}
        <div className="flex items-center gap-2">
          <button
            type="button"
            onClick={() => setActiveFilter("all")}
            className={`px-3 py-1.5 rounded-lg text-xs font-medium transition-all cursor-pointer ${
              activeFilter === "all"
                ? "bg-[#695FA4] text-[#FFFBFF] border border-[#B7ADDF] shadow-[0_2px_8px_rgba(105,95,164,0.35)]"
                : "bg-[#2B2450]/50 text-[#D4CCF1] hover:text-[#FFFBFF] border border-[rgba(183,173,223,0.12)]"
            }`}
          >
            Tümü ({projects.length})
          </button>
          <button
            type="button"
            onClick={() => setActiveFilter("running")}
            className={`px-3 py-1.5 rounded-lg text-xs font-medium transition-all cursor-pointer ${
              activeFilter === "running"
                ? "bg-[#695FA4] text-[#FFFBFF] border border-[#B7ADDF] shadow-[0_2px_8px_rgba(105,95,164,0.35)]"
                : "bg-[#2B2450]/50 text-[#D4CCF1] hover:text-[#FFFBFF] border border-[rgba(183,173,223,0.12)]"
            }`}
          >
            Çalışanlar ({runningCount})
          </button>
          <button
            type="button"
            onClick={() => setActiveFilter("stopped")}
            className={`px-3 py-1.5 rounded-lg text-xs font-medium transition-all cursor-pointer ${
              activeFilter === "stopped"
                ? "bg-[#695FA4] text-[#FFFBFF] border border-[#B7ADDF] shadow-[0_2px_8px_rgba(105,95,164,0.35)]"
                : "bg-[#2B2450]/50 text-[#D4CCF1] hover:text-[#FFFBFF] border border-[rgba(183,173,223,0.12)]"
            }`}
          >
            Durdurulanlar ({stoppedCount})
          </button>
          <button
            type="button"
            onClick={() => setActiveFilter("external")}
            className={`px-3 py-1.5 rounded-lg text-xs font-medium transition-all cursor-pointer ${
              activeFilter === "external"
                ? "bg-[#695FA4] text-[#FFFBFF] border border-[#B7ADDF] shadow-[0_2px_8px_rgba(105,95,164,0.35)]"
                : "bg-[#2B2450]/50 text-[#D4CCF1] hover:text-[#FFFBFF] border border-[rgba(183,173,223,0.12)]"
            }`}
          >
            Harici APFS ({externalCount})
          </button>
        </div>
      </div>

      {/* Projects Table / Card List */}
      {filteredProjects.length === 0 ? (
        <div className="w-full py-16 text-center rounded-2xl bg-[#2B2450]/40 border border-[rgba(183,173,223,0.14)] flex flex-col items-center justify-center">
          <p className="text-sm text-[#D4CCF1] mb-3">Filtreye uygun proje bulunamadı.</p>
          <button
            type="button"
            onClick={onCreateProject}
            className="text-xs font-semibold text-[#B7ADDF] hover:underline cursor-pointer"
          >
            + Yeni Proje Oluştur
          </button>
        </div>
      ) : (
        <div className="flex flex-col gap-3">
          {filteredProjects.map((project) => {
            const vol = volumes.find((v) => v.uuid === project.volume_uuid) || null;
            const isRunning = project.status === "running";
            const isExternal = vol ? !vol.is_internal || vol.is_removable : false;
            const isMutatingThis = mutatingProject === project.id;

            return (
              <div
                key={project.id}
                className="flex items-center justify-between p-4 rounded-2xl bg-[#2B2450]/65 hover:bg-[#2B2450]/85 border border-[rgba(183,173,223,0.14)] hover:border-[rgba(183,173,223,0.3)] transition-all shadow-[0_8px_24px_rgba(10,6,30,0.4)] group"
              >
                {/* Left: Project Identity */}
                <div className="flex items-center gap-4">
                  <div className="w-10 h-10 rounded-xl bg-[#695FA4]/30 border border-[rgba(183,173,223,0.25)] text-[#FFFBFF] font-['Plus_Jakarta_Sans',sans-serif] font-bold text-base flex items-center justify-center">
                    {project.name.charAt(0).toUpperCase()}
                  </div>

                  <div>
                    <div className="flex items-center gap-2.5">
                      <h3 className="font-['Plus_Jakarta_Sans',sans-serif] font-bold text-base text-[#FFFBFF]">
                        {project.name}
                      </h3>
                      <span className="font-mono text-xs text-[#B7ADDF]">
                        {project.local_domain}
                      </span>
                      {isExternal && (
                        <span className="flex items-center gap-1 font-mono text-[10px] text-[#F59E0B] bg-[#F59E0B]/12 border border-[#F59E0B]/25 px-2 py-0.5 rounded-md">
                          <HardDrive className="w-3 h-3" />
                          <span>Harici</span>
                        </span>
                      )}
                    </div>
                    <p className="font-mono text-xs text-[#D4CCF1]/70 truncate max-w-lg mt-0.5">
                      {project.path}
                    </p>
                  </div>
                </div>

                {/* Center: Tech Specs */}
                <div className="flex items-center gap-4 font-mono text-xs">
                  <div className="flex items-center gap-1.5 px-2.5 py-1 rounded-lg bg-[#191337]/60 border border-[rgba(183,173,223,0.12)] text-[#D4CCF1]">
                    <span className="text-[#B7ADDF] font-semibold">PHP</span>
                    <span>{project.php_version}</span>
                  </div>

                  <div className="flex items-center gap-1.5 px-2.5 py-1 rounded-lg bg-[#191337]/60 border border-[rgba(183,173,223,0.12)] text-[#D4CCF1]">
                    <span>Belge Kökü:</span>
                    <span className="text-[#FFFBFF]">{project.document_root || "public"}</span>
                  </div>

                  <div
                    className={`flex items-center gap-1.5 px-2.5 py-1 rounded-full text-[11px] font-semibold ${
                      isRunning
                        ? "bg-[#10B981]/15 text-[#10B981] border border-[#10B981]/30"
                        : "bg-[#6B7280]/15 text-[#D4CCF1] border border-[#6B7280]/30"
                    }`}
                  >
                    <span
                      className={`w-1.5 h-1.5 rounded-full ${
                        isRunning ? "bg-[#10B981] shadow-[0_0_6px_#10B981]" : "bg-[#6B7280]"
                      }`}
                    />
                    <span>{isRunning ? "çevrimiçi" : "beklemede"}</span>
                  </div>
                </div>

                {/* Right: Actions */}
                <div className="flex items-center gap-2">
                  {isRunning ? (
                    <>
                      <button
                        type="button"
                        onClick={() => onOpenBrowser(project)}
                        className="flex items-center gap-1 px-3 py-1.5 rounded-lg bg-[#2B2450]/60 hover:bg-[#695FA4]/30 border border-[rgba(183,173,223,0.18)] text-xs font-medium text-[#D4CCF1] hover:text-[#FFFBFF] transition-colors cursor-pointer"
                        title="Tarayıcıda Aç"
                      >
                        <ExternalLink className="w-3.5 h-3.5 text-[#B7ADDF]" />
                        <span>Aç</span>
                      </button>
                      <button
                        type="button"
                        onClick={() => onStopProject(project.id)}
                        disabled={isMutatingThis}
                        className="flex items-center gap-1 px-3 py-1.5 rounded-lg bg-[#EF4444]/15 hover:bg-[#EF4444]/25 border border-[#EF4444]/30 text-xs font-semibold text-[#EF4444] transition-colors cursor-pointer disabled:opacity-40"
                      >
                        <Square className="w-3.5 h-3.5 fill-current" />
                        <span>Durdur</span>
                      </button>
                    </>
                  ) : (
                    <button
                      type="button"
                      onClick={() => onStartProject(project.id)}
                      disabled={isMutatingThis}
                      className="flex items-center gap-1 px-3 py-1.5 rounded-lg bg-[#10B981]/15 hover:bg-[#10B981]/25 border border-[#10B981]/30 text-xs font-semibold text-[#10B981] transition-colors cursor-pointer disabled:opacity-40"
                    >
                      <Play className="w-3.5 h-3.5 fill-current" />
                      <span>Başlat</span>
                    </button>
                  )}

                  <button
                    type="button"
                    onClick={() => setSelectedProjectId(project.id)}
                    className="flex items-center gap-1 px-3 py-1.5 rounded-lg bg-gradient-to-r from-[#695FA4] to-[#2B2450] border border-[#B7ADDF] text-xs font-semibold text-[#FFFBFF] hover:brightness-110 shadow-[0_2px_8px_rgba(105,95,164,0.3)] transition-all cursor-pointer"
                  >
                    <Eye className="w-3.5 h-3.5" />
                    <span>Müfettiş</span>
                  </button>
                </div>
              </div>
            );
          })}
        </div>
      )}

      {/* Slide-Over Project Inspector */}
      <ProjectInspector
        project={selectedProject}
        volume={selectedVolume}
        runtime={runtime}
        isOpen={selectedProjectId !== null}
        onClose={() => setSelectedProjectId(null)}
        onStartProject={onStartProject}
        onStopProject={onStopProject}
        onOpenBrowser={onOpenBrowser}
        isMutating={mutatingProject === selectedProjectId}
      />
    </div>
  );
};
