import React from "react";
import { Folder, Globe, Play, Square, AlertOctagon, ExternalLink, Cpu } from "lucide-react";
import { ProjectRecord } from "../types";

interface ProjectListProps {
  projects: ProjectRecord[];
  onToggleStatus: (id: string, currentStatus: string) => void;
}

export const ProjectList: React.FC<ProjectListProps> = ({ projects, onToggleStatus }) => {
  if (projects.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center p-12 border border-dashed border-slate-800 rounded-xl bg-slate-900/40 text-center">
        <Folder className="w-12 h-12 text-slate-600 mb-3" />
        <h3 className="text-lg font-medium text-slate-200">No Projects Yet</h3>
        <p className="text-sm text-slate-400 max-w-sm mt-1">
          Create a new project or import an existing PHP workspace directly from your internal SSD or external drive.
        </p>
      </div>
    );
  }

  return (
    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
      {projects.map((p) => {
        const isDisconnected = p.status === "disconnected";
        const isRunning = p.status === "running";

        return (
          <div
            key={p.id}
            className={`flex flex-col justify-between p-5 rounded-xl border transition-all duration-200 ${
              isDisconnected
                ? "bg-rose-950/20 border-rose-900/50 shadow-sm"
                : isRunning
                ? "bg-slate-900/90 border-indigo-500/50 shadow-indigo-500/10 shadow-lg"
                : "bg-slate-900/60 border-slate-800 hover:border-slate-700"
            }`}
          >
            <div>
              <div className="flex items-start justify-between gap-2">
                <h3 className="font-semibold text-base text-slate-100 truncate" title={p.name}>
                  {p.name}
                </h3>
                <span
                  className={`inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium uppercase tracking-wider ${
                    isRunning
                      ? "bg-emerald-950 text-emerald-400 border border-emerald-800"
                      : isDisconnected
                      ? "bg-rose-950 text-rose-400 border border-rose-800"
                      : "bg-slate-800 text-slate-400 border border-slate-700"
                  }`}
                >
                  {p.status}
                </span>
              </div>

              <div className="mt-3 space-y-1.5 text-xs text-slate-400">
                <div className="flex items-center gap-1.5 truncate">
                  <Globe className="w-3.5 h-3.5 text-slate-500 flex-shrink-0" />
                  <a
                    href={`http://${p.local_domain}`}
                    target="_blank"
                    rel="noreferrer"
                    className="text-indigo-400 hover:underline truncate"
                  >
                    {p.local_domain}
                  </a>
                </div>

                <div className="flex items-center gap-1.5 truncate" title={p.path}>
                  <Folder className="w-3.5 h-3.5 text-slate-500 flex-shrink-0" />
                  <span className="truncate">{p.path}</span>
                </div>

                <div className="flex items-center gap-1.5">
                  <Cpu className="w-3.5 h-3.5 text-slate-500 flex-shrink-0" />
                  <span>PHP {p.php_version}</span>
                </div>
              </div>
            </div>

            <div className="mt-5 pt-3 border-t border-slate-800/80 flex items-center justify-between">
              {isDisconnected ? (
                <div className="flex items-center gap-1.5 text-xs text-rose-400 font-medium">
                  <AlertOctagon className="w-4 h-4 flex-shrink-0" />
                  <span>Storage volume disconnected</span>
                </div>
              ) : (
                <button
                  onClick={() => onToggleStatus(p.id, p.status)}
                  className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium transition-colors ${
                    isRunning
                      ? "bg-slate-800 hover:bg-slate-700 text-slate-200"
                      : "bg-indigo-600 hover:bg-indigo-500 text-white"
                  }`}
                >
                  {isRunning ? (
                    <>
                      <Square className="w-3.5 h-3.5 fill-current" />
                      <span>Stop</span>
                    </>
                  ) : (
                    <>
                      <Play className="w-3.5 h-3.5 fill-current" />
                      <span>Start</span>
                    </>
                  )}
                </button>
              )}

              <a
                href={`http://${p.local_domain}`}
                target="_blank"
                rel="noreferrer"
                className={`p-1.5 rounded-lg hover:bg-slate-800 text-slate-400 hover:text-slate-200 transition-colors ${
                  isDisconnected ? "opacity-30 pointer-events-none" : ""
                }`}
                title="Open in Browser"
              >
                <ExternalLink className="w-4 h-4" />
              </a>
            </div>
          </div>
        );
      })}
    </div>
  );
};
