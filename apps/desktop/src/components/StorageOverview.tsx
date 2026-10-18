import React from "react";
import { HardDrive, RefreshCw, AlertCircle } from "lucide-react";
import { VolumeInfo, ProjectRecord } from "../types";
import { StorageTopologyCard } from "./StorageTopologyCard";

export interface StorageOverviewProps {
  volumes: VolumeInfo[];
  projects: ProjectRecord[];
  onRefresh: () => void;
}

export const StorageOverview: React.FC<StorageOverviewProps> = ({
  volumes,
  projects,
  onRefresh,
}) => {
  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-xl font-bold text-white tracking-tight flex items-center gap-2.5">
            <HardDrive className="w-6 h-6 text-amber-400" />
            Storage Topology & Volume Boundaries
          </h2>
          <p className="text-xs text-zinc-400 mt-0.5">
            APFS volume isolation, persistent UUID tracking, and external workspace safety barriers
          </p>
        </div>
        <button
          type="button"
          onClick={onRefresh}
          className="flex items-center gap-1.5 px-3 py-1.5 rounded-xl bg-white/5 hover:bg-white/10 border border-white/10 text-xs font-medium text-zinc-300 hover:text-white transition-colors"
        >
          <RefreshCw className="w-3.5 h-3.5 text-indigo-400" />
          <span>Scan Volumes</span>
        </button>
      </div>

      {/* Disconnect Resilience Notice */}
      <div className="p-4 rounded-xl bg-amber-950/20 border border-amber-500/20 flex items-start gap-3 text-xs text-amber-200/90">
        <AlertCircle className="w-5 h-5 text-amber-400 shrink-0 mt-0.5" />
        <div>
          <span className="font-semibold text-white block mb-0.5">
            External Workspace Safety Boundary (ADR-0004 &amp; ADR-0012)
          </span>
          Projects on external volumes (like <code className="text-amber-300">/Volumes/Mac Harici Disk</code>) are bound by persistent APFS UUID. If an external drive is unmounted, operations fail safely without recreating directories or ghost data on the internal disk.
        </div>
      </div>

      {/* Volume Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-5">
        {volumes.map((volume) => {
          const attachedProjects = projects.filter(
            (p) => p.volume_uuid === volume.uuid
          ).length;

          return (
            <StorageTopologyCard
              key={volume.uuid}
              volume={volume}
              projectCount={attachedProjects}
            />
          );
        })}
      </div>
    </div>
  );
};
