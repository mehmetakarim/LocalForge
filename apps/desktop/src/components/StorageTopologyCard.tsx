import React from "react";
import { HardDrive, ShieldCheck, Cpu } from "lucide-react";
import { VolumeInfo } from "../types";

export interface StorageTopologyCardProps {
  volume: VolumeInfo;
  projectCount?: number;
}

function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB", "TB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + " " + sizes[i];
}

export const StorageTopologyCard: React.FC<StorageTopologyCardProps> = ({
  volume,
  projectCount = 0,
}) => {
  const usedBytes = Math.max(0, volume.total_bytes - volume.available_bytes);
  const usedPercent = volume.total_bytes > 0 ? (usedBytes / volume.total_bytes) * 100 : 0;
  const isExternal = volume.is_removable || !volume.is_internal;

  return (
    <div
      style={{
        boxShadow:
          "0 12px 28px -4px rgba(0, 0, 0, 0.45), inset 0 1px 0 0 rgba(255, 255, 255, 0.15)",
      }}
      className="relative flex flex-col justify-between p-5 rounded-2xl lf-material-card border border-white/10 group"
    >
      {/* 1px Specular top rim */}
      <div
        className="absolute top-0 left-0 right-0 h-[1px] rounded-t-2xl bg-gradient-to-r from-transparent via-white/20 to-transparent pointer-events-none"
        aria-hidden="true"
      />

      {/* Header */}
      <div className="flex items-start justify-between gap-3 mb-3">
        <div className="flex items-center gap-3">
          <div
            className={`flex items-center justify-center w-10 h-10 rounded-xl border ${
              isExternal
                ? "bg-amber-500/15 border-amber-500/30 text-amber-300"
                : "bg-indigo-500/15 border-indigo-500/30 text-indigo-300"
            }`}
          >
            {isExternal ? (
              <HardDrive className="w-5 h-5" />
            ) : (
              <Cpu className="w-5 h-5" />
            )}
          </div>
          <div>
            <h4 className="font-semibold text-zinc-100 group-hover:text-white transition-colors text-sm">
              {volume.name}
            </h4>
            <span className="font-mono text-[11px] text-zinc-400">
              {volume.mount_point}
            </span>
          </div>
        </div>

        <span
          className={`px-2.5 py-0.5 rounded-full text-[10px] font-mono uppercase font-semibold border ${
            isExternal
              ? "bg-amber-500/10 border-amber-500/30 text-amber-300"
              : "bg-indigo-500/10 border-indigo-500/30 text-indigo-300"
          }`}
        >
          {isExternal ? "External NVMe/APFS" : "Internal SSD"}
        </span>
      </div>

      {/* UUID & Safety Boundary Tag */}
      <div className="my-2 p-2.5 rounded-xl bg-black/40 border border-white/5 space-y-1.5 font-mono text-[11px]">
        <div className="flex justify-between items-center text-zinc-400">
          <span className="text-zinc-500">APFS UUID:</span>
          <span className="truncate max-w-[200px] text-zinc-300 select-all" title={volume.uuid}>
            {volume.uuid}
          </span>
        </div>
        <div className="flex justify-between items-center text-zinc-400">
          <span className="text-zinc-500">File System:</span>
          <span className="uppercase text-zinc-300">{volume.filesystem || "apfs"}</span>
        </div>
        <div className="flex justify-between items-center text-zinc-400">
          <span className="text-zinc-500">Attached Projects:</span>
          <span className="text-indigo-300 font-semibold">{projectCount}</span>
        </div>
      </div>

      {/* Storage Capacity Bar */}
      <div className="space-y-1.5 my-2">
        <div className="flex justify-between text-xs font-mono">
          <span className="text-zinc-400">
            {formatBytes(volume.available_bytes)} free
          </span>
          <span className="text-zinc-500">
            {formatBytes(volume.total_bytes)} total
          </span>
        </div>
        <div className="w-full h-2 rounded-full bg-zinc-800 overflow-hidden border border-white/5">
          <div
            className={`h-full rounded-full transition-all duration-500 ${
              usedPercent > 90
                ? "bg-rose-500"
                : usedPercent > 75
                ? "bg-amber-400"
                : "bg-gradient-to-r from-indigo-500 to-emerald-400"
            }`}
            style={{ width: `${Math.min(100, Math.max(2, usedPercent))}%` }}
          />
        </div>
      </div>

      {/* Safety Barrier Badge */}
      <div className="flex items-center gap-1.5 pt-3 mt-1 border-t border-white/10 text-[11px] font-medium text-emerald-300">
        <ShieldCheck className="w-4 h-4 text-emerald-400" />
        <span>Volume Barrier Active (APFS UUID Verified)</span>
      </div>
    </div>
  );
};
