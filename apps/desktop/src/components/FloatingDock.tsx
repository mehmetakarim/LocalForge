import React from "react";
import {
  Compass,
  Layers,
  Cpu,
  HardDrive,
  CloudUpload,
  Bot,
  Settings,
  Sparkles,
} from "lucide-react";
import { ViewMode } from "../types";

export interface FloatingDockProps {
  activeMode: ViewMode;
  onSelectMode: (mode: ViewMode) => void;
  onOpenWelcome?: () => void;
}

interface NavItem {
  id: ViewMode;
  label: string;
  icon: React.ComponentType<{ className?: string }>;
  shortcut: string;
}

const NAV_ITEMS: NavItem[] = [
  { id: "studio", label: "Spatial Studio", icon: Compass, shortcut: "⌘1" },
  { id: "projects", label: "Projeler", icon: Layers, shortcut: "⌘2" },
  { id: "services", label: "Servisler & Çalışma Zamanı", icon: Cpu, shortcut: "⌘3" },
  { id: "storage", label: "Depolama & Birimler", icon: HardDrive, shortcut: "⌘4" },
  { id: "deployments", label: "Dağıtım", icon: CloudUpload, shortcut: "⌘5" },
  { id: "mcp", label: "AI & MCP", icon: Bot, shortcut: "⌘6" },
  { id: "settings", label: "Sistem Tercihleri", icon: Settings, shortcut: "⌘7" },
];

export const FloatingDock: React.FC<FloatingDockProps> = ({
  activeMode,
  onSelectMode,
  onOpenWelcome,
}) => {
  return (
    <aside
      aria-label="Uygulama Gezinti Dokı"
      className="fixed left-6 top-24 z-40 flex flex-col items-center p-2 rounded-2xl select-none bg-[#2B2450]/65 backdrop-blur-2xl border border-[rgba(183,173,223,0.16)] border-t-[rgba(183,173,223,0.32)] shadow-[0_20px_44px_-10px_rgba(10,6,30,0.7)]"
    >
      <nav className="flex flex-col gap-2 relative" role="tablist">
        {NAV_ITEMS.map((item) => {
          const isActive = activeMode === item.id;
          const Icon = item.icon;

          return (
            <div key={item.id} className="relative group">
              <button
                type="button"
                role="tab"
                aria-selected={isActive}
                aria-label={`${item.label} (${item.shortcut})`}
                tabIndex={0}
                onClick={() => onSelectMode(item.id)}
                className={`relative flex items-center justify-center w-10 h-10 rounded-xl transition-all duration-200 outline-none focus-visible:ring-2 focus-visible:ring-[#B7ADDF] ${
                  isActive
                    ? "bg-[#695FA4] text-[#FFFBFF] border border-[#B7ADDF] shadow-[0_4px_16px_rgba(105,95,164,0.45)]"
                    : "text-[#D4CCF1] hover:text-[#FFFBFF] hover:bg-[rgba(183,173,223,0.12)] border border-transparent"
                }`}
              >
                <Icon
                  className={`w-4 h-4 transition-transform duration-200 ${
                    isActive ? "scale-110 text-[#FFFBFF]" : "group-hover:scale-105"
                  }`}
                />
                {isActive && (
                  <span
                    className="absolute left-[-6px] top-1/2 -translate-y-1/2 w-1 h-4 bg-[#B7ADDF] rounded-full shadow-[0_0_8px_rgba(183,173,223,0.8)]"
                    aria-hidden="true"
                  />
                )}
              </button>

              {/* Flyout Tooltip to the right */}
              <div
                role="tooltip"
                className="pointer-events-none absolute left-[calc(100%+14px)] top-1/2 -translate-y-1/2 hidden group-hover:flex items-center gap-2 px-3 py-1.5 rounded-lg text-xs font-medium text-[#FFFBFF] bg-[#191337]/95 border border-[rgba(183,173,223,0.22)] backdrop-blur-md shadow-xl whitespace-nowrap z-50 animate-in fade-in zoom-in-95 duration-150"
              >
                <span className="font-['Plus_Jakarta_Sans',sans-serif]">{item.label}</span>
                <kbd className="px-1.5 py-0.5 text-[10px] uppercase font-mono tracking-wider bg-[rgba(183,173,223,0.15)] rounded text-[#B7ADDF] border border-[rgba(183,173,223,0.25)]">
                  {item.shortcut}
                </kbd>
              </div>
            </div>
          );
        })}
      </nav>

      {onOpenWelcome && (
        <>
          <div className="w-5 h-px my-2.5 bg-[rgba(183,173,223,0.14)]" aria-hidden="true" />
          <div className="relative group">
            <button
              type="button"
              onClick={onOpenWelcome}
              aria-label="Hoş Geldiniz Ekranı (İlk Çalıştırma)"
              className="flex items-center justify-center w-10 h-10 rounded-xl text-[#D4CCF1] hover:text-[#FFFBFF] hover:bg-[rgba(183,173,223,0.12)] transition-all outline-none focus-visible:ring-2 focus-visible:ring-[#B7ADDF]"
            >
              <Sparkles className="w-4 h-4 text-[#B7ADDF]" />
            </button>
            <div
              role="tooltip"
              className="pointer-events-none absolute left-[calc(100%+14px)] top-1/2 -translate-y-1/2 hidden group-hover:flex items-center gap-2 px-3 py-1.5 rounded-lg text-xs font-medium text-[#FFFBFF] bg-[#191337]/95 border border-[rgba(183,173,223,0.22)] backdrop-blur-md shadow-xl whitespace-nowrap z-50"
            >
              <span className="font-['Plus_Jakarta_Sans',sans-serif]">Başlangıç Ekranı</span>
            </div>
          </div>
        </>
      )}
    </aside>
  );
};
