import React from "react";
import { RefreshCw, ShieldCheck, CheckCircle2, AlertTriangle, XCircle } from "lucide-react";
import { RuntimeOverview } from "../types";
import { deriveServiceStates } from "../utils/servicesTelemetry";

export interface ServicesRuntimeProps {
  runtime: RuntimeOverview | null;
  onRefresh: () => void;
  onStartService?: (service: string) => Promise<void>;
  onStopService?: (service: string) => Promise<void>;
}

export const ServicesRuntime: React.FC<ServicesRuntimeProps> = ({
  runtime,
  onRefresh,
  onStartService,
  onStopService,
}) => {
  const { nginxStatus, phpFpmStatus, mariadbStatus } = deriveServiceStates(runtime);

  return (
    <div className="w-full max-w-6xl py-8 px-6 flex flex-col gap-6">
      {/* Header */}
      <div className="flex items-end justify-between border-b border-[rgba(183,173,223,0.12)] pb-5">
        <div>
          <h1 className="font-['Plus_Jakarta_Sans',sans-serif] font-extrabold text-3xl tracking-[-0.03em] text-[#FFFBFF]">
            Servisler ve Çalışma Zamanı
          </h1>
          <p className="text-sm text-[#D4CCF1] mt-1 font-['Figtree',sans-serif]">
            LocalForge mimarisi tarafından yönetilen doğrulanmış arka plan süreçleri ve port sahiplikleri.
          </p>
        </div>

        <button
          type="button"
          onClick={onRefresh}
          className="flex items-center gap-2 px-4 py-2 rounded-xl bg-[#2B2450]/65 hover:bg-[#695FA4]/30 border border-[rgba(183,173,223,0.2)] text-xs font-semibold text-[#D4CCF1] hover:text-[#FFFBFF] transition-all cursor-pointer"
        >
          <RefreshCw className="w-4 h-4 text-[#B7ADDF]" />
          <span>Servisleri Yeniden Tara</span>
        </button>
      </div>

      {/* 3 Main Service Cards Grid */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
        {/* Card 1: Nginx Gateway */}
        <div className="rounded-2xl p-6 bg-[#2B2450]/65 border border-[rgba(183,173,223,0.16)] border-t-[1.5px] border-t-[rgba(183,173,223,0.35)] shadow-[0_16px_36px_-8px_rgba(10,6,30,0.6)] flex flex-col justify-between min-h-[350px]">
          <div>
            <div className="flex items-start justify-between mb-4">
              <div>
                <h3 className="font-['Plus_Jakarta_Sans',sans-serif] font-bold text-lg text-[#FFFBFF]">
                  Nginx Ağ Geçidi
                </h3>
                <span className="font-mono text-xs text-[#D4CCF1]">Ters Proxy & Yönlendirici</span>
              </div>

              {nginxStatus === "running" ? (
                <div className="flex items-center gap-1.5 font-mono text-[11px] font-semibold text-[#10B981] bg-[#10B981]/15 border border-[#10B981]/30 px-2.5 py-0.5 rounded-full">
                  <CheckCircle2 className="w-3.5 h-3.5 text-[#10B981]" />
                  <span>DOĞRULANDI</span>
                </div>
              ) : nginxStatus === "conflict" ? (
                <div className="flex items-center gap-1.5 font-mono text-[11px] font-semibold text-[#EF4444] bg-[#EF4444]/15 border border-[#EF4444]/30 px-2.5 py-0.5 rounded-full">
                  <XCircle className="w-3.5 h-3.5 text-[#EF4444]" />
                  <span>PORT ÇAKIŞMASI</span>
                </div>
              ) : nginxStatus === "stopped" ? (
                <div className="flex items-center gap-1.5 font-mono text-[11px] font-semibold text-[#D4CCF1] bg-[#6B7280]/15 border border-[#6B7280]/30 px-2.5 py-0.5 rounded-full">
                  <span>DURDURULDU</span>
                </div>
              ) : (
                <div className="flex items-center gap-1.5 font-mono text-[11px] font-semibold text-[#F59E0B] bg-[#F59E0B]/15 border border-[#F59E0B]/30 px-2.5 py-0.5 rounded-full">
                  <AlertTriangle className="w-3.5 h-3.5 text-[#F59E0B]" />
                  <span>BİLİNMİYOR</span>
                </div>
              )}
            </div>

            {/* Telemetry rows */}
            <div className="flex flex-col gap-2.5 mb-4">
              <div className="p-2.5 rounded-xl bg-[#191337]/60 border border-[rgba(183,173,223,0.10)] flex flex-col gap-1">
                <span className="font-mono text-[10px] uppercase text-[#B7ADDF] tracking-wider">
                  Dinleme Portu
                </span>
                <span className="font-mono text-xs text-[#FFFBFF] font-semibold">
                  {runtime?.nginx_port ? `127.0.0.1:${runtime.nginx_port}` : "Bilinmiyor"}
                </span>
              </div>

              <div className="p-2.5 rounded-xl bg-[#191337]/60 border border-[rgba(183,173,223,0.10)] flex flex-col gap-1">
                <span className="font-mono text-[10px] uppercase text-[#B7ADDF] tracking-wider">
                  Süreç Sahipliği & Kimliği
                </span>
                <span className="font-mono text-xs text-[#D4CCF1]">
                  {nginxStatus === "running"
                    ? "LocalForge Yönetiminde (Doğrulanmış)"
                    : nginxStatus === "conflict"
                    ? "Yabancı Süreç Tarafından İşgal Edildi"
                    : nginxStatus === "stopped"
                    ? "Süreç Beklemede"
                    : "Telemetri Alınamadı (Bilinmiyor)"}
                </span>
              </div>

              <div className="p-2.5 rounded-xl bg-[#191337]/60 border border-[rgba(183,173,223,0.10)] flex flex-col gap-1">
                <span className="font-mono text-[10px] uppercase text-[#B7ADDF] tracking-wider">
                  Doğrulama Modeli
                </span>
                <span className="font-mono text-xs text-[#D4CCF1]">
                  Kanonik Yapılandırma & Port-PID Eşleşmesi
                </span>
              </div>
            </div>
          </div>

          <div className="pt-3 border-t border-[rgba(183,173,223,0.12)] flex items-center justify-between text-xs">
            <span className="font-mono text-[11px] text-[#B7ADDF]">HTTP/1.1 Loopback</span>
            <span className="text-[#D4CCF1]">İzole Mod</span>
          </div>
        </div>

        {/* Card 2: PHP-FPM Pool */}
        <div className="rounded-2xl p-6 bg-[#2B2450]/65 border border-[rgba(183,173,223,0.16)] border-t-[1.5px] border-t-[rgba(183,173,223,0.35)] shadow-[0_16px_36px_-8px_rgba(10,6,30,0.6)] flex flex-col justify-between min-h-[350px]">
          <div>
            <div className="flex items-start justify-between mb-4">
              <div>
                <h3 className="font-['Plus_Jakarta_Sans',sans-serif] font-bold text-lg text-[#FFFBFF]">
                  PHP-FPM Havuzu
                </h3>
                <span className="font-mono text-xs text-[#D4CCF1]">Hızlı CGI Süreç Yöneticisi</span>
              </div>

              {phpFpmStatus === "running" ? (
                <div className="flex items-center gap-1.5 font-mono text-[11px] font-semibold text-[#10B981] bg-[#10B981]/15 border border-[#10B981]/30 px-2.5 py-0.5 rounded-full">
                  <CheckCircle2 className="w-3.5 h-3.5 text-[#10B981]" />
                  <span>AKTİF</span>
                </div>
              ) : phpFpmStatus === "stopped" ? (
                <div className="flex items-center gap-1.5 font-mono text-[11px] font-semibold text-[#D4CCF1] bg-[#6B7280]/15 border border-[#6B7280]/30 px-2.5 py-0.5 rounded-full">
                  <span>BEKLEMEDE</span>
                </div>
              ) : (
                <div className="flex items-center gap-1.5 font-mono text-[11px] font-semibold text-[#F59E0B] bg-[#F59E0B]/15 border border-[#F59E0B]/30 px-2.5 py-0.5 rounded-full">
                  <AlertTriangle className="w-3.5 h-3.5 text-[#F59E0B]" />
                  <span>BİLİNMİYOR</span>
                </div>
              )}
            </div>

            {/* Telemetry rows */}
            <div className="flex flex-col gap-2.5 mb-4">
              <div className="p-2.5 rounded-xl bg-[#191337]/60 border border-[rgba(183,173,223,0.10)] flex flex-col gap-1">
                <span className="font-mono text-[10px] uppercase text-[#B7ADDF] tracking-wider">
                  Tespit Edilen Sürümler
                </span>
                <span className="font-mono text-xs text-[#FFFBFF] font-semibold">
                  {runtime?.php_versions && runtime.php_versions.length > 0
                    ? runtime.php_versions.join(", ")
                    : runtime
                    ? "Tespit Edilmedi"
                    : "Bilinmiyor"}
                </span>
              </div>

              <div className="p-2.5 rounded-xl bg-[#191337]/60 border border-[rgba(183,173,223,0.10)] flex flex-col gap-1">
                <span className="font-mono text-[10px] uppercase text-[#B7ADDF] tracking-wider">
                  Süreç Mimarisi
                </span>
                <span className="font-mono text-xs text-[#D4CCF1]">
                  Kullanıcı Seviyesi (Non-Root) UNIX Soketleri
                </span>
              </div>

              <div className="p-2.5 rounded-xl bg-[#191337]/60 border border-[rgba(183,173,223,0.10)] flex flex-col gap-1">
                <span className="font-mono text-[10px] uppercase text-[#B7ADDF] tracking-wider">
                  Havuz İzolasyonu
                </span>
                <span className="font-mono text-xs text-[#D4CCF1]">
                  Proje Başına Ayrı php-fpm.conf ve Soket
                </span>
              </div>
            </div>
          </div>

          <div className="pt-3 border-t border-[rgba(183,173,223,0.12)] flex items-center justify-between text-xs">
            <span className="font-mono text-[11px] text-[#B7ADDF]">Apple Silicon arm64</span>
            <span className="text-[#D4CCF1]">Dinamik Havuz</span>
          </div>
        </div>

        {/* Card 3: MariaDB Engine */}
        <div className="rounded-2xl p-6 bg-[#2B2450]/65 border border-[rgba(183,173,223,0.16)] border-t-[1.5px] border-t-[rgba(183,173,223,0.35)] shadow-[0_16px_36px_-8px_rgba(10,6,30,0.6)] flex flex-col justify-between min-h-[350px]">
          <div>
            <div className="flex items-start justify-between mb-4">
              <div>
                <h3 className="font-['Plus_Jakarta_Sans',sans-serif] font-bold text-lg text-[#FFFBFF]">
                  MariaDB Motoru
                </h3>
                <span className="font-mono text-xs text-[#D4CCF1]">İlişkisel Veritabanı Sunucusu</span>
              </div>

              {mariadbStatus === "running" ? (
                <div className="flex items-center gap-1.5 font-mono text-[11px] font-semibold text-[#10B981] bg-[#10B981]/15 border border-[#10B981]/30 px-2.5 py-0.5 rounded-full">
                  <CheckCircle2 className="w-3.5 h-3.5 text-[#10B981]" />
                  <span>AKTİF</span>
                </div>
              ) : mariadbStatus === "stopped" ? (
                <div className="flex items-center gap-1.5 font-mono text-[11px] font-semibold text-[#D4CCF1] bg-[#6B7280]/15 border border-[#6B7280]/30 px-2.5 py-0.5 rounded-full">
                  <span>DURDURULDU</span>
                </div>
              ) : (
                <div className="flex items-center gap-1.5 font-mono text-[11px] font-semibold text-[#F59E0B] bg-[#F59E0B]/15 border border-[#F59E0B]/30 px-2.5 py-0.5 rounded-full">
                  <AlertTriangle className="w-3.5 h-3.5 text-[#F59E0B]" />
                  <span>BİLİNMİYOR</span>
                </div>
              )}
            </div>

            {/* Telemetry rows */}
            <div className="flex flex-col gap-2.5 mb-4">
              <div className="p-2.5 rounded-xl bg-[#191337]/60 border border-[rgba(183,173,223,0.10)] flex flex-col gap-1">
                <span className="font-mono text-[10px] uppercase text-[#B7ADDF] tracking-wider">
                  Dinleme Portu & Soket
                </span>
                <span className="font-mono text-xs text-[#FFFBFF] font-semibold">
                  {runtime?.mariadb_port ? `127.0.0.1:${runtime.mariadb_port}` : "Bilinmiyor"}
                </span>
              </div>

              <div className="p-2.5 rounded-xl bg-[#191337]/60 border border-[rgba(183,173,223,0.10)] flex flex-col gap-1">
                <span className="font-mono text-[10px] uppercase text-[#B7ADDF] tracking-wider">
                  Veritabanı Sürümü
                </span>
                <span className="font-mono text-xs text-[#D4CCF1]">
                  {runtime?.mariadb_version || (runtime ? "Tespit Edilemedi" : "Bilinmiyor")}
                </span>
              </div>

              <div className="p-2.5 rounded-xl bg-[#191337]/60 border border-[rgba(183,173,223,0.10)] flex flex-col gap-1">
                <span className="font-mono text-[10px] uppercase text-[#B7ADDF] tracking-wider">
                  Veri Dizini &amp; İlklendirme
                </span>
                <span className="font-mono text-xs text-[#D4CCF1]">
                  {runtime?.mariadb_datadir_initialized
                    ? "İlklendirilmiş (LocalForge İzolasyonu)"
                    : "Boş / İlklendirilmemiş (Güvenlik Korumasında)"}
                </span>
              </div>

              <div className="p-2.5 rounded-xl bg-[#191337]/60 border border-[rgba(183,173,223,0.10)] flex flex-col gap-1">
                <span className="font-mono text-[10px] uppercase text-[#B7ADDF] tracking-wider">
                  Canlı Yoklama Gecikmesi
                </span>
                <span className="font-mono text-xs text-[#D4CCF1]">
                  {runtime?.mariadb_latency_ms !== undefined && runtime?.mariadb_latency_ms !== null
                    ? `${runtime.mariadb_latency_ms} ms (Doğrulandı)`
                    : runtime
                    ? "Yoklama Tamamlanmadı"
                    : "Bilinmiyor"}
                </span>
              </div>
            </div>
          </div>

          <div className="pt-3 border-t border-[rgba(183,173,223,0.12)] flex items-center justify-between text-xs">
            <span className="font-mono text-[11px] text-[#B7ADDF]">
              {mariadbStatus === "running" ? "127.0.0.1:3306" : "Durduruldu"}
            </span>
            {onStartService && onStopService && (
              mariadbStatus === "running" ? (
                <button
                  type="button"
                  onClick={() => onStopService("mariadb")}
                  className="px-3 py-1 rounded-lg bg-red-500/20 hover:bg-red-500/30 border border-red-500/40 text-red-200 text-xs font-semibold cursor-pointer transition-all"
                >
                  Durdur
                </button>
              ) : (
                <button
                  type="button"
                  onClick={() => onStartService("mariadb")}
                  className="px-3 py-1 rounded-lg bg-[#10B981]/20 hover:bg-[#10B981]/30 border border-[#10B981]/40 text-[#10B981] text-xs font-semibold cursor-pointer transition-all"
                >
                  Başlat
                </button>
              )
            )}
          </div>
        </div>
      </div>

      {/* Grounded Ownership & Process Identity Panel */}
      <div className="rounded-2xl p-6 bg-gradient-to-r from-[#2B2450]/80 to-[#191337]/90 border border-[rgba(183,173,223,0.18)] border-l-4 border-l-[#B7ADDF] shadow-xl flex items-start gap-4">
        <ShieldCheck className="w-6 h-6 text-[#B7ADDF] shrink-0 mt-0.5" />
        <div className="flex flex-col gap-1.5">
          <h4 className="font-['Plus_Jakarta_Sans',sans-serif] font-bold text-base text-[#FFFBFF]">
            Süreç Kimliği ve Sahiplik Doğrulama Mimarisi (R1.3B)
          </h4>
          <p className="text-xs text-[#D4CCF1] leading-relaxed">
            LocalForge, port 8080'de rastgele bir Nginx çalışıyor diye gateway'i sağlıklı kabul etmez. Homebrew veya yabancı Nginx süreçleri ile LocalForge tarafından başlatılmış izole süreçler ayrıştırılır. Port başka bir süreç tarafından işgal edilmişse sistem durumu otomatik olarak <strong>Port Çakışması</strong> olarak raporlanır ve güvenli izole mod korunur.
          </p>
        </div>
      </div>
    </div>
  );
};
