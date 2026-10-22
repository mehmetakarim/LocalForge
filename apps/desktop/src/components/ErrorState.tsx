import React from "react";
import { AlertTriangle, RefreshCw, X } from "lucide-react";

export interface ErrorStateProps {
  title?: string;
  message: string;
  onRetry?: () => void;
  onDismiss?: () => void;
}

export const ErrorState: React.FC<ErrorStateProps> = ({
  title = "Sistem İşlem Hatası",
  message,
  onRetry,
  onDismiss,
}) => {
  return (
    <div
      role="alert"
      className="p-5 rounded-2xl bg-[#191337]/95 border border-[#EF4444]/40 shadow-[-10px_10px_35px_rgba(10,6,30,0.85)] backdrop-blur-3xl flex flex-col gap-3 select-none text-left"
    >
      <div className="flex items-start justify-between gap-3">
        <div className="flex items-center gap-3">
          <div className="flex items-center justify-center w-8 h-8 rounded-xl bg-[#EF4444]/15 border border-[#EF4444]/35 text-[#EF4444] shadow-[0_0_15px_rgba(239,68,68,0.3)] shrink-0">
            <AlertTriangle className="w-4 h-4" />
          </div>
          <div>
            <h3 className="text-xs font-['Plus_Jakarta_Sans',sans-serif] font-bold text-[#FFFBFF] tracking-tight">
              {title}
            </h3>
            <span className="font-mono text-[10px] text-[#EF4444] uppercase tracking-wider font-semibold">
              Çalışma Zamanı Uyarısı
            </span>
          </div>
        </div>

        {onDismiss && (
          <button
            type="button"
            onClick={onDismiss}
            className="w-6 h-6 rounded-lg bg-[#2B2450]/60 hover:bg-[#695FA4]/30 border border-[rgba(183,173,223,0.18)] text-[#D4CCF1] hover:text-[#FFFBFF] flex items-center justify-center transition-colors cursor-pointer"
            title="Kapat"
          >
            <X className="w-3.5 h-3.5" />
          </button>
        )}
      </div>

      <p className="text-xs text-[#D4CCF1] font-mono select-all bg-[#0A061E]/80 border border-[rgba(183,173,223,0.12)] p-2.5 rounded-xl leading-relaxed max-h-32 overflow-y-auto">
        {message}
      </p>

      {onRetry && (
        <div className="flex justify-end gap-2 pt-1 border-t border-[rgba(183,173,223,0.10)]">
          <button
            type="button"
            onClick={onRetry}
            className="flex items-center gap-1.5 px-3 py-1.5 rounded-xl bg-[#2B2450]/70 hover:bg-[#695FA4]/40 border border-[rgba(183,173,223,0.22)] text-xs font-semibold text-[#D4CCF1] hover:text-[#FFFBFF] transition-all cursor-pointer"
          >
            <RefreshCw className="w-3.5 h-3.5 text-[#B7ADDF]" />
            <span>Yeniden Dene / Temizle</span>
          </button>
        </div>
      )}
    </div>
  );
};
