import React from "react";
import { Disc, HardDrive, ShieldCheck } from "lucide-react";
import { VolumeInfo } from "../types";

interface DriveStatusBadgeProps {
  volumes: VolumeInfo[];
}

export const DriveStatusBadge: React.FC<DriveStatusBadgeProps> = ({ volumes }) => {
  const formatGb = (bytes: number) => {
    if (bytes === 0) return "0 GB";
    const gb = bytes / (1024 * 1024 * 1024);
    return `${gb.toFixed(0)} GB`;
  };

  return (
    <div className="flex flex-col gap-2.5">
      <div className="flex items-center justify-between text-[11px] font-medium text-neutral-400 tracking-wider uppercase">
        <span className="flex items-center gap-1.5">
          <HardDrive className="w-3.5 h-3.5 text-neutral-400" />
          Depolama ve Donanım
        </span>
        <span className="flex items-center gap-1 text-emerald-400 font-medium">
          <ShieldCheck className="w-3.5 h-3.5" />
          Bariyer Devrede
        </span>
      </div>

      <div className="grid grid-cols-1 gap-2">
        {volumes.map((vol) => {
          const isExternal = !vol.is_internal || vol.mount_point.startsWith("/Volumes");
          const totalGb = vol.total_bytes / (1024 * 1024 * 1024);
          const freeGb = vol.available_bytes / (1024 * 1024 * 1024);
          const usedPercent = totalGb > 0 ? Math.min(100, Math.max(0, ((totalGb - freeGb) / totalGb) * 100)) : 0;

          return (
            <div
              key={vol.uuid}
              className={`p-3 rounded-xl border transition-all ${
                isExternal
                  ? "bg-gradient-to-br from-amber-500/10 via-neutral-900/60 to-neutral-950/80 border-amber-500/25 shadow-sm"
                  : "bg-neutral-900/40 border-white/[0.06]"
              }`}
            >
              <div className="flex items-start justify-between mb-2">
                <div className="flex items-center gap-2">
                  <div
                    className={`w-7 h-7 rounded-lg flex items-center justify-center ${
                      isExternal
                        ? "bg-amber-500/20 text-amber-400 border border-amber-500/30"
                        : "bg-blue-500/15 text-blue-400 border border-blue-500/20"
                    }`}
                  >
                    {isExternal ? (
                      <Disc className="w-4 h-4 animate-spin-slow" />
                    ) : (
                      <HardDrive className="w-4 h-4" />
                    )}
                  </div>
                  <div>
                    <div className="flex items-center gap-1.5">
                      <span className="text-xs font-semibold text-white tracking-tight">{vol.name}</span>
                      {isExternal && (
                        <span className="px-1.5 py-0.5 rounded-full text-[9px] font-bold uppercase tracking-wider bg-amber-500/20 text-amber-300 border border-amber-500/30">
                          Harici SSD
                        </span>
                      )}
                    </div>
                    <span className="text-[10px] text-neutral-400 font-mono">
                      {vol.filesystem.toUpperCase()} • {vol.mount_point}
                    </span>
                  </div>
                </div>

                <div className="text-right">
                  <span className="text-xs font-semibold text-neutral-200">
                    {formatGb(vol.available_bytes)} boş
                  </span>
                  <span className="block text-[10px] text-neutral-400">
                    / {formatGb(vol.total_bytes)}
                  </span>
                </div>
              </div>

              {/* Apple-style Segmented Capacity Bar */}
              <div className="w-full h-1.5 bg-neutral-800 rounded-full overflow-hidden p-[1px]">
                <div
                  className={`h-full rounded-full transition-all duration-500 ${
                    isExternal
                      ? "bg-gradient-to-r from-amber-400 to-orange-500"
                      : "bg-gradient-to-r from-blue-500 to-indigo-500"
                  }`}
                  style={{ width: `${usedPercent}%` }}
                />
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
};
