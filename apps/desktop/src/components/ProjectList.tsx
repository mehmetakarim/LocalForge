import React from "react";
import {
  Globe,
  Play,
  Square,
  AlertTriangle,
  ExternalLink,
  Cpu,
  HardDrive,
  Copy,
  Check,
  Sparkles,
} from "lucide-react";
import { ProjectRecord } from "../types";

interface ProjectListProps {
  projects: ProjectRecord[];
  onToggleStatus: (id: string, currentStatus: string) => void;
  onOpenCreate: () => void;
}

export const ProjectList: React.FC<ProjectListProps> = ({
  projects,
  onToggleStatus,
  onOpenCreate,
}) => {
  const [copiedId, setCopiedId] = React.useState<string | null>(null);

  const copyUrl = (domain: string, id: string) => {
    navigator.clipboard.writeText(`http://${domain}:8080`);
    setCopiedId(id);
    setTimeout(() => setCopiedId(null), 1800);
  };

  if (projects.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center py-20 px-6 rounded-3xl border border-dashed border-white/10 apple-glass text-center">
        <div className="w-16 h-16 rounded-2xl bg-gradient-to-tr from-blue-500/20 via-indigo-500/20 to-purple-500/20 border border-white/10 flex items-center justify-center text-blue-400 mb-4 shadow-xl shadow-blue-500/5">
          <Sparkles className="w-8 h-8" />
        </div>
        <h3 className="text-lg font-semibold text-white tracking-tight">Kayıtlı Proje Bulunamadı</h3>
        <p className="text-sm text-neutral-400 max-w-md mt-1.5 leading-relaxed">
          Harici SSD veya dahili diskinizdeki mevcut PHP projelerinizi tek tıklamayla içe aktarın veya sıfırdan modern bir çalışma alanı başlatın.
        </p>
        <button
          onClick={onOpenCreate}
          className="mt-6 px-5 py-2.5 rounded-xl text-xs font-semibold text-white apple-button-primary flex items-center gap-2"
        >
          <Play className="w-3.5 h-3.5 fill-current" />
          İlk Projeyi Oluştur
        </button>
      </div>
    );
  }

  return (
    <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-5">
      {projects.map((p) => {
        const isDisconnected = p.status === "disconnected";
        const isRunning = p.status === "running";
        const isExternal = p.path.startsWith("/Volumes");

        // Generate monogram initials
        const initials = p.name
          .split(/[\s-_]+/)
          .map((n) => n[0])
          .slice(0, 2)
          .join("")
          .toUpperCase() || "LF";

        return (
          <div
            key={p.id}
            className={`group relative flex flex-col justify-between p-5 rounded-2xl border transition-all duration-300 ${
              isDisconnected
                ? "bg-gradient-to-b from-rose-950/20 via-neutral-900/60 to-neutral-950/80 border-rose-500/30 shadow-lg shadow-rose-950/20"
                : isRunning
                ? "bg-gradient-to-b from-neutral-800/60 via-neutral-900/70 to-neutral-950/90 border-blue-500/35 shadow-xl shadow-blue-500/5"
                : "apple-card hover:border-white/15"
            }`}
          >
            {/* Top Row: Monogram, Title, Status Indicator */}
            <div>
              <div className="flex items-start justify-between gap-3">
                <div className="flex items-center gap-3">
                  {/* Apple Style Gradient Monogram */}
                  <div
                    className={`w-11 h-11 rounded-xl flex items-center justify-center font-bold text-sm tracking-wider shadow-inner transition-transform group-hover:scale-105 duration-200 ${
                      isRunning
                        ? "bg-gradient-to-tr from-blue-600 via-indigo-600 to-cyan-500 text-white shadow-blue-400/20"
                        : isDisconnected
                        ? "bg-gradient-to-tr from-rose-700 to-orange-700 text-white"
                        : "bg-gradient-to-tr from-neutral-700 via-neutral-800 to-neutral-900 text-neutral-300 border border-white/10"
                    }`}
                  >
                    {initials}
                  </div>

                  <div className="overflow-hidden">
                    <h3 className="font-semibold text-sm text-white tracking-tight truncate group-hover:text-blue-400 transition-colors" title={p.name}>
                      {p.name}
                    </h3>
                    <div className="flex items-center gap-1.5 mt-0.5">
                      <span className="text-[11px] font-mono text-neutral-400 truncate">
                        {p.local_domain}
                      </span>
                    </div>
                  </div>
                </div>

                {/* Status Pill */}
                <div className="flex items-center">
                  <span
                    className={`inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-[10px] font-semibold tracking-wider uppercase border transition-all ${
                      isRunning
                        ? "bg-emerald-500/10 text-emerald-400 border-emerald-500/30 shadow-xs shadow-emerald-500/20"
                        : isDisconnected
                        ? "bg-rose-500/15 text-rose-400 border-rose-500/30"
                        : "bg-neutral-800/80 text-neutral-400 border-white/5"
                    }`}
                  >
                    <span
                      className={`w-1.5 h-1.5 rounded-full ${
                        isRunning
                          ? "bg-emerald-400 animate-pulse"
                          : isDisconnected
                          ? "bg-rose-400"
                          : "bg-neutral-500"
                      }`}
                    />
                    {isRunning ? "Çalışıyor" : isDisconnected ? "Bağlantı Yok" : "Durduruldu"}
                  </span>
                </div>
              </div>

              {/* Specs & Storage Metadata */}
              <div className="mt-4 pt-3 border-t border-white/[0.06] grid grid-cols-2 gap-2 text-[11px]">
                <div className="flex items-center gap-1.5 text-neutral-400">
                  <Cpu className="w-3.5 h-3.5 text-blue-400 flex-shrink-0" />
                  <span className="font-medium text-neutral-300">PHP {p.php_version}</span>
                  <span className="text-neutral-500 text-[10px]">(FPM)</span>
                </div>

                <div className="flex items-center gap-1.5 text-neutral-400 truncate">
                  <HardDrive className={`w-3.5 h-3.5 flex-shrink-0 ${isExternal ? "text-amber-400" : "text-neutral-500"}`} />
                  <span className="truncate text-neutral-300">
                    {isExternal ? "Harici SSD" : "Dahili Disk"}
                  </span>
                </div>
              </div>

              {/* Workspace Directory Path */}
              <div className="mt-2 text-[10px] font-mono text-neutral-400 truncate bg-neutral-950/40 p-2 rounded-lg border border-white/[0.04]" title={p.path}>
                <span className="text-neutral-400 select-none">root: </span>
                <span className="text-neutral-300">{p.path}</span>
              </div>
            </div>

            {/* Bottom Actions Row */}
            <div className="mt-5 pt-3 border-t border-white/[0.06] flex items-center justify-between gap-2">
              {isDisconnected ? (
                <div className="flex items-center gap-2 text-xs text-rose-300 font-medium">
                  <AlertTriangle className="w-4 h-4 text-rose-400 flex-shrink-0" />
                  <span className="text-[11px]">Harici disk ayrıldı (Güvenli kilit)</span>
                </div>
              ) : (
                <div className="flex items-center gap-2">
                  <button
                    onClick={() => onToggleStatus(p.id, p.status)}
                    className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-semibold transition-all ${
                      isRunning
                        ? "bg-neutral-800 hover:bg-neutral-700/80 text-rose-300 border border-white/10"
                        : "apple-button-primary text-white"
                    }`}
                  >
                    {isRunning ? (
                      <>
                        <Square className="w-3 h-3 fill-current" />
                        <span>Durdur</span>
                      </>
                    ) : (
                      <>
                        <Play className="w-3 h-3 fill-current" />
                        <span>Başlat</span>
                      </>
                    )}
                  </button>

                  <button
                    onClick={() => copyUrl(p.local_domain, p.id)}
                    className="p-1.5 rounded-lg text-neutral-400 hover:text-white apple-button-ghost transition-colors"
                    title="Yerel URL'yi Kopyala (http://...:8080)"
                  >
                    {copiedId === p.id ? (
                      <Check className="w-3.5 h-3.5 text-emerald-400" />
                    ) : (
                      <Copy className="w-3.5 h-3.5" />
                    )}
                  </button>
                </div>
              )}

              <div className="flex items-center gap-1">
                <a
                  href={`http://${p.local_domain}:8080`}
                  target="_blank"
                  rel="noreferrer"
                  className={`flex items-center gap-1 px-2.5 py-1.5 rounded-lg text-xs font-medium text-neutral-300 hover:text-white apple-button-ghost transition-colors ${
                    isDisconnected || !isRunning ? "opacity-35 pointer-events-none" : ""
                  }`}
                  title="Tarayıcıda Aç"
                >
                  <Globe className="w-3.5 h-3.5 text-blue-400" />
                  <span>Aç</span>
                  <ExternalLink className="w-3 h-3 text-neutral-400 ml-0.5" />
                </a>
              </div>
            </div>
          </div>
        );
      })}
    </div>
  );
};
