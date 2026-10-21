import React, { useState } from "react";
import { X, FolderDown, HardDrive, AlertCircle } from "lucide-react";
import { VolumeInfo } from "../types";

export interface ImportProjectModalProps {
  isOpen: boolean;
  onClose: () => void;
  onImport: (path: string) => Promise<void>;
  volumes: VolumeInfo[];
}

export const ImportProjectModal: React.FC<ImportProjectModalProps> = ({
  isOpen,
  onClose,
  onImport,
  volumes,
}) => {
  const [projectPath, setProjectPath] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  if (!isOpen) return null;

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    const trimmed = projectPath.trim();
    if (!trimmed) {
      setError("Lütfen geçerli bir proje klasör yolu girin.");
      return;
    }

    setLoading(true);
    setError(null);
    try {
      await onImport(trimmed);
      setProjectPath("");
      setError(null);
      onClose();
    } catch (err: any) {
      const errorMsg =
        typeof err === "string"
          ? err
          : err?.message || String(err || "Proje içe aktarılamadı.");
      setError(errorMsg);
    } finally {
      setLoading(false);
    }
  };

  const externalVol = volumes.find((v) => !v.is_internal || v.is_removable);

  return (
    <div
      role="dialog"
      aria-modal="true"
      aria-label="Projeyi İçe Aktar"
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
        <div className="flex items-center gap-3.5 mb-6">
          <div className="w-10 h-10 rounded-2xl bg-gradient-to-tr from-[#695FA4] to-[#2B2450] border border-[#B7ADDF] flex items-center justify-center text-[#FFFBFF] shadow-lg shadow-[#695FA4]/20">
            <FolderDown className="w-5 h-5" />
          </div>
          <div>
            <h3 className="font-['Plus_Jakarta_Sans',sans-serif] font-bold text-lg text-[#FFFBFF]">
              Mevcut Projeyi İçe Aktar
            </h3>
            <p className="text-xs text-[#D4CCF1] font-['Figtree',sans-serif]">
              Var olan yerel PHP klasörünü LocalForge çalışma alanına bağlayın.
            </p>
          </div>
        </div>

        {error && (
          <div className="mb-5 p-3.5 rounded-xl bg-red-500/15 border border-red-500/30 text-red-200 text-xs flex items-center gap-2 font-mono">
            <AlertCircle className="w-4 h-4 text-red-400 shrink-0" />
            <span className="truncate">{error}</span>
          </div>
        )}

        {/* Import Form */}
        <form onSubmit={handleSubmit} className="space-y-4 font-['Figtree',sans-serif]">
          <div>
            <label className="block text-xs font-semibold text-[#D4CCF1] mb-1.5">
              Proje Klasör Yolu
            </label>
            <input
              type="text"
              value={projectPath}
              onChange={(e) => setProjectPath(e.target.value)}
              placeholder={externalVol ? `${externalVol.mount_point}/benim-projem` : "/Users/.../benim-projem"}
              className="w-full bg-[#2B2450]/50 border border-[rgba(183,173,223,0.2)] rounded-xl px-3.5 py-2.5 text-xs text-[#FFFBFF] placeholder-[#D4CCF1]/40 focus:outline-none focus:border-[#B7ADDF] font-mono"
              autoFocus
            />
            <p className="text-[11px] text-[#D4CCF1]/70 mt-1">
              Tam dosya yolu belirtilmelidir. Klasörde <code>localforge.json</code> yoksa otomatik oluşturulur.
            </p>
          </div>

          {externalVol && (
            <div className="p-3 rounded-xl bg-[#2B2450]/30 border border-[rgba(183,173,223,0.12)] flex items-center justify-between">
              <div className="flex items-center gap-2">
                <HardDrive className="w-4 h-4 text-[#B7ADDF]" />
                <span className="text-xs text-[#D4CCF1]">Önerilen Birim:</span>
                <span className="font-mono text-xs text-[#FFFBFF] font-semibold">{externalVol.name}</span>
              </div>
              <button
                type="button"
                onClick={() => setProjectPath(externalVol.mount_point + "/")}
                className="text-[11px] font-mono text-[#B7ADDF] hover:text-[#FFFBFF] underline cursor-pointer"
              >
                Yolu Kullan
              </button>
            </div>
          )}

          {/* Action Buttons */}
          <div className="flex items-center justify-end gap-3 pt-3 border-t border-[rgba(183,173,223,0.12)]">
            <button
              type="button"
              onClick={onClose}
              disabled={loading}
              className="px-4 py-2 rounded-xl text-xs font-semibold text-[#D4CCF1] hover:text-[#FFFBFF] transition-all cursor-pointer"
            >
              İptal
            </button>
            <button
              type="submit"
              disabled={loading || !projectPath.trim()}
              className="px-5 py-2 rounded-xl bg-[#695FA4] hover:bg-[#695FA4]/90 disabled:opacity-50 text-xs font-semibold text-[#FFFBFF] border border-[#B7ADDF] shadow-md transition-all cursor-pointer flex items-center gap-2"
            >
              {loading ? (
                <span>İçe Aktarılıyor...</span>
              ) : (
                <>
                  <FolderDown className="w-3.5 h-3.5" />
                  <span>Projeyi İçe Aktar</span>
                </>
              )}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
};
