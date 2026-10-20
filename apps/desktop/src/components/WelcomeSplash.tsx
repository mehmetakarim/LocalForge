import React from "react";
import { FolderOpen, Plus, HardDrive, ArrowRight, X } from "lucide-react";

export interface WelcomeSplashProps {
  isOpen: boolean;
  onClose: () => void;
  onCreateProject: () => void;
  onOpenFolder?: () => void;
  onDefineExternalDisk?: () => void;
}

export const WelcomeSplash: React.FC<WelcomeSplashProps> = ({
  isOpen,
  onClose,
  onCreateProject,
  onOpenFolder,
  onDefineExternalDisk,
}) => {
  if (!isOpen) return null;

  return (
    <div
      role="dialog"
      aria-modal="true"
      aria-label="LocalForge'a Hoş Geldiniz"
      className="fixed inset-0 z-50 flex items-center justify-center p-6 bg-[#191337]/90 backdrop-blur-2xl animate-in fade-in duration-200 select-none"
    >
      {/* Background ambient lighting */}
      <div
        className="absolute top-1/4 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[700px] h-[400px] pointer-events-none blur-[100px] z-0"
        style={{
          background:
            "radial-gradient(ellipse at center, rgba(105, 95, 164, 0.35) 0%, rgba(43, 36, 80, 0.15) 60%, transparent 80%)",
        }}
        aria-hidden="true"
      />

      <div className="relative z-10 w-full max-w-2xl rounded-3xl bg-[#2B2450]/80 border border-[rgba(183,173,223,0.22)] border-t-[1.5px] border-t-[rgba(183,173,223,0.45)] shadow-[0_40px_100px_rgba(10,6,30,0.95)] p-8 flex flex-col items-center text-center animate-in zoom-in-95 duration-200">
        {/* Dismiss Button */}
        <button
          type="button"
          onClick={onClose}
          aria-label="Kapat"
          className="absolute top-5 right-5 w-8 h-8 rounded-lg bg-[#191337]/60 hover:bg-[#695FA4]/30 border border-[rgba(183,173,223,0.18)] text-[#D4CCF1] hover:text-[#FFFBFF] flex items-center justify-center transition-colors cursor-pointer"
        >
          <X className="w-4 h-4" />
        </button>

        {/* Master Apple Squircle App Icon */}
        <div className="mb-4">
          <img
            src="/brand/localforge-app-icon-512.png"
            alt="LocalForge App Icon Master"
            className="w-20 h-20 rounded-2xl shadow-[0_12px_32px_rgba(10,6,30,0.8)] border border-[rgba(183,173,223,0.3)] object-cover"
          />
        </div>

        {/* Titles */}
        <div className="mb-6 max-w-lg">
          <div className="font-mono text-[11px] font-semibold uppercase tracking-[0.12em] text-[#B7ADDF] mb-2">
            LocalForge · Sürüm 0.1.0-alpha
          </div>
          <h1 className="font-['Plus_Jakarta_Sans',sans-serif] font-extrabold text-3xl tracking-[-0.03em] text-[#FFFBFF] mb-2">
            LocalForge'a Hoş Geldiniz
          </h1>
          <p className="text-xs md:text-sm text-[#D4CCF1] leading-relaxed">
            macOS Apple Silicon mimarisi için tasarlanmış bağımsız yerel geliştirme ve dağıtım stüdyosu.
          </p>
        </div>

        {/* 3 Action Cards */}
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4 w-full mb-6 text-left">
          {/* Action 1: Open Folder */}
          <div
            role="button"
            tabIndex={0}
            onClick={() => {
              if (onOpenFolder) onOpenFolder();
              onClose();
            }}
            onKeyDown={(e) => {
              if (e.key === "Enter" || e.key === " ") {
                e.preventDefault();
                if (onOpenFolder) onOpenFolder();
                onClose();
              }
            }}
            className="p-4 rounded-2xl bg-[#191337]/60 hover:bg-[#695FA4]/15 border border-[rgba(183,173,223,0.14)] hover:border-[rgba(183,173,223,0.3)] transition-all cursor-pointer flex flex-col justify-between min-h-[160px] outline-none focus-visible:ring-2 focus-visible:ring-[#B7ADDF]"
          >
            <div>
              <div className="w-8 h-8 rounded-xl bg-[#695FA4]/25 text-[#B7ADDF] flex items-center justify-center mb-3">
                <FolderOpen className="w-4 h-4" />
              </div>
              <h3 className="font-['Plus_Jakarta_Sans',sans-serif] font-bold text-sm text-[#FFFBFF] mb-1">
                Mevcut Projeyi Aç
              </h3>
              <p className="text-[11px] text-[#D4CCF1] leading-snug">
                Var olan yerel PHP klasörünü LocalForge çalışma alanına bağlayın.
              </p>
            </div>
            <span className="font-mono text-[10px] text-[#B7ADDF] mt-3">⌘O Klasör Seç</span>
          </div>

          {/* Action 2: New Project (Primary Highlight) */}
          <div
            role="button"
            tabIndex={0}
            onClick={() => {
              onCreateProject();
              onClose();
            }}
            onKeyDown={(e) => {
              if (e.key === "Enter" || e.key === " ") {
                e.preventDefault();
                onCreateProject();
                onClose();
              }
            }}
            className="p-4 rounded-2xl bg-gradient-to-b from-[#3B376A]/70 to-[#191337]/80 hover:brightness-110 border border-[#B7ADDF] shadow-[0_8px_24px_rgba(105,95,164,0.35)] transition-all cursor-pointer flex flex-col justify-between min-h-[160px] outline-none focus-visible:ring-2 focus-visible:ring-[#B7ADDF]"
          >
            <div>
              <div className="w-8 h-8 rounded-xl bg-[#695FA4] text-[#FFFBFF] flex items-center justify-center mb-3">
                <Plus className="w-4 h-4" />
              </div>
              <h3 className="font-['Plus_Jakarta_Sans',sans-serif] font-bold text-sm text-[#FFFBFF] mb-1">
                Yeni Proje Başlat
              </h3>
              <p className="text-[11px] text-[#D4CCF1] leading-snug">
                Nginx, PHP-FPM ve MariaDB ile izole geliştirme ortamı kurun.
              </p>
            </div>
            <span className="font-mono text-[10px] text-[#FFFBFF] bg-[#695FA4] px-2 py-0.5 rounded-md self-start mt-3">
              ⌘N Proje Sihirbazı
            </span>
          </div>

          {/* Action 3: Define External Disk */}
          <div
            role="button"
            tabIndex={0}
            onClick={() => {
              if (onDefineExternalDisk) onDefineExternalDisk();
              onClose();
            }}
            onKeyDown={(e) => {
              if (e.key === "Enter" || e.key === " ") {
                e.preventDefault();
                if (onDefineExternalDisk) onDefineExternalDisk();
                onClose();
              }
            }}
            className="p-4 rounded-2xl bg-[#191337]/60 hover:bg-[#695FA4]/15 border border-[rgba(183,173,223,0.14)] hover:border-[rgba(183,173,223,0.3)] transition-all cursor-pointer flex flex-col justify-between min-h-[160px] outline-none focus-visible:ring-2 focus-visible:ring-[#B7ADDF]"
          >
            <div>
              <div className="w-8 h-8 rounded-xl bg-[#F59E0B]/20 text-[#F59E0B] flex items-center justify-center mb-3">
                <HardDrive className="w-4 h-4" />
              </div>
              <h3 className="font-['Plus_Jakarta_Sans',sans-serif] font-bold text-sm text-[#FFFBFF] mb-1">
                Harici Disk Tanımla
              </h3>
              <p className="text-[11px] text-[#D4CCF1] leading-snug">
                Projeleri harici APFS SSD üzerinde güvenle çalıştırın.
              </p>
            </div>
            <span className="font-mono text-[10px] text-[#B7ADDF] mt-3">⌘D Disk Bağla</span>
          </div>
        </div>

        {/* Footer */}
        <div className="flex items-center justify-between w-full pt-4 border-t border-[rgba(183,173,223,0.12)] text-xs text-[#D4CCF1]">
          <div className="flex items-center gap-2 font-mono text-[11px]">
            <span>Apple Silicon Tier-1</span>
            <span>·</span>
            <span>APFS Güvenli</span>
            <span>·</span>
            <span>Loopback :8080</span>
          </div>

          <button
            type="button"
            onClick={onClose}
            className="flex items-center gap-1.5 px-4 py-1.5 rounded-xl bg-[rgba(183,173,223,0.12)] hover:bg-[rgba(183,173,223,0.22)] border border-[rgba(183,173,223,0.20)] text-xs font-semibold text-[#FFFBFF] transition-all cursor-pointer"
          >
            <span>Stüdyoya Geç</span>
            <ArrowRight className="w-3.5 h-3.5" />
          </button>
        </div>
      </div>
    </div>
  );
};
