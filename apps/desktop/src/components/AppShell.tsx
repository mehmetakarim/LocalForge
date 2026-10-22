import React, { useEffect } from "react";
import { ViewMode, ProjectRecord, VolumeInfo, RuntimeOverview } from "../types";
import { FloatingDock } from "./FloatingDock";
import { AICommandCapsule } from "./AICommandCapsule";

export interface AppShellProps {
  activeMode: ViewMode;
  onSelectMode: (mode: ViewMode) => void;
  projects: ProjectRecord[];
  volumes: VolumeInfo[];
  runtime: RuntimeOverview | null;
  isCapsuleOpen: boolean;
  onOpenCapsule: () => void;
  onCloseCapsule: () => void;
  onSelectProject?: (project: ProjectRecord) => void;
  onCreateProject?: () => void;
  onRefreshData?: () => void;
  onOpenWelcome?: () => void;
  zoomLevel?: number;
  onZoomIn?: () => void;
  onZoomOut?: () => void;
  onZoomReset?: () => void;
  isInspectorOpen?: boolean;
  children: React.ReactNode;
}

export const AppShell: React.FC<AppShellProps> = ({
  activeMode,
  onSelectMode,
  projects,
  volumes,
  runtime,
  isCapsuleOpen,
  onOpenCapsule,
  onCloseCapsule,
  onSelectProject,
  onCreateProject,
  onRefreshData,
  onOpenWelcome,
  zoomLevel = 100,
  onZoomIn,
  onZoomOut,
  onZoomReset,
  isInspectorOpen = false,
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

  const externalVol = volumes.find((v) => !v.is_internal || v.is_removable);
  const primaryVolumeName = externalVol?.mount_point || volumes[0]?.mount_point || "/Volumes/Mac Harici Disk";

  return (
    <div className="h-screen max-h-screen relative flex flex-col font-['Figtree',sans-serif] text-[#FFFBFF] select-none overflow-hidden bg-[#191337]">
      {/* Studio Deep Atmospheric Backdrop */}
      <div
        className="fixed inset-0 pointer-events-none z-0"
        style={{
          background:
            "radial-gradient(circle at 50% 18%, rgba(59, 55, 106, 0.48) 0%, rgba(43, 36, 80, 0.65) 45%, #191337 90%)",
        }}
        aria-hidden="true"
      />

      {/* Atmospheric Top Violet Ambient Halo */}
      <div
        className="fixed top-[-120px] left-1/2 -translate-x-1/2 w-[1000px] h-[480px] pointer-events-none blur-[80px] z-0"
        style={{
          background:
            "radial-gradient(ellipse at center, rgba(105, 95, 164, 0.32) 0%, rgba(43, 36, 80, 0.15) 50%, transparent 80%)",
        }}
        aria-hidden="true"
      />

      {/* Unified macOS Titlebar with Official Brand Lockup */}
      <header
        data-tauri-drag-region
        className="fixed top-0 left-0 right-0 h-[52px] z-50 flex items-center justify-between px-7 border-b border-[rgba(183,173,223,0.10)] bg-[#2B2450]/35 backdrop-blur-xl"
      >
        {/* Genuine macOS Native Traffic Lights Clearance Spacing (No Duplicate HTML Dots) */}
        <div
          data-tauri-drag-region
          className="w-20 h-full flex items-center select-none pointer-events-none"
          aria-hidden="true"
        />

        {/* Official Brand Lockup with Rendered Elephant Emblem */}
        <div className="flex items-center gap-3" data-tauri-drag-region>
          <img
            src="/brand/localforge-emblem.png"
            alt="LocalForge Elephant Emblem"
            className="w-6 h-6 rounded-md object-cover shadow-[0_2px_8px_rgba(10,6,30,0.5)] border border-[rgba(183,173,223,0.2)]"
          />
          <span className="font-['Plus_Jakarta_Sans',sans-serif] font-extrabold text-[15px] tracking-[-0.02em] text-[#FFFBFF]">
            LocalForge
          </span>
          <span className="font-mono text-[10px] font-semibold tracking-[0.08em] uppercase text-[#B7ADDF] bg-[rgba(183,173,223,0.12)] border border-[rgba(183,173,223,0.22)] px-2 py-0.5 rounded-md">
            Spatial Studio
          </span>
        </div>

        {/* System Health Indicators & Volume Badge */}
        <div className="flex items-center gap-3 font-mono text-[11px]">
          {/* Nginx Gateway Pill */}
          <div className="flex items-center gap-2 text-[#D4CCF1] bg-[#2B2450]/55 px-2.5 py-1 rounded-lg border border-[rgba(183,173,223,0.12)]">
            <span
              className={`w-2 h-2 rounded-full ${
                runtime?.nginx_running
                  ? "bg-[#10B981] shadow-[0_0_8px_#10B981]"
                  : runtime?.nginx_port_conflict
                  ? "bg-[#EF4444] shadow-[0_0_8px_#EF4444]"
                  : "bg-[#6B7280]"
              }`}
            />
            <span>
              Ağ Geçidi: {runtime?.nginx_running ? "Doğrulandı" : runtime?.nginx_port_conflict ? "Port Çakışması" : "Durduruldu"} ({runtime?.nginx_port || 8080})
            </span>
          </div>

          {/* MariaDB Pill */}
          <div className="flex items-center gap-2 text-[#D4CCF1] bg-[#2B2450]/55 px-2.5 py-1 rounded-lg border border-[rgba(183,173,223,0.12)]">
            <span
              className={`w-2 h-2 rounded-full ${
                runtime?.mariadb_alive
                  ? "bg-[#10B981] shadow-[0_0_8px_#10B981]"
                  : "bg-[#6B7280]"
              }`}
            />
            <span>
              MariaDB: {runtime?.mariadb_alive ? "Aktif" : "Durduruldu"} ({runtime?.mariadb_port || 3306})
            </span>
          </div>

          {/* Volume Badge */}
          <div className="text-[#FFFBFF] bg-[#2B2450]/55 px-2.5 py-1 rounded-lg border border-[rgba(183,173,223,0.28)] truncate max-w-[200px]">
            <span>{primaryVolumeName}</span>
          </div>
        </div>
      </header>

      {/* Floating Vertical Navigation Dock on Left */}
      <FloatingDock
        activeMode={activeMode}
        onSelectMode={onSelectMode}
        onOpenWelcome={onOpenWelcome}
      />

      {/* Floating Functional Zoom Controls on Right (Hidden when Project Inspector is open) */}
      {onZoomIn && onZoomOut && onZoomReset && !isInspectorOpen && (
        <aside
          aria-label="Sahne Yakınlaştırma Denetimleri"
          className="fixed right-6 top-1/2 -translate-y-1/2 z-40 flex flex-col items-center gap-1.5 p-2 rounded-2xl bg-[#2B2450]/55 border border-[rgba(183,173,223,0.16)] backdrop-blur-2xl shadow-xl font-mono text-xs text-[#D4CCF1]"
        >
          <button
            type="button"
            onClick={onZoomIn}
            className="w-7 h-7 rounded-lg bg-[rgba(183,173,223,0.10)] hover:bg-[rgba(183,173,223,0.20)] border border-[rgba(183,173,223,0.15)] text-[#FFFBFF] flex items-center justify-center transition-colors cursor-pointer"
            title="Yakınlaştır (+5%)"
          >
            +
          </button>
          <button
            type="button"
            onClick={onZoomReset}
            className="text-[11px] font-semibold text-[#D4CCF1] hover:text-[#FFFBFF] px-1 py-0.5 rounded transition-colors cursor-pointer"
            title="Sıfırla (100%)"
          >
            %{zoomLevel}
          </button>
          <button
            type="button"
            onClick={onZoomOut}
            className="w-7 h-7 rounded-lg bg-[rgba(183,173,223,0.10)] hover:bg-[rgba(183,173,223,0.20)] border border-[rgba(183,173,223,0.15)] text-[#FFFBFF] flex items-center justify-center transition-colors cursor-pointer"
            title="Uzaklaştır (-5%)"
          >
            -
          </button>
        </aside>
      )}

      {/* Main Studio Viewport with Independent Vertical Scrolling */}
      <main className="relative z-10 flex-1 ml-24 mr-4 mt-[52px] h-[calc(100vh-52px)] overflow-y-auto overflow-x-hidden flex flex-col items-center justify-start min-h-0">
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
