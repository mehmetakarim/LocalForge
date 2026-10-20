import React, { useState } from "react";
import {
  X,
  Play,
  Square,
  ExternalLink,
  Copy,
  Check,
  HardDrive,
  Cpu,
  Lock,
  Database,
  CloudUpload,
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
  const [copied, setCopied] = useState(false);

  if (!isOpen || !project) return null;

  const isRunning = project.status === "running";
  const activePort = runtime?.nginx_port || 8080;
  const isExternal = volume ? volume.is_removable || !volume.is_internal : false;
  const cleanDbName = `localforge_${project.name.toLowerCase().replace(/[^a-z0-9]/g, "_")}`;

  const copyCurlHelper = () => {
    const cmd = `curl -s -H "Host: ${project.local_domain}" http://127.0.0.1:${activePort}/`;
    navigator.clipboard.writeText(cmd);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div
      role="dialog"
      aria-labelledby="inspector-title"
      aria-modal="true"
      className="fixed inset-y-0 right-0 w-[420px] z-50 flex flex-col bg-[#191337]/95 backdrop-blur-3xl border-l border-[rgba(183,173,223,0.18)] shadow-[-20px_0_60px_rgba(10,6,30,0.85)] p-6 transition-transform duration-300 ease-out select-none"
    >
      {/* Drawer Header with Official Elephant Emblem */}
      <div className="flex items-center justify-between pb-4 border-b border-[rgba(183,173,223,0.14)] mb-5">
        <div className="flex items-center gap-3">
          <img
            src="/brand/localforge-emblem.png"
            alt="LocalForge Elephant Emblem"
            className="w-9 h-9 rounded-xl object-cover shadow-[0_2px_8px_rgba(10,6,30,0.5)] border border-[rgba(183,173,223,0.25)]"
          />
          <div>
            <span className="font-mono text-[10px] uppercase text-[#B7ADDF] tracking-wider font-semibold">
              Proje Müfettişi
            </span>
            <h2 id="inspector-title" className="font-['Plus_Jakarta_Sans',sans-serif] text-xl font-bold text-[#FFFBFF] tracking-tight">
              {project.name}
            </h2>
          </div>
        </div>

        <button
          type="button"
          onClick={onClose}
          className="w-8 h-8 rounded-lg bg-[#2B2450]/60 hover:bg-[#695FA4]/30 border border-[rgba(183,173,223,0.18)] text-[#D4CCF1] hover:text-[#FFFBFF] flex items-center justify-center transition-colors cursor-pointer"
          title="Kapat (Esc)"
        >
          <X className="w-4 h-4" />
        </button>
      </div>

      <div className="flex-1 overflow-y-auto space-y-4 pr-1">
        {/* Status Chip */}
        <div className="p-3.5 rounded-xl bg-[#2B2450]/55 border border-[rgba(183,173,223,0.12)] flex items-center justify-between">
          <span className="text-xs text-[#D4CCF1]">Çalışma Durumu</span>
          <div className="flex items-center gap-2">
            <span
              className={`w-2 h-2 rounded-full ${
                isRunning
                  ? "bg-[#10B981] shadow-[0_0_8px_#10B981]"
                  : project.status === "failed"
                  ? "bg-[#EF4444] shadow-[0_0_8px_#EF4444]"
                  : project.status === "degraded" || project.status === "disconnected"
                  ? "bg-[#F59E0B] shadow-[0_0_8px_#F59E0B]"
                  : "bg-[#6B7280]"
              }`}
            />
            <span
              className={`text-xs font-mono font-bold uppercase ${
                isRunning
                  ? "text-[#10B981]"
                  : project.status === "failed"
                  ? "text-[#EF4444]"
                  : project.status === "degraded" || project.status === "disconnected"
                  ? "text-[#F59E0B]"
                  : "text-[#D4CCF1]"
              }`}
            >
              {isRunning ? "çevrimiçi" : project.status}
            </span>
          </div>
        </div>

        {/* Runtime Engine Details */}
        <div className="space-y-2">
          <div className="flex items-center gap-1.5 text-[11px] font-mono text-[#B7ADDF] uppercase tracking-wider font-semibold">
            <Cpu className="w-3.5 h-3.5 text-[#B7ADDF]" />
            <span>Çalışma Zamanı &amp; Ağ Geçidi</span>
          </div>
          <div className="p-3.5 rounded-xl bg-[#2B2450]/45 border border-[rgba(183,173,223,0.12)] space-y-2 font-mono text-xs">
            <div className="flex justify-between">
              <span className="text-[#D4CCF1]/70">PHP Sürümü:</span>
              <span className="text-[#FFFBFF] font-semibold">PHP {project.php_version} (FPM)</span>
            </div>
            <div className="flex justify-between">
              <span className="text-[#D4CCF1]/70">Ağ Geçidi Portu:</span>
              <span className="text-[#FFFBFF]">127.0.0.1:{activePort}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-[#D4CCF1]/70">Yerel Etki Alanı:</span>
              <span className="text-[#B7ADDF] font-semibold">{project.local_domain}</span>
            </div>
          </div>
        </div>

        {/* Database & Persistence */}
        <div className="space-y-2">
          <div className="flex items-center gap-1.5 text-[11px] font-mono text-[#B7ADDF] uppercase tracking-wider font-semibold">
            <Database className="w-3.5 h-3.5 text-[#10B981]" />
            <span>MariaDB Veritabanı</span>
          </div>
          <div className="p-3.5 rounded-xl bg-[#2B2450]/45 border border-[rgba(183,173,223,0.12)] space-y-2 font-mono text-xs">
            <div className="flex justify-between">
              <span className="text-[#D4CCF1]/70">Veritabanı Adı:</span>
              <span className="text-[#10B981] font-semibold">{cleanDbName}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-[#D4CCF1]/70">Bağlantı:</span>
              <span className="text-[#FFFBFF]">127.0.0.1:3306</span>
            </div>
          </div>
        </div>

        {/* Strict Safety Guard: Database Deletion Lock (R1.2B Rule) */}
        <div className="p-3.5 rounded-xl bg-[#EF4444]/10 border border-[#EF4444]/30 space-y-1.5">
          <div className="flex items-center gap-2 text-xs font-semibold text-[#EF4444]">
            <Lock className="w-3.5 h-3.5" />
            <span>Veritabanı Silme Kilidi: Aktif (Phase R1 Güvenlik Kuralı)</span>
          </div>
          <p className="text-[11px] text-[#D4CCF1] leading-relaxed">
            Kullanıcı veri kaybını engellemek amacıyla veritabanı silme butonu kalıcı olarak devre dışı bırakılmıştır.
          </p>
          <button
            type="button"
            disabled
            className="w-full py-1.5 px-3 rounded-lg bg-white/5 border border-white/10 text-[11px] font-mono text-[#D4CCF1]/50 cursor-not-allowed flex items-center justify-center gap-1.5"
          >
            <Lock className="w-3 h-3" />
            <span>Veritabanını Sil (Kilitli & Devre Dışı)</span>
          </button>
        </div>

        {/* Strict Safety Guard: SFTP Deployment Target (Dry-Run Only) */}
        <div className="p-3.5 rounded-xl bg-[#F59E0B]/10 border border-[#F59E0B]/30 space-y-1.5">
          <div className="flex items-center gap-2 text-xs font-semibold text-[#F59E0B]">
            <CloudUpload className="w-3.5 h-3.5" />
            <span>SFTP Dağıtım Kapısı: Salt-Okunur Önizleme (Dry-Run Only)</span>
          </div>
          <p className="text-[11px] text-[#D4CCF1] leading-relaxed">
            Dağıtım mekanizması yalnızca simülasyon ve sağlama doğrulaması yapar. Canlı sunucuya yazma devre dışıdır.
          </p>
        </div>

        {/* Storage & APFS Security */}
        <div className="space-y-2">
          <div className="flex items-center gap-1.5 text-[11px] font-mono text-[#B7ADDF] uppercase tracking-wider font-semibold">
            <HardDrive className="w-3.5 h-3.5 text-[#F59E0B]" />
            <span>Depolama &amp; APFS Güvenliği</span>
          </div>
          <div className="p-3.5 rounded-xl bg-[#2B2450]/45 border border-[rgba(183,173,223,0.12)] space-y-2 font-mono text-xs">
            <div className="flex justify-between">
              <span className="text-[#D4CCF1]/70">Depolama Sürücüsü:</span>
              <span
                className={`font-semibold ${
                  isExternal ? "text-[#F59E0B]" : "text-[#D4CCF1]"
                }`}
              >
                {volume?.name || "Bilinmeyen Sürücü"}
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-[#D4CCF1]/70">APFS UUID:</span>
              <span className="text-[#D4CCF1] text-[10px] truncate max-w-[170px]" title={project.volume_uuid}>
                {project.volume_uuid}
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-[#D4CCF1]/70">Document Root:</span>
              <span className="text-[#FFFBFF]">/{project.document_root}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-[#D4CCF1]/70">Fiziksel Yol:</span>
              <span className="text-[#D4CCF1] text-[10px] truncate max-w-[170px]" title={project.path}>
                {project.path}
              </span>
            </div>
          </div>
        </div>

        {/* Diagnostic cURL Helper */}
        <div className="p-3.5 rounded-xl bg-[#2B2450]/45 border border-[rgba(183,173,223,0.12)] space-y-2">
          <div className="flex items-center justify-between text-xs text-[#D4CCF1]">
            <span>cURL Test Komutu:</span>
            <button
              type="button"
              onClick={copyCurlHelper}
              className="flex items-center gap-1 text-[11px] font-mono text-[#B7ADDF] hover:underline cursor-pointer"
            >
              {copied ? <Check className="w-3 h-3 text-[#10B981]" /> : <Copy className="w-3 h-3 text-[#B7ADDF]" />}
              <span>{copied ? "Kopyalandı" : "Kopyala"}</span>
            </button>
          </div>
          <code className="block p-2 rounded-lg bg-[#191337]/90 text-[10px] font-mono text-[#D4CCF1] break-all select-all border border-[rgba(183,173,223,0.10)]">
            curl -s -H "Host: {project.local_domain}" http://127.0.0.1:{activePort}/
          </code>
        </div>
      </div>

      {/* Action Footer */}
      <div className="pt-4 border-t border-[rgba(183,173,223,0.14)] flex items-center gap-3">
        {isRunning ? (
          <button
            type="button"
            disabled={isMutating}
            onClick={() => onStopProject(project.id)}
            className="flex-1 flex items-center justify-center gap-2 py-2.5 px-4 rounded-xl bg-[#EF4444]/15 hover:bg-[#EF4444]/25 border border-[#EF4444]/30 text-[#EF4444] font-semibold text-xs transition-colors cursor-pointer"
          >
            <Square className="w-3.5 h-3.5" />
            <span>{isMutating ? "Durduruluyor..." : "Durdur ■"}</span>
          </button>
        ) : (
          <button
            type="button"
            disabled={isMutating || project.status === "disconnected"}
            onClick={() => onStartProject(project.id)}
            className="flex-1 flex items-center justify-center gap-2 py-2.5 px-4 rounded-xl bg-gradient-to-r from-[#695FA4] to-[#2B2450] hover:brightness-110 border border-[#B7ADDF] text-[#FFFBFF] font-semibold text-xs transition-all shadow-[0_4px_16px_rgba(105,95,164,0.35)] cursor-pointer disabled:opacity-40 disabled:cursor-not-allowed"
          >
            <Play className="w-3.5 h-3.5 fill-current" />
            <span>
              {isMutating
                ? "Başlatılıyor..."
                : project.status === "disconnected"
                ? "Disk Bağlı Değil"
                : "Başlat ▶"}
            </span>
          </button>
        )}

        <button
          type="button"
          onClick={() => onOpenBrowser(project)}
          className="flex-1 flex items-center justify-center gap-2 py-2.5 px-4 rounded-xl bg-[#2B2450]/60 hover:bg-[#695FA4]/30 border border-[rgba(183,173,223,0.2)] text-[#FFFBFF] font-semibold text-xs transition-colors cursor-pointer"
        >
          <ExternalLink className="w-3.5 h-3.5 text-[#B7ADDF]" />
          <span>Tarayıcıda Aç ↗</span>
        </button>
      </div>
    </div>
  );
};
