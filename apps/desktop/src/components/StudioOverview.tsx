import React, { useState } from "react";
import { Plus, RefreshCw, ExternalLink } from "lucide-react";
import { ProjectRecord, VolumeInfo, RuntimeOverview } from "../types";
import { SpatialCard } from "./SpatialCard";
import { ProjectInspector } from "./ProjectInspector";
import { computeHeroHealthContent } from "../utils/healthClassification";

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
  const [selectedProjectId, setSelectedProjectId] = useState<string | null>(null);
  const [pageIndex, setPageIndex] = useState(0);
  const runningProjects = projects.filter((p) => p.status === "running");

  // Inspector auto-selection in DEV mode
  React.useEffect(() => {
    if (import.meta.env.DEV && typeof window !== "undefined" && projects.length > 0) {
      const param = new URLSearchParams(window.location.search).get("inspector");
      if (param) {
        const found = projects.find((p) => p.id === param || p.name === param);
        if (found) {
          setSelectedProjectId(found.id);
        }
      }
    }
  }, [projects]);

  const PAGE_SIZE = 6;
  const totalPages = Math.max(1, Math.ceil(projects.length / PAGE_SIZE));
  const safePageIndex = Math.min(pageIndex, totalPages - 1);

  // Keyboard navigation for multi-page collections (ArrowLeft / ArrowRight)
  React.useEffect(() => {
    if (totalPages <= 1) return;

    const handleKeyDown = (e: KeyboardEvent) => {
      if (
        document.activeElement?.tagName === "INPUT" ||
        document.activeElement?.tagName === "TEXTAREA"
      ) {
        return;
      }
      if (e.key === "ArrowLeft") {
        setPageIndex((prev) => Math.max(0, prev - 1));
      } else if (e.key === "ArrowRight") {
        setPageIndex((prev) => Math.min(totalPages - 1, prev + 1));
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [totalPages]);

  const displayedProjects = projects.slice(
    safePageIndex * PAGE_SIZE,
    (safePageIndex + 1) * PAGE_SIZE
  );

  const selectedProject = projects.find((p) => p.id === selectedProjectId) || null;
  const selectedVolume = selectedProject
    ? volumes.find((v) => v.uuid === selectedProject.volume_uuid) || null
    : null;

  const hero = computeHeroHealthContent(projects, volumes, runtime);
  const scaleMode = displayedProjects.length > 4 ? "compact" : "default";

  return (
    <div className="relative w-full flex-1 flex flex-col justify-between items-center select-none pb-20 overflow-hidden">
      {/* Perspective Floor & Reflections */}
      <div
        className="absolute bottom-[-40px] w-[1400px] h-[500px] pointer-events-none z-0"
        style={{
          background:
            "radial-gradient(ellipse at 50% 40%, rgba(59, 55, 106, 0.45) 0%, rgba(43, 36, 80, 0.3) 40%, transparent 75%)",
          transform: "rotateX(65deg)",
        }}
        aria-hidden="true"
      />
      <div
        className="absolute bottom-[100px] w-[900px] h-[140px] pointer-events-none blur-[40px] z-0"
        style={{
          background:
            "radial-gradient(ellipse at center, rgba(105, 95, 164, 0.22) 0%, rgba(183, 173, 223, 0.08) 50%, transparent 80%)",
        }}
        aria-hidden="true"
      />

      {/* Stage Hero Headline */}
      <div className="text-center z-10 pt-4 max-w-2xl px-4">
        <div className="font-mono text-[11px] font-semibold tracking-[0.16em] uppercase text-[#B7ADDF] mb-2">
          Yerel Çalışma Alanı · Apple Silicon APFS
        </div>
        <h1 className="font-['Plus_Jakarta_Sans',sans-serif] text-3xl md:text-4xl font-extrabold tracking-[-0.03em] text-[#FFFBFF] mb-2 leading-tight">
          {hero.prefix} <strong className="text-[#B7ADDF]">{hero.highlight}</strong>
        </h1>
        <p className="text-xs md:text-sm text-[#D4CCF1] max-w-xl mx-auto font-normal leading-relaxed">
          {hero.desc}
        </p>
      </div>

      {/* Stage Monoliths Row */}
      <div className="relative w-full max-w-6xl my-auto py-6 flex flex-col items-center justify-center z-10">
        {projects.length === 0 ? (
          /* 0 Projects: Honest Empty State Wireframe */
          <div
            role="button"
            tabIndex={0}
            onClick={onCreateProject}
            onKeyDown={(e) => {
              if (e.key === "Enter" || e.key === " ") {
                e.preventDefault();
                onCreateProject();
              }
            }}
            style={{
              WebkitBoxReflect:
                "below 10px linear-gradient(to bottom, transparent 65%, rgba(183, 173, 223, 0.08) 100%)",
            }}
            className="w-80 min-h-[300px] rounded-3xl p-8 border-2 border-dashed border-[rgba(183,173,223,0.28)] hover:border-[#B7ADDF] bg-[#2B2450]/20 hover:bg-[#695FA4]/15 backdrop-blur-md transition-all flex flex-col items-center justify-center text-center cursor-pointer shadow-[0_30px_60px_-15px_rgba(10,6,30,0.8)] outline-none focus-visible:ring-2 focus-visible:ring-[#B7ADDF]"
          >
            <div className="w-12 h-12 rounded-2xl bg-[#695FA4]/25 border border-[rgba(183,173,223,0.3)] text-[#B7ADDF] flex items-center justify-center text-xl mb-4">
              <Plus className="w-6 h-6" />
            </div>
            <h3 className="font-['Plus_Jakarta_Sans',sans-serif] text-base font-bold text-[#FFFBFF] mb-1">
              İlk Projeni Oluştur
            </h3>
            <p className="text-xs text-[#D4CCF1] leading-relaxed mb-4 max-w-xs">
              Harici APFS SSD (/Volumes/Mac Harici Disk) veya dahili diskten PHP projesi bağlayın.
            </p>
            <span className="px-4 py-2 rounded-xl bg-gradient-to-r from-[#695FA4] to-[#2B2450] border border-[#B7ADDF] hover:brightness-110 text-[#FFFBFF] text-xs font-semibold shadow-[0_4px_16px_rgba(105,95,164,0.35)]">
              + Proje Oluştur
            </span>
          </div>
        ) : (
          /* Projects Rendered (2, 6, or more with pagination) */
          <div
            className={`flex flex-wrap items-center justify-center z-10 ${
              scaleMode === "compact" ? "gap-4 max-w-5xl" : "gap-7"
            }`}
          >
            {displayedProjects.map((project) => {
              const vol = volumes.find((v) => v.uuid === project.volume_uuid) || null;
              return (
                <SpatialCard
                  key={project.id}
                  project={project}
                  volume={vol}
                  isSelected={selectedProjectId === project.id}
                  onSelect={(p) => setSelectedProjectId(p.id)}
                  scaleMode={scaleMode}
                />
              );
            })}

            {/* Quick Add Slot when space permits on current page and total projects < 6 */}
            {projects.length < 6 && (
              <div
                role="button"
                tabIndex={0}
                onClick={onCreateProject}
                onKeyDown={(e) => {
                  if (e.key === "Enter" || e.key === " ") {
                    e.preventDefault();
                    onCreateProject();
                  }
                }}
                className="w-48 min-h-[280px] rounded-3xl p-5 border-2 border-dashed border-[rgba(183,173,223,0.25)] hover:border-[#B7ADDF] bg-[#2B2450]/20 hover:bg-[#695FA4]/15 backdrop-blur-md transition-all flex flex-col items-center justify-center text-center cursor-pointer outline-none focus-visible:ring-2 focus-visible:ring-[#B7ADDF]"
              >
                <div className="w-9 h-9 rounded-xl bg-[#695FA4]/20 border border-[rgba(183,173,223,0.25)] text-[#B7ADDF] flex items-center justify-center mb-3">
                  <Plus className="w-4 h-4" />
                </div>
                <span className="font-['Plus_Jakarta_Sans',sans-serif] text-xs font-semibold text-[#FFFBFF] mb-1">
                  Proje Ekle
                </span>
                <span className="text-[10px] text-[#D4CCF1] leading-tight">
                  Yeni çalışma alanı bağla
                </span>
              </div>
            )}
          </div>
        )}

        {/* Truthful Total Project Counter & Pagination Navigation for >6 Projects */}
        {projects.length > 6 && (
          <div className="flex items-center justify-between w-full max-w-xl mx-auto mt-6 px-4 py-2.5 rounded-2xl bg-[#2B2450]/80 backdrop-blur-md border border-[rgba(183,173,223,0.18)] text-xs text-[#D4CCF1] shadow-[0_8px_24px_rgba(10,6,30,0.6)] z-20">
            <div className="flex items-center gap-2">
              <span className="font-mono text-[11px] text-[#B7ADDF] font-semibold tracking-wider uppercase">
                Toplam {projects.length} Proje
              </span>
              <span className="text-[#695FA4]">·</span>
              <span className="text-[#D4CCF1]">
                Sayfa {safePageIndex + 1} / {totalPages}
              </span>
            </div>

            <div className="flex items-center gap-2">
              <button
                type="button"
                onClick={() => setPageIndex((p) => Math.max(0, p - 1))}
                disabled={safePageIndex === 0}
                aria-label="Önceki Proje Sayfası"
                className="px-3 py-1 rounded-xl bg-[#191337]/60 hover:bg-[#695FA4]/30 disabled:opacity-30 border border-[rgba(183,173,223,0.15)] text-[#D4CCF1] hover:text-[#FFFBFF] transition-all text-xs font-medium cursor-pointer disabled:cursor-not-allowed outline-none focus-visible:ring-2 focus-visible:ring-[#B7ADDF]"
              >
                ← Önceki
              </button>

              <div className="flex items-center gap-1.5 px-1">
                {Array.from({ length: totalPages }).map((_, idx) => (
                  <button
                    key={idx}
                    type="button"
                    onClick={() => setPageIndex(idx)}
                    aria-label={`Sayfa ${idx + 1}`}
                    className={`h-2 rounded-full transition-all ${
                      idx === safePageIndex
                        ? "bg-[#B7ADDF] w-4 shadow-[0_0_8px_rgba(183,173,223,0.8)]"
                        : "bg-[rgba(183,173,223,0.25)] hover:bg-[rgba(183,173,223,0.45)] w-2"
                    }`}
                  />
                ))}
              </div>

              <button
                type="button"
                onClick={() => setPageIndex((p) => Math.min(totalPages - 1, p + 1))}
                disabled={safePageIndex >= totalPages - 1}
                aria-label="Sonraki Proje Sayfası"
                className="px-3 py-1 rounded-xl bg-[#191337]/60 hover:bg-[#695FA4]/30 disabled:opacity-30 border border-[rgba(183,173,223,0.15)] text-[#D4CCF1] hover:text-[#FFFBFF] transition-all text-xs font-medium cursor-pointer disabled:cursor-not-allowed outline-none focus-visible:ring-2 focus-visible:ring-[#B7ADDF]"
              >
                Sonraki →
              </button>
            </div>
          </div>
        )}
      </div>

      {/* Quick Action Bar at Bottom of Stage */}
      <div className="w-full max-w-3xl flex items-center justify-center gap-3 z-10">
        {runningProjects.length > 0 && (
          <button
            type="button"
            onClick={() => onOpenBrowser(runningProjects[0])}
            className="flex items-center gap-1.5 px-3.5 py-1.5 rounded-xl bg-[#2B2450]/60 hover:bg-[#695FA4]/30 border border-[rgba(183,173,223,0.18)] text-xs font-medium text-[#D4CCF1] hover:text-[#FFFBFF] transition-colors cursor-pointer"
          >
            <ExternalLink className="w-3.5 h-3.5 text-[#B7ADDF]" />
            <span>{runningProjects[0].local_domain} · tarayıcıda aç</span>
          </button>
        )}

        <button
          type="button"
          onClick={onRefresh}
          className="flex items-center gap-1.5 px-3.5 py-1.5 rounded-xl bg-[#2B2450]/60 hover:bg-[#695FA4]/30 border border-[rgba(183,173,223,0.18)] text-xs font-medium text-[#D4CCF1] hover:text-[#FFFBFF] transition-colors cursor-pointer"
        >
          <RefreshCw className="w-3.5 h-3.5 text-[#D4CCF1]" />
          <span>Servisleri Yenile</span>
        </button>

        <button
          type="button"
          onClick={onCreateProject}
          className="flex items-center gap-1.5 px-3.5 py-1.5 rounded-xl bg-gradient-to-r from-[#695FA4] to-[#2B2450] border border-[#B7ADDF] text-xs font-semibold text-[#FFFBFF] hover:brightness-110 shadow-[0_2px_12px_rgba(105,95,164,0.3)] transition-all cursor-pointer"
        >
          <Plus className="w-3.5 h-3.5 text-[#10B981]" />
          <span>Proje Ekle</span>
        </button>
      </div>

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
