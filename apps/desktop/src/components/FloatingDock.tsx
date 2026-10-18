import React from "react";
import {
  Compass,
  Layers,
  Cpu,
  HardDrive,
  CloudUpload,
  Bot,
  Settings,
  Sun,
  Moon,
} from "lucide-react";
import { ViewMode } from "../types";

export interface FloatingDockProps {
  activeMode: ViewMode;
  onSelectMode: (mode: ViewMode) => void;
  theme: "dark" | "light";
  onToggleTheme: () => void;
}

interface NavItem {
  id: ViewMode;
  label: string;
  icon: React.ComponentType<{ className?: string }>;
  shortcut: string;
}

const NAV_ITEMS: NavItem[] = [
  { id: "studio", label: "Studio", icon: Compass, shortcut: "⌘1" },
  { id: "projects", label: "Projects", icon: Layers, shortcut: "⌘2" },
  { id: "services", label: "Services", icon: Cpu, shortcut: "⌘3" },
  { id: "storage", label: "Storage", icon: HardDrive, shortcut: "⌘4" },
  { id: "deployments", label: "Deployments", icon: CloudUpload, shortcut: "⌘5" },
  { id: "mcp", label: "AI & MCP", icon: Bot, shortcut: "⌘6" },
  { id: "settings", label: "Settings", icon: Settings, shortcut: "⌘7" },
];

export const FloatingDock: React.FC<FloatingDockProps> = ({
  activeMode,
  onSelectMode,
  theme,
  onToggleTheme,
}) => {
  return (
    <aside
      aria-label="Application Navigation Dock"
      className="fixed left-5 top-20 z-40 flex flex-col items-center p-2 rounded-2xl lf-material-dock select-none"
      style={{
        boxShadow:
          "0 16px 40px -8px rgba(0, 0, 0, 0.5), 0 0 24px -4px rgba(79, 110, 247, 0.18), inset 0 1px 0 0 rgba(255, 255, 255, 0.2)",
      }}
    >
      <nav className="flex flex-col gap-2 relative" role="tablist">
        {NAV_ITEMS.map((item) => {
          const isActive = activeMode === item.id;
          const Icon = item.icon;

          return (
            <div key={item.id} className="relative group">
              <button
                role="tab"
                aria-selected={isActive}
                aria-label={`${item.label} (${item.shortcut})`}
                tabIndex={0}
                onClick={() => onSelectMode(item.id)}
                className={`relative flex items-center justify-center w-11 h-11 rounded-xl transition-all duration-200 outline-none focus-visible:ring-2 focus-visible:ring-indigo-400 ${
                  isActive
                    ? "text-white bg-indigo-600/35 border border-indigo-400/50 shadow-[0_0_16px_rgba(79,110,247,0.45)]"
                    : "text-zinc-400 hover:text-zinc-100 hover:bg-white/5 border border-transparent"
                }`}
              >
                <Icon className={`w-5 h-5 transition-transform duration-200 ${isActive ? "scale-110 text-indigo-300" : "group-hover:scale-105"}`} />
                {isActive && (
                  <span
                    className="absolute left-[-6px] top-1/2 -translate-y-1/2 w-1 h-5 bg-indigo-400 rounded-full shadow-[0_0_8px_rgba(129,140,248,0.8)]"
                    aria-hidden="true"
                  />
                )}
              </button>

              {/* Flyout Tooltip to the right */}
              <div
                role="tooltip"
                className="pointer-events-none absolute left-[calc(100%+14px)] top-1/2 -translate-y-1/2 hidden group-hover:flex items-center gap-2 px-3 py-1.5 rounded-lg text-xs font-medium text-zinc-100 bg-zinc-900/90 border border-white/15 backdrop-blur-md shadow-xl whitespace-nowrap z-50 animate-in fade-in zoom-in-95 duration-150"
              >
                <span>{item.label}</span>
                <kbd className="px-1.5 py-0.5 text-[10px] uppercase font-mono tracking-wider bg-white/10 rounded text-zinc-300 border border-white/10">
                  {item.shortcut}
                </kbd>
              </div>
            </div>
          );
        })}
      </nav>

      <div className="w-6 h-px my-3 bg-white/10" aria-hidden="true" />

      {/* Theme Toggle Button */}
      <div className="relative group">
        <button
          type="button"
          aria-label={`Switch to ${theme === "dark" ? "Light" : "Dark"} mode`}
          onClick={onToggleTheme}
          className="flex items-center justify-center w-11 h-11 rounded-xl text-zinc-400 hover:text-amber-300 hover:bg-white/5 transition-all outline-none focus-visible:ring-2 focus-visible:ring-amber-400"
        >
          {theme === "dark" ? (
            <Sun className="w-5 h-5 transition-transform duration-200 group-hover:rotate-45" />
          ) : (
            <Moon className="w-5 h-5 transition-transform duration-200 group-hover:-rotate-12 text-indigo-300" />
          )}
        </button>
        <div
          role="tooltip"
          className="pointer-events-none absolute left-[calc(100%+14px)] top-1/2 -translate-y-1/2 hidden group-hover:flex items-center px-3 py-1.5 rounded-lg text-xs font-medium text-zinc-100 bg-zinc-900/90 border border-white/15 backdrop-blur-md shadow-xl whitespace-nowrap z-50"
        >
          <span>{theme === "dark" ? "Light Theme" : "Dark Theme"}</span>
        </div>
      </div>
    </aside>
  );
};
