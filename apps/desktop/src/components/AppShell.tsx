import React, { useEffect } from "react";
import { ViewMode, ProjectRecord } from "../types";
import { FloatingDock } from "./FloatingDock";
import { AICommandCapsule } from "./AICommandCapsule";

export interface AppShellProps {
  activeMode: ViewMode;
  onSelectMode: (mode: ViewMode) => void;
  theme: "dark" | "light";
  onToggleTheme: () => void;
  isCapsuleOpen: boolean;
  onOpenCapsule: () => void;
  onCloseCapsule: () => void;
  projects: ProjectRecord[];
  onSelectProject?: (project: ProjectRecord) => void;
  onCreateProject?: () => void;
  onRefreshData?: () => void;
  zoomLevel?: number;
  onZoomIn?: () => void;
  onZoomOut?: () => void;
  onZoomReset?: () => void;
  children: React.ReactNode;
}

export const AppShell: React.FC<AppShellProps> = ({
  activeMode,
  onSelectMode,
  theme,
  onToggleTheme,
  isCapsuleOpen,
  onOpenCapsule,
  onCloseCapsule,
  projects,
  onSelectProject,
  onCreateProject,
  onRefreshData,
  zoomLevel = 100,
  onZoomIn,
  onZoomOut,
  onZoomReset,
  children,
}) => {
  // Global Cmd+1..7 shortcut handler
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && !e.shiftKey && !e.altKey) {
        switch (e.key) {
          case "1":
            e.preventDefault();
            onSelectMode("studio");
            break;
          case "2":
            e.preventDefault();
            onSelectMode("projects");
            break;
          case "3":
            e.preventDefault();
            onSelectMode("services");
            break;
          case "4":
            e.preventDefault();
            onSelectMode("storage");
            break;
          case "5":
            e.preventDefault();
            onSelectMode("deployments");
            break;
          case "6":
            e.preventDefault();
            onSelectMode("mcp");
            break;
          case "7":
            e.preventDefault();
            onSelectMode("settings");
            break;
        }
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [onSelectMode]);

  return (
    <div
      className={`min-h-screen relative flex flex-col font-sans text-zinc-100 select-none overflow-hidden ${
        theme === "dark" ? "bg-[#07090F]" : "bg-slate-900"
      }`}
    >
      {/* Studio Radial Ambient Backdrop */}
      <div
        className="fixed inset-0 pointer-events-none z-0"
        style={{
          background:
            "radial-gradient(circle at 50% 20%, rgba(30, 42, 74, 0.45) 0%, rgba(10, 13, 20, 0.95) 60%, #07090F 100%)",
        }}
        aria-hidden="true"
      />

      {/* Atmospheric Lighting Halos (Hardware-Capped strictly to 2 layers) */}
      <div
        className="fixed top-[-120px] left-1/2 -translate-x-1/2 w-[900px] h-[480px] rounded-full pointer-events-none opacity-30 blur-[80px] bg-gradient-to-br from-indigo-500/20 via-purple-600/10 to-transparent z-0"
        aria-hidden="true"
      />
      <div
        className="fixed bottom-[-100px] right-[15%] w-[600px] h-[400px] rounded-full pointer-events-none opacity-20 blur-[90px] bg-gradient-to-tl from-indigo-600/20 to-transparent z-0"
        aria-hidden="true"
      />

      {/* macOS Native Titlebar Header */}
      <header
        data-tauri-drag-region
        className="fixed top-0 left-0 right-0 h-12 z-40 flex items-center justify-between px-24 border-b border-white/[0.04] bg-[#0A0D14]/80 backdrop-blur-md"
      >
        <div className="flex items-center gap-2 text-xs font-medium text-zinc-400 pointer-events-none">
          <span className="font-semibold text-zinc-200">LocalForge</span>
          <span className="text-zinc-600">/</span>
          <span className="capitalize text-zinc-400 font-mono text-[11px]">{activeMode}</span>
        </div>

        <div className="flex items-center gap-4 text-[11px] font-mono text-zinc-400">
          <div className="flex items-center gap-1.5">
            <span className="w-1.5 h-1.5 rounded-full bg-emerald-400 shadow-[0_0_8px_#10B981]" />
            <span className="text-zinc-400">senkronize · yerel loopback :8080</span>
          </div>
          <span className="text-zinc-600">•</span>
          <span className="text-zinc-500">v0.1.0 (Phase R1.3A-V)</span>
        </div>
      </header>

      {/* Floating Vertical Navigation Dock on Left */}
      <FloatingDock
        activeMode={activeMode}
        onSelectMode={onSelectMode}
        theme={theme}
        onToggleTheme={onToggleTheme}
      />

      {/* Floating Functional Zoom Controls on Right */}
      {onZoomIn && onZoomOut && onZoomReset && (
        <aside
          aria-label="Scene Zoom Controls"
          className="fixed right-6 top-1/2 -translate-y-1/2 z-40 flex flex-col items-center gap-2 p-2 rounded-2xl bg-[#101522]/55 border border-white/10 backdrop-blur-2xl shadow-xl font-mono text-xs text-zinc-400"
        >
          <button
            type="button"
            onClick={onZoomIn}
            className="w-7 h-7 rounded-lg bg-white/5 hover:bg-white/10 border border-white/10 text-white flex items-center justify-center transition-colors"
            title="Yakınlaştır (+5%)"
          >
            +
          </button>
          <button
            type="button"
            onClick={onZoomReset}
            className="text-[11px] font-semibold text-zinc-300 hover:text-white px-1 py-0.5 rounded transition-colors"
            title="Sıfırla (100%)"
          >
            %{zoomLevel}
          </button>
          <button
            type="button"
            onClick={onZoomOut}
            className="w-7 h-7 rounded-lg bg-white/5 hover:bg-white/10 border border-white/10 text-white flex items-center justify-center transition-colors"
            title="Uzaklaştır (-5%)"
          >
            -
          </button>
        </aside>
      )}

      {/* Main Studio Viewport (Centered) */}
      <main className="relative z-10 flex-1 ml-20 mr-16 mt-12 flex flex-col items-center justify-center overflow-y-auto">
        {children}
      </main>

      {/* Floating AI Command Capsule */}
      <AICommandCapsule
        isOpen={isCapsuleOpen}
        onOpen={onOpenCapsule}
        onClose={onCloseCapsule}
        projects={projects}
        onSelectProject={onSelectProject}
        onNavigate={onSelectMode}
        onCreateProject={onCreateProject}
        onRefreshData={onRefreshData}
      />
    </div>
  );
};
