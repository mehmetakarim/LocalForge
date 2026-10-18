import React, { useState, useEffect, useRef } from "react";
import {
  Sparkles,
  Command,
  Search,
  Plus,
  RefreshCw,
  Layers,
  Server,
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
      {/* Floating Closed Capsule at bottom */}
      {!isOpen && (
        <div className="fixed bottom-6 left-1/2 -translate-x-1/2 z-40">
          <button
            type="button"
            aria-label="Open AI Command Capsule (⌘K)"
            onClick={onOpen}
            style={{
              boxShadow:
                "0 12px 36px rgba(0, 0, 0, 0.5), 0 0 20px rgba(79, 110, 247, 0.22), inset 0 1px 0 0 rgba(255, 255, 255, 0.2)",
            }}
            className="flex items-center justify-between gap-4 w-[460px] h-[46px] px-4 rounded-full lf-material-capsule border border-white/15 backdrop-blur-xl text-zinc-300 hover:text-white hover:border-white/25 transition-all group outline-none focus-visible:ring-2 focus-visible:ring-indigo-400"
          >
            <div className="flex items-center gap-2.5">
              <Sparkles className="w-4 h-4 text-indigo-400 animate-pulse" />
              <span className="text-xs text-zinc-400 group-hover:text-zinc-200">
                Ask LocalForge or jump to project...
              </span>
            </div>
            <div className="flex items-center gap-1.5 font-mono text-[10px] text-zinc-400 bg-white/10 px-2 py-0.5 rounded-md border border-white/10">
              <Command className="w-3 h-3" />
              <span>K</span>
            </div>
          </button>
        </div>
      )}

      {/* Expanded Modal Command Center */}
      {isOpen && (
        <div
          role="dialog"
          aria-modal="true"
          aria-label="LocalForge AI Command Center"
          className="fixed inset-0 z-50 flex items-start justify-center pt-24 bg-black/60 backdrop-blur-md animate-in fade-in duration-150"
          onClick={onClose}
        >
          <div
            onClick={(e) => e.stopPropagation()}
            style={{
              boxShadow:
                "0 24px 64px -12px rgba(0, 0, 0, 0.75), 0 0 32px -4px rgba(79, 110, 247, 0.3), inset 0 1px 0 0 rgba(255, 255, 255, 0.25)",
            }}
            className="w-[640px] max-h-[520px] rounded-2xl bg-zinc-950/90 border border-white/20 backdrop-blur-2xl flex flex-col overflow-hidden text-zinc-200"
          >
            {/* Search Input Bar */}
            <div className="flex items-center gap-3 px-4 py-3 border-b border-white/10 bg-white/5">
              <Search className="w-5 h-5 text-indigo-400" />
              <input
                ref={inputRef}
                type="text"
                value={query}
                onChange={(e) => setQuery(e.target.value)}
                placeholder="Search projects, services, or run diagnostic commands..."
                className="w-full bg-transparent text-sm text-white placeholder-zinc-500 outline-none font-medium"
              />
              {query ? (
                <button
                  onClick={() => setQuery("")}
                  className="p-1 text-zinc-400 hover:text-white rounded"
                >
                  <X className="w-4 h-4" />
                </button>
              ) : (
                <kbd className="px-2 py-0.5 text-[10px] font-mono text-zinc-400 bg-white/10 rounded border border-white/10">
                  ESC
                </kbd>
              )}
            </div>

            {/* Scrollable Command Results */}
            <div className="p-3 overflow-y-auto space-y-4 max-h-[420px]">
              {/* Quick Actions */}
              <div>
                <span className="px-3 text-[10px] uppercase font-mono tracking-wider text-zinc-500 font-semibold block mb-1">
                  Quick Actions
                </span>
                <div className="space-y-1">
                  {onCreateProject && (
                    <button
                      type="button"
                      onClick={() => {
                        onCreateProject();
                        onClose();
                      }}
                      className="w-full flex items-center justify-between px-3 py-2 rounded-xl hover:bg-white/10 text-left text-xs transition-colors"
                    >
                      <div className="flex items-center gap-2.5 text-zinc-200">
                        <Plus className="w-4 h-4 text-emerald-400" />
                        <span className="font-medium">Create New PHP Project</span>
                      </div>
                      <span className="font-mono text-[10px] text-zinc-500">Action</span>
                    </button>
                  )}

                  {onRefreshData && (
                    <button
                      type="button"
                      onClick={() => {
                        onRefreshData();
                        onClose();
                      }}
                      className="w-full flex items-center justify-between px-3 py-2 rounded-xl hover:bg-white/10 text-left text-xs transition-colors"
                    >
                      <div className="flex items-center gap-2.5 text-zinc-200">
                        <RefreshCw className="w-4 h-4 text-indigo-400" />
                        <span className="font-medium">Scan & Resync Volumes and Runtimes</span>
                      </div>
                      <span className="font-mono text-[10px] text-zinc-500">Action</span>
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
                        className="w-full flex items-center justify-between px-3 py-2 rounded-xl hover:bg-white/10 text-left text-xs transition-colors"
                      >
                        <div className="flex items-center gap-2.5 text-zinc-200">
                          <Server className="w-4 h-4 text-indigo-300" />
                          <span className="font-medium">Inspect Services (PHP-FPM, Nginx, MariaDB)</span>
                        </div>
                        <span className="font-mono text-[10px] text-zinc-500">⌘3</span>
                      </button>

                      <button
                        type="button"
                        onClick={() => {
                          onNavigate("storage");
                          onClose();
                        }}
                        className="w-full flex items-center justify-between px-3 py-2 rounded-xl hover:bg-white/10 text-left text-xs transition-colors"
                      >
                        <div className="flex items-center gap-2.5 text-zinc-200">
                          <HardDrive className="w-4 h-4 text-amber-300" />
                          <span className="font-medium">View External Storage Topologies</span>
                        </div>
                        <span className="font-mono text-[10px] text-zinc-500">⌘4</span>
                      </button>
                    </>
                  )}
                </div>
              </div>

              {/* Matching Projects */}
              <div>
                <span className="px-3 text-[10px] uppercase font-mono tracking-wider text-zinc-500 font-semibold block mb-1">
                  Projects ({filteredProjects.length})
                </span>
                {filteredProjects.length === 0 ? (
                  <div className="px-3 py-2 text-xs text-zinc-500 font-mono">
                    No matching projects found
                  </div>
                ) : (
                  <div className="space-y-1">
                    {filteredProjects.map((p) => (
                      <button
                        key={p.id}
                        type="button"
                        onClick={() => {
                          if (onSelectProject) onSelectProject(p);
                          onClose();
                        }}
                        className="w-full flex items-center justify-between px-3 py-2 rounded-xl hover:bg-white/10 text-left text-xs transition-colors group"
                      >
                        <div className="flex items-center gap-2.5">
                          <Layers className="w-4 h-4 text-indigo-400" />
                          <div>
                            <span className="font-medium text-zinc-100 group-hover:text-white block">
                              {p.name}
                            </span>
                            <span className="font-mono text-[11px] text-zinc-400">
                              {p.local_domain} • PHP {p.php_version}
                            </span>
                          </div>
                        </div>
                        <span
                          className={`px-2 py-0.5 rounded text-[10px] font-mono uppercase ${
                            p.status === "running"
                              ? "bg-emerald-500/10 text-emerald-400 border border-emerald-500/30"
                              : "bg-white/5 text-zinc-400"
                          }`}
                        >
                          {p.status}
                        </span>
                      </button>
                    ))}
                  </div>
                )}
              </div>

              {/* AI & MCP Diagnostics Preview */}
              <div>
                <div className="flex items-center justify-between px-3 mb-1">
                  <span className="text-[10px] uppercase font-mono tracking-wider text-zinc-500 font-semibold flex items-center gap-1.5">
                    <Bot className="w-3.5 h-3.5 text-indigo-400" />
                    AI Diagnostics & MCP Engine
                  </span>
                  <span className="text-[10px] font-mono text-emerald-400 bg-emerald-500/10 px-2 py-0.2 rounded border border-emerald-500/20">
                    Local Core Safe
                  </span>
                </div>
                <div className="p-2.5 rounded-xl bg-indigo-950/30 border border-indigo-500/20 text-xs text-indigo-200/90 space-y-1 font-mono">
                  <div className="text-[11px] text-zinc-300">
                    💡 <strong>Tip:</strong> Query LocalForge directly via Model Context Protocol or CLI:
                  </div>
                  <div className="text-[11px] bg-black/40 p-1.5 rounded select-all text-indigo-300">
                    localforge service status --project &lt;id&gt;
                  </div>
                </div>
              </div>
            </div>

            {/* Modal Footer */}
            <div className="flex items-center justify-between px-4 py-2 border-t border-white/10 bg-white/5 text-[11px] text-zinc-400 font-mono">
              <div className="flex items-center gap-2">
                <span>Navigation:</span>
                <kbd className="px-1.5 py-0.5 bg-white/10 rounded text-[10px] text-zinc-300">↑</kbd>
                <kbd className="px-1.5 py-0.5 bg-white/10 rounded text-[10px] text-zinc-300">↓</kbd>
                <span>Select:</span>
                <kbd className="px-1.5 py-0.5 bg-white/10 rounded text-[10px] text-zinc-300">↵</kbd>
              </div>
              <div className="flex items-center gap-1.5 text-emerald-400">
                <span className="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse" />
                <span>Spatial Engine Live</span>
              </div>
            </div>
          </div>
        </div>
      )}
    </>
  );
};
