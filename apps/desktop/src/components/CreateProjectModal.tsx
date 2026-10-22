import React, { useState, useEffect } from "react";
import { X, FolderPlus, Disc, HardDrive } from "lucide-react";
import { VolumeInfo, RuntimeOverview } from "../types";

interface CreateProjectModalProps {
  isOpen: boolean;
  onClose: () => void;
  volumes: VolumeInfo[];
  runtime?: RuntimeOverview | null;
  onCreate: (name: string, parentDir: string, docroot: string, phpVersion: string) => Promise<void>;
}

export const CreateProjectModal: React.FC<CreateProjectModalProps> = ({
  isOpen,
  onClose,
  volumes,
  runtime,
  onCreate,
}) => {
  const [name, setName] = useState("");
  const [parentDir, setParentDir] = useState(
    volumes.find((v) => v.mount_point.startsWith("/Volumes"))?.mount_point || "/Volumes"
  );
  const [docroot, setDocroot] = useState("public");

  // Dynamically discover verified installed versions from runtime telemetry
  const verifiedList = (runtime?.php_details?.filter((d) => d.is_fpm_verified) || []).map((d) => d.major_minor);
  const fallbackList = runtime?.php_versions?.filter((v) => ["8.5", "8.4", "8.3", "8.2", "8.1"].includes(v)) || [];
  const installedVersions = Array.from(new Set([...verifiedList, ...fallbackList]));
  const defaultVersion = installedVersions[0] || "8.5";

  const [phpVersion, setPhpVersion] = useState(defaultVersion);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (installedVersions.length > 0 && !installedVersions.includes(phpVersion)) {
      setPhpVersion(installedVersions[0]);
    }
  }, [installedVersions.join(",")]);

  if (!isOpen) return null;

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!name.trim() || !parentDir.trim()) return;

    setLoading(true);
    try {
      await onCreate(name, parentDir, docroot, phpVersion);
      setName("");
      onClose();
    } finally {
      setLoading(false);
    }
  };

  const allSupportedVersions = ["8.5", "8.4", "8.3", "8.2", "8.1"];

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/65 backdrop-blur-xl p-4 animate-in fade-in duration-200 select-none">
      <div className="bg-[#191337] border border-[rgba(183,173,223,0.22)] rounded-3xl w-full max-w-lg p-7 shadow-2xl shadow-black/80 relative">
        {/* Close Button */}
        <button
          onClick={onClose}
          className="absolute top-5 right-5 w-7 h-7 rounded-full bg-[#2B2450]/60 hover:bg-[#695FA4]/30 border border-[rgba(183,173,223,0.18)] flex items-center justify-center text-[#D4CCF1] hover:text-[#FFFBFF] transition-all cursor-pointer"
        >
          <X className="w-4 h-4" />
        </button>

        {/* Modal Header */}
        <div className="flex items-center gap-3.5 mb-6">
          <div className="w-10 h-10 rounded-2xl bg-gradient-to-tr from-[#695FA4] to-[#2B2450] border border-[#B7ADDF] flex items-center justify-center text-[#FFFBFF] shadow-lg shadow-[#695FA4]/20">
            <FolderPlus className="w-5 h-5" />
          </div>
          <div>
            <h2 className="font-['Plus_Jakarta_Sans',sans-serif] text-base font-bold text-[#FFFBFF] tracking-tight">
              Yeni Proje Oluştur
            </h2>
            <p className="text-xs text-[#D4CCF1]">Harici veya dahili diskte izole PHP çalışma alanı başlatın</p>
          </div>
        </div>

        <form onSubmit={handleSubmit} className="space-y-5">
          {/* Project Name */}
          <div>
            <label className="block text-xs font-medium text-[#D4CCF1] mb-1.5">
              Proje Adı
            </label>
            <input
              type="text"
              required
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="ör. localforge-store"
              className="w-full px-3.5 py-2.5 bg-[#2B2450]/60 border border-[rgba(183,173,223,0.18)] rounded-xl text-sm text-[#FFFBFF] placeholder-[#D4CCF1]/40 focus:outline-none focus:border-[#B7ADDF] focus:ring-1 focus:ring-[#B7ADDF] transition-all font-medium"
            />
            <p className="text-[11px] text-[#D4CCF1]/70 mt-1">
              Yerel alan adı otomatik atanır: <span className="text-[#B7ADDF] font-mono">{name ? `${name.toLowerCase().replace(/[^a-z0-9]/g, "")}.test` : "proje.test"}</span>
            </p>
          </div>

          {/* Storage Volume Picker */}
          <div>
            <label className="block text-xs font-medium text-[#D4CCF1] mb-1.5">
              Depolama Konumu (Hedef Disk)
            </label>
            <div className="space-y-2">
              {volumes.map((v) => {
                const isSelected = parentDir === v.mount_point;
                const isExternal = !v.is_internal || v.mount_point.startsWith("/Volumes");

                return (
                  <div
                    key={v.uuid}
                    onClick={() => setParentDir(v.mount_point)}
                    className={`p-3 rounded-xl border flex items-center justify-between cursor-pointer transition-all ${
                      isSelected
                        ? "bg-[#695FA4]/30 border-[#B7ADDF] shadow-[0_0_12px_rgba(105,95,164,0.3)]"
                        : "bg-[#2B2450]/40 border-[rgba(183,173,223,0.12)] hover:border-[rgba(183,173,223,0.25)]"
                    }`}
                  >
                    <div className="flex items-center gap-3">
                      <div
                        className={`w-7 h-7 rounded-lg flex items-center justify-center ${
                          isExternal ? "bg-[#F59E0B]/20 text-[#F59E0B]" : "bg-[#695FA4]/20 text-[#B7ADDF]"
                        }`}
                      >
                        {isExternal ? <HardDrive className="w-4 h-4" /> : <Disc className="w-4 h-4" />}
                      </div>
                      <div>
                        <div className="text-xs font-semibold text-[#FFFBFF]">{v.name}</div>
                        <div className="text-[10px] font-mono text-[#D4CCF1]/60 truncate max-w-[240px]">
                          {v.mount_point}
                        </div>
                      </div>
                    </div>
                    {isExternal && (
                      <span className="text-[10px] font-mono text-[#F59E0B] bg-[#F59E0B]/12 border border-[#F59E0B]/25 px-2 py-0.5 rounded-md">
                        Önerilen
                      </span>
                    )}
                  </div>
                );
              })}
            </div>
          </div>

          {/* PHP Version & Docroot */}
          <div className="grid grid-cols-2 gap-4">
            <div>
              <label className="block text-xs font-medium text-[#D4CCF1] mb-1.5">
                PHP Sürümü
              </label>
              <select
                value={phpVersion}
                onChange={(e) => setPhpVersion(e.target.value)}
                className="w-full px-3.5 py-2.5 bg-[#2B2450]/60 border border-[rgba(183,173,223,0.18)] rounded-xl text-xs text-[#FFFBFF] font-mono focus:outline-none focus:border-[#B7ADDF]"
              >
                {allSupportedVersions.map((v) => {
                  const isInstalled = installedVersions.includes(v);
                  return (
                    <option key={v} value={v} className="bg-[#191337] text-white">
                      PHP {v} {isInstalled ? "(Kurulu · FPM arm64)" : "(Kurulu Değil)"}
                    </option>
                  );
                })}
              </select>
            </div>

            <div>
              <label className="block text-xs font-medium text-[#D4CCF1] mb-1.5">
                Belge Kökü
              </label>
              <input
                type="text"
                value={docroot}
                onChange={(e) => setDocroot(e.target.value)}
                placeholder="public"
                className="w-full px-3.5 py-2.5 bg-[#2B2450]/60 border border-[rgba(183,173,223,0.18)] rounded-xl text-xs text-[#FFFBFF] font-mono focus:outline-none focus:border-[#B7ADDF]"
              />
            </div>
          </div>

          {/* Actions */}
          <div className="pt-2 flex items-center justify-end gap-3">
            <button
              type="button"
              onClick={onClose}
              className="px-4 py-2 rounded-xl border border-[rgba(183,173,223,0.18)] text-xs text-[#D4CCF1] hover:text-[#FFFBFF] hover:bg-white/5 transition-all cursor-pointer"
            >
              Vazgeç
            </button>
            <button
              type="submit"
              disabled={loading || !name.trim()}
              className="px-5 py-2 rounded-xl bg-gradient-to-r from-[#695FA4] to-[#2B2450] hover:brightness-110 border border-[#B7ADDF] text-xs font-semibold text-[#FFFBFF] transition-all shadow-[0_4px_16px_rgba(105,95,164,0.35)] disabled:opacity-40 cursor-pointer"
            >
              {loading ? "Oluşturuluyor..." : "Proje Oluştur"}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
};
