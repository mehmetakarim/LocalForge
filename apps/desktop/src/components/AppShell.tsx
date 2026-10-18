import React, { useEffect } from "react";
import { ViewMode } from "../types";
import { FloatingDock } from "./FloatingDock";
import { AICommandCapsule } from "./AICommandCapsule";
import { ProjectRecord } from "../types";

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
      className={`min-h-screen relative flex flex-col font-sans text-zinc-100 select-none overflow-x-hidden ${
        theme === "dark" ? "bg-[#060913]" : "bg-slate-900"
      }`}
    >
      {/* Ambient Lighting Halos (Strictly limited to 2 subtle radial gradients, no heavy 3D) */}
      <div
        className="fixed top-[-120px] left-[15%] w-[640px] h-[480px] rounded-full pointer-events-none opacity-40 blur-[100px] bg-gradient-to-br from-indigo-600/30 via-indigo-900/15 to-transparent z-0"
        aria-hidden="true"
      />
      <div
        className="fixed bottom-[-100px] right-[10%] w-[540px] h-[420px] rounded-full pointer-events-none opacity-30 blur-[90px] bg-gradient-to-tl from-indigo-500/25 via-blue-900/10 to-transparent z-0"
        aria-hidden="true"
      />

      {/* macOS Native Titlebar Drag Strip & Window Traffic-Light Safety Margin */}
      <header
        data-tauri-drag-region
        className="fixed top-0 left-0 right-0 h-10 z-30 flex items-center justify-between px-20 border-b border-white/5 bg-[#060913]/70 backdrop-blur-md"
      >
        <div className="flex items-center gap-2 text-xs font-medium text-zinc-400 pointer-events-none">
          <span className="font-semibold text-zinc-200">LocalForge</span>
          <span className="text-zinc-600">/</span>
          <span className="capitalize text-zinc-400 font-mono text-[11px]">{activeMode}</span>
        </div>

        <div className="flex items-center gap-3 text-[11px] font-mono text-zinc-500 pointer-events-none">
          <span>v0.1.0 (Phase R1.3A)</span>
        </div>
      </header>

      {/* Floating Vertical Navigation Dock */}
      <FloatingDock
        activeMode={activeMode}
        onSelectMode={onSelectMode}
        theme={theme}
        onToggleTheme={onToggleTheme}
      />

      {/* Main Studio Viewport (With left margin to clear dock + top margin for traffic lights) */}
      <main className="relative z-10 flex-1 ml-24 mr-8 mt-14 max-w-7xl">
        {children}
      </main>

      {/* Floating AI Command Capsule & Modal */}
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
