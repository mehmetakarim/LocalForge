import React from "react";
import { HardDrive, RefreshCw, ShieldCheck, CheckCircle2, Lock } from "lucide-react";
import { VolumeInfo, ProjectRecord } from "../types";
import { computeStorageCapacity } from "../utils/storageCapacity";

export interface StorageVolumesProps {
  volumes: VolumeInfo[];
  projects: ProjectRecord[];
  onRefresh: () => void;
}

export const StorageVolumes: React.FC<StorageVolumesProps> = ({
  volumes,
  projects,
  onRefresh,
}) => {
  const formatBytes = (bytes: number) => {
    if (bytes === 0) return "0 GB";
    const gb = bytes / (1024 * 1024 * 1024);
    if (gb >= 1000) {
      return `${(gb / 1024).toFixed(1)} TB`;
    }
    return `${gb.toFixed(0)} GB`;
  };

  return (
    <div className="w-full max-w-6xl py-8 px-6 flex flex-col gap-6">
      {/* Header */}
      <div className="flex items-end justify-between border-b border-[rgba(183,173,223,0.12)] pb-5">
        <div>
          <h1 className="font-['Plus_Jakarta_Sans',sans-serif] font-extrabold text-3xl tracking-[-0.03em] text-[#FFFBFF]">
            Depolama ve Birimler
          </h1>
          <p className="text-sm text-[#D4CCF1] mt-1 font-['Figtree',sans-serif]">
            Apple Silicon APFS topolojisi, harici SSD çalışma alanları ve VolumeBarrier güvenliği ({volumes.length} birim bağlı).
          </p>
        </div>

        <button
          type="button"
          onClick={onRefresh}
          className="flex items-center gap-2 px-4 py-2 rounded-xl bg-[#2B2450]/65 hover:bg-[#695FA4]/30 border border-[rgba(183,173,223,0.2)] text-xs font-semibold text-[#D4CCF1] hover:text-[#FFFBFF] transition-all cursor-pointer"
        >
          <RefreshCw className="w-4 h-4 text-[#B7ADDF]" />
          <span>Birimleri Yeniden Tara</span>
        </button>
      </div>

      {/* Volumes Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        {volumes.map((vol) => {
          const isExternal = !vol.is_internal || vol.is_removable;
          const assignedProjects = projects.filter((p) => p.volume_uuid === vol.uuid);
          const { hasValidCapacity, usedBytes, usagePercent, statusLabel } = computeStorageCapacity(vol);

          return (
            <div
              key={vol.uuid}
              className="rounded-2xl p-6 bg-[#2B2450]/65 border border-[rgba(183,173,223,0.16)] border-t-[1.5px] border-t-[rgba(183,173,223,0.35)] shadow-[0_16px_36px_-8px_rgba(10,6,30,0.6)] flex flex-col justify-between min-h-[350px]"
            >
              <div>
                <div className="flex items-start justify-between mb-4">
                  <div className="flex items-center gap-3">
                    <div
                      className={`w-10 h-10 rounded-xl flex items-center justify-center ${
                        !hasValidCapacity
                          ? "bg-slate-700/20 text-[#D4CCF1] border border-[rgba(183,173,223,0.2)]"
                          : isExternal
                          ? "bg-[#F59E0B]/20 text-[#F59E0B] border border-[#F59E0B]/30"
                          : "bg-[#695FA4]/20 text-[#B7ADDF] border border-[rgba(183,173,223,0.3)]"
                      }`}
                    >
                      <HardDrive className="w-5 h-5" />
                    </div>
                    <div>
                      <h3 className="font-['Plus_Jakarta_Sans',sans-serif] font-bold text-lg text-[#FFFBFF]">
                        {vol.name}
                      </h3>
                      <span className="font-mono text-xs text-[#D4CCF1]">
                        {vol.mount_point}
                      </span>
                    </div>
                  </div>

                  <div
                    className={`flex items-center gap-1.5 font-mono text-[11px] font-semibold px-2.5 py-0.5 rounded-full ${
                      !hasValidCapacity
                        ? "text-[#F59E0B] bg-[#F59E0B]/15 border border-[#F59E0B]/30"
                        : isExternal
                        ? "text-[#10B981] bg-[#10B981]/15 border border-[#10B981]/30"
                        : "text-[#D4CCF1] bg-[#6B7280]/15 border border-[#6B7280]/30"
                    }`}
                  >
                    <CheckCircle2
                      className={`w-3.5 h-3.5 ${
                        !hasValidCapacity ? "text-[#F59E0B]" : "text-[#10B981]"
                      }`}
                    />
                    <span>{statusLabel}</span>
                  </div>
                </div>

                {/* Disk Gauge */}
                <div className="my-5 p-4 rounded-xl bg-[#191337]/60 border border-[rgba(183,173,223,0.10)]">
                  <div className="flex justify-between font-mono text-xs text-[#D4CCF1] mb-2">
                    <span>
                      {hasValidCapacity
                        ? `Kullanılan: ${formatBytes(usedBytes)} (%${usagePercent})`
                        : "Kullanılan: BİLİNMİYOR"}
                    </span>
                    <span className="text-[#FFFBFF] font-semibold">
                      {hasValidCapacity
                        ? `Toplam: ${formatBytes(vol.total_bytes)}`
                        : "Toplam: KAPASİTE OKUNAMADI"}
                    </span>
                  </div>
                  <div className="w-full h-2 rounded-full bg-white/10 overflow-hidden">
                    <div
                      className={`h-full rounded-full transition-all duration-500 ${
                        !hasValidCapacity
                          ? "bg-slate-600/30"
                          : isExternal
                          ? "bg-[#B7ADDF]"
                          : "bg-[#695FA4]"
                      }`}
                      style={{ width: `${usagePercent ?? 0}%` }}
                    />
                  </div>
                </div>

                {/* Telemetry rows */}
                <div className="flex flex-col gap-2 font-mono text-xs">
                  <div className="flex justify-between py-1.5 border-b border-[rgba(183,173,223,0.08)]">
                    <span className="text-[#B7ADDF]">Kalıcı UUID:</span>
                    <span className="text-[#FFFBFF] truncate max-w-[220px]" title={vol.uuid}>
                      {vol.uuid}
                    </span>
                  </div>
                  <div className="flex justify-between py-1.5 border-b border-[rgba(183,173,223,0.08)]">
                    <span className="text-[#B7ADDF]">Dosya Sistemi:</span>
                    <span className="text-[#D4CCF1]">{vol.filesystem || "APFS"}</span>
                  </div>
                  <div className="flex justify-between py-1.5 border-b border-[rgba(183,173,223,0.08)]">
                    <span className="text-[#B7ADDF]">Birim Tipi:</span>
                    <span className="text-[#D4CCF1]">
                      {isExternal ? "Harici SSD (Tier-1 Çalışma Alanı)" : "Dahili APFS Kapsayıcısı"}
                    </span>
                  </div>
                  <div className="flex justify-between py-1.5">
                    <span className="text-[#B7ADDF]">Bağlı Projeler:</span>
                    <span className="text-[#10B981] font-semibold">
                      {assignedProjects.length} Proje
                    </span>
                  </div>
                </div>
              </div>

              <div className="pt-3 border-t border-[rgba(183,173,223,0.12)] flex items-center justify-between text-xs">
                <span className="font-mono text-[11px] text-[#B7ADDF]">
                  {isExternal ? "VolumeBarrier Koruması" : "Sistem Deposu"}
                </span>
                <span className="text-[#10B981] flex items-center gap-1">
                  <Lock className="w-3.5 h-3.5" />
                  <span>Güvenli İzolasyon</span>
                </span>
              </div>
            </div>
          );
        })}
      </div>

      {/* Safety Policy Banner */}
      <div className="rounded-2xl p-6 bg-gradient-to-r from-[#2B2450]/80 to-[#191337]/90 border border-[rgba(183,173,223,0.18)] border-l-4 border-l-[#B7ADDF] shadow-xl flex items-start gap-4">
        <ShieldCheck className="w-6 h-6 text-[#B7ADDF] shrink-0 mt-0.5" />
        <div className="flex flex-col gap-1.5">
          <h4 className="font-['Plus_Jakarta_Sans',sans-serif] font-bold text-base text-[#FFFBFF]">
            APFS Harici Disk Güvenlik Garantisi (R1.2B & R1.3B)
          </h4>
          <p className="text-xs text-[#D4CCF1] leading-relaxed">
            Harici disk (/Volumes/Mac Harici Disk) bağlantısı kesildiğinde veya çıkarıldığında, LocalForge asla dahili diske gizlice yedek dizin veya veritabanı kopyalamaz. <strong>VolumeBarrier</strong> mekanizması fail-closed (güvenli-kapalı) prensibiyle çalışarak disk tekrar bağlanana kadar yazma işlemlerini güvenle bekletir.
          </p>
        </div>
      </div>
    </div>
  );
};
