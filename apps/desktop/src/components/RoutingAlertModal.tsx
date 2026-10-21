import React, { useState } from "react";
import { X, AlertTriangle, Copy, Check, Terminal } from "lucide-react";
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
  const [copied, setCopied] = useState(false);

  if (!isOpen || !project) return null;

  const handleCopy = () => {
    navigator.clipboard.writeText(curlCommand);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div
      role="dialog"
      aria-modal="true"
      aria-label="Alan Adı Yönlendirme Uyarısı"
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/75 backdrop-blur-xl p-4 animate-in fade-in duration-200 select-none"
    >
      <div className="bg-[#191337] border border-[rgba(183,173,223,0.22)] rounded-3xl w-full max-w-lg p-7 shadow-2xl shadow-black/80 relative">
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
          <div className="w-10 h-10 rounded-2xl bg-amber-500/20 border border-amber-400/40 flex items-center justify-center text-amber-300 shadow-lg shadow-amber-500/20">
            <AlertTriangle className="w-5 h-5" />
          </div>
          <div>
            <h3 className="font-['Plus_Jakarta_Sans',sans-serif] font-bold text-lg text-[#FFFBFF]">
              Alan Adı Yönlendirmesi Yapılandırılmadı
            </h3>
            <p className="text-xs text-[#D4CCF1] font-mono">
              {url}
            </p>
          </div>
        </div>

        {/* Explanation */}
        <div className="space-y-3.5 text-xs text-[#D4CCF1] font-['Figtree',sans-serif] leading-relaxed">
          <p>
            Tarayıcınızın <strong className="text-[#FFFBFF]">{url}</strong> ({project.local_domain}) adresini doğrudan açabilmesi için bu alan adının yerel makinenizde (<code className="text-[#B7ADDF]">127.0.0.1</code>) çözümlenmesi gerekmektedir.
          </p>

          <div className="p-3.5 rounded-xl bg-amber-500/10 border border-amber-500/25 text-amber-200 text-[11px] leading-snug">
            <strong>Güvenlik Garantisi:</strong> Yanlış veya başka bir projenin sitesinin sunulmasını önlemek amacıyla, doğrulanmamış ham IP (<code className="text-white">http://127.0.0.1:{port}/</code>) üzerinden tarayıcı açılması engellenmiştir.
          </div>

          <div>
            <span className="block font-semibold text-[#FFFBFF] mb-1">
              Kalıcı Çözüm:
            </span>
            <p className="text-[11px] text-[#D4CCF1]">
              macOS <code className="text-[#B7ADDF]">/etc/hosts</code> dosyanıza şu satırı ekleyin:
            </p>
            <div className="mt-1 p-2 rounded-lg bg-[#2B2450]/60 border border-[rgba(183,173,223,0.14)] font-mono text-xs text-[#FFFBFF]">
              127.0.0.1 {project.local_domain}
            </div>
          </div>

          <div>
            <div className="flex items-center justify-between mb-1">
              <span className="font-semibold text-[#FFFBFF] flex items-center gap-1.5">
                <Terminal className="w-3.5 h-3.5 text-[#B7ADDF]" />
                <span>Alternatif Hızlı Test (Terminal / cURL):</span>
              </span>
              <button
                type="button"
                onClick={handleCopy}
                className="flex items-center gap-1 text-[11px] font-mono text-[#B7ADDF] hover:text-[#FFFBFF] cursor-pointer"
              >
                {copied ? (
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
            </div>
            <pre className="p-2.5 rounded-lg bg-[#191337] border border-[rgba(183,173,223,0.18)] font-mono text-[11px] text-[#B7ADDF] overflow-x-auto whitespace-pre-wrap break-all">
              {curlCommand}
            </pre>
          </div>
        </div>

        {/* Footer */}
        <div className="flex items-center justify-end gap-3 pt-5 mt-4 border-t border-[rgba(183,173,223,0.12)]">
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
