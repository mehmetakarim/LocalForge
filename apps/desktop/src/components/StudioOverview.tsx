import React, { useState } from "react";
import { Plus, RefreshCw, ExternalLink } from "lucide-react";
import { ProjectRecord, VolumeInfo, RuntimeOverview } from "../types";
import { SpatialCard } from "./SpatialCard";
import { ProjectInspector } from "./ProjectInspector";

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

import { computeHeroHealthContent } from "../utils/healthClassification";

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

  // Inspector auto-selection: only active in DEV mode
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
    <div className="relative min-h-[calc(100vh-140px)] flex flex-col justify-between items-center select-none pb-24">
      {/* Perspective Grid Plane */}
      <div
        className="absolute inset-x-[-100px] top-[140px] bottom-[120px] pointer-events-none opacity-25 z-0"
        style={{
          backgroundImage:
            "radial-gradient(circle, rgba(255, 255, 255, 0.12) 1px, transparent 1px)",
          backgroundSize: "36px 36px",
          transform: "rotateX(60deg)",
          transformOrigin: "center bottom",
          maskImage: "radial-gradient(ellipse at center, rgba(0,0,0,1) 30%, transparent 75%)",
          WebkitMaskImage: "radial-gradient(ellipse at center, rgba(0,0,0,1) 30%, transparent 75%)",
        }}
        aria-hidden="true"
      />

      {/* Centered Hero Heading */}
      <div className="text-center z-10 pt-4">
        <div className="font-mono text-[11px] tracking-widest text-zinc-400 uppercase mb-2">
          Yerel Çalışma Alanı · Apple Silicon APFS
        </div>
        <h1 className="text-4xl md:text-5xl font-light tracking-tight text-white mb-2">
          {hero.prefix} <strong className="font-bold text-white">{hero.highlight}</strong>
        </h1>
        <p className="text-xs md:text-sm text-zinc-400 max-w-xl mx-auto font-normal">
          {hero.desc}
        </p>
      </div>

      {/* Spatial 3D Stage */}
      <div className="relative w-full max-w-5xl my-auto py-8 flex flex-col items-center justify-center z-10">
        {/* Connecting Conduits Lines */}
        <svg
          className="absolute inset-0 w-full h-full pointer-events-none z-0"
          viewBox="0 0 1000 400"
          preserveAspectRatio="none"
        >
          <path
            d="M 280 230 Q 280 320 500 350"
            fill="none"
            stroke="rgba(255,255,255,0.10)"
            strokeDasharray="3,3"
            strokeWidth="1.2"
          />
          <path
            d="M 720 230 Q 720 320 500 350"
            fill="none"
            stroke="rgba(255,255,255,0.10)"
            strokeDasharray="3,3"
            strokeWidth="1.2"
          />
          <line
            x1="500"
            y1="230"
            x2="500"
            y2="350"
            stroke="rgba(255,255,255,0.10)"
            strokeDasharray="3,3"
            strokeWidth="1.2"
          />
        </svg>

        {/* Spatial Monoliths Row */}
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
                "below 8px linear-gradient(to bottom, transparent 70%, rgba(255, 255, 255, 0.05) 100%)",
            }}
            className="w-72 min-h-[280px] rounded-3xl p-8 border-2 border-dashed border-white/20 hover:border-indigo-400/50 bg-white/[0.015] hover:bg-white/[0.04] transition-all flex flex-col items-center justify-center text-center cursor-pointer shadow-[0_32px_64px_-16px_rgba(0,0,0,0.6)] outline-none focus-visible:ring-2 focus-visible:ring-indigo-400"
          >
            <div className="w-12 h-12 rounded-full bg-indigo-500/15 border border-indigo-400/35 text-indigo-400 flex items-center justify-center text-xl mb-4">
              <Plus className="w-6 h-6" />
            </div>
            <h3 className="text-base font-bold text-white mb-1">İlk Projeni Oluştur</h3>
            <p className="text-xs text-zinc-400 leading-relaxed mb-4">
              Harici APFS SSD (Mac Harici Disk) veya dahili diskten PHP projesi bağla.
            </p>
            <span className="px-4 py-2 rounded-xl bg-indigo-600 hover:bg-indigo-500 text-white text-xs font-semibold shadow-[0_0_16px_rgba(79,110,247,0.4)]">
              + Proje Oluştur
            </span>
          </div>
        ) : (
          /* Projects Rendered (2, 6, or more with pagination) */
          <div
            className={`flex flex-wrap items-center justify-center z-10 ${
              scaleMode === "compact" ? "gap-4 max-w-5xl" : "gap-8"
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
                className="w-44 min-h-[250px] rounded-3xl p-5 border-2 border-dashed border-white/15 hover:border-white/35 bg-white/[0.015] hover:bg-white/[0.04] transition-all flex flex-col items-center justify-center text-center cursor-pointer outline-none focus-visible:ring-2 focus-visible:ring-indigo-400"
              >
                <div className="w-8 h-8 rounded-full bg-white/5 text-zinc-400 flex items-center justify-center mb-3">
                  <Plus className="w-4 h-4" />
                </div>
                <span className="text-xs font-semibold text-white mb-1">Proje Ekle</span>
                <span className="text-[10px] text-zinc-500 leading-tight">
                  Yeni çalışma alanı bağla
                </span>
              </div>
            )}
          </div>
        )}

        {/* Truthful Total Project Counter & Pagination Navigation for >6 Projects */}
        {projects.length > 6 && (
          <div className="flex items-center justify-between w-full max-w-xl mx-auto mt-6 px-4 py-2.5 rounded-2xl bg-[#0F1424]/80 backdrop-blur-md border border-white/10 text-xs text-zinc-300 shadow-[0_8px_24px_rgba(0,0,0,0.5)] z-20">
            <div className="flex items-center gap-2">
              <span className="font-mono text-[11px] text-indigo-400 font-semibold tracking-wider uppercase">
                Toplam {projects.length} Proje
              </span>
              <span className="text-zinc-600">·</span>
              <span className="text-zinc-400">
                Sayfa {safePageIndex + 1} / {totalPages}
              </span>
            </div>

            <div className="flex items-center gap-2">
              <button
                type="button"
                onClick={() => setPageIndex((p) => Math.max(0, p - 1))}
                disabled={safePageIndex === 0}
                aria-label="Önceki Proje Sayfası"
                className="px-3 py-1 rounded-xl bg-white/5 hover:bg-white/10 disabled:opacity-30 disabled:hover:bg-white/5 border border-white/10 text-zinc-300 hover:text-white transition-all text-xs font-medium cursor-pointer disabled:cursor-not-allowed outline-none focus-visible:ring-2 focus-visible:ring-indigo-400"
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
                    className={`w-2 h-2 rounded-full transition-all ${
                      idx === safePageIndex
                        ? "bg-indigo-400 w-4 shadow-[0_0_8px_rgba(99,102,241,0.8)]"
                        : "bg-white/20 hover:bg-white/40"
                    }`}
                  />
                ))}
              </div>

              <button
                type="button"
                onClick={() => setPageIndex((p) => Math.min(totalPages - 1, p + 1))}
                disabled={safePageIndex >= totalPages - 1}
                aria-label="Sonraki Proje Sayfası"
                className="px-3 py-1 rounded-xl bg-white/5 hover:bg-white/10 disabled:opacity-30 disabled:hover:bg-white/5 border border-white/10 text-zinc-300 hover:text-white transition-all text-xs font-medium cursor-pointer disabled:cursor-not-allowed outline-none focus-visible:ring-2 focus-visible:ring-indigo-400"
              >
                Sonraki →
              </button>
            </div>
          </div>
        )}

        {/* Bu Mac Central Node */}
        <div className="mt-8 flex items-center gap-3 px-6 py-2.5 rounded-2xl bg-[#121826]/70 border border-white/10 backdrop-blur-xl shadow-xl z-10">
          <svg
            className="w-5 h-5 text-indigo-300"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="1.8"
          >
            <rect x="2" y="3" width="20" height="14" rx="2" />
            <line x1="8" y1="21" x2="16" y2="21" />
            <line x1="12" y1="17" x2="12" y2="21" />
          </svg>
          <div className="flex flex-col text-left">
            <span className="text-xs font-semibold text-white">Bu Mac (Apple Silicon)</span>
            <span className="text-[10px] font-mono text-zinc-400">
              Loopback 127.0.0.1 · {volumes.length} APFS Birimi Bağlı
            </span>
          </div>
        </div>
      </div>

      {/* Bottom Controls Deck */}
      <div className="w-full max-w-3xl flex flex-col items-center gap-3.5 z-10">

        {/* Quick Action Pills */}
        <div className="flex items-center gap-2.5">
          {runningProjects.length > 0 && (
            <button
              type="button"
              onClick={() => onOpenBrowser(runningProjects[0])}
              className="flex items-center gap-1.5 px-3 py-1.5 rounded-xl bg-white/5 hover:bg-white/10 border border-white/10 text-xs font-medium text-zinc-300 hover:text-white transition-colors"
            >
              <ExternalLink className="w-3.5 h-3.5 text-indigo-400" />
              <span>{runningProjects[0].local_domain} · tarayıcıda aç</span>
            </button>
          )}

          <button
            type="button"
            onClick={onRefresh}
            className="flex items-center gap-1.5 px-3 py-1.5 rounded-xl bg-white/5 hover:bg-white/10 border border-white/10 text-xs font-medium text-zinc-300 hover:text-white transition-colors"
          >
            <RefreshCw className="w-3.5 h-3.5 text-zinc-400" />
            <span>Servisleri Yenile</span>
          </button>

          <button
            type="button"
            onClick={onCreateProject}
            className="flex items-center gap-1.5 px-3 py-1.5 rounded-xl bg-white/5 hover:bg-white/10 border border-white/10 text-xs font-medium text-zinc-300 hover:text-white transition-colors"
          >
            <Plus className="w-3.5 h-3.5 text-emerald-400" />
            <span>Proje Ekle</span>
          </button>
        </div>
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
