import React, { useState } from "react";
import { X, FolderPlus, Disc, HardDrive } from "lucide-react";
import { VolumeInfo } from "../types";

interface CreateProjectModalProps {
  isOpen: boolean;
  onClose: () => void;
  volumes: VolumeInfo[];
  onCreate: (name: string, parentDir: string, docroot: string, phpVersion: string) => Promise<void>;
}

export const CreateProjectModal: React.FC<CreateProjectModalProps> = ({
  isOpen,
  onClose,
  volumes,
  onCreate,
}) => {
  const [name, setName] = useState("");
  const [parentDir, setParentDir] = useState(
    volumes.find((v) => v.mount_point.startsWith("/Volumes"))?.mount_point || "/Volumes"
  );
  const [docroot, setDocroot] = useState("public");
  const [phpVersion, setPhpVersion] = useState("8.4");
  const [loading, setLoading] = useState(false);

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

  const phpOptions = ["8.4", "8.3", "8.2", "8.1"];

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/65 backdrop-blur-xl p-4 animate-in fade-in duration-200">
      <div className="bg-neutral-900/90 border border-white/15 rounded-3xl w-full max-w-lg p-7 shadow-2xl shadow-black/80 relative apple-glass">
        {/* Close Button */}
        <button
          onClick={onClose}
          className="absolute top-5 right-5 w-7 h-7 rounded-full bg-white/10 hover:bg-white/20 flex items-center justify-center text-neutral-300 hover:text-white transition-all"
        >
          <X className="w-4 h-4" />
        </button>

        {/* Modal Header */}
        <div className="flex items-center gap-3.5 mb-6">
          <div className="w-10 h-10 rounded-2xl bg-gradient-to-tr from-blue-600 to-indigo-600 flex items-center justify-center text-white shadow-lg shadow-blue-500/20">
            <FolderPlus className="w-5 h-5" />
          </div>
          <div>
            <h2 className="text-base font-semibold text-white tracking-tight">Yeni Proje Oluştur</h2>
            <p className="text-xs text-neutral-400">Harici veya dahili diskte izole PHP çalışma alanı başlatın</p>
          </div>
        </div>

        <form onSubmit={handleSubmit} className="space-y-5">
          {/* Project Name */}
          <div>
            <label className="block text-xs font-medium text-neutral-300 mb-1.5">
              Proje Adı
            </label>
            <input
              type="text"
              required
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="ör. stepperskip-store"
              className="w-full px-3.5 py-2.5 bg-neutral-950/70 border border-white/10 rounded-xl text-sm text-white placeholder-neutral-500 focus:outline-none focus:border-blue-500 focus:ring-1 focus:ring-blue-500 transition-all font-medium"
            />
            <p className="text-[11px] text-neutral-500 mt-1">
              Yerel alan adı otomatik atanır: <span className="text-blue-400 font-mono">{name ? `${name.toLowerCase().replace(/[^a-z0-9]/g, "")}.test` : "proje.test"}</span>
            </p>
          </div>

          {/* Storage Volume Picker */}
          <div>
            <label className="block text-xs font-medium text-neutral-300 mb-1.5">
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
                        ? "bg-blue-600/15 border-blue-500 text-white shadow-sm"
                        : "bg-neutral-950/40 border-white/5 hover:border-white/15 text-neutral-300"
                    }`}
                  >
                    <div className="flex items-center gap-3">
                      {isExternal ? (
                        <Disc className={`w-4 h-4 ${isSelected ? "text-blue-400" : "text-amber-400"}`} />
                      ) : (
                        <HardDrive className={`w-4 h-4 ${isSelected ? "text-blue-400" : "text-neutral-500"}`} />
                      )}
                      <div>
                        <div className="flex items-center gap-2">
                          <span className="text-xs font-semibold">{v.name}</span>
                          {isExternal && (
                            <span className="px-1.5 py-0.2 rounded-full text-[9px] font-bold uppercase tracking-wider bg-amber-500/20 text-amber-300 border border-amber-500/30">
                              Harici SSD
                            </span>
                          )}
                        </div>
                        <span className="text-[10px] text-neutral-500 font-mono">{v.mount_point}</span>
                      </div>
                    </div>

                    <div className={`w-4 h-4 rounded-full border flex items-center justify-center ${
                      isSelected ? "border-blue-500 bg-blue-500" : "border-neutral-600"
                    }`}>
                      {isSelected && <div className="w-1.5 h-1.5 rounded-full bg-white" />}
                    </div>
                  </div>
                );
              })}
            </div>
          </div>

          {/* PHP Version Segmented Control */}
          <div>
            <label className="block text-xs font-medium text-neutral-300 mb-1.5">
              PHP Çalışma Ortamı
            </label>
            <div className="grid grid-cols-4 gap-2 p-1 bg-neutral-950/70 border border-white/10 rounded-xl">
              {phpOptions.map((ver) => (
                <button
                  key={ver}
                  type="button"
                  onClick={() => setPhpVersion(ver)}
                  className={`py-2 text-xs font-semibold rounded-lg transition-all ${
                    phpVersion === ver
                      ? "bg-blue-600 text-white shadow-sm"
                      : "text-neutral-400 hover:text-white"
                  }`}
                >
                  PHP {ver}
                </button>
              ))}
            </div>
          </div>

          {/* Document Root */}
          <div>
            <label className="block text-xs font-medium text-neutral-300 mb-1.5">
              Document Root (Web Dizini)
            </label>
            <input
              type="text"
              value={docroot}
              onChange={(e) => setDocroot(e.target.value)}
              placeholder="public"
              className="w-full px-3.5 py-2.5 bg-neutral-950/70 border border-white/10 rounded-xl text-sm text-white placeholder-neutral-500 focus:outline-none focus:border-blue-500 focus:ring-1 focus:ring-blue-500 transition-all font-mono text-xs"
            />
          </div>

          {/* Action Buttons */}
          <div className="pt-3 border-t border-white/10 flex items-center justify-end gap-2.5">
            <button
              type="button"
              onClick={onClose}
              className="px-4 py-2.5 rounded-xl text-xs font-medium text-neutral-300 hover:text-white apple-button-ghost transition-all"
            >
              Vazgeç
            </button>
            <button
              type="submit"
              disabled={loading || !name.trim()}
              className="px-5 py-2.5 rounded-xl text-xs font-semibold text-white apple-button-primary disabled:opacity-40 transition-all"
            >
              {loading ? "Oluşturuluyor..." : "Projeyi Başlat"}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
};
