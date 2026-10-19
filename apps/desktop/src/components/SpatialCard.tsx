import React, { useState } from "react";
import { ProjectRecord, VolumeInfo } from "../types";

export interface SpatialCardProps {
  project: ProjectRecord;
  volume: VolumeInfo | null;
  isSelected?: boolean;
  onSelect: (project: ProjectRecord) => void;
  scaleMode?: "default" | "compact";
}

export const SpatialCard: React.FC<SpatialCardProps> = ({
  project,
  volume,
  isSelected = false,
  onSelect,
  scaleMode = "default",
}) => {
  const [tilt, setTilt] = useState({ x: 8, y: 0, lift: -8 });

  const isRunning = project.status === "running";
  const isFailed = project.status === "failed";
  const isDegraded = project.status === "degraded";
  const isDisconnected = project.status === "disconnected";
  const isExternal = volume ? volume.is_removable || !volume.is_internal : false;

  const getStatusBadge = () => {
    switch (project.status) {
      case "running":
        return {
          dot: "bg-emerald-400 shadow-[0_0_8px_rgba(52,211,153,0.9)]",
          text: "text-emerald-400",
          label: "çevrimiçi",
        };
      case "failed":
        return {
          dot: "bg-rose-500 shadow-[0_0_8px_rgba(244,63,94,0.9)]",
          text: "text-rose-400",
          label: "arızalı",
        };
      case "degraded":
        return {
          dot: "bg-amber-400 shadow-[0_0_8px_rgba(251,191,36,0.8)]",
          text: "text-amber-400",
          label: "bozulmuş",
        };
      case "disconnected":
        return {
          dot: "bg-amber-500 shadow-[0_0_8px_rgba(245,158,11,0.8)]",
          text: "text-amber-400",
          label: "bağlantı kesik",
        };
      case "starting":
        return {
          dot: "bg-blue-400 shadow-[0_0_8px_rgba(96,165,250,0.8)]",
          text: "text-blue-400",
          label: "başlatılıyor",
        };
      case "stopping":
        return {
          dot: "bg-blue-400 shadow-[0_0_8px_rgba(96,165,250,0.8)]",
          text: "text-blue-400",
          label: "durduruluyor",
        };
      case "unknown":
        return {
          dot: "bg-zinc-400",
          text: "text-zinc-400",
          label: "bilinmiyor",
        };
      case "stopped":
      default:
        return {
          dot: "bg-zinc-500",
          text: "text-zinc-400",
          label: "beklemede",
        };
    }
  };

  const statusBadge = getStatusBadge();

  const handleMouseMove = (e: React.MouseEvent<HTMLDivElement>) => {
    if (window.matchMedia("(prefers-reduced-motion: reduce)").matches) {
      return;
    }
    const rect = e.currentTarget.getBoundingClientRect();
    const x = (e.clientX - rect.left) / rect.width - 0.5;
    const y = (e.clientY - rect.top) / rect.height - 0.5;
    setTilt({
      x: 8 + -y * 6,
      y: x * 6,
      lift: -16,
    });
  };

  const handleMouseLeave = () => {
    setTilt({ x: 8, y: 0, lift: -8 });
  };

  const isCompact = scaleMode === "compact";

  const getCardToneClasses = () => {
    if (isFailed || isDegraded) {
      return "bg-gradient-to-b from-[#24141E]/75 via-[#120A10]/85 to-[#F43F5E]/20 border-t-[1.5px] border-t-rose-400/40 border-x border-b border-rose-500/20 shadow-[0_32px_64px_-16px_rgba(0,0,0,0.8),0_0_32px_-4px_rgba(244,63,94,0.2),inset_0_1px_0_rgba(255,255,255,0.2)]";
    }
    if (isDisconnected) {
      return "bg-gradient-to-b from-[#241C14]/75 via-[#140F0A]/85 to-[#F59E0B]/20 border-t-[1.5px] border-t-amber-400/40 border-x border-b border-amber-500/20 shadow-[0_32px_64px_-16px_rgba(0,0,0,0.8),0_0_32px_-4px_rgba(245,158,11,0.2),inset_0_1px_0_rgba(255,255,255,0.2)]";
    }
    if (isRunning) {
      return "bg-gradient-to-b from-[#182034]/75 via-[#0C101C]/85 to-[#10B981]/20 border-t-[1.5px] border-t-white/30 border-x border-b border-white/10 shadow-[0_32px_64px_-16px_rgba(0,0,0,0.8),0_0_32px_-4px_rgba(16,185,129,0.2),inset_0_1px_0_rgba(255,255,255,0.2)]";
    }
    return "bg-gradient-to-b from-[#161C2C]/70 via-[#0C101A]/85 to-[#64748B]/15 border-t-[1.5px] border-t-white/20 border-x border-b border-white/10 shadow-[0_32px_64px_-16px_rgba(0,0,0,0.8),inset_0_1px_0_rgba(255,255,255,0.15)]";
  };

  return (
    <div
      role="button"
      tabIndex={0}
      aria-label={`${project.name} - ${project.status}`}
      onClick={() => onSelect(project)}
      onKeyDown={(e) => {
        if (e.key === "Enter" || e.key === " ") {
          e.preventDefault();
          onSelect(project);
        }
      }}
      onMouseMove={handleMouseMove}
      onMouseLeave={handleMouseLeave}
      style={{
        transform: `perspective(1400px) rotateX(${tilt.x}deg) rotateY(${tilt.y}deg) translateY(${tilt.lift}px)`,
        WebkitBoxReflect:
          "below 8px linear-gradient(to bottom, transparent 65%, rgba(255, 255, 255, 0.08) 100%)",
      }}
      className={`relative select-none transition-all duration-300 ease-out cursor-pointer outline-none focus-visible:ring-2 focus-visible:ring-indigo-400 ${
        isCompact ? "w-40 min-h-[230px] p-4 rounded-2xl" : "w-64 min-h-[290px] p-6 rounded-3xl"
      } ${getCardToneClasses()} ${
        isSelected ? "ring-2 ring-indigo-400/80 shadow-[0_0_36px_rgba(79,110,247,0.35)]" : ""
      }`}
    >
      {/* Monolith Header */}
      <div className="flex flex-col gap-1 mb-5">
        <h3
          className={`font-bold text-white tracking-tight truncate ${
            isCompact ? "text-sm" : "text-lg"
          }`}
          title={project.name}
        >
          {project.name}
        </h3>
        <div className="flex items-center gap-1.5 font-mono text-[11px]">
          <span className={`w-1.5 h-1.5 rounded-full ${statusBadge.dot}`} />
          <span className={statusBadge.text}>{statusBadge.label}</span>
        </div>
      </div>

      {/* Nested Sub-Component Pills */}
      <div className="flex flex-col gap-2.5">
        <div className="flex items-center justify-between p-2 rounded-xl bg-black/35 border border-white/5 text-xs text-zinc-200">
          <div className="flex items-center gap-2 truncate">
            <span
              className={`w-4 h-4 rounded-md font-bold text-[10px] flex items-center justify-center ${
                isExternal ? "bg-amber-400 text-black" : "bg-indigo-500 text-white"
              }`}
            >
              {project.name.charAt(0).toUpperCase()}
            </span>
            <span className="truncate">{project.document_root || "public"}</span>
          </div>
          <div className="flex items-center gap-1">
            <span
              className={`w-1.5 h-1.5 rounded-full ${
                isRunning ? "bg-emerald-400" : "bg-zinc-600"
              }`}
            />
            <span
              className={`w-1.5 h-1.5 rounded-full ${
                isRunning ? "bg-emerald-400" : "bg-zinc-600"
              }`}
            />
          </div>
        </div>

        <div className="flex items-center justify-between p-2 rounded-xl bg-black/35 border border-white/5 text-xs text-zinc-300">
          <div className="flex items-center gap-2 truncate">
            <span className="w-4 h-4 rounded-md font-mono text-[9px] font-bold bg-emerald-500 text-black flex items-center justify-center">
              P
            </span>
            <span className="font-mono text-[11px] text-zinc-300">
              PHP {project.php_version}
            </span>
          </div>
          <div className="flex items-center gap-1">
            <span
              className={`w-1.5 h-1.5 rounded-full ${
                isRunning ? "bg-emerald-400" : "bg-zinc-600"
              }`}
            />
          </div>
        </div>
      </div>

      {/* Drive Badge Hint at Bottom */}
      <div className="absolute bottom-3 left-4 right-4 flex items-center justify-between text-[10px] font-mono text-zinc-500 pt-2 border-t border-white/5">
        <span className={isExternal ? "text-amber-400/90 font-medium truncate" : "truncate"}>
          {volume?.name || "APFS"}
        </span>
        <span className="text-zinc-600 text-[9px]">Detay ↗</span>
      </div>
    </div>
  );
};
