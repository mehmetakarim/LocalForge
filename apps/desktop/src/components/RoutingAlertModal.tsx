import React, { useState } from "react";
import { X, AlertTriangle, Copy, Check, Terminal, CheckCircle2, Globe } from "lucide-react";
import { ProjectRecord } from "../types";

export interface RoutingAlertModalProps {
  isOpen: boolean;
  onClose: () => void;
  project: ProjectRecord | null;
  port: number;
  url: string;
  curlCommand: string;
  onOpenDirect?: (url: string) => void;
}

export const RoutingAlertModal: React.FC<RoutingAlertModalProps> = ({
  isOpen,
  onClose,
  project,
  port,
  url,
  curlCommand,
  onOpenDirect,
}) => {
  const [copiedCurl, setCopiedCurl] = useState(false);
  const [copiedHosts, setCopiedHosts] = useState(false);
  const [copiedLoopback, setCopiedLoopback] = useState(false);

  if (!isOpen || !project) return null;

  const slug = project.name.toLowerCase().replace(/[^a-z0-9]/g, "-").replace(/-+/g, "-").replace(/^-|-$/g, "");
  const preferredDomain = `${slug}.localforge`;
  const isPortValid = typeof project.loopback_port === "number" && project.loopback_port >= 18081 && project.loopback_port <= 18999;
  const loopbackPort = isPortValid ? project.loopback_port : (typeof port === "number" && port >= 18081 && port <= 18999 ? port : null);
  const loopbackUrl = loopbackPort ? `http://127.0.0.1:${loopbackPort}/` : "";
  const localhostCandidateUrl = loopbackPort ? `http://${slug}.localforge.localhost:${loopbackPort}/` : "";

  const hostsCmd = `sudo sh -c 'echo "127.0.0.1 ${preferredDomain} ${project.local_domain}" >> /etc/hosts'`;

  const handleCopyCurl = () => {
    navigator.clipboard.writeText(curlCommand);
    setCopiedCurl(true);
    setTimeout(() => setCopiedCurl(false), 2000);
  };

  const handleCopyHosts = () => {
    navigator.clipboard.writeText(hostsCmd);
    setCopiedHosts(true);
    setTimeout(() => setCopiedHosts(false), 2000);
  };

  const handleCopyLoopback = () => {
    if (!loopbackUrl) return;
    navigator.clipboard.writeText(loopbackUrl);
    setCopiedLoopback(true);
    setTimeout(() => setCopiedLoopback(false), 2000);
  };

  const handleOpenDirectClick = () => {
    if (!loopbackUrl) return;
    if (onOpenDirect) {
      onOpenDirect(loopbackUrl);
    } else {
      window.open(loopbackUrl, "_blank");
    }
    onClose();
  };

  return (
    <div
      role="dialog"
      aria-modal="true"
      aria-label="Alan Adı Yönlendirme Uyarısı ve Tanılama"
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/75 backdrop-blur-xl p-4 animate-in fade-in duration-200 select-none"
    >
      <div className="bg-[#191337] border border-[rgba(183,173,223,0.22)] rounded-3xl w-full max-w-xl p-7 shadow-2xl shadow-black/80 relative max-h-[90vh] overflow-y-auto">
        {/* Close Button */}
        <button
          type="button"
          onClick={onClose}
          aria-label="Kapat"
          className="absolute top-5 right-5 w-7 h-7 rounded-full bg-[#2B2450]/60 hover:bg-[#695FA4]/30 border border-[rgba(183,173,223,0.18)] flex items-center justify-center text-[#D4CCF1] hover:text-[#FFFBFF] transition-all cursor-pointer"
        >
          <X className="w-4 h-4" />
        </button>

        {/* Modal Header */}
        <div className="flex items-center gap-3.5 mb-5">
          <div className="w-10 h-10 rounded-2xl bg-amber-500/20 border border-amber-400/40 flex items-center justify-center text-amber-300 shadow-lg shadow-amber-500/20 shrink-0">
            <AlertTriangle className="w-5 h-5" />
          </div>
          <div>
            <h3 className="font-['Plus_Jakarta_Sans',sans-serif] font-bold text-lg text-[#FFFBFF]">
              Alan Adı Yönlendirmesi ve DNS Tanılaması
            </h3>
            <p className="text-xs text-[#D4CCF1] font-mono">
              {url} ({project.local_domain})
            </p>
          </div>
        </div>

        {/* Diagnostic Pipeline Matrix */}
        <div className="grid grid-cols-2 gap-2.5 mb-4 text-xs font-mono">
          <div className="p-2.5 rounded-xl bg-[#2B2450]/50 border border-[rgba(183,173,223,0.12)] flex items-center justify-between">
            <span className="text-[#D4CCF1]">Nginx Ağ Geçidi (8080):</span>
            <span className="text-[#10B981] font-semibold flex items-center gap-1">
              <CheckCircle2 className="w-3.5 h-3.5" /> Dinlemede
            </span>
          </div>
          <div className="p-2.5 rounded-xl bg-[#2B2450]/50 border border-[rgba(183,173,223,0.12)] flex items-center justify-between">
            <span className="text-[#D4CCF1]">VHost Eşleşmesi:</span>
            <span className="text-[#10B981] font-semibold flex items-center gap-1">
              <CheckCircle2 className="w-3.5 h-3.5" /> Tanımlı
            </span>
          </div>
          <div className="p-2.5 rounded-xl bg-[#2B2450]/50 border border-[rgba(183,173,223,0.12)] flex items-center justify-between">
            <span className="text-[#D4CCF1]">PHP-FPM Havuzu:</span>
            <span className={project.status === "running" ? "text-[#10B981] font-semibold" : "text-[#F59E0B] font-semibold"}>
              {project.status === "running" ? "Çevrimiçi" : "Durduruldu"}
            </span>
          </div>
          <div className="p-2.5 rounded-xl bg-[#2B2450]/50 border border-[rgba(183,173,223,0.12)] flex items-center justify-between">
            <span className="text-[#D4CCF1]">macOS Sistem DNS:</span>
            <span className="text-[#EF4444] font-semibold flex items-center gap-1">
              <AlertTriangle className="w-3.5 h-3.5" /> NXDOMAIN
            </span>
          </div>
        </div>

        {/* Explanation */}
        <div className="space-y-3.5 text-xs text-[#D4CCF1] font-['Figtree',sans-serif] leading-relaxed">
          <p>
            macOS varsayılan sistem DNS'i <code className="text-[#B7ADDF]">.localforge</code> ve <code className="text-[#B7ADDF]">.test</code> uzantılarını henüz yerel IP adresine (<code className="text-[#B7ADDF]">127.0.0.1</code>) yönlendirmemektedir.
          </p>

          {loopbackPort ? (
            <div className="p-3.5 rounded-xl bg-[#10B981]/10 border border-[#10B981]/30 text-emerald-200 text-[11px] leading-snug space-y-1">
              <strong className="text-white block flex items-center gap-1.5">
                <CheckCircle2 className="w-3.5 h-3.5 text-[#10B981]" />
                Gate U5.B Ayrık Loopback Güvencesi:
              </strong>
              <span>
                Bu projeye özel ve izole port <strong className="text-white font-mono">127.0.0.1:{loopbackPort}</strong> tahsis edilmiştir. Yanlış kiracı riski olmaksızın doğrudan bu adresi tarayıcınızda açabilirsiniz!
              </span>
            </div>
          ) : (
            <div className="p-3.5 rounded-xl bg-amber-500/10 border border-amber-500/30 text-amber-200 text-[11px] leading-snug space-y-1">
              <strong className="text-white block flex items-center gap-1.5">
                <AlertTriangle className="w-3.5 h-3.5 text-amber-400" />
                Loopback Port Tahsis Edilmedi:
              </strong>
              <span>
                Bu proje için henüz 18081-18999 aralığında geçerli bir ayrık port atanmamıştır. Port tahsisi yapılana kadar doğrudan loopback erişimi kısıtlanmıştır.
              </span>
            </div>
          )}

          {/* Action 1: Direct Loopback Access */}
          <div className="p-3 rounded-xl bg-[#2B2450]/45 border border-[rgba(183,173,223,0.14)] flex items-center justify-between">
            <div className="font-mono text-[11px]">
              <span className="text-[#D4CCF1]/70 block text-[10px]">Doğrudan Loopback Uç Noktası:</span>
              {loopbackPort ? (
                <>
                  <span className="text-[#10B981] font-bold">{loopbackUrl}</span>
                  <span className="text-[#D4CCF1]/60 block text-[10px] mt-0.5">RFC 6761 Adayı: {localhostCandidateUrl}</span>
                </>
              ) : (
                <span className="text-amber-400 font-bold">Port Atanmadı</span>
              )}
            </div>
            {loopbackPort && (
              <button
                type="button"
                onClick={handleCopyLoopback}
                className="flex items-center gap-1 text-[11px] font-mono text-[#B7ADDF] hover:text-[#FFFBFF] cursor-pointer"
              >
                {copiedLoopback ? (
                  <>
                    <Check className="w-3 h-3 text-emerald-400" />
                    <span className="text-emerald-400">Kopyalandı</span>
                  </>
                ) : (
                  <>
                    <Copy className="w-3 h-3" />
                    <span>Kopyala</span>
                  </>
                )}
              </button>
            )}
          </div>

          {/* Action 2: Add to /etc/hosts */}
          <div>
            <div className="flex items-center justify-between mb-1">
              <span className="font-semibold text-[#FFFBFF] flex items-center gap-1.5">
                <Globe className="w-3.5 h-3.5 text-[#B7ADDF]" />
                <span>Tercih Edilen Alan Adı /etc/hosts Kaydı (İsteğe Bağlı):</span>
              </span>
              <button
                type="button"
                onClick={handleCopyHosts}
                className="flex items-center gap-1 text-[11px] font-mono text-[#B7ADDF] hover:text-[#FFFBFF] cursor-pointer"
              >
                {copiedHosts ? (
                  <>
                    <Check className="w-3 h-3 text-emerald-400" />
                    <span className="text-emerald-400">Kopyalandı</span>
                  </>
                ) : (
                  <>
                    <Copy className="w-3 h-3" />
                    <span>Komutu Kopyala</span>
                  </>
                )}
              </button>
            </div>
            <pre className="p-2 rounded-lg bg-[#0A061E]/90 border border-[rgba(183,173,223,0.18)] font-mono text-[11px] text-[#FFFBFF] overflow-x-auto whitespace-pre-wrap break-all">
              {hostsCmd}
            </pre>
          </div>

          {/* Action 3: cURL helper */}
          <div>
            <div className="flex items-center justify-between mb-1">
              <span className="font-semibold text-[#FFFBFF] flex items-center gap-1.5">
                <Terminal className="w-3.5 h-3.5 text-[#B7ADDF]" />
                <span>Anında cURL Test Komutu:</span>
              </span>
              <button
                type="button"
                onClick={handleCopyCurl}
                className="flex items-center gap-1 text-[11px] font-mono text-[#B7ADDF] hover:text-[#FFFBFF] cursor-pointer"
              >
                {copiedCurl ? (
                  <>
                    <Check className="w-3 h-3 text-emerald-400" />
                    <span className="text-emerald-400">Kopyalandı</span>
                  </>
                ) : (
                  <>
                    <Copy className="w-3 h-3" />
                    <span>Komutu Kopyala</span>
                  </>
                )}
              </button>
            </div>
            <pre className="p-2 rounded-lg bg-[#0A061E]/90 border border-[rgba(183,173,223,0.18)] font-mono text-[11px] text-[#B7ADDF] overflow-x-auto whitespace-pre-wrap break-all">
              {curlCommand}
            </pre>
          </div>
        </div>

        {/* Footer */}
        <div className="flex items-center justify-between gap-3 pt-4 mt-4 border-t border-[rgba(183,173,223,0.12)]">
          <button
            type="button"
            onClick={onClose}
            className="px-4 py-2 rounded-xl bg-[#2B2450]/60 hover:bg-[#695FA4]/30 border border-[rgba(183,173,223,0.18)] text-xs text-[#D4CCF1] hover:text-[#FFFBFF] transition-all cursor-pointer"
          >
            Kapat
          </button>
          {loopbackPort ? (
            <button
              type="button"
              onClick={handleOpenDirectClick}
              className="px-4 py-2 rounded-xl bg-gradient-to-r from-[#695FA4] to-[#2B2450] hover:brightness-110 border border-[#B7ADDF] text-xs font-semibold text-[#FFFBFF] transition-all shadow-[0_4px_16px_rgba(105,95,164,0.35)] cursor-pointer"
            >
              Doğrudan Aç (127.0.0.1:{loopbackPort}) ↗
            </button>
          ) : (
            <button
              type="button"
              disabled
              className="px-4 py-2 rounded-xl bg-[#2B2450]/40 border border-[rgba(183,173,223,0.1)] text-xs font-semibold text-[#D4CCF1]/40 cursor-not-allowed"
            >
              Port Tahsis Edilmedi
            </button>
          )}
        </div>
      </div>
    </div>
  );
};
