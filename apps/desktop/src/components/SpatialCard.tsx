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

  const getStatusBadge = () => {
    switch (project.status) {
      case "running":
        return {
          dot: "bg-[#10B981] shadow-[0_0_8px_#10B981]",
          text: "text-[#10B981]",
          bg: "bg-[#10B981]/12 border-[#10B981]/30",
          label: "çevrimiçi",
        };
      case "failed":
        return {
          dot: "bg-[#EF4444] shadow-[0_0_8px_#EF4444]",
          text: "text-[#EF4444]",
          bg: "bg-[#EF4444]/12 border-[#EF4444]/30",
          label: "arızalı",
        };
      case "degraded":
        return {
          dot: "bg-[#F59E0B] shadow-[0_0_8px_#F59E0B]",
          text: "text-[#F59E0B]",
          bg: "bg-[#F59E0B]/12 border-[#F59E0B]/30",
          label: "bozulmuş",
        };
      case "disconnected":
        return {
          dot: "bg-[#F59E0B] shadow-[0_0_8px_#F59E0B]",
          text: "text-[#F59E0B]",
          bg: "bg-[#F59E0B]/12 border-[#F59E0B]/30",
          label: "bağlantı kesik",
        };
      case "starting":
        return {
          dot: "bg-[#60A5FA] shadow-[0_0_8px_#60A5FA]",
          text: "text-[#60A5FA]",
          bg: "bg-[#60A5FA]/12 border-[#60A5FA]/30",
          label: "başlatılıyor",
        };
      case "stopping":
        return {
          dot: "bg-[#9CA3AF]",
          text: "text-[#9CA3AF]",
          bg: "bg-[#9CA3AF]/12 border-[#9CA3AF]/30",
          label: "durduruluyor",
        };
      case "unknown":
        return {
          dot: "bg-[#6B7280]",
          text: "text-[#D4CCF1]",
          bg: "bg-[#6B7280]/14 border-[#6B7280]/30",
          label: "bilinmiyor",
        };
      case "stopped":
      default:
        return {
          dot: "bg-[#6B7280]",
          text: "text-[#D4CCF1]",
          bg: "bg-[#6B7280]/14 border-[#6B7280]/30",
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
  const cleanName = project.name.toLowerCase().replace(/[^a-z0-9]/g, "_");

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
          "below 10px linear-gradient(to bottom, transparent 65%, rgba(183, 173, 223, 0.08) 100%)",
        background:
          "linear-gradient(180deg, rgba(43, 36, 80, 0.72) 0%, rgba(25, 19, 55, 0.85) 100%)",
      }}
      className={`relative select-none transition-all duration-300 ease-out cursor-pointer outline-none focus-visible:ring-2 focus-visible:ring-[#B7ADDF] backdrop-blur-2xl border border-[rgba(183,173,223,0.18)] border-t-[1.5px] border-t-[rgba(183,173,223,0.45)] shadow-[0_30px_60px_-15px_rgba(10,6,30,0.8),inset_0_1px_0_rgba(183,173,223,0.35)] hover:border-[#B7ADDF] hover:shadow-[0_40px_80px_-20px_rgba(10,6,30,0.9),0_0_30px_rgba(105,95,164,0.25)] ${
        isCompact ? "w-44 min-h-[260px] p-4 rounded-2xl" : "w-64 min-h-[320px] p-6 rounded-3xl"
      } ${
        isSelected ? "ring-2 ring-[#B7ADDF] shadow-[0_0_36px_rgba(105,95,164,0.45)]" : ""
      }`}
    >
      {/* Monolith Header */}
      <div className="flex items-start justify-between gap-2 mb-4">
        <div className="truncate">
          <h3
            className={`font-['Plus_Jakarta_Sans',sans-serif] font-bold text-[#FFFBFF] tracking-[-0.02em] truncate ${
              isCompact ? "text-base" : "text-xl"
            }`}
            title={project.name}
          >
            {project.name}
          </h3>
          <span className="font-mono text-[10px] text-[#B7ADDF] truncate block">
            {project.local_domain}
          </span>
        </div>
        <div
          className={`flex items-center gap-1.5 font-mono text-[11px] font-semibold px-2 py-0.5 rounded-full border ${statusBadge.bg} ${statusBadge.text}`}
        >
          <span className={`w-1.5 h-1.5 rounded-full ${statusBadge.dot}`} />
          <span>{statusBadge.label}</span>
        </div>
      </div>

      {/* Nested Sub-Component Pills */}
      <div className="flex flex-col gap-2 mb-4">
        {/* Nginx / Web Root Pill */}
        <div className="flex items-center justify-between p-2 rounded-xl bg-[#191337]/55 border border-[rgba(183,173,223,0.12)] text-xs text-[#D4CCF1]">
          <div className="flex items-center gap-2 truncate">
            <span className="w-5 h-5 rounded-md font-['Plus_Jakarta_Sans',sans-serif] text-[10px] font-bold bg-[#695FA4] text-[#FFFBFF] flex items-center justify-center">
              {project.name.charAt(0).toUpperCase()}
            </span>
            <span className="truncate">{project.document_root || "public"}</span>
          </div>
          <div className="flex items-center gap-1 font-mono text-[10px] text-[#B7ADDF]">
            <span>:8080</span>
          </div>
        </div>

        {/* PHP Version Pill */}
        <div className="flex items-center justify-between p-2 rounded-xl bg-[#191337]/55 border border-[rgba(183,173,223,0.12)] text-xs text-[#D4CCF1]">
          <div className="flex items-center gap-2 truncate">
            <span className="w-5 h-5 rounded-md font-mono text-[9px] font-bold bg-[#2B2450] border border-[#B7ADDF] text-[#B7ADDF] flex items-center justify-center">
              PHP
            </span>
            <span className="font-mono text-[11px] text-[#FFFBFF]">
              {project.php_version}
            </span>
          </div>
          <span
            className={`w-1.5 h-1.5 rounded-full ${
              isRunning ? "bg-[#10B981] shadow-[0_0_6px_#10B981]" : "bg-[#6B7280]"
            }`}
          />
        </div>

        {/* MariaDB Isolated DB Pill */}
        <div className="flex items-center justify-between p-2 rounded-xl bg-[#191337]/55 border border-[rgba(183,173,223,0.12)] text-xs text-[#D4CCF1]">
          <div className="flex items-center gap-2 truncate">
            <span className="w-5 h-5 rounded-md font-mono text-[9px] font-bold bg-[#059669] text-white flex items-center justify-center">
              DB
            </span>
            <span className="font-mono text-[10px] text-[#D4CCF1] truncate">
              {cleanName}
            </span>
          </div>
          <span className="font-mono text-[10px] text-[#B7ADDF]">:3306</span>
        </div>
      </div>

      {/* Monolith Footer with Inspect Button */}
      <div className="pt-3 border-t border-[rgba(183,173,223,0.10)] flex items-center justify-between">
        <span
          className={`font-mono text-[10px] truncate max-w-[120px] ${
            isExternal ? "text-[#F59E0B] font-medium" : "text-[#D4CCF1]"
          }`}
          title={volume?.name || "APFS"}
        >
          {volume?.name || "APFS"}
        </span>
        <button
          type="button"
          onClick={(e) => {
            e.stopPropagation();
            onSelect(project);
          }}
          className="bg-gradient-to-r from-[#695FA4] to-[#2B2450] border border-[#B7ADDF] text-[#FFFBFF] font-['Plus_Jakarta_Sans',sans-serif] font-semibold text-[11px] px-3 py-1.5 rounded-lg shadow-[0_2px_8px_rgba(105,95,164,0.3)] hover:brightness-110 transition-all cursor-pointer"
        >
          Müfettiş ↗
        </button>
      </div>
    </div>
  );
};
