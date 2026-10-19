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
  const isExternal = volume ? volume.is_removable || !volume.is_internal : false;

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
      } ${
        isRunning
          ? "bg-gradient-to-b from-[#182034]/75 via-[#0C101C]/85 to-[#10B981]/20 border-t-[1.5px] border-t-white/30 border-x border-b border-white/10 shadow-[0_32px_64px_-16px_rgba(0,0,0,0.8),0_0_32px_-4px_rgba(16,185,129,0.2),inset_0_1px_0_rgba(255,255,255,0.2)]"
          : "bg-gradient-to-b from-[#161C2C]/70 via-[#0C101A]/85 to-[#64748B]/15 border-t-[1.5px] border-t-white/20 border-x border-b border-white/10 shadow-[0_32px_64px_-16px_rgba(0,0,0,0.8),inset_0_1px_0_rgba(255,255,255,0.15)]"
      } ${isSelected ? "ring-2 ring-indigo-400/80 shadow-[0_0_36px_rgba(79,110,247,0.35)]" : ""}`}
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
          <span
            className={`w-1.5 h-1.5 rounded-full ${
              isRunning
                ? "bg-emerald-400 shadow-[0_0_8px_rgba(52,211,153,0.9)]"
                : "bg-zinc-500"
            }`}
          />
          <span className={isRunning ? "text-emerald-400" : "text-zinc-400"}>
            {isRunning ? "çevrimiçi" : "beklemede"}
          </span>
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
