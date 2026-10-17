import React from "react";
import { HardDrive, Disc } from "lucide-react";
import { VolumeInfo } from "../types";

interface DriveStatusBadgeProps {
  volumes: VolumeInfo[];
}

export const DriveStatusBadge: React.FC<DriveStatusBadgeProps> = ({ volumes }) => {
  const formatBytes = (bytes: number) => {
    if (bytes === 0) return "0 GB";
    const gb = bytes / (1024 * 1024 * 1024);
    return `${gb.toFixed(1)} GB`;
  };

  return (
    <div className="flex flex-wrap items-center gap-2">
      {volumes.map((vol) => {
        const isExternal = vol.mount_point.startsWith("/Volumes");
        const freeGb = formatBytes(vol.available_bytes);

        return (
          <div
            key={vol.uuid}
            className={`flex items-center gap-2 px-3 py-1.5 rounded-lg border text-xs font-medium transition-all ${
              isExternal
                ? "bg-amber-950/30 border-amber-800/50 text-amber-200"
                : "bg-slate-900 border-slate-800 text-slate-300"
            }`}
            title={`Mount: ${vol.mount_point}\nUUID: ${vol.uuid}\nFS: ${vol.filesystem}`}
          >
            {isExternal ? (
              <Disc className="w-3.5 h-3.5 text-amber-400 animate-pulse" />
            ) : (
              <HardDrive className="w-3.5 h-3.5 text-indigo-400" />
            )}
            <span className="font-semibold">{vol.name}</span>
            <span className="text-slate-400">({freeGb} free)</span>
            <span className="inline-block w-1.5 h-1.5 rounded-full bg-emerald-500"></span>
          </div>
        );
      })}
    </div>
  );
};
