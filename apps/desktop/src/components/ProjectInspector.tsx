import React from "react";
import {
  X,
  Play,
  Square,
  ExternalLink,
  Copy,
  Check,
  HardDrive,
  Cpu,
} from "lucide-react";
import { ProjectRecord, VolumeInfo, RuntimeOverview } from "../types";

export interface ProjectInspectorProps {
  project: ProjectRecord | null;
  volume: VolumeInfo | null;
  runtime: RuntimeOverview | null;
  isOpen: boolean;
  onClose: () => void;
  onStartProject: (id: string) => void;
  onStopProject: (id: string) => void;
  onOpenBrowser: (project: ProjectRecord) => void;
  isMutating?: boolean;
}

export const ProjectInspector: React.FC<ProjectInspectorProps> = ({
  project,
  volume,
  runtime,
  isOpen,
  onClose,
  onStartProject,
  onStopProject,
  onOpenBrowser,
  isMutating = false,
}) => {
  const [copied, setCopied] = React.useState(false);

  if (!isOpen || !project) return null;

  const isRunning = project.status === "running";
  const activePort = runtime?.nginx_port || 8080;

  const copyCurlHelper = () => {
    const cmd = `curl -s -H "Host: ${project.local_domain}" http://127.0.0.1:${activePort}/`;
    navigator.clipboard.writeText(cmd);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const isExternal = volume ? volume.is_removable || !volume.is_internal : false;

  return (
    <div
      className="fixed inset-y-0 right-0 w-96 z-50 flex flex-col bg-[#0E121E]/95 backdrop-blur-3xl border-l border-white/10 shadow-[-20px_0_60px_rgba(0,0,0,0.85)] p-6 transition-transform duration-300 ease-out select-none"
      role="dialog"
      aria-labelledby="inspector-title"
    >
      {/* Drawer Header */}
      <div className="flex items-center justify-between pb-4 border-b border-white/10 mb-5">
        <div>
          <span className="text-[10px] font-mono uppercase text-indigo-400 tracking-wider font-semibold">
            Project Inspector
          </span>
          <h2 id="inspector-title" className="text-xl font-bold text-white tracking-tight">
            {project.name}
          </h2>
        </div>
        <button
          type="button"
          onClick={onClose}
          className="w-8 h-8 rounded-lg bg-white/5 hover:bg-white/10 border border-white/10 text-zinc-400 hover:text-white flex items-center justify-center transition-colors"
          title="Kapat (Esc)"
        >
          <X className="w-4 h-4" />
        </button>
      </div>

      <div className="flex-1 overflow-y-auto space-y-5 pr-1">
        {/* Status Chip */}
        <div className="p-3 rounded-xl bg-black/40 border border-white/5 flex items-center justify-between">
          <span className="text-xs text-zinc-400">Çalışma Durumu</span>
          <div className="flex items-center gap-2">
            <span
              className={`w-2 h-2 rounded-full ${
                isRunning
                  ? "bg-emerald-400 shadow-[0_0_8px_rgba(52,211,153,0.8)]"
                  : "bg-zinc-500"
              }`}
            />
            <span
              className={`text-xs font-mono font-bold uppercase ${
                isRunning ? "text-emerald-400" : "text-zinc-400"
              }`}
            >
              {project.status}
            </span>
          </div>
        </div>

        {/* Runtime Engine Details */}
        <div className="space-y-2">
          <div className="flex items-center gap-1.5 text-[11px] font-mono text-zinc-400 uppercase tracking-wider">
            <Cpu className="w-3.5 h-3.5 text-indigo-400" />
            <span>Çalışma Zamanı (Runtime)</span>
          </div>
          <div className="p-3.5 rounded-xl bg-black/40 border border-white/5 space-y-2 font-mono text-xs">
            <div className="flex justify-between">
              <span className="text-zinc-500">PHP Version:</span>
              <span className="text-zinc-200 font-semibold">PHP {project.php_version} (FPM)</span>
            </div>
            <div className="flex justify-between">
              <span className="text-zinc-500">Loopback Gateway:</span>
              <span className="text-zinc-200">127.0.0.1:{activePort}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-zinc-500">Local Domain:</span>
              <span className="text-indigo-400">{project.local_domain}</span>
            </div>
            <div className="text-[10px] text-zinc-500 pt-1 border-t border-white/5 font-sans">
              Not: `.test` yönlendirmesi HTTP Host-Header tabanlıdır (ayrı DNS yapılandırması gerektirir).
            </div>
          </div>
        </div>

        {/* Storage & APFS Security */}
        <div className="space-y-2">
          <div className="flex items-center gap-1.5 text-[11px] font-mono text-zinc-400 uppercase tracking-wider">
            <HardDrive className="w-3.5 h-3.5 text-amber-400" />
            <span>Depolama &amp; APFS Güvenliği</span>
          </div>
          <div className="p-3.5 rounded-xl bg-black/40 border border-white/5 space-y-2 font-mono text-xs">
            <div className="flex justify-between">
              <span className="text-zinc-500">Depolama Sürücüsü:</span>
              <span
                className={`font-semibold ${
                  isExternal ? "text-amber-400" : "text-zinc-300"
                }`}
              >
                {volume?.name || "Bilinmeyen Sürücü"}
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-zinc-500">APFS UUID:</span>
              <span className="text-zinc-400 text-[10px] truncate max-w-[170px]" title={project.volume_uuid}>
                {project.volume_uuid}
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-zinc-500">Document Root:</span>
              <span className="text-zinc-200">/{project.document_root}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-zinc-500">Fiziksel Yol:</span>
              <span className="text-zinc-400 text-[10px] truncate max-w-[170px]" title={project.path}>
                {project.path}
              </span>
            </div>
          </div>
        </div>

        {/* Diagnostic cURL Helper */}
        <div className="p-3 rounded-xl bg-white/[0.03] border border-white/5 space-y-2">
          <div className="flex items-center justify-between text-xs text-zinc-400">
            <span>cURL Test Komutu:</span>
            <button
              type="button"
              onClick={copyCurlHelper}
              className="flex items-center gap-1 text-[11px] font-mono text-indigo-400 hover:text-indigo-300 transition-colors"
            >
              {copied ? <Check className="w-3 h-3 text-emerald-400" /> : <Copy className="w-3 h-3" />}
              <span>{copied ? "Kopyalandı" : "Kopyala"}</span>
            </button>
          </div>
          <code className="block p-2 rounded-lg bg-black/60 text-[10px] font-mono text-zinc-400 break-all select-all">
            curl -s -H "Host: {project.local_domain}" http://127.0.0.1:{activePort}/
          </code>
        </div>
      </div>

      {/* Action Footer */}
      <div className="pt-4 border-t border-white/10 flex items-center gap-3">
        {isRunning ? (
          <button
            type="button"
            disabled={isMutating}
            onClick={() => onStopProject(project.id)}
            className="flex-1 flex items-center justify-center gap-2 py-2.5 px-4 rounded-xl bg-red-500/15 hover:bg-red-500/25 border border-red-500/30 text-red-400 font-semibold text-xs transition-colors"
          >
            <Square className="w-3.5 h-3.5" />
            <span>{isMutating ? "Durduruluyor..." : "Durdur ■"}</span>
          </button>
        ) : (
          <button
            type="button"
            disabled={isMutating}
            onClick={() => onStartProject(project.id)}
            className="flex-1 flex items-center justify-center gap-2 py-2.5 px-4 rounded-xl bg-indigo-600 hover:bg-indigo-500 text-white font-semibold text-xs transition-all shadow-[0_0_20px_rgba(79,110,247,0.4)]"
          >
            <Play className="w-3.5 h-3.5 fill-current" />
            <span>{isMutating ? "Başlatılıyor..." : "Başlat ▶"}</span>
          </button>
        )}

        <button
          type="button"
          onClick={() => onOpenBrowser(project)}
          className="flex-1 flex items-center justify-center gap-2 py-2.5 px-4 rounded-xl bg-white/5 hover:bg-white/10 border border-white/10 text-white font-semibold text-xs transition-colors"
        >
          <ExternalLink className="w-3.5 h-3.5" />
          <span>Tarayıcıda Aç ↗</span>
        </button>
      </div>
    </div>
  );
};
