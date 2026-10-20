import React, { useState, useEffect, useRef } from "react";
import {
  Command,
  Search,
  Plus,
  RefreshCw,
  Cpu,
  HardDrive,
  Bot,
  X,
} from "lucide-react";
import { ProjectRecord, ViewMode } from "../types";

export interface AICommandCapsuleProps {
  isOpen: boolean;
  onOpen: () => void;
  onClose: () => void;
  projects: ProjectRecord[];
  onSelectProject?: (project: ProjectRecord) => void;
  onNavigate?: (mode: ViewMode) => void;
  onCreateProject?: () => void;
  onRefreshData?: () => void;
}

export const AICommandCapsule: React.FC<AICommandCapsuleProps> = ({
  isOpen,
  onOpen,
  onClose,
  projects,
  onSelectProject,
  onNavigate,
  onCreateProject,
  onRefreshData,
}) => {
  const [query, setQuery] = useState("");
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "k") {
        e.preventDefault();
        if (isOpen) {
          onClose();
        } else {
          onOpen();
        }
      } else if (e.key === "Escape" && isOpen) {
        e.preventDefault();
        onClose();
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [isOpen, onOpen, onClose]);

  useEffect(() => {
    if (isOpen) {
      setTimeout(() => inputRef.current?.focus(), 50);
    } else {
      setQuery("");
    }
  }, [isOpen]);

  const filteredProjects = projects.filter(
    (p) =>
      p.name.toLowerCase().includes(query.toLowerCase()) ||
      p.local_domain.toLowerCase().includes(query.toLowerCase()) ||
      p.php_version.toLowerCase().includes(query.toLowerCase())
  );

  return (
    <>
      {/* Direction A+ Bottom AI Command Capsule */}
      {!isOpen && (
        <div className="fixed bottom-5 left-1/2 -translate-x-1/2 z-40">
          <button
            type="button"
            aria-label="Komut Kapsülünü Aç (⌘K)"
            onClick={onOpen}
            style={{
              boxShadow:
                "0 20px 48px -10px rgba(10, 6, 30, 0.85), 0 0 24px rgba(105, 95, 164, 0.22), inset 0 1px 0 0 rgba(183, 173, 223, 0.30)",
            }}
            className="flex items-center justify-between gap-4 w-[520px] h-[46px] px-5 rounded-full bg-[#2B2450]/78 backdrop-blur-2xl border border-[rgba(183,173,223,0.22)] border-t-[rgba(183,173,223,0.40)] text-[#D4CCF1] hover:text-[#FFFBFF] hover:border-[#B7ADDF] transition-all group outline-none focus-visible:ring-2 focus-visible:ring-[#B7ADDF] cursor-pointer"
          >
            <div className="flex items-center gap-3">
              <span className="w-2 h-2 rounded-full bg-[#B7ADDF] shadow-[0_0_10px_#B7ADDF] animate-pulse" />
              <span className="text-xs font-medium text-[#D4CCF1] group-hover:text-[#FFFBFF]">
                LocalForge'a sorun veya projeye atlayın...
              </span>
            </div>

            <div className="flex items-center gap-2">
              <span className="font-mono text-[10px] text-[#D4CCF1] bg-[#191337]/85 border border-[rgba(183,173,223,0.25)] px-2 py-0.5 rounded-md text-[#B7ADDF] font-semibold flex items-center gap-1">
                <Command className="w-3 h-3" />
                <span>K</span>
              </span>
            </div>
          </button>
        </div>
      )}

      {/* Expanded Modal Overlay */}
      {isOpen && (
        <div
          role="dialog"
          aria-modal="true"
          aria-label="Komut Kapsülü"
          className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-md animate-in fade-in duration-150"
          onClick={(e) => {
            if (e.target === e.currentTarget) onClose();
          }}
        >
          <div className="w-full max-w-xl rounded-2xl bg-[#191337] border border-[rgba(183,173,223,0.25)] shadow-[0_30px_90px_rgba(10,6,30,0.95)] overflow-hidden flex flex-col animate-in zoom-in-95 duration-150">
            {/* Search Input Bar */}
            <div className="flex items-center gap-3 px-4 py-3.5 border-b border-[rgba(183,173,223,0.14)] bg-[#2B2450]/40">
              <Search className="w-4 h-4 text-[#B7ADDF]" />
              <input
                ref={inputRef}
                type="text"
                value={query}
                onChange={(e) => setQuery(e.target.value)}
                placeholder="Proje ara, servisleri denetle veya komut çalıştır..."
                className="flex-1 bg-transparent text-sm text-[#FFFBFF] placeholder-[#D4CCF1]/50 outline-none font-['Figtree',sans-serif]"
              />
              <button
                type="button"
                onClick={onClose}
                aria-label="Kapat"
                className="text-[#D4CCF1] hover:text-[#FFFBFF] p-1 rounded-md hover:bg-white/5 transition-colors cursor-pointer"
              >
                <X className="w-4 h-4" />
              </button>
            </div>

            {/* Quick Actions List */}
            <div className="p-3 max-h-80 overflow-y-auto space-y-1">
              {filteredProjects.length > 0 && (
                <div className="mb-2">
                  <div className="px-2 py-1 text-[10px] font-mono uppercase text-[#B7ADDF] tracking-wider">
                    Projeler ({filteredProjects.length})
                  </div>
                  {filteredProjects.map((p) => (
                    <button
                      key={p.id}
                      type="button"
                      onClick={() => {
                        onSelectProject?.(p);
                        onClose();
                      }}
                      className="w-full flex items-center justify-between px-3 py-2 rounded-xl hover:bg-[#695FA4]/20 text-left transition-colors text-xs text-[#FFFBFF] group cursor-pointer"
                    >
                      <div className="flex items-center gap-2.5">
                        <span className="w-2 h-2 rounded-full bg-[#10B981] shadow-[0_0_6px_#10B981]" />
                        <span className="font-semibold">{p.name}</span>
                        <span className="font-mono text-[11px] text-[#D4CCF1]">
                          {p.local_domain}
                        </span>
                      </div>
                      <span className="font-mono text-[10px] text-[#B7ADDF] group-hover:underline">
                        İncele ↗
                      </span>
                    </button>
                  ))}
                </div>
              )}

              <div className="px-2 py-1 text-[10px] font-mono uppercase text-[#B7ADDF] tracking-wider">
                Hızlı Komutlar
              </div>

              {onCreateProject && (
                <button
                  type="button"
                  onClick={() => {
                    onCreateProject();
                    onClose();
                  }}
                  className="w-full flex items-center gap-2.5 px-3 py-2 rounded-xl hover:bg-[#695FA4]/20 text-left text-xs text-[#D4CCF1] hover:text-[#FFFBFF] transition-colors cursor-pointer"
                >
                  <Plus className="w-4 h-4 text-[#10B981]" />
                  <span>Yeni PHP Projesi Oluştur</span>
                </button>
              )}

              {onRefreshData && (
                <button
                  type="button"
                  onClick={() => {
                    onRefreshData();
                    onClose();
                  }}
                  className="w-full flex items-center gap-2.5 px-3 py-2 rounded-xl hover:bg-[#695FA4]/20 text-left text-xs text-[#D4CCF1] hover:text-[#FFFBFF] transition-colors cursor-pointer"
                >
                  <RefreshCw className="w-4 h-4 text-[#B7ADDF]" />
                  <span>Sistem ve Servis Durumunu Yenile</span>
                </button>
              )}

              {onNavigate && (
                <>
                  <button
                    type="button"
                    onClick={() => {
                      onNavigate("services");
                      onClose();
                    }}
                    className="w-full flex items-center gap-2.5 px-3 py-2 rounded-xl hover:bg-[#695FA4]/20 text-left text-xs text-[#D4CCF1] hover:text-[#FFFBFF] transition-colors cursor-pointer"
                  >
                    <Cpu className="w-4 h-4 text-[#60A5FA]" />
                    <span>Servisler & Çalışma Zamanı Tanılarını Aç</span>
                  </button>
                  <button
                    type="button"
                    onClick={() => {
                      onNavigate("storage");
                      onClose();
                    }}
                    className="w-full flex items-center gap-2.5 px-3 py-2 rounded-xl hover:bg-[#695FA4]/20 text-left text-xs text-[#D4CCF1] hover:text-[#FFFBFF] transition-colors cursor-pointer"
                  >
                    <HardDrive className="w-4 h-4 text-[#F59E0B]" />
                    <span>Depolama & Harici APFS Birimlerini Görüntüle</span>
                  </button>
                  <button
                    type="button"
                    onClick={() => {
                      onNavigate("mcp");
                      onClose();
                    }}
                    className="w-full flex items-center gap-2.5 px-3 py-2 rounded-xl hover:bg-[#695FA4]/20 text-left text-xs text-[#D4CCF1] hover:text-[#FFFBFF] transition-colors cursor-pointer"
                  >
                    <Bot className="w-4 h-4 text-[#B7ADDF]" />
                    <span>Model Context Protocol (MCP) Durumunu İncele</span>
                  </button>
                </>
              )}
            </div>

            {/* Modal Footer */}
            <div className="flex items-center justify-between px-4 py-2 border-t border-[rgba(183,173,223,0.12)] bg-[#191337] text-[11px] font-mono text-[#D4CCF1]/70">
              <div className="flex items-center gap-2">
                <span>Çalışma Alanı:</span>
                <span className="text-[#FFFBFF]">/Volumes/Mac Harici Disk</span>
              </div>
              <div className="flex items-center gap-2">
                <kbd className="px-1.5 py-0.5 rounded bg-[rgba(183,173,223,0.12)] text-[#B7ADDF]">
                  ESC
                </kbd>
                <span>kapat</span>
              </div>
            </div>
          </div>
        </div>
      )}
    </>
  );
};
