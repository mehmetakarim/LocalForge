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
  AlertTriangle,
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
  onUpdatePhpVersion?: (id: string, version: string) => Promise<void>;
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
  onUpdatePhpVersion,
  isMutating = false,
}) => {
  const [copied, setCopied] = useState(false);
  const [copiedLoopback, setCopiedLoopback] = useState(false);
  const [isChangingPhp, setIsChangingPhp] = useState(false);
  const [targetPhp, setTargetPhp] = useState("");
  const [updatingPhp, setUpdatingPhp] = useState(false);
  const [updateError, setUpdateError] = useState<string | null>(null);

  if (!isOpen || !project) return null;

  const isRunning = project.status === "running";
  const activePort = runtime?.nginx_port || 8080;
  const isExternal = volume ? volume.is_removable || !volume.is_internal : false;
  const cleanDbName = `localforge_${project.name.toLowerCase().replace(/[^a-z0-9]/g, "_")}`;

  const slug = project.name.toLowerCase().replace(/[^a-z0-9]/g, "-").replace(/-+/g, "-").replace(/^-|-$/g, "");
  const preferredDomain = `${slug}.localforge`;
  const localhostCandidate = `${slug}.localforge.localhost`;
  const isPortAssigned =
    project.loopback_port !== undefined &&
    project.loopback_port !== null &&
    project.loopback_port > 0;
  const loopbackPort = isPortAssigned ? project.loopback_port : null;
  const loopbackEndpoint = loopbackPort ? `127.0.0.1:${loopbackPort}` : "Tahsis Edilmedi";
  const directLoopbackUrl = loopbackPort ? `http://${loopbackEndpoint}/` : "";

  // Installed & verified PHP version detection
  const verifiedList = (runtime?.php_details?.filter((d) => d.is_fpm_verified) || []).map((d) => d.major_minor);
  const fallbackList = runtime?.php_versions?.filter((v) => ["8.5", "8.4", "8.3", "8.2", "8.1"].includes(v)) || [];
  const installedVersions = Array.from(new Set([...verifiedList, ...fallbackList]));
  const isCurrentPhpInstalled = installedVersions.some(
    (v) => v === project.php_version || v.startsWith(project.php_version) || project.php_version.startsWith(v)
  );

  const copyCurlHelper = () => {
    if (!loopbackPort) return;
    const cmd = `curl -s -H "Host: ${preferredDomain}" http://127.0.0.1:${loopbackPort}/`;
    navigator.clipboard.writeText(cmd);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const copyLoopbackHelper = () => {
    if (!directLoopbackUrl) return;
    navigator.clipboard.writeText(directLoopbackUrl);
    setCopiedLoopback(true);
    setTimeout(() => setCopiedLoopback(false), 2000);
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
            <div className="flex justify-between items-center">
              <span className="text-[#D4CCF1]/70">PHP Sürümü:</span>
              <div className="flex items-center gap-2">
                <span className="text-[#FFFBFF] font-semibold">
                  PHP {project.php_version} (FPM)
                </span>
                {onUpdatePhpVersion && (
                  <button
                    type="button"
                    onClick={() => {
                      setTargetPhp(installedVersions[0] || "8.5");
                      setIsChangingPhp(true);
                      setUpdateError(null);
                    }}
                    className="text-[10px] px-2 py-0.5 rounded bg-[rgba(183,173,223,0.15)] hover:bg-[rgba(183,173,223,0.25)] text-[#B7ADDF] hover:text-[#FFFBFF] transition-all cursor-pointer font-sans"
                  >
                    Değiştir
                  </button>
                )}
              </div>
            </div>
            <div className="flex justify-between items-center">
              <span className="text-[#D4CCF1]/70">Tercih Edilen Alan Adı:</span>
              <div className="flex items-center gap-1.5">
                <span className="text-[#B7ADDF] font-semibold">{preferredDomain}</span>
                <span className="text-[9px] px-1.5 py-0.2 rounded bg-[#695FA4]/40 text-[#D4CCF1] font-sans">
                  Önerilen
                </span>
              </div>
            </div>
            <div className="flex justify-between items-center">
              <span className="text-[#D4CCF1]/70">Doğrudan Loopback:</span>
              <div className="flex items-center gap-1.5">
                <span className="text-[#10B981] font-semibold">{loopbackEndpoint}</span>
                <button
                  type="button"
                  onClick={copyLoopbackHelper}
                  className="text-[10px] text-[#B7ADDF] hover:text-[#FFFBFF] cursor-pointer"
                  title="Loopback URL'sini Kopyala"
                >
                  {copiedLoopback ? <Check className="w-3 h-3 text-[#10B981]" /> : <Copy className="w-3 h-3" />}
                </button>
              </div>
            </div>
            <div className="flex justify-between">
              <span className="text-[#D4CCF1]/70">Bağımsız Alan Adı:</span>
              <span className="text-[#D4CCF1]">{localhostCandidate}:{loopbackPort}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-[#D4CCF1]/70">Eski Alan Adı (Legacy):</span>
              <span className="text-[#D4CCF1]/80">{project.local_domain}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-[#D4CCF1]/70">Ağ Geçidi Portu:</span>
              <span className="text-[#FFFBFF]">127.0.0.1:{activePort}</span>
            </div>
          </div>

          {/* Missing PHP Runtime Warning Banner (Turkish Guidance) */}
          {!isCurrentPhpInstalled && (
            <div className="p-3.5 rounded-xl bg-amber-500/15 border border-amber-500/35 text-amber-200 text-xs space-y-2 select-text font-['Figtree',sans-serif]">
              <div className="flex items-center gap-2 font-bold text-amber-300">
                <AlertTriangle className="w-4 h-4 text-amber-400 shrink-0" />
                <span>Eksik Çalışma Zamanı (PHP {project.php_version})</span>
              </div>
              <p className="text-[11px] leading-relaxed text-[#D4CCF1]">
                Bu proje için yapılandırılmış olan <strong className="text-white">PHP {project.php_version}</strong> FPM ikilisi ana makinenizde bulunamadı. Sisteminizde doğrulanmış sürüm: <strong className="text-white">PHP {installedVersions.join(", ") || "Yok"} (FPM arm64)</strong>.
              </p>
              <div className="pt-1 flex items-center justify-between text-[11px]">
                <span className="text-amber-300/80">Otomatik geçiş engellendi (PO Kuralı).</span>
                {onUpdatePhpVersion && (
                  <button
                    type="button"
                    onClick={() => {
                      setTargetPhp(installedVersions[0] || "8.5");
                      setIsChangingPhp(true);
                      setUpdateError(null);
                    }}
                    className="px-2.5 py-1 rounded-lg bg-amber-500/25 hover:bg-amber-500/40 border border-amber-500/40 text-amber-100 font-semibold cursor-pointer transition-colors"
                  >
                    Sürümü Değiştir...
                  </button>
                )}
              </div>
            </div>
          )}

          {/* Confirmed Version Switcher Dialog */}
          {isChangingPhp && (
            <div className="p-4 rounded-xl bg-[#191337] border border-[#B7ADDF]/40 shadow-2xl space-y-3 font-['Figtree',sans-serif]">
              <div className="flex items-center justify-between">
                <h4 className="text-xs font-bold text-[#FFFBFF]">PHP Sürümünü Güncelle (Onaylı)</h4>
                <button
                  type="button"
                  onClick={() => setIsChangingPhp(false)}
                  className="text-[#D4CCF1] hover:text-[#FFFBFF]"
                >
                  <X className="w-3.5 h-3.5" />
                </button>
              </div>
              <p className="text-[11px] text-[#D4CCF1] leading-relaxed">
                "{project.name}" projesinin PHP sürümü değiştirilecektir. Bu işlem SQLite veritabanındaki kaydı günceller.
              </p>
              <div>
                <label className="block text-[11px] text-[#B7ADDF] mb-1">Yeni PHP Sürümü Seçin:</label>
                <select
                  value={targetPhp}
                  onChange={(e) => setTargetPhp(e.target.value)}
                  className="w-full px-3 py-2 rounded-lg bg-[#2B2450]/80 border border-[rgba(183,173,223,0.25)] text-xs text-white font-mono"
                >
                  {installedVersions.map((v) => (
                    <option key={v} value={v}>
                      PHP {v} (Doğrulanmış FPM arm64)
                    </option>
                  ))}
                </select>
              </div>
              {updateError && (
                <div className="text-[11px] text-red-300 bg-red-950/40 p-2 rounded border border-red-500/30">
                  {updateError}
                </div>
              )}
              <div className="flex justify-end gap-2 pt-2 border-t border-[rgba(183,173,223,0.12)]">
                <button
                  type="button"
                  onClick={() => setIsChangingPhp(false)}
                  className="px-3 py-1 rounded-lg border border-[rgba(183,173,223,0.2)] text-xs text-[#D4CCF1]"
                >
                  Vazgeç
                </button>
                <button
                  type="button"
                  disabled={updatingPhp || !targetPhp}
                  onClick={async () => {
                    if (!onUpdatePhpVersion) return;
                    setUpdatingPhp(true);
                    setUpdateError(null);
                    try {
                      await onUpdatePhpVersion(project.id, targetPhp);
                      setIsChangingPhp(false);
                    } catch (err: any) {
                      setUpdateError(err?.message || String(err));
                    } finally {
                      setUpdatingPhp(false);
                    }
                  }}
                  className="px-3 py-1 rounded-lg bg-[#695FA4] hover:bg-[#695FA4]/90 border border-[#B7ADDF] text-xs font-semibold text-white transition-all disabled:opacity-50"
                >
                  {updatingPhp ? "Kaydediliyor..." : "Onayla ve Kaydet"}
                </button>
              </div>
            </div>
          )}
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
            {loopbackPort && (
              <button
                type="button"
                onClick={copyCurlHelper}
                className="flex items-center gap-1 text-[11px] font-mono text-[#B7ADDF] hover:underline cursor-pointer"
              >
                {copied ? <Check className="w-3 h-3 text-[#10B981]" /> : <Copy className="w-3 h-3 text-[#B7ADDF]" />}
                <span>{copied ? "Kopyalandı" : "Kopyala"}</span>
              </button>
            )}
          </div>
          <code className="block p-2 rounded-lg bg-[#191337]/90 text-[10px] font-mono text-[#D4CCF1] break-all select-all border border-[rgba(183,173,223,0.10)]">
            {loopbackPort
              ? `curl -s -H "Host: ${preferredDomain}" http://127.0.0.1:${loopbackPort}/`
              : `# Port tahsis edilmedi (Projeyi başlattığınızda otomatik atanır)`}
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
          disabled={!project.loopback_port || project.status !== "running"}
          className={`flex-1 flex items-center justify-center gap-2 py-2.5 px-4 rounded-xl border font-semibold text-xs transition-colors ${
            !project.loopback_port || project.status !== "running"
              ? "bg-[#2B2450]/30 border-[rgba(183,173,223,0.1)] text-[#D4CCF1]/40 cursor-not-allowed"
              : "bg-[#2B2450]/60 hover:bg-[#695FA4]/30 border-[rgba(183,173,223,0.2)] text-[#FFFBFF] cursor-pointer"
          }`}
          title={
            !project.loopback_port
              ? "Port henüz atanmadı. Önce projeyi başlatınız."
              : project.status !== "running"
              ? "Proje durdurulmuş durumda."
              : "Tarayıcıda Aç"
          }
        >
          <ExternalLink className="w-3.5 h-3.5 text-[#B7ADDF]" />
          <span>Tarayıcıda Aç ↗</span>
        </button>
      </div>
    </div>
  );
};
