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
}

export const RoutingAlertModal: React.FC<RoutingAlertModalProps> = ({
  isOpen,
  onClose,
  project,
  port,
  url,
  curlCommand,
}) => {
  const [copiedCurl, setCopiedCurl] = useState(false);
  const [copiedHosts, setCopiedHosts] = useState(false);

  if (!isOpen || !project) return null;

  const hostsCmd = `sudo sh -c 'echo "127.0.0.1 ${project.local_domain}" >> /etc/hosts'`;

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
            macOS varsayılan sistem DNS'i <code className="text-[#B7ADDF]">.test</code> uzantılı alan adlarını yerel IP adresine (<code className="text-[#B7ADDF]">127.0.0.1</code>) otomatik yönlendirmez. Bu nedenle tarayıcınız bu adrese doğrudan erişememektedir.
          </p>

          <div className="p-3.5 rounded-xl bg-amber-500/10 border border-amber-500/25 text-amber-200 text-[11px] leading-snug">
            <strong>Kiracı İzolasyonu Güvencesi:</strong> Yanlış veya başka bir projenin sitesinin sunulmasını önlemek amacıyla, doğrulanmamış ham IP (<code className="text-white">http://127.0.0.1:{port}/</code>) üzerinden tarayıcı açılması engellenmiştir.
          </div>

          {/* Action 1: Add to /etc/hosts */}
          <div>
            <div className="flex items-center justify-between mb-1">
              <span className="font-semibold text-[#FFFBFF] flex items-center gap-1.5">
                <Globe className="w-3.5 h-3.5 text-[#B7ADDF]" />
                <span>Çözüm 1: Yerel /etc/hosts Kaydı Ekleme (Terminal):</span>
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

          {/* Action 2: cURL helper */}
          <div>
            <div className="flex items-center justify-between mb-1">
              <span className="font-semibold text-[#FFFBFF] flex items-center gap-1.5">
                <Terminal className="w-3.5 h-3.5 text-[#B7ADDF]" />
                <span>Çözüm 2: Anında cURL ile HTTP Başlığı Doğrulama:</span>
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

          {/* Architectural Discussion for PO Review */}
          <div className="p-3 rounded-xl bg-[#2B2450]/40 border border-[rgba(183,173,223,0.14)] text-[11px] space-y-1">
            <span className="font-bold text-[#FFFBFF] block">
              Gelecek Aşama İçin Mimari Yaklaşım Seçenekleri:
            </span>
            <ul className="list-disc pl-4 space-y-1 text-[#D4CCF1]">
              <li>
                <strong>Yaklaşım A (Ayrık Loopback Portları):</strong> Her projeye ayrı port verilmesi (<code className="text-white">127.0.0.1:8081</code>). DNS müdahalesi gerektirmez.
              </li>
              <li>
                <strong>Yaklaşım B (RFC 6761 .localhost Alt Alan Adı):</strong> Tarayıcıların yerel olarak çözümlediği <code className="text-white">proje.localforge.localhost:8080</code> formatına geçiş.
              </li>
            </ul>
          </div>
        </div>

        {/* Footer */}
        <div className="flex items-center justify-end gap-3 pt-4 mt-4 border-t border-[rgba(183,173,223,0.12)]">
          <button
            type="button"
            onClick={onClose}
            className="px-5 py-2 rounded-xl bg-[#695FA4] hover:bg-[#695FA4]/90 text-xs font-semibold text-[#FFFBFF] border border-[#B7ADDF] transition-all cursor-pointer"
          >
            Anladım, Kapat
          </button>
        </div>
      </div>
    </div>
  );
};
