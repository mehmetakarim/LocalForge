import React, { useState } from "react";
import {
  Play,
  Square,
  ExternalLink,
  HardDrive,
  Copy,
  Check,
  Terminal,
  Activity,
} from "lucide-react";
import { ProjectStatus } from "../types";

export interface SpatialCardProps {
  id: string;
  title: string;
  status: ProjectStatus;
  statusMessage?: string;
  phpVersion: string;
  localDomain: string;
  volumeName: string;
  isExternalVolume: boolean;
  documentRoot?: string;
  activePort?: number;
  cpuPercent?: number;
  memoryMb?: number;
  isMutating?: boolean;
  onStart?: () => void;
  onStop?: () => void;
  onOpenBrowser?: () => void;
}

export const SpatialCard: React.FC<SpatialCardProps> = ({
  id: _id,
  title,
  status,
  statusMessage,
  phpVersion,
  localDomain,
  volumeName,
  isExternalVolume,
  documentRoot = "public",
  activePort = 8080,
  cpuPercent: _cpuPercent,
  memoryMb: _memoryMb,
  isMutating = false,
  onStart,
  onStop,
  onOpenBrowser,
}) => {
  const [transform, setTransform] = useState<string>("perspective(1000px) rotateX(0deg) rotateY(0deg)");
  const [copied, setCopied] = useState(false);

  const handleMouseMove = (e: React.MouseEvent<HTMLDivElement>) => {
    // Respect user motion preference
    if (window.matchMedia("(prefers-reduced-motion: reduce)").matches) {
      return;
    }
    const rect = e.currentTarget.getBoundingClientRect();
    const x = (e.clientX - rect.left) / rect.width - 0.5;
    const y = (e.clientY - rect.top) / rect.height - 0.5;
    setTransform(
      `perspective(1000px) rotateY(${x * 4}deg) rotateX(${-y * 4}deg) translateY(-2px)`
    );
  };

  const handleMouseLeave = () => {
    setTransform("perspective(1000px) rotateX(0deg) rotateY(0deg) translateY(0px)");
  };

  const copyCurlHelper = () => {
    const cmd = `curl -s -H "Host: ${localDomain}" http://127.0.0.1:${activePort}/`;
    navigator.clipboard.writeText(cmd);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const getStatusDisplay = () => {
    switch (status) {
      case "running":
        return {
          label: "RUNNING",
          color: "text-emerald-400",
          border: "border-emerald-500/30",
          bg: "bg-emerald-500/10",
          dot: "bg-emerald-400 shadow-[0_0_10px_rgba(52,211,153,0.8)]",
        };
      case "starting":
        return {
          label: "STARTING",
          color: "text-indigo-400",
          border: "border-indigo-500/30",
          bg: "bg-indigo-500/10",
          dot: "bg-indigo-400 animate-ping",
        };
      case "stopping":
        return {
          label: "STOPPING",
          color: "text-amber-400",
          border: "border-amber-500/30",
          bg: "bg-amber-500/10",
          dot: "bg-amber-400",
        };
      case "degraded":
        return {
          label: "DEGRADED",
          color: "text-amber-400",
          border: "border-amber-500/30",
          bg: "bg-amber-500/10",
          dot: "bg-amber-400",
        };
      case "disconnected":
        return {
          label: "DISCONNECTED",
          color: "text-rose-400",
          border: "border-rose-500/30",
          bg: "bg-rose-500/10",
          dot: "bg-rose-400",
        };
      case "stopped":
      default:
        return {
          label: "STOPPED",
          color: "text-zinc-400",
          border: "border-white/10",
          bg: "bg-zinc-800/20",
          dot: "bg-zinc-500",
        };
    }
  };

  const statusConfig = getStatusDisplay();
  const isRunning = status === "running";

  return (
    <div
      style={{ perspective: "1000px" }}
      className="transition-transform duration-200 ease-out"
    >
      <div
        onMouseMove={handleMouseMove}
        onMouseLeave={handleMouseLeave}
        style={{
          transform,
          boxShadow: isRunning
            ? "0 16px 36px -4px rgba(0, 0, 0, 0.55), 0 0 24px -2px rgba(79, 110, 247, 0.16), inset 0 1px 0 0 rgba(255, 255, 255, 0.22)"
            : "0 12px 28px -4px rgba(0, 0, 0, 0.45), inset 0 1px 0 0 rgba(255, 255, 255, 0.15)",
        }}
        className="relative flex flex-col justify-between p-5 rounded-2xl lf-material-card border border-white/10 transition-all duration-200 group"
      >
        {/* Specular Top Rim */}
        <div
          className="absolute top-0 left-0 right-0 h-[1px] rounded-t-2xl bg-gradient-to-r from-transparent via-white/30 to-transparent pointer-events-none"
          aria-hidden="true"
        />

        {/* Card Header */}
        <div className="flex items-start justify-between gap-3 mb-4">
          <div className="flex items-center gap-2.5">
            <span
              className={`flex items-center gap-1.5 px-2.5 py-1 rounded-full text-[10px] font-mono font-semibold tracking-wider uppercase border ${statusConfig.bg} ${statusConfig.border} ${statusConfig.color}`}
            >
              <span className={`w-1.5 h-1.5 rounded-full ${statusConfig.dot}`} />
              {statusConfig.label}
            </span>
            <h3 className="font-semibold text-base text-zinc-100 group-hover:text-white transition-colors tracking-tight">
              {title}
            </h3>
          </div>

          {/* Volume badge */}
          <div
            className={`flex items-center gap-1.5 px-2 py-0.5 rounded-md text-[11px] font-mono border ${
              isExternalVolume
                ? "bg-amber-500/10 border-amber-500/30 text-amber-300"
                : "bg-white/5 border-white/10 text-zinc-400"
            }`}
            title={`Mounted on: ${volumeName}`}
          >
            <HardDrive className="w-3 h-3" />
            <span className="truncate max-w-[120px]">{volumeName}</span>
          </div>
        </div>

        {/* Card Body Metrics */}
        <div className="space-y-2 mb-5 text-xs text-zinc-400">
          <div className="flex items-center justify-between py-1 border-b border-white/5">
            <span className="text-zinc-500">PHP Runtime:</span>
            <span className="font-mono text-zinc-200 font-medium">
              v{phpVersion} (FPM)
            </span>
          </div>

          <div className="flex items-center justify-between py-1 border-b border-white/5">
            <span className="text-zinc-500">Local Gateway:</span>
            <div className="flex items-center gap-1 font-mono text-indigo-300">
              <span>{localDomain}</span>
              <button
                type="button"
                onClick={copyCurlHelper}
                title="Copy loopback curl command"
                aria-label="Copy loopback curl command"
                className="p-1 hover:text-white rounded hover:bg-white/10 transition-colors"
              >
                {copied ? <Check className="w-3 h-3 text-emerald-400" /> : <Copy className="w-3 h-3" />}
              </button>
            </div>
          </div>

          <div className="flex items-center justify-between py-1 border-b border-white/5">
            <span className="text-zinc-500">Document Root:</span>
            <span className="font-mono text-zinc-300 truncate max-w-[180px]">
              /{documentRoot}
            </span>
          </div>

          {isRunning && (
            <div className="flex items-center justify-between py-1">
              <span className="text-zinc-500 flex items-center gap-1">
                <Activity className="w-3 h-3 text-emerald-400" /> Port:
              </span>
              <span className="font-mono text-emerald-300">
                127.0.0.1:{activePort}
              </span>
            </div>
          )}

          {statusMessage && (
            <div className="p-2 rounded bg-red-500/10 border border-red-500/20 text-red-300 text-[11px]">
              {statusMessage}
            </div>
          )}
        </div>

        {/* Card Action Footer */}
        <div className="flex items-center justify-between pt-3 border-t border-white/10 gap-2">
          {isRunning ? (
            <button
              type="button"
              disabled={isMutating}
              onClick={onStop}
              className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-zinc-800/80 hover:bg-zinc-700/80 border border-white/10 text-xs font-medium text-zinc-200 transition-all outline-none focus-visible:ring-2 focus-visible:ring-red-400"
            >
              <Square className="w-3.5 h-3.5 text-red-400 fill-red-400" />
              <span>{isMutating ? "Stopping..." : "Stop"}</span>
            </button>
          ) : (
            <button
              type="button"
              disabled={isMutating || status === "disconnected"}
              onClick={onStart}
              className={`flex items-center gap-1.5 px-3.5 py-1.5 rounded-lg text-xs font-semibold text-white transition-all shadow-[0_0_12px_rgba(79,110,247,0.3)] outline-none focus-visible:ring-2 focus-visible:ring-indigo-400 ${
                status === "disconnected"
                  ? "bg-zinc-800 text-zinc-500 cursor-not-allowed border border-white/5"
                  : "bg-indigo-600 hover:bg-indigo-500 border border-indigo-400/50"
              }`}
            >
              <Play className="w-3.5 h-3.5 fill-white" />
              <span>{isMutating ? "Starting..." : "Start Project"}</span>
            </button>
          )}

          <div className="flex items-center gap-2">
            <button
              type="button"
              onClick={copyCurlHelper}
              className="flex items-center gap-1 px-2.5 py-1.5 rounded-lg bg-white/5 hover:bg-white/10 border border-white/10 text-xs font-medium text-zinc-300 hover:text-white transition-colors"
              title="Copy curl command with Host header"
            >
              <Terminal className="w-3.5 h-3.5 text-indigo-400" />
              <span>cURL</span>
            </button>

            {isRunning && onOpenBrowser && (
              <button
                type="button"
                onClick={onOpenBrowser}
                className="flex items-center gap-1 px-2.5 py-1.5 rounded-lg bg-indigo-600/20 hover:bg-indigo-600/35 border border-indigo-400/30 text-xs font-medium text-indigo-200 hover:text-white transition-colors"
                title={`Open http://${localDomain}`}
              >
                <span>Visit</span>
                <ExternalLink className="w-3.5 h-3.5" />
              </button>
            )}
          </div>
        </div>
      </div>
    </div>
  );
};
