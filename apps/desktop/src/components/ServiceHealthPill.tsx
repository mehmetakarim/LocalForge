import React, { useState } from "react";
import { CheckCircle2, AlertTriangle, Square, Activity, X } from "lucide-react";

export type ServiceStatus = "running" | "degraded" | "stopped" | "starting";

export interface ServiceHealthPillProps {
  name: string;
  status: ServiceStatus;
  port?: number;
  socketPath?: string;
  version?: string;
  latencyMs?: number;
  errorMessage?: string;
  onRefresh?: () => void;
}

export const ServiceHealthPill: React.FC<ServiceHealthPillProps> = ({
  name,
  status,
  port,
  socketPath,
  version,
  latencyMs,
  errorMessage,
  onRefresh,
}) => {
  const [showPopover, setShowPopover] = useState(false);

  const getStatusConfig = () => {
    switch (status) {
      case "running":
        return {
          bg: "bg-emerald-500/10 hover:bg-emerald-500/15 border-emerald-500/30 text-emerald-300",
          dot: "bg-emerald-400 shadow-[0_0_8px_rgba(52,211,153,0.8)]",
          icon: CheckCircle2,
          label: port ? `Port :${port} Active` : "Running",
        };
      case "degraded":
        return {
          bg: "bg-amber-500/10 hover:bg-amber-500/15 border-amber-500/30 text-amber-300",
          dot: "bg-amber-400",
          icon: AlertTriangle,
          label: "Degraded",
        };
      case "starting":
        return {
          bg: "bg-indigo-500/10 hover:bg-indigo-500/15 border-indigo-500/30 text-indigo-300",
          dot: "bg-indigo-400 animate-ping",
          icon: Activity,
          label: "Starting...",
        };
      case "stopped":
      default:
        return {
          bg: "bg-zinc-800/40 hover:bg-zinc-800/60 border-white/10 text-zinc-400",
          dot: "bg-zinc-500",
          icon: Square,
          label: "Stopped",
        };
    }
  };

  const config = getStatusConfig();
  const Icon = config.icon;

  return (
    <div className="relative inline-block">
      <button
        type="button"
        aria-label={`${name} service: ${config.label}`}
        onClick={() => setShowPopover(!showPopover)}
        className={`flex items-center gap-2 px-3 py-1.5 rounded-full border text-xs font-medium backdrop-blur-md transition-all duration-150 outline-none focus-visible:ring-2 focus-visible:ring-indigo-400 cursor-pointer ${config.bg}`}
      >
        <span className={`w-2 h-2 rounded-full ${config.dot}`} aria-hidden="true" />
        <span className="font-semibold text-zinc-200">{name}</span>
        <span className="text-[11px] opacity-85 font-mono">{config.label}</span>
        <Icon className="w-3.5 h-3.5 opacity-70 ml-0.5" />
      </button>

      {/* Detail Popover */}
      {showPopover && (
        <div className="absolute top-[calc(100%+8px)] left-0 z-50 w-72 p-3.5 rounded-xl bg-zinc-900/95 border border-white/15 backdrop-blur-xl shadow-2xl text-xs text-zinc-300 animate-in fade-in zoom-in-95 duration-150">
          <div className="flex items-center justify-between pb-2 mb-2 border-b border-white/10">
            <div className="flex items-center gap-2">
              <span className={`w-2 h-2 rounded-full ${config.dot}`} />
              <span className="font-semibold text-white uppercase tracking-wider text-[11px]">
                {name} Service
              </span>
            </div>
            <button
              onClick={() => setShowPopover(false)}
              className="text-zinc-400 hover:text-white p-0.5 rounded transition-colors"
              aria-label="Close popover"
            >
              <X className="w-3.5 h-3.5" />
            </button>
          </div>

          <div className="space-y-1.5 font-mono text-[11px]">
            <div className="flex justify-between">
              <span className="text-zinc-500">Status:</span>
              <span className="text-zinc-200 font-semibold">{status.toUpperCase()}</span>
            </div>
            {port && (
              <div className="flex justify-between">
                <span className="text-zinc-500">Port:</span>
                <span className="text-zinc-200">127.0.0.1:{port}</span>
              </div>
            )}
            {version && (
              <div className="flex justify-between">
                <span className="text-zinc-500">Version:</span>
                <span className="text-zinc-200">{version}</span>
              </div>
            )}
            {latencyMs !== undefined && (
              <div className="flex justify-between">
                <span className="text-zinc-500">Ping:</span>
                <span className="text-emerald-400">{latencyMs.toFixed(1)} ms</span>
              </div>
            )}
            {socketPath && (
              <div className="pt-1 border-t border-white/5">
                <span className="text-zinc-500 block mb-0.5">Socket:</span>
                <span className="text-zinc-300 break-all select-all text-[10px] bg-black/40 p-1 rounded block">
                  {socketPath}
                </span>
              </div>
            )}
            {errorMessage && (
              <div className="pt-1 text-red-400 border-t border-red-500/20">
                <span className="font-semibold block">Error:</span>
                <span>{errorMessage}</span>
              </div>
            )}
          </div>

          {onRefresh && (
            <button
              type="button"
              onClick={() => {
                onRefresh();
                setShowPopover(false);
              }}
              className="mt-3 w-full py-1 px-2 text-center rounded-lg bg-indigo-600/30 hover:bg-indigo-600/50 border border-indigo-400/40 text-indigo-200 font-medium transition-colors"
            >
              Check Health
            </button>
          )}
        </div>
      )}
    </div>
  );
};
